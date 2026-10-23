# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Revalidate samples into review groups without changing labels or moving bytes."""

import argparse
import hashlib
import json
from collections import Counter, defaultdict
from pathlib import Path

import pyarrow as pa
import pyarrow.parquet as pq

from .acquisition import object_path
from .adjudication import load as load_adjudications
from .detector_conflicts import observed_conflict
from .normalization import canonical_type, normalize_annotation
from .origins import Provenance
from .parquet_metadata import LABEL_STATUSES
from .runtime import private_output, write
from .validators import observe

STATUSES = LABEL_STATUSES
PRECEDENCE = (
    "validated_manual",
    "llm-validated",
    "validated_auto",
    "validated_tools",
    "validated_origin",
)
"""Evidence tiers, strongest first. A stronger tier settles identity; a weaker one that
disagrees is kept as a hard case, never as a veto."""
SUBJECT_DETECTORS = frozenset({"magika"})
"""Detectors under evaluation. Their claims are evidence about the detector, not the bytes."""
MINIMUM_VOTES = 2


def _mapped_claims(annotation: dict, formats: dict) -> dict:
    markings = annotation.get("vt_markings_history", [annotation.get("vt_markings", {})])
    return observed_conflict(markings, annotation.get("verdicts", {}), formats)


def detectors_disagree(annotation: dict, formats: dict) -> tuple[bool, set[str]]:
    """Whether detector claims conflict with each other or the label, and what they claim."""
    evidence = _mapped_claims(annotation, formats)
    claims = {canonical_type(k) for k in evidence["mapped_claims"].values()}
    disagreement = bool(
        annotation.get("conflicting")
        or annotation.get("label_status") == "conflicting"
        or evidence["conflicting"]
    )
    return disagreement, claims


def passes(validation: dict) -> list[dict]:
    """Auto-eligible passes, with generic ones set aside when a specific one exists.

    A LightGBM model is also valid ASCII text and a JAR is also a zip. When two modules both
    pass, the one naming a container or encoding is evidence of integrity, not a competing
    identity, so it must not turn a proof into a tie.
    """
    eligible = [
        o for o in validation["observations"] if o["status"] == "pass" and o.get("auto_eligible")
    ]
    specific = [o for o in eligible if not o.get("generic")]
    return specific or eligible


def proven(validation: dict) -> str | None:
    """The single format a structural validator proved, when no validator failed it.

    A failure of another format does not contradict the proof: a gzip stream the zip
    validator refutes is still proven gzip. The refuted format stays recorded.
    """
    successful = {o["format_id"] for o in passes(validation)}
    failed = {o["format_id"] for o in validation["observations"] if o["status"] == "fail"}
    if len(successful) == 1 and not successful & failed:
        return next(iter(successful))
    return None


def refuted(annotation: dict, validation: dict) -> set[str]:
    """Formats a validator has failed on, in this run or any earlier one.

    A structural refutation is a statement about bytes, and the bytes do not change, so it
    is remembered. Forgetting it made labels oscillate: a weak label supplied the hint that
    let a validator run and refute it, losing the label lost the hint, and the next pass
    restored the label because the refutation was no longer visible.
    """
    return {o["format_id"] for o in validation["observations"] if o["status"] == "fail"} | set(
        annotation.get("refuted_format_ids", [])
    )


def tool_consensus(annotation: dict, validation: dict, formats: dict) -> str | None:
    """The one format independent detectors agree on, when no validator refutes it.

    Magika is excluded: a corpus that admits its own predictions as truth would measure
    agreement with itself. Two distinct detectors are the minimum, so a single tool
    repeated across VirusTotal history entries never carries a label on its own.
    """
    votes: dict[str, set[str]] = {}
    for key, kind in _mapped_claims(annotation, formats)["mapped_claims"].items():
        name = key.rsplit(".", 1)[-1]
        if name not in SUBJECT_DETECTORS:
            votes.setdefault(name, set()).add(canonical_type(kind))
    kinds = {kind for claimed in votes.values() for kind in claimed}
    if len(votes) < MINIMUM_VOTES or len(kinds) != 1:
        return None
    kind = next(iter(kinds))
    if kind in refuted(annotation, validation) or kind not in formats:
        return None
    return kind


def _review(annotation: dict, field: str, actor: str, formats: dict) -> dict | None:
    review = annotation.get(field, {})
    labels = review.get("format_ids", [])
    if (
        review.get(actor)
        and review.get("decision") == "validated"
        and review.get("evidence")
        and labels
        and set(labels) <= formats.keys()
    ):
        return {"format_ids": sorted(set(labels)), "evidence": review}
    return None


def _provenance_stands(named, validation, formats, annotation, prior) -> bool:
    """Whether nothing stronger contradicts what a pinned source path says.

    A curated negative label outranks provenance: an `invalid` sample was chosen because
    it is malformed, and its plausible extension is exactly what makes it a useful
    negative rather than a mislabelled positive.
    """
    if prior is not None and prior in formats:
        role = json.loads(formats[prior].get("metadata_json") or "{}").get("class_role")
        if role == "negative":
            return False
    # Only a refutation of the named class contradicts it. "Not IGES" says nothing against a
    # path naming C, and treating it as a veto withheld labels whenever a weakly applicable
    # validator happened to run.
    if named in refuted(annotation, validation):
        return False
    disagreement, claims = detectors_disagree(annotation, formats)
    return not disagreement and not (claims - {named})


def decide(
    annotation: dict,
    validation: dict,
    formats: dict,
    origins=(),
    prior: str | None = None,
    provenance: Provenance | None = None,
) -> dict | None:
    """The strongest identity decision the evidence supports, or None.

    A legacy label_status string is not evidence: an unattributed "accepted" records that
    some earlier pass admitted the file, not that anyone or anything verified it.
    """
    attributed = _review(annotation, "manual_validation", "reviewer", formats)
    if attributed:
        return {"basis": "validated_manual", **attributed}
    reviewed = _review(annotation, "llm_validation", "model", formats)
    if reviewed:
        return {"basis": "llm-validated", **reviewed}
    validated = proven(validation)
    if validated is not None and validated in formats:
        evidence = [o for o in passes(validation) if o["format_id"] == validated]
        return {"basis": "validated_auto", "format_ids": [validated], "evidence": evidence}
    agreed = tool_consensus(annotation, validation, formats)
    if agreed is not None:
        return {
            "basis": "validated_tools",
            "format_ids": [agreed],
            "evidence": "detector_consensus",
        }
    named = provenance.identity(origins) if origins and provenance else None
    if named is not None and _provenance_stands(named, validation, formats, annotation, prior):
        return {
            "basis": "validated_origin",
            "format_ids": [named],
            "evidence": "reproducible_generator"
            if any(o.startswith("generated:") for o in origins)
            else "pinned_source_path",
        }
    return None


def classify(
    annotation: dict,
    validation: dict,
    formats: dict,
    origins=(),
    prior: str | None = None,
    provenance: Provenance | None = None,
) -> str:
    """Review group. The strongest evidence settles identity; detector disagreement is
    retained as evidence and as a hard-case flag, not as a veto over the bytes."""
    provenance = provenance or (Provenance(formats) if origins else None)
    decision = decide(annotation, validation, formats, origins, prior, provenance)
    if decision:
        return decision["basis"]
    disagreement, claims = detectors_disagree(annotation, formats)
    successful = {o["format_id"] for o in passes(validation)}
    # A validator that refutes the one format every detector named is a conflict between
    # evidence sources, not an absence of evidence.
    if disagreement or len(claims | successful) > 1 or claims & refuted(annotation, validation):
        return "conflicting"
    return "need_review"


def apply_label(
    annotation: dict,
    validation: dict,
    formats: dict,
    origins=(),
    prior: str | None = None,
    provenance: Provenance | None = None,
) -> dict:
    """Assign supported labels while retaining earlier labels and independent conflicts."""
    annotation = normalize_annotation(annotation)
    result = dict(annotation)
    legacy = result.pop("label_status", None)
    if legacy is not None and legacy not in STATUSES:
        result["legacy_label_status"] = legacy
    result.pop("conflicting", None)
    remembered = refuted(annotation, validation)
    if remembered:
        result["refuted_format_ids"] = sorted(remembered)
    provenance = provenance or (Provenance(formats) if origins else None)
    decision = decide(annotation, validation, formats, origins, prior, provenance)
    result["validation_status"] = classify(
        annotation, validation, formats, origins, prior, provenance
    )
    result["label_status"] = result["validation_status"]
    if decision:
        disagreement, claims = detectors_disagree(annotation, formats)
        proofs = [o for o in passes(validation) if o["format_id"] in formats]
        # Weaker evidence never vetoes the decision, but naming a different format makes
        # this exactly the discriminative sample a benchmark wants kept as a hard case.
        named = provenance.identity(origins) if origins and provenance else None
        weaker = claims | {o["format_id"] for o in proofs} | ({named} if named else set())
        contested = disagreement or bool(weaker - set(decision["format_ids"]))
        result["tags"] = sorted(
            set(result.get("tags", []))
            | {tag for o in proofs for tag in o.get("tags", [])}
            | ({"detectors_disagree"} if contested else set())
        )
        result.setdefault("previous_format_ids", annotation.get("format_ids", []))
        result["format_ids"] = decision["format_ids"]
        result["label_decision"] = decision
        # Detector disagreement on a proven file is exactly the discriminative case a
        # benchmark wants, so it stays a hard case even though the label is settled.
        result["hard_case"] = bool(annotation.get("hard_case") or contested)
    return result


def _digest(path: Path) -> str:
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


UNKNOWN = "unknown"


def sample_hints(row: dict, annotation: dict, formats: dict, provenance: Provenance) -> set[str]:
    """What the validators are told this sample might be.

    Every source here is fixed for the life of the sample -- where it came from, what
    discovery claimed, what it was previously labelled -- so the hint set does not move
    when the label does. That matters because hints gate which validators run: deriving
    them from the current label let a provenance-based label unlock a validator that
    refuted it, which withdrew the label, which re-enabled the label, oscillating forever.
    """
    hints = set(annotation.get("format_ids", [])) | set(annotation.get("discovery_format_ids", []))
    hints |= set(annotation.get("previous_format_ids", []))
    hints |= set(annotation.get("existing_accepted_format_ids", []))
    named = provenance.identity(row["origins"]) if row["origins"] else None
    if named:
        hints.add(named)
    if not row["format_id"].startswith("__"):
        hints.add(row["format_id"])
    return hints


def _class_for(decision: dict, formats: dict) -> str:
    labels = decision.get("format_ids") or []
    return labels[0] if decision["validation_status"] in PRECEDENCE and labels else UNKNOWN


def _relabelled_row(row: dict, decision: dict, formats: dict) -> dict:
    """One sample row carrying the decision this run reached."""
    kind = _class_for(decision, formats)
    annotation = {k: v for k, v in decision.items() if k not in ("label_status", "hard_case")}
    annotation.pop("validation_status", None)
    annotation["format_ids"] = [kind]
    return {
        "class_ordinal": formats[kind]["ordinal"],
        "sample_ordinal": 0,
        "format_id": kind,
        "label_status": decision["validation_status"],
        "sha256": row["sha256"],
        "size": row["size"],
        "origins": row["origins"],
        "hard_case": bool(decision.get("hard_case")),
        "annotation_json": json.dumps(annotation, sort_keys=True),
    }


def validate_dataset(
    metadata: Path, store: Path, output: Path, apply: bool = False, adjudications=None
) -> dict:
    taxonomy = metadata / "taxonomy.parquet"
    if not taxonomy.exists():
        taxonomy = metadata / "classes.parquet"
    formats = {r["format_id"]: r for r in pq.read_table(taxonomy).to_pylist()}
    provenance = Provenance(formats)
    # Recorded human decisions are applied on every run, so a correction added to the file
    # takes effect the next time the corpus is validated rather than needing a migration.
    if adjudications is None:
        adjudications = metadata / "config/label-adjudications.json"
    reviews = load_adjudications(adjudications, formats) if Path(adjudications).exists() else {}
    if apply and UNKNOWN not in formats:
        raise ValueError(f"Applying labels needs an {UNKNOWN!r} class for unverified samples")
    schema = pa.schema(
        [
            ("sha256", pa.binary(32)),
            ("format_id", pa.string()),
            ("validation_status", pa.string()),
            ("validation_json", pa.string()),
            ("assigned_format_ids", pa.list_(pa.string())),
            ("tags", pa.list_(pa.string())),
            ("label_decision_json", pa.string()),
        ]
    )
    counts = Counter()
    relabelled: dict[str, list[dict]] = defaultdict(list)
    output.parent.mkdir(parents=True, exist_ok=True)
    temporary = output.with_suffix(".next")
    with pq.ParquetWriter(temporary, schema, compression="zstd") as writer:
        for batch in pq.ParquetFile(metadata / "samples.parquet").iter_batches(batch_size=128):
            rows = []
            for row in batch.to_pylist():
                sha = row["sha256"].hex()
                annotation = json.loads(row["annotation_json"])
                if row["sha256"] in reviews:
                    annotation = {**annotation, **reviews[row["sha256"]]}
                hints = sample_hints(row, annotation, formats, provenance)
                report = observe(object_path(store, sha), sha, hints=hints)
                # Origins and the existing label are provenance evidence; without them a
                # revalidation silently drops the labels only a pinned source path supports.
                prior = None if row["format_id"].startswith("__") else row["format_id"]
                decision = apply_label(
                    annotation, report, formats, row["origins"], prior, provenance
                )
                status = decision["validation_status"]
                counts[status] += 1
                if apply:
                    relabelled[_class_for(decision, formats)].append(
                        _relabelled_row(row, decision, formats)
                    )
                rows.append(
                    {
                        "sha256": row["sha256"],
                        "format_id": row["format_id"],
                        "validation_status": status,
                        "assigned_format_ids": decision.get("format_ids", []),
                        "tags": decision.get("tags", []),
                        "label_decision_json": json.dumps(
                            decision.get("label_decision"), sort_keys=True
                        ),
                        "validation_json": json.dumps(report, sort_keys=True),
                    }
                )
            writer.write_table(pa.Table.from_pylist(rows, schema))
    if apply:
        # Writing the decisions back makes the corpus a fixed point of its own labelling:
        # without it `validate` keeps reporting labels the published table does not carry.
        from .parquet_metadata import write_samples
        from .receipt import refresh

        write_samples(relabelled, formats, metadata / "samples.parquet")
        refresh(metadata)
    result = {
        "counts": {s: counts[s] for s in sorted(STATUSES)},
        "samples": sum(counts.values()),
        "applied": bool(apply),
        "validation_sha256": _digest(temporary),
        "scope": "Logical review groups; original labels and bytes preserved",
    }
    # Publish the table and the counts back to back, and name the table the counts came
    # from: two files cannot be renamed atomically, so a stale pairing has to be visible
    # rather than silently believed.
    temporary.replace(output)
    write(output.with_suffix(".json"), result)
    return result


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--metadata", type=Path, default=Path("."))
    parser.add_argument("--store", type=Path, default=Path("local/corpus"))
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument(
        "--apply",
        action="store_true",
        help="Write the decided labels back into samples.parquet and rebind the receipt.",
    )
    args = parser.parse_args(argv)
    print(
        json.dumps(
            validate_dataset(
                args.metadata, args.store, private_output(args.output), apply=args.apply
            ),
            sort_keys=True,
        )
    )
