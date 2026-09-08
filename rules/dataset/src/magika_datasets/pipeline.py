# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0

"""Resumable candidate collection, observations and hydration. Never reconciles labels."""

import argparse
import fcntl
import json
import os
import shutil
import subprocess
import sys
import tempfile
import time
from collections import defaultdict
from pathlib import Path

import pyarrow.parquet as pq

from .acquisition import acquired_records, sha256_file
from .candidate_conflicts import ConflictSearch, conflict_plan, disagreement
from .candidate_export import export_candidates
from .configuration import validate_build_config
from .observation_cache import seed_journal
from .parquet_corpus import check_sources, pack, verify
from .runtime import implementation_hashes
from .unattended import Runner, read, status, write
from .verdict_export import VerdictIndex
from .verdicts import TOOLS, collect, tool_identity


def progress(root, stage, **values):
    write(
        root / "status.json",
        {
            "phase": stage,
            "pid": os.getpid(),
            "updated_at": time.time(),
            "reconciliation": "manual; never performed by this pipeline",
            **values,
        },
    )


def taxonomy(root):
    rows = []
    for r in pq.read_table(root / "classes.parquet").to_pylist():
        rows.append(
            {
                **json.loads(r["metadata_json"]),
                **{k: r[k] for k in ("format_id", "name", "categories", "extensions")},
                "samples": [],
            }
        )
    by_kind = {r["format_id"]: r for r in rows}
    for batch in pq.ParquetFile(root / "samples.parquet").iter_batches(
        columns=["format_id", "sha256"]
    ):
        for r in batch.to_pylist():
            by_kind[r["format_id"]]["samples"].append({"sha256": r["sha256"].hex()})
    return rows


def observation_priority(record, formats):
    """Known disagreements get detector workers first; every record remains queued."""
    conflicting = any(
        disagreement(origin.get("claim") or {}, formats)
        for origin in record["origins"]
        if origin.get("provider") == "virustotal"
    )
    return (not conflicting, record["sha256"])


def run(root):
    config = read(root / "config.json")
    os.chdir(config["cwd"])
    os.nice(10)
    with (root / "run.lock").open("a") as lock:
        fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
        try:
            for name, expected in config.get("implementation_sha256", {}).items():
                if sha256_file((Path(__file__).parent / name)) != expected:
                    raise ValueError(
                        "Pipeline implementation changed; preserve this run and use an explicit upgrade or new run"
                    )
            collection = Path(config["collection_run"]).resolve()
            if status(collection)["process_alive"]:
                raise ValueError("Collection is still running; resume after its completion")
            collection_config = (
                read(collection / "config.json")
                if (collection / "config.json").exists()
                else read(Path(config["collection_config"]))
            )
            store = Path(collection_config["store"])
            store.mkdir(parents=True, exist_ok=True, mode=0o700)
            baseline = root / "baseline"
            if not (baseline / "complete.json").exists():
                baseline.mkdir(exist_ok=True)
                for name in ("classes.parquet", "samples.parquet", "corpus-parquet-receipt.json"):
                    shutil.copyfile(Path(config["metadata_dir"]) / name, baseline / name)
                check_sources(
                    baseline / "samples.parquet",
                    baseline / "classes.parquet",
                    read(baseline / "corpus-parquet-receipt.json"),
                )
                write(baseline / "complete.json", {"frozen": True})
            check_sources(
                baseline / "samples.parquet",
                baseline / "classes.parquet",
                read(baseline / "corpus-parquet-receipt.json"),
            )
            rows = taxonomy(baseline)
            formats = {r["format_id"]: r for r in rows}
            conflicts = root / "vt-conflicts"
            if not (conflicts / "collection-report.json").exists():
                conflicts.mkdir(exist_ok=True)
                progress(root, "vt_disagreement_collection")
                if not (conflicts / "plan.json").exists():
                    records = list(acquired_records(store))
                    plan = conflict_plan(
                        rows,
                        [read(Path(p)) for p in collection_config["vt_recipes"]],
                        records,
                        per_class=config.get("conflicts_per_class", 100),
                    )
                    write(conflicts / "plan.json", plan)
                plan = read(conflicts / "plan.json")
                with (store / "hydrator.lock").open("a") as writer_lock:
                    fcntl.flock(writer_lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
                    Runner(
                        conflicts,
                        collection_config,
                        plan,
                        search=ConflictSearch(
                            conflicts, plan, formats, config.get("conflict_pages_per_class", 20)
                        ),
                    ).run()
            if not (collection / "collection-report.json").exists():
                progress(root, "collecting")
                command = [
                    sys.executable,
                    "-m",
                    "magika_datasets.unattended",
                    "resume" if (collection / "config.json").exists() else "start",
                    "--run-dir",
                    str(collection),
                    "--foreground",
                ]
                if not (collection / "config.json").exists():
                    command += ["--config", config["collection_config"]]
                subprocess.run(command, check=True)
            discovery = defaultdict(set)
            for path in (collection / "candidates.parquet", conflicts / "candidates.parquet"):
                for batch in pq.ParquetFile(path).iter_batches():
                    for row in batch.to_pylist():
                        discovery[row["sha256"].hex()].update(row["discovery_classes"])
            records = [r for r in acquired_records(store) if r["sha256"] in discovery]
            if len(records) != len(discovery):
                raise ValueError("Candidate pool has unavailable acquired objects")
            records.sort(key=lambda record: observation_priority(record, formats))
            tools = read(Path(config["tools"]))
            if {t["id"] for t in tools} != TOOLS:
                raise ValueError(
                    "Configure all eight tools; missing installations must be explicitly unavailable"
                )
            identities = dict(tool_identity(t) for t in tools)
            identity_path = root / "tool-identities.json"
            if identity_path.exists() and read(identity_path) != identities:
                raise ValueError("Tool binaries/signatures/configuration changed within the run")
            write(identity_path, identities)
            journal = root / "verdicts.jsonl"
            if not (root / "verdicts-complete.json").exists():
                progress(root, "seeding_observation_cache", candidates=len(records))
                paths = [Path(p) for p in config.get("prior_journals", [])]
                for directory in config.get("journal_roots", []):
                    paths += [
                        p.with_suffix("").with_suffix(".jsonl")
                        for p in Path(directory).rglob("*.tools.json")
                        if not p.resolve().is_relative_to(root)
                    ]
                paths = sorted({p.resolve() for p in paths if p.is_file()})
                seed_journal(paths, journal, records, tools)
                result = collect(
                    records,
                    tools,
                    journal,
                    workers=config.get("verdict_workers", 4),
                    progress=lambda value: progress(
                        root, "observing", candidates=len(records), **value
                    ),
                )
                write(root / "verdicts-complete.json", result)
            metadata = root / "metadata"
            if not (metadata / "corpus-parquet-receipt.json").exists():
                progress(root, "exporting_candidate_metadata", candidates=len(records))
                receipt = export_candidates(
                    records, discovery, rows, VerdictIndex([journal]), tools, metadata
                )
            else:
                receipt = read(metadata / "corpus-parquet-receipt.json")
            shutil.copyfile(baseline / "classes.parquet", metadata / "taxonomy.parquet")
            progress(root, "hydrating", candidates=len(records))
            output = root / "snapshot"
            if output.exists():
                result = verify(output, metadata / "samples.parquet", metadata / "classes.parquet")
            else:
                result = pack(
                    metadata / "samples.parquet",
                    metadata / "classes.parquet",
                    receipt,
                    store,
                    output,
                    progress=lambda value: progress(
                        root, "hydrating", candidates=len(records), **value
                    ),
                )
            if result["samples"] != len(records) or not result["verified"]:
                raise ValueError("Candidate hydration incomplete")
            if config.get("public_metadata_dir"):
                destination = Path(config["public_metadata_dir"])
                if not destination.exists():
                    destination.parent.mkdir(parents=True, exist_ok=True)
                    with tempfile.TemporaryDirectory(
                        prefix=destination.name + ".building-", dir=destination.parent
                    ) as directory:
                        temporary = Path(directory) / "metadata"
                        shutil.copytree(metadata, temporary)
                        write(temporary / "hydration-verification.json", result)
                        temporary.replace(destination)
                else:
                    for name in receipt["files"]:
                        if sha256_file(destination / name) != receipt["files"][name]["sha256"]:
                            raise ValueError(
                                "Choose a new public metadata directory; existing output differs"
                            )
            progress(
                root,
                "complete",
                candidates=len(records),
                label_status_counts=receipt["label_status_counts"],
                tool_status_counts=receipt["tool_status_counts"],
                verification=result,
                metadata=str(metadata),
                snapshot=str(output),
                accepted_dataset_changed=False,
            )
        except BaseException as error:
            progress(root, "failed", error=str(error)[:500])
            raise


def pipeline_status(root):
    result = status(root)
    if result.get("phase") == "interrupted" and (root / "status.json").exists():
        saved = read(root / "status.json")
        if saved["phase"] == "complete":
            result["phase"] = "complete"
    if (
        result.get("phase") == "vt_disagreement_collection"
        and (root / "vt-conflicts/status.json").exists()
    ):
        detail = read(root / "vt-conflicts/status.json")
        result["collection"] = {
            k: detail.get(k)
            for k in ("jobs", "jobs_finished", "new_objects", "pending_batches", "providers")
        }
    return result


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__, prog="magika-datasets build")
    parser.add_argument("action", choices=["start", "resume", "status", "_run"])
    parser.add_argument("--run-dir", type=Path, required=True)
    parser.add_argument("--config", type=Path)
    parser.add_argument("--foreground", action="store_true")
    args = parser.parse_args(argv)
    root = args.run_dir.resolve()
    if args.action == "status":
        print(json.dumps(pipeline_status(root), sort_keys=True))
        return
    if args.action == "_run":
        run(root)
        return
    from .runtime import private_output

    private_output(root)
    if args.action == "start":
        if root.exists() or not args.config:
            parser.error("Use a new run directory and --config, or resume an existing run")
        config = read(args.config)
        validate_build_config(config)
        root.mkdir(parents=True)
        write(
            root / "config.json",
            {**config, "cwd": str(Path.cwd()), "implementation_sha256": implementation_hashes()},
        )
    if pipeline_status(root)["process_alive"]:
        print(json.dumps({"already_running": True}))
        return
    if args.foreground:
        run(root)
    else:
        with (root / "process.log").open("a") as log:
            child = subprocess.Popen(
                [sys.executable, "-m", "magika_datasets.pipeline", "_run", "--run-dir", str(root)],
                cwd=read(root / "config.json")["cwd"],
                stdin=subprocess.DEVNULL,
                stdout=log,
                stderr=subprocess.STDOUT,
                start_new_session=True,
            )
        print(json.dumps({"started_pid": child.pid, "run_dir": str(root)}))


if __name__ == "__main__":
    main()
