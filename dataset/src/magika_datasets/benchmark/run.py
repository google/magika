# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Run every detector over a snapshot: quality on all scored files, then Hyperfine timings."""

import argparse
import gzip
import json
import os
import random
import re
import shlex
import shutil
from datetime import UTC, datetime
from pathlib import Path

from ..acquisition import sha256_file
from ..runtime import write
from . import metrics, process, snapshot, tools

CHUNK = 128
"""Files per quality invocation; the workloads below are what gets timed."""


def workloads(samples: list[dict], counts: list[int], seed: int) -> list[dict]:
    """Distinct files per invocation, sorted because TrID sorts its inputs anyway."""
    hashes = [s["sha256"] for s in samples]
    cases = []
    for count in counts:
        if count > len(hashes):
            continue  # repeating files would pass a smaller workload off as a larger one
        selected = sorted(process.sample_order(hashes, count, process.trial_seed(seed, count)))
        cases.append(dict(id=f"files-{count}", file_count=count, samples=selected))
    return cases


def check_workload(samples, actual, expected, *, adaptive=False):
    """Timed output must repeat the quality observation.

    A Magika configuration that starts on CPU and moves work to a GPU may make a different
    inference decision under load; that difference is recorded, never a rule decision or an error.
    """
    differences = []
    for sample, row, baseline in zip(samples, actual, expected, strict=True):
        if row["error"]:
            raise ValueError("Workload returned a tool error")
        if row == baseline:
            continue
        if not adaptive or row["deterministic"] or baseline["deterministic"]:
            raise ValueError("Workload outputs differ from quality observations")
        differences.append(
            {
                "sha256": sample["sha256"],
                "quality_observation": baseline,
                "workload_observation": row,
            }
        )
    return metrics.quality(samples, actual), differences


def validate_config(spec: dict) -> None:
    tools.validate_default_policy(spec)
    counts = spec["file_counts"]
    if not counts or any(type(n) is not int or n < 1 for n in counts):
        raise ValueError("File counts must be positive integers")
    if spec["runs"] < 2 or spec["warmup"] < 1:
        raise ValueError("At least two runs and one warmup are required")
    if len({t["id"] for t in spec["tools"]}) != len(spec["tools"]):
        raise ValueError("Duplicate tool id")
    for tool in spec["tools"]:
        tools.Adapter(tool["adapter"])
        if not re.fullmatch(r"[a-z0-9][a-z0-9_-]*", tool["id"]):
            raise ValueError("Tool ids must be safe lowercase names")
        if not tool.get("unavailable") and (
            not tool.get("artifacts") or not tool.get("version_command")
        ):
            raise ValueError(f"{tool['id']}: artifact hashes and a version command are required")


def run(
    spec: dict, dataset: Path, output: Path, revision: str, hyperfine: str, timeout: float
) -> dict:
    validate_config(spec)
    output = output.resolve()
    output.mkdir(parents=True, exist_ok=False)  # a run is never overwritten
    classes, everything = snapshot.prepare(dataset, output)
    samples = sorted((s for s in everything if s["truth"]), key=lambda s: s["sha256"])
    if not samples:
        raise ValueError("The snapshot holds no verified samples to score")
    for directory in ("raw", "file-lists", "home", "tmp"):
        (output / directory).mkdir()
    env = {
        "PATH": os.defpath,
        "HOME": str(output / "home"),
        "TMPDIR": str(output / "tmp"),
        "LC_ALL": "C",
        "LANG": "C",
        "TZ": "UTC",
        **spec.get("environment", {}),
    }
    hyperfine = str(Path(shutil.which(hyperfine) or hyperfine).resolve())
    result = dict(
        schema=1,
        revision=revision,
        created_at=datetime.now(UTC).isoformat(),
        status="in_progress",
        host=process.host_info(output),
        config=spec,
        hyperfine=dict(
            version=process.run_cli([hyperfine, "--version"], env, timeout)[1].decode().strip(),
            sha256=sha256_file(Path(hyperfine)),
        ),
        snapshot=dict(
            manifest_sha256=sha256_file(Path(dataset) / "manifest.json"),
            samples=len(everything),
            scored=len(samples),
        ),
        tools={},
        quality={},
        measurements=[],
        unavailable={},
    )
    relative = {s["sha256"]: "files/" + s["sha256"] for s in samples}
    mappings, observations = {}, {}
    for tool in spec["tools"]:
        if tool.get("unavailable"):
            result["unavailable"][tool["id"]] = tool["unavailable"]
            continue
        result["tools"][tool["id"]] = tools.identify(tool, env, timeout)
        mappings[tool["id"]] = tools.label_mapping(classes, tools.Adapter(tool["adapter"]))
        rows = []
        for start in range(0, len(samples), CHUNK):
            paths = [relative[s["sha256"]] for s in samples[start : start + CHUNK]]
            raw = process.run_cli(
                tools.command(tool, paths, output / "file-lists"), env, timeout, cwd=output
            )[1]
            (output / "raw" / f"{tool['id']}-{start}.txt.gz").write_bytes(
                gzip.compress(raw, mtime=0)
            )
            rows.extend(tools.parse_output(tool["adapter"], raw, paths, mappings[tool["id"]]))
        observations[tool["id"]] = rows
        result["quality"][tool["id"]] = metrics.quality(samples, rows)
        write(output / "results.json", result)
        print(f"Quality: {tool['id']}, {len(rows)} files", flush=True)
    (output / "observations.json.gz").write_bytes(
        gzip.compress(json.dumps(observations, sort_keys=True).encode(), mtime=0)
    )
    write(output / "label-mappings.json", mappings)
    # All-file metrics stay primary; this slice only removes classes a tool cannot name.
    common = set(classes)
    for mapping in mappings.values():
        common &= {names[0] for names in mapping.values() if len(names) == 1}
    indices = [i for i, s in enumerate(samples) if s["truth"] in common]
    result["common_vocabulary"] = dict(
        classes=sorted(common),
        files=len(indices),
        quality={
            tool: metrics.quality([samples[i] for i in indices], [rows[i] for i in indices])
            for tool, rows in observations.items()
        },
    )
    cases = workloads(samples, spec["file_counts"], spec["seed"])
    write(output / "workloads.json", cases)
    by_hash = {s["sha256"]: (i, s) for i, s in enumerate(samples)}
    jobs = [(tool, case) for case in cases for tool in spec["tools"] if tool["id"] in observations]
    random.Random(spec["seed"]).shuffle(jobs)
    for tool, case in jobs:
        paths = [relative[h] for h in case["samples"]]
        argv = tools.command(tool, paths, output / "file-lists")
        # Check the exact workload before timing it: never time a broken or early-exit path.
        raw = process.run_cli(argv, env, timeout, cwd=output)[1]
        name = f"{tool['id']}-{case['id']}"
        (output / "raw" / f"{name}-validation.txt.gz").write_bytes(gzip.compress(raw, mtime=0))
        actual = tools.parse_output(tool["adapter"], raw, paths, mappings[tool["id"]])
        expected = [observations[tool["id"]][by_hash[h][0]] for h in case["samples"]]
        workload_quality, differences = check_workload(
            [by_hash[h][1] for h in case["samples"]],
            actual,
            expected,
            adaptive=tool["adapter"] == tools.Adapter.MAGIKA
            and tool.get("settings", {}).get("startup_backend") == "CPU",
        )
        export = output / "raw" / f"{name}.json"
        process.run_cli(
            [
                hyperfine,
                "--shell=none",
                "--style=none",
                "--runs",
                str(spec["runs"]),
                "--warmup",
                str(spec["warmup"]),
                "--export-json",
                str(export),
                shlex.join(argv),
            ],
            env,
            timeout * (spec["runs"] + spec["warmup"]),
            cwd=output,
        )
        timed = json.loads(export.read_text())["results"]
        if len(timed) != 1:
            raise ValueError("Expected one Hyperfine result per workload")
        result["measurements"].append(
            metrics.timing(timed[0], case["file_count"])
            | dict(
                tool=tool["id"],
                case=case["id"],
                file_count=case["file_count"],
                command=argv,
                workload_quality=workload_quality,
                differences_from_quality=differences,
            )
        )
        write(output / "results.json", result)
        print(f"Timed: {name}", flush=True)
    result["measurements"].sort(key=lambda m: (m["file_count"], m["tool"]))
    for tool in spec["tools"]:
        identity = result["tools"].get(tool["id"])
        if identity and (
            sha256_file(Path(tool["command"][0])) != identity["executable_sha256"]
            or any(
                sha256_file(Path(p)) != identity["artifacts"][r]
                for r, p in tool["artifacts"].items()
            )
        ):
            raise ValueError("Tool artifacts changed during measurement")
    if any(sha256_file(Path(s["path"])) != s["sha256"] for s in samples):
        raise ValueError("Files changed during measurement")
    result["status"] = "partial" if result["unavailable"] else "complete"
    write(output / "results.json", result)
    (output / "report.md").write_text(metrics.render(result))
    return result


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__, prog="magika-datasets benchmark")
    parser.add_argument("--config", type=Path, help="Tool configuration from `benchmark-config`")
    parser.add_argument("--snapshot", type=Path, help="Hydrated snapshot directory")
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--revision", help="Source revision of the tools under test")
    parser.add_argument("--hyperfine", default="hyperfine")
    parser.add_argument("--timeout", type=float, default=120)
    parser.add_argument("--render", type=Path, help="Render a saved results.json; run nothing")
    args = parser.parse_args(argv)
    if args.render:
        args.output.write_text(metrics.render(json.loads(args.render.read_text())))
        return
    if not (args.config and args.snapshot and args.revision):
        parser.error("--config, --snapshot and --revision are required to measure")
    run(
        json.loads(args.config.read_text()),
        args.snapshot,
        args.output,
        args.revision,
        args.hyperfine,
        args.timeout,
    )
