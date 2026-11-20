# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0

"""Resolve and publish the licence of every GitHub repository the corpus draws from.

A redistributable evaluation corpus has to say where its samples came from and under what
terms. Evidence arrives from two places already on disk -- the frozen repository inventory,
whose licence was read at a pinned revision, and licence records carried on the samples
themselves -- and the offline join of those two needs no credentials, so the default path
is reproducible by anyone with the metadata. What neither covers can be asked of GitHub,
explicitly and opt-in.
"""

import argparse
import json
from collections import Counter
from pathlib import Path

import pyarrow as pa
import pyarrow.parquet as pq

from .runtime import private_output, write

INVENTORY = "frozen_repository_inventory"
ANNOTATION = "sample_annotation"
RESIDUAL = "github_api"
UNRESOLVED = "unresolved"
BEGIN, END = "<!-- BEGIN licenses -->", "<!-- END licenses -->"
NOT_A_LICENSE = frozenset({"NOASSERTION", "NONE"})
"""What GitHub returns when it found a licence file it could not identify. It records that
a question was asked, not that it was answered, so it must not close the question."""

SCHEMA = pa.schema(
    [
        ("repository", pa.string()),
        ("url", pa.string()),
        ("spdx_id", pa.string()),
        ("license_name", pa.string()),
        ("license_permalink", pa.string()),
        ("revision", pa.string()),
        ("basis", pa.string()),
        ("samples", pa.uint64()),
        ("allowed_by_policy", pa.bool_()),
    ]
)


def _observations(blob):
    """Licence records split by where they came from.

    The inventory read its licence at a pinned revision, so it wins. An annotation records
    what a sample's own provenance claimed, which is a useful cross-check and a fallback.
    """
    inventory, annotation = [], []
    for record in json.loads(blob or "[]"):
        if not record.get("spdx_id") or record["spdx_id"] in NOT_A_LICENSE:
            continue
        (inventory if str(record.get("basis", "")).startswith(INVENTORY) else annotation).append(
            record
        )
    return inventory, annotation


def resolve(sources_path, policy, client=None):
    """One row per cited repository, and a receipt saying how each licence was established.

    `client` is the only path that touches the network. Without it the result depends on
    nothing but the metadata, which is what makes the published table reproducible.
    """
    allowed = set(policy.get("allowed_spdx", []))
    rows, receipt = [], Counter()
    conflicts, agreements = [], 0
    for record in pq.read_table(sources_path).to_pylist():
        name = record["repository"]
        inventory, annotation = _observations(record["license_observations_json"])
        chosen, basis = (inventory[0], INVENTORY) if inventory else (None, UNRESOLVED)
        if chosen is None and annotation:
            chosen, basis = annotation[0], ANNOTATION
        if inventory and annotation:
            claimed = {r["spdx_id"] for r in annotation}
            if claimed == {inventory[0]["spdx_id"]}:
                agreements += 1
            else:
                conflicts.append(
                    {
                        "repository": name,
                        "inventory": inventory[0]["spdx_id"],
                        "annotation": sorted(claimed)[0],
                    }
                )
        if chosen is None and client is not None:
            # One dead or renamed repository must not abort the rest of the resolution.
            answer = client.license(name)
            if answer and answer.get("spdx_id"):
                chosen, basis = answer, RESIDUAL
        spdx = chosen.get("spdx_id") if chosen else None
        rows.append(
            {
                "repository": name,
                "url": record["url"],
                "spdx_id": spdx,
                "license_name": (chosen or {}).get("name"),
                "license_permalink": (chosen or {}).get("license_permalink")
                or (chosen or {}).get("permalink"),
                "revision": (chosen or {}).get("revision"),
                "basis": basis,
                "samples": record["samples"],
                "allowed_by_policy": spdx in allowed,
            }
        )
        receipt[basis] += 1
    return rows, {
        "repositories": len(rows),
        "resolved": sum(1 for r in rows if r["spdx_id"]),
        "unresolved": sum(1 for r in rows if not r["spdx_id"]),
        "basis_counts": dict(sorted(receipt.items())),
        "annotation_agreements": agreements,
        "annotation_conflicts": conflicts,
        "allowed_spdx": sorted(allowed),
        "scope": (
            "Licence reported for the repository at the pinned revision. File-level and "
            "dependency licences are not resolved, and a sample may carry its own terms."
        ),
    }


def _by_license(rows):
    grouped = {}
    for row in rows:
        grouped.setdefault(row["spdx_id"] or "unresolved", []).append(row)
    return {
        name: sorted(items, key=lambda r: r["repository"])
        for name, items in sorted(grouped.items())
    }


def markdown(rows):
    """The full list, grouped by licence. Generated: run `magika-datasets licenses`."""
    lines = [
        "# Source repositories",
        "",
        "Every GitHub repository this corpus draws samples from, with the licence reported",
        "for it at the revision the samples were pinned to. Generated by",
        "`magika-datasets licenses`; do not edit by hand.",
        "",
        f"{len(rows):,} repositories, "
        f"{sum(1 for r in rows if r['spdx_id']):,} with a resolved licence.",
    ]
    for name, items in _by_license(rows).items():
        lines += [
            "",
            f"## {name}",
            "",
            "| repository | samples | basis |",
            "| --- | --- | --- |",
        ]
        for row in items:
            lines.append(
                f"| [{row['repository']}]({row['url']}) | {row['samples']} | {row['basis']} |"
            )
    return "\n".join(lines) + "\n"


def readme_block(rows):
    """The aggregate table the README carries, between its licence markers."""
    lines = [
        BEGIN,
        "",
        "| licence | repositories | samples | allowed by policy |",
        "| --- | --- | --- | --- |",
    ]
    for name, items in _by_license(rows).items():
        allowed = "yes" if all(r["allowed_by_policy"] for r in items) else "no"
        lines.append(f"| {name} | {len(items)} | {sum(r['samples'] for r in items)} | {allowed} |")
    lines += [
        "",
        f"Full list: [REPOSITORIES.md](REPOSITORIES.md) ({len(rows):,} repositories).",
        "",
        END,
    ]
    return "\n".join(lines)


def update_readme(path, rows):
    """Replace the licence block in place. Returns whether anything changed."""
    path = Path(path)
    text = path.read_text()
    if BEGIN not in text or END not in text:
        raise ValueError(f"README has no {BEGIN} / {END} block to fill")
    head, rest = text.split(BEGIN, 1)
    _, tail = rest.split(END, 1)
    updated = head + readme_block(rows) + tail
    if updated == text:
        return False
    path.write_text(updated)
    return True


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__, prog="magika-datasets licenses")
    parser.add_argument("--sources", type=Path, default=Path("github-used.parquet"))
    parser.add_argument(
        "--policy", type=Path, default=Path("config/repository-license-policy.json")
    )
    parser.add_argument("--output", type=Path, default=Path("repository-licenses.parquet"))
    parser.add_argument("--receipt", type=Path, default=Path("repository-licenses-receipt.json"))
    parser.add_argument("--markdown", type=Path, default=Path("REPOSITORIES.md"))
    parser.add_argument("--readme", type=Path, default=Path("README.md"))
    parser.add_argument(
        "--resolve-residual",
        action="store_true",
        help="Ask GitHub about repositories no local evidence covers. Needs GITHUB_TOKEN.",
    )
    parser.add_argument(
        "--check",
        action="store_true",
        help="Report whether the published markdown is current; write nothing.",
    )
    args = parser.parse_args(argv)
    from .acquisition import sha256_file
    from .github_api import MetadataClient

    # Answers already cached are local evidence, so the default path uses them and makes no
    # request. Without this a plain run silently drops every licence the API had resolved.
    client = MetadataClient(offline=not args.resolve_residual)
    rows, receipt = resolve(args.sources, json.loads(args.policy.read_text()), client)
    rendered = markdown(rows)
    if args.check:
        current = args.markdown.exists() and args.markdown.read_text() == rendered
        block = readme_block(rows) in args.readme.read_text() if args.readme.exists() else False
        receipt["markdown_current"] = current
        receipt["readme_current"] = block
        print(json.dumps(receipt, sort_keys=True))
        return 0 if current and block else 1
    pq.write_table(
        pa.Table.from_pylist(rows, SCHEMA), private_output(args.output), compression="zstd"
    )
    args.markdown.write_text(rendered)
    # A README without the markers is reported rather than silently skipped, so a stale
    # published table cannot hide behind a successful run.
    has_block = args.readme.exists() and BEGIN in args.readme.read_text()
    receipt["readme_updated"] = has_block and update_readme(args.readme, rows)
    receipt["readme_block_present"] = has_block
    receipt["sources_sha256"] = sha256_file(args.sources)
    receipt["policy_sha256"] = sha256_file(args.policy)
    receipt["parquet_sha256"] = sha256_file(args.output)
    write(args.receipt, receipt)
    print(json.dumps(receipt, sort_keys=True))
