# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Dataset and tool identities for the benchmark catalogue and derived reports."""

import json
import re
from datetime import UTC, datetime

from .comparison import command_settings, fingerprint, load_json
from .trid import enabled


def utc_timestamp(value):
    stamp = datetime.fromisoformat(value)
    if stamp.tzinfo is None or stamp.utcoffset() is None:
        raise ValueError("Measurement timestamp must include a timezone")
    return stamp.astimezone(UTC).isoformat().replace("+00:00", "Z")


def dataset_record(result, descriptor):
    if not descriptor or any(
        not isinstance(descriptor.get(k), str) or not descriptor[k].strip()
        for k in ["id", "name", "version"]
    ):
        raise ValueError("Dataset id, name and version are required")
    if not re.fullmatch(r"[a-z0-9][a-z0-9._-]*", descriptor["id"]):
        raise ValueError("Dataset id must be a stable lowercase identifier")
    digest = result["compatibility"]["corpus"]
    if descriptor.get("corpus_sha256", digest) != digest:
        raise ValueError("Dataset descriptor names a different corpus snapshot")
    counts = {q["files"] for q in result["quality"].values()}
    if len(counts) != 1:
        raise ValueError("Dataset scoring counts must agree across tools")
    count = counts.pop()
    if descriptor.get("scored_files", count) != count:
        raise ValueError("Dataset scoring count differs from measurements")
    total = descriptor.get("source_files", count)
    if type(total) is not int or total < count:
        raise ValueError("Dataset source size cannot be smaller than its scored subset")
    return descriptor | {"corpus_sha256": digest, "scored_files": count, "source_files": total}


def tool_record(result, tool_id):
    spec = next(t for t in result["config"]["tools"] if t["id"] == tool_id)
    identity = result["tools"][tool_id]
    raw = identity["version"]
    settings = dict(spec.get("settings") or {})
    corrections = []
    adapter = spec["adapter"]
    revision = spec.get("source_revision")
    if adapter == "magika-jsonl":
        name = "Magika"
        match = re.match(r"magika\s+(\S+)", raw)
        if not match:
            raise ValueError("Unrecognized Magika version output")
        software = match[1]
        if not settings.get("release"):
            revision = revision or result["revision"]
        rules = settings.get("rules", "off")
        mode = {"off": "ML", "enforce": "rules + ML", "only": "rules-only"}.get(rules, rules)
        backend = settings.get("backend", "unspecified")
        if rules == "only":
            backend = "CPU"
        elif "GPU" in backend:
            backend = "Metal" if "Metal" in backend else "GPU"
        config = f"{backend}; {mode}"
        if settings.get("startup_backend") == "CPU":
            config += "; CPU warmup"
    elif adapter == "file-mime":
        name = "libmagic"
        match = re.match(r"file-(\S+)", raw)
        if not match:
            raise ValueError("Unrecognized file version output")
        software = match[1]
        config = "MIME; serial"
    elif adapter == "trid":
        name = "TrID"
        match = re.search(r"File Identifier v(\S+)", raw)
        if not match:
            raise ValueError("Unrecognized TrID version output")
        software = match[1]
        observed = enabled(raw)
        verified = identity.get("stringzilla", {}).get("version")
        effective = "off" if observed is False else verified or "unverified"
        if settings.get("stringzilla") != effective:
            corrections.append(
                {
                    "field": "stringzilla",
                    "declared": settings.get("stringzilla"),
                    "observed": effective,
                    "basis": "saved runtime version output and module probe",
                }
            )
        settings["stringzilla"] = effective
        config = (
            f"strings={settings.get('strings')}; StringZilla={settings.get('stringzilla', 'off')}"
        )
    else:
        raise ValueError(f"Unsupported tool adapter: {adapter}")
    version = software + (f" @ {revision[:8]}" if revision else "")
    configuration = {
        "settings": settings,
        "command": spec["command"],
        "environment": result["config"].get("environment", {}),
    }
    config_id = fingerprint(
        {
            "tool": command_settings(spec | {"settings": settings}),
            "environment": configuration["environment"],
        }
    )
    build = {
        "executable_sha256": identity["executable_sha256"],
        "artifacts": identity.get("artifacts", {}),
    }
    return {
        "tool": name,
        "version": version,
        "software_version": software,
        "source_revision": revision,
        "raw_version": raw,
        "config": config,
        "config_id": config_id,
        "configuration": configuration,
        "build_id": fingerprint(build),
        "corrections": corrections,
        **build,
    }


def tool_cells(row):
    identity = row["tool_identity"]
    return [
        identity["tool"],
        identity["version"],
        f"{identity['config']} (`{identity['config_id'][:8]}`)",
    ]


def index_entry(run_id, result, descriptor, receipt_hash):
    return {
        "id": run_id,
        "dataset": dataset_record(result, descriptor),
        "measured_at_utc": utc_timestamp(result["created_at"]),
        "benchmark_version": result["benchmark_version"],
        "revision": result["revision"],
        "status": result["status"],
        "artifacts_sha256": receipt_hash,
    }


def report_metadata(result, descriptor):
    return {
        "dataset_identity": dataset_record(result, descriptor),
        "measured_at_utc": utc_timestamp(result["created_at"]),
        "benchmark_version": result["benchmark_version"],
    }


def source_dataset(source, result):
    if result.get("config", {}).get("dataset"):
        return result["config"]["dataset"]
    index = load_json(source.parent / "index.json")
    return next(r["dataset"] for r in index["runs"] if r["id"] == source.name)


def report_identity_lines(summary):
    d = summary["dataset_identity"]
    lines = [
        f"Dataset: **{d['name']}**, version `{d['version']}`. "
        f"Measured at **{summary['measured_at_utc']}**; benchmark protocol `{summary['benchmark_version']}`.",
        "",
        "<details><summary>Exact tool configurations and build identities</summary>",
        "",
    ]
    for row in summary["rows"]:
        t = row["tool_identity"]
        lines += [
            f"- **{t['tool']} {t['version']}** — config `{t['config_id'][:8]}`, build `{t['build_id']}`: "
            f"`{json.dumps(t['configuration']['settings'], sort_keys=True)}`"
        ]
    lines += [
        "",
        "Full commands, environment, source revisions and executable/artifact hashes are retained in `overview.json`.",
        "",
        "</details>",
        "",
    ]
    if any(r["tool_identity"].get("corrections") for r in summary["rows"]):
        lines += [
            "Configuration correction: TrID acceleration is labeled from saved runtime evidence. "
            "Earlier declared StringZilla settings were inaccurate; the raw records and timings "
            "are preserved unchanged. Correction details are recorded in `overview.json`.",
            "",
        ]
    return lines
