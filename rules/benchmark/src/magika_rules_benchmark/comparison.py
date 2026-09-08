# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0

"""Versioned cross-tool observations and Hyperfine JSON; deterministic reports."""

import argparse
import gzip
import hashlib
import json
import os
import random
import re
import shlex
import shutil
import statistics
from collections import defaultdict
from datetime import UTC, datetime
from enum import StrEnum
from pathlib import Path

from tabulate import tabulate

from . import corpus, runner

VERSION = "1.0.0"


class Adapter(StrEnum):
    MAGIKA = "magika-jsonl"
    FILE = "file-mime"
    TRID = "trid"


def fingerprint(value):
    return hashlib.sha256(
        json.dumps(value, sort_keys=True, separators=(",", ":"), allow_nan=False).encode()
    ).hexdigest()


def label_mapping(classes, adapter):
    aliases = defaultdict(set)
    for name, row in classes.items():
        if adapter == Adapter.MAGIKA:
            meta = row.get("magika", {})
            labels = [name, *meta.get("output_labels", []), *meta.get("kb_labels", [])]
        elif adapter == Adapter.FILE:
            labels = row.get("mimes", [])
        else:
            labels = [x.lower().lstrip(".") for x in row.get("extensions", [])]
        for label in labels:
            if not isinstance(label, str):
                raise ValueError("Class aliases must be strings")
            aliases[label].add(name)
    return {key: sorted(value) for key, value in sorted(aliases.items())}


def mapped(raw, aliases, *, abstain=False, tie=False, error=None, deterministic=None):
    candidates = sorted({name for label in raw for name in aliases.get(label, [])})
    state = (
        "error"
        if error
        else "abstained"
        if abstain
        else "ambiguous"
        if tie or len(candidates) > 1
        else "mapped"
        if candidates
        else "unmapped"
    )
    return dict(
        raw_labels=raw,
        candidates=candidates,
        mapping=state,
        prediction=candidates[0] if state == "mapped" else None,
        error=error,
        deterministic=deterministic,
    )


def parse_output(adapter, raw, paths, aliases):
    adapter = Adapter(adapter)
    if adapter == Adapter.MAGIKA:
        values = [json.loads(line) for line in raw.splitlines()]
        if [v["path"] for v in values] != list(paths):
            raise ValueError("Magika output paths are missing or reordered")
        rows = []
        for value in values:
            result = value["result"]
            if result["status"] != "ok":
                rows.append(mapped([], aliases, error=result["status"]))
                continue
            value = result["value"]
            label = value["output"]["label"]
            rows.append(
                mapped(
                    [label],
                    aliases,
                    abstain=label in ("unknown", "undefined"),
                    deterministic=runner.deterministic(value),
                )
            )
        return rows
    if adapter == Adapter.FILE:
        if b"\0" in raw:
            # file -0 -0 delimits both names and complete descriptions, including
            # Apple's multiline fat-binary output. Score the top-level MIME;
            # architecture details stay in the saved raw output.
            fields = raw.decode().split("\0")
            if fields[-1] != "" or fields[:-1:2] != list(paths):
                raise ValueError("file output paths are missing or reordered")
            labels = [field.splitlines()[0] for field in fields[1:-1:2]]
            return [
                mapped(
                    [label],
                    aliases,
                    abstain=label == "application/octet-stream",
                    error=label if label.startswith("ERROR:") else None,
                )
                for label in labels
            ]
        lines = raw.decode().splitlines()
        rows, index = [], 0
        for path in paths:
            if index >= len(lines):
                raise ValueError("file output count differs from input count")
            labels = [lines[index]]
            index += 1
            # Apple file emits architecture continuations even with --brief.
            continuation = re.compile(re.escape(path) + r" \(for architecture .+\):\s*(.*)")
            while index < len(lines) and (match := continuation.fullmatch(lines[index])):
                labels.append(match[1])
                index += 1
            labels = sorted(set(labels))
            rows.append(
                mapped(
                    labels,
                    aliases,
                    abstain=labels == ["application/octet-stream"],
                    error=next((s for s in labels if s.startswith("ERROR:")), None),
                )
            )
        if index != len(lines):
            raise ValueError("file output count differs from input count")
        return rows
    groups = re.split(r"(?m)^File: (.+)\r?$", raw.decode())
    if groups[1::2] != list(paths):
        raise ValueError("TrID output paths are missing or reordered")
    rows = []
    for group in groups[2::2]:
        matches = re.findall(r"(?m)^\s*([\d.]+)% \(\.([^)]*)\)", group)
        if not matches:
            if "Unknown!" not in group:
                raise ValueError("TrID output has neither predictions nor an abstention")
            rows.append(mapped([], aliases, abstain=True))
        else:
            # Never select a lower-ranked candidate because it happens to match truth.
            score, extensions = matches[0]
            tie = len(matches) > 1 and matches[1][0] == score
            rows.append(
                mapped(extensions.lower().split("/"), aliases, tie=tie, abstain=not extensions)
            )
    return rows


def quality_metrics(samples, rows):
    per_class = defaultdict(lambda: dict(files=0, correct=0, decisions=0, errors=0))
    counts = dict(
        files=len(samples),
        correct=0,
        wrong=0,
        decisions=0,
        errors=0,
        ambiguous=0,
        unmapped=0,
        abstained=0,
        native_identifications=0,
    )
    for sample, row in zip(samples, rows, strict=True):
        truth, prediction = sample["truth"], row["prediction"]
        counts["correct"] += prediction == truth
        counts["wrong"] += prediction is not None and prediction != truth
        counts["decisions"] += prediction is not None
        counts["errors"] += bool(row["error"])
        for state in ("ambiguous", "unmapped", "abstained"):
            counts[state] += row.get("mapping") == state
        counts["native_identifications"] += bool(row.get("raw_labels")) and row.get(
            "mapping"
        ) not in ("abstained", "error")
        c = per_class[truth]
        c["files"] += 1
        c["correct"] += prediction == truth
        c["decisions"] += prediction is not None
        c["errors"] += bool(row["error"])
    n = counts["files"]
    return counts | dict(
        accuracy=counts["correct"] / n if n else None,
        decision_coverage=counts["decisions"] / n if n else None,
        precision=counts["correct"] / counts["decisions"] if counts["decisions"] else None,
        macro_recall=statistics.mean(v["correct"] / v["files"] for v in per_class.values())
        if per_class
        else None,
        formats_with_correct_decision=sum(v["correct"] > 0 for v in per_class.values()),
        formats=len(per_class),
        per_class=dict(sorted(per_class.items())),
    )


def make_workloads(samples, hits, counts, rates, seed):
    hashes = [s["sha256"] for s in samples]
    hit_pool = [h for h in hashes if h in hits]
    miss_pool = [h for h in hashes if h not in hits]
    cases = []
    for count in counts:
        for rate in [None, *rates]:
            if rate is not None and (
                count * rate % 100 or (rate and not hit_pool) or (rate < 100 and not miss_pool)
            ):
                continue
            case_seed = runner.trial_seed(seed, count, rate)
            if rate is None:
                selected = runner.sample_order(hashes, count, case_seed)
            else:
                n = count * rate // 100
                selected = runner.sample_order(hit_pool, n, case_seed)
                selected += runner.sample_order(miss_pool, count - n, case_seed + 1)
                random.Random(case_seed).shuffle(selected)
            # TrID deduplicates and sorts filenames. Use that order for every tool,
            # and never present repeated files as a larger workload.
            if len(set(selected)) != count:
                continue
            selected.sort()
            cases.append(
                dict(
                    id=f"files-{count}-hits-{rate if rate is not None else 'natural'}",
                    file_count=count,
                    requested_rule_hit_percent=rate,
                    samples=selected,
                    distinct_files=len(set(selected)),
                )
            )
    return cases


def timing_summary(result, count):
    times, exits = result["times"], result["exit_codes"]
    if not times or len(exits) != len(times) or any(exits):
        raise ValueError("Hyperfine trials have missing data or nonzero exit codes")
    if any(t <= 0 for t in times):
        raise ValueError("Hyperfine timing must be positive")
    median = statistics.median(times)
    return dict(
        trials=len(times),
        median_seconds=median,
        min_seconds=min(times),
        max_seconds=max(times),
        files_per_second=count / median,
    )


def compare_previous(current, previous):
    reasons = []
    if current["benchmark_version"] != previous["benchmark_version"]:
        reasons.append("benchmark_version")
    for key in current["compatibility"].keys() | previous["compatibility"].keys():
        if key in ("settings", "mappings"):
            a, b = current["compatibility"].get(key, {}), previous["compatibility"].get(key, {})
            if any(a[tool] != b[tool] for tool in a.keys() & b.keys()):
                reasons.append(key)
            continue
        if current["compatibility"].get(key) != previous["compatibility"].get(key):
            reasons.append(key)
    if reasons:
        return dict(comparable=False, reasons=sorted(reasons), deltas=[])
    old = {(m["tool"], m["case"]): m for m in previous["measurements"]}
    deltas = []
    for measurement in current["measurements"]:
        before = old.get((measurement["tool"], measurement["case"]))
        if before:
            a, b = before["median_seconds"], measurement["median_seconds"]
            deltas.append(
                dict(
                    tool=measurement["tool"],
                    case=measurement["case"],
                    before_seconds=a,
                    after_seconds=b,
                    speedup=a / b,
                    elapsed_change_percent=100 * (b / a - 1),
                )
            )
    quality_deltas = []
    for tool, after in current.get("quality", {}).items():
        before = previous.get("quality", {}).get(tool)
        if before:
            quality_deltas.append(
                dict(
                    tool=tool,
                    **{
                        key: after[key] - before[key]
                        if after.get(key) is not None and before.get(key) is not None
                        else None
                        for key in (
                            "accuracy",
                            "decision_coverage",
                            "macro_recall",
                            "rule_hit_percent",
                        )
                    },
                )
            )
    return dict(comparable=True, reasons=[], deltas=deltas, quality_deltas=quality_deltas)


def command_settings(tool):
    """Compare actual flags, permitting artifact paths/revisions to move."""
    aliases = {str(path): f"<{role}>" for role, path in tool.get("artifacts", {}).items()}
    args = tool.get("command", [])
    return {
        "id": tool["id"],
        "adapter": tool["adapter"],
        "argv": ["<executable>", *[aliases.get(arg, arg) for arg in args[1:]]],
        "settings": tool.get("settings"),
    }


def render(result):
    def percent(value):
        return "—" if value is None else f"{100 * value:.2f}%"

    quality = []
    for tool, value in result["quality"].items():
        quality.append(
            [
                tool,
                value["files"],
                percent(value["accuracy"]),
                percent(value["precision"]),
                percent(value["decision_coverage"]),
                percent(value["macro_recall"]),
                value["wrong"],
                value["errors"],
                value["ambiguous"],
                value["unmapped"],
                value.get("rule_hit_percent"),
                f"{value['formats_with_correct_decision']}/{value['formats']}",
            ]
        )
    timings = [
        [
            m["tool"],
            m["file_count"],
            m["requested_rule_hit_percent"],
            m["observed_rule_hit_percent"],
            round(1000 * m["median_seconds"], 3),
            round(m["files_per_second"], 1),
            m["trials"],
        ]
        for m in result["measurements"]
    ]
    text = f"# Cross-tool benchmark {result['benchmark_version']}\n\n"
    text += f"Revision: `{result['revision']}`. Status: {result['status']}.\n\n"
    text += "Whole-process wall time, warm OS/tool caches, shell disabled. One file includes startup; large workloads amortize it. No isolated model-initialization or cold-disk claim.\n\n"
    text += (
        tabulate(
            quality,
            headers=[
                "Tool",
                "Files",
                "Accuracy",
                "Precision",
                "Mapped coverage",
                "Macro recall",
                "Wrong",
                "Errors",
                "Ambiguous",
                "Unmapped",
                "Rule hit %",
                "Formats correct",
            ],
            tablefmt="github",
        )
        + "\n\n"
    )
    text += (
        tabulate(
            timings,
            headers=[
                "Tool",
                "Files",
                "Requested rule %",
                "Observed rule %",
                "Median ms",
                "Files/s",
                "Trials",
            ],
            tablefmt="github",
        )
        + "\n\n"
    )
    text += "Rule ratios refer to the named Magika rules reference versus its ML-only control. A dash is the natural sample mix or a tool without a rules mode.\n\n"
    if result.get("common_vocabulary"):
        common = result["common_vocabulary"]
        text += f"Common unambiguous vocabulary: {len(common['classes'])} classes, {common['files']} files. Selected from class metadata and model support, before scoring; this is not proof of each tool's format support.\n\n"
        text += (
            tabulate(
                [
                    [
                        tool,
                        percent(q["accuracy"]),
                        percent(q["decision_coverage"]),
                        percent(q["macro_recall"]),
                    ]
                    for tool, q in common["quality"].items()
                ],
                headers=["Tool", "Common-vocabulary accuracy", "Mapped coverage", "Macro recall"],
                tablefmt="github",
            )
            + "\n\n"
        )
    if result.get("unavailable"):
        text += (
            tabulate(
                sorted(result["unavailable"].items()),
                headers=["Unavailable tool", "Reason"],
                tablefmt="github",
            )
            + "\n\n"
        )
    if result.get("previous"):
        p = result["previous"]
        text += "Previous run comparable: " + str(p["comparable"]) + ".\n\n"
        if p["reasons"]:
            text += "Different comparison keys: " + ", ".join(p["reasons"]) + ".\n\n"
        if p["deltas"]:
            text += (
                tabulate(
                    [
                        [
                            d["tool"],
                            d["case"],
                            round(d["speedup"], 4),
                            round(d["elapsed_change_percent"], 2),
                        ]
                        for d in p["deltas"]
                    ],
                    headers=["Tool", "Workload", "Before / after", "Elapsed change %"],
                    tablefmt="github",
                )
                + "\n"
            )
        if p.get("quality_deltas"):
            text += (
                "\n"
                + tabulate(
                    [
                        [
                            d["tool"],
                            *[
                                None if d[k] is None else round(d[k] * scale, 4)
                                for k, scale in (
                                    ("accuracy", 100),
                                    ("decision_coverage", 100),
                                    ("macro_recall", 100),
                                    ("rule_hit_percent", 1),
                                )
                            ],
                        ]
                        for d in p["quality_deltas"]
                    ],
                    headers=[
                        "Tool",
                        "Accuracy Δ pp",
                        "Coverage Δ pp",
                        "Macro recall Δ pp",
                        "Rule hits Δ pp",
                    ],
                    tablefmt="github",
                )
                + "\n"
            )
    return text


def load_json(path):
    path = Path(path)
    return json.loads(
        gzip.decompress(path.read_bytes()) if path.suffix == ".gz" else path.read_bytes()
    )


def input_identity(source):
    return fingerprint(
        dict(
            classes=source["classes"],
            samples=sorted((s["sha256"], s["size"], s["truth"]) for s in source["samples"]),
        )
    )


def save_gzip(path, data):
    path.write_bytes(
        gzip.compress(json.dumps(data, sort_keys=True, allow_nan=False).encode(), mtime=0)
    )


def invoke(command, env, timeout, cwd=None):
    # Reuse the existing process isolation, timeout and process-group cleanup.
    return runner.run_cli(command, env, timeout, cwd=cwd)[1]


def tool_command(tool, paths, directory):
    if tool["adapter"] == Adapter.TRID:
        listing = directory / (fingerprint(paths) + ".txt")
        listing.write_text("\n".join(paths) + "\n")
        return [*tool["command"], "-f", str(listing)]
    return [*tool["command"], "--", *paths]


def run(args):
    spec = load_json(args.config)
    if spec["schema"] != 1:
        raise ValueError("Unsupported comparison config schema")
    counts, rates = spec["file_counts"], spec["rule_hit_percentages"]
    if (
        not counts
        or any(type(n) is not int or n < 1 for n in counts)
        or any(type(n) is not int or not 0 <= n <= 100 for n in rates)
        or spec["runs"] < 2
        or spec["warmup"] < 1
        or spec.get("quality_chunk_files", 128) < 1
        or args.timeout <= 0
    ):
        raise ValueError("Positive file counts, >=2 runs, >=1 warmup and 0..100 rates required")
    tools = spec["tools"]
    if len({t["id"] for t in tools}) != len(tools):
        raise ValueError("Duplicate tool id")
    for tool in tools:
        Adapter(tool["adapter"])
        if not re.fullmatch(r"[a-z0-9][a-z0-9_-]*", tool["id"]):
            raise ValueError("Tool ids must be safe lowercase names")
    source = load_json(args.inputs)
    samples = sorted(source["samples"], key=lambda s: s["sha256"])
    if not samples or any(not s.get("truth") or s.get("ambiguous") for s in samples):
        raise ValueError("Supply an explicitly adjudicated, unambiguous input snapshot")
    if len({s["sha256"] for s in samples}) != len(samples):
        raise ValueError("Duplicate sample hashes")
    for sample in samples:
        path = args.files_root / sample["sha256"] if args.files_root else Path(sample["path"])
        sample["path"] = str(path.resolve())
        if path.stat().st_size != sample["size"] or corpus.file_hash(path) != sample["sha256"]:
            raise ValueError("Sample bytes do not match the input snapshot")
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)  # Each revision/run is immutable.
    (output / "files").mkdir()
    for sample in samples:
        target = output / "files" / sample["sha256"]
        try:
            os.link(sample["path"], target)
        except OSError:
            shutil.copyfile(sample["path"], target)
        sample["path"] = str(target)
    (output / "raw").mkdir()
    lists = output / "file-lists"
    lists.mkdir()
    corpus.atomic_json(output / "config.json", spec)
    portable = {k: v for k, v in source.items() if k != "samples"}
    portable["samples"] = [{k: s[k] for k in ("sha256", "size", "truth")} for s in samples]
    save_gzip(output / "inputs.json.gz", portable)
    env = {
        "PATH": os.defpath,
        "HOME": str(output / "home"),
        "TMPDIR": str(output / "tmp"),
        "LC_ALL": "C",
        "LANG": "C",
        "TZ": "UTC",
        **spec.get("environment", {}),
    }
    Path(env["HOME"]).mkdir()
    Path(env["TMPDIR"]).mkdir()
    env["MAGIKA_RULES_CACHE"] = str(output / "rule-cache")
    hyperfine = str(Path(shutil.which(args.hyperfine) or args.hyperfine).resolve())
    hyperfine_id = dict(
        version=invoke([hyperfine, "--version"], env, args.timeout).decode().strip(),
        sha256=corpus.file_hash(hyperfine),
    )
    host = runner.host_info(output)
    stable_host = {k: v for k, v in host.items() if k != "filesystem_available_bytes"}
    result = dict(
        benchmark_version=VERSION,
        schema=1,
        revision=args.revision,
        created_at=datetime.now(UTC).isoformat(),
        status="in_progress",
        host=host,
        config=spec,
        hyperfine=hyperfine_id,
        tools={},
        quality={},
        measurements=[],
        unavailable={},
        compatibility={},
        source_identity=source.get("identity"),
    )
    mappings, observations = {}, {}
    reuse = None
    if args.reuse_quality:
        reuse = load_json(args.reuse_quality / "results.json")
        old_inputs = load_json(args.reuse_quality / "inputs.json.gz")
        if input_identity(old_inputs) != input_identity(source):
            raise ValueError("Cannot reuse quality output from changed inputs or labels")
        for key in ("environment", "quality_chunk_files"):
            if reuse["config"].get(key) != spec.get(key):
                raise ValueError("Cannot reuse quality output with changed execution settings")
        result["reused_quality"] = {
            "source_results_sha256": corpus.file_hash(args.reuse_quality / "results.json"),
            "raw_sha256": {},
        }
    for tool in tools:
        tool_id = tool["id"]
        if tool.get("unavailable"):
            result["unavailable"][tool_id] = tool["unavailable"]
            continue
        if not tool.get("artifacts") or not tool.get("version_command"):
            raise ValueError("Every tool requires artifact hashes and a version command")
        executable = str(Path(shutil.which(tool["command"][0]) or tool["command"][0]).resolve())
        tool["command"][0] = executable
        identity = dict(
            version=invoke(tool["version_command"], env, args.timeout).decode().strip(),
            executable_sha256=corpus.file_hash(executable),
            artifacts={role: corpus.file_hash(path) for role, path in tool["artifacts"].items()},
            command=tool["command"],
        )
        result["tools"][tool_id] = identity
        mapping = label_mapping(source["classes"], Adapter(tool["adapter"]))
        mappings[tool_id] = mapping
        rows = []
        for start in range(0, len(samples), spec.get("quality_chunk_files", 128)):
            group = samples[start : start + spec.get("quality_chunk_files", 128)]
            paths = ["files/" + s["sha256"] for s in group]
            command = tool_command(tool, paths, lists)
            cached = args.reuse_quality / "raw" / f"{tool_id}-{start}.txt.gz" if reuse else None
            if cached and cached.exists() and reuse["tools"].get(tool_id) == identity:
                raw = gzip.decompress(cached.read_bytes())
                result["reused_quality"]["raw_sha256"][cached.name] = corpus.file_hash(cached)
            else:
                raw = invoke(command, env, args.timeout, cwd=output)
            (output / "raw" / f"{tool_id}-{start}.txt.gz").write_bytes(gzip.compress(raw, mtime=0))
            rows.extend(parse_output(Adapter(tool["adapter"]), raw, paths, mapping))
        observations[tool_id] = rows
        result["quality"][tool_id] = quality_metrics(samples, rows)
        print(f"Quality: {tool_id}, {len(rows)} files", flush=True)
        corpus.atomic_json(output / "results.json", result)
    save_gzip(output / "observations.json.gz", observations)
    corpus.atomic_json(output / "label-mappings.json", mappings)
    # This vocabulary intersection is independent of successful predictions.
    # All-file metrics remain the primary denominator.
    common = set(source.get("supported", source["classes"]))
    for mapping in mappings.values():
        common &= {names[0] for names in mapping.values() if len(names) == 1}
    indices = [i for i, sample in enumerate(samples) if sample["truth"] in common]
    result["common_vocabulary"] = dict(
        classes=sorted(common),
        files=len(indices),
        quality={
            tool: quality_metrics([samples[i] for i in indices], [rows[i] for i in indices])
            for tool, rows in observations.items()
        },
    )
    ref, ml = spec["rules_reference"], spec["ml_reference"]
    rule_hits = {
        s["sha256"]
        for s, a, b in zip(samples, observations[ref], observations[ml], strict=True)
        if a["deterministic"] and not b["deterministic"] and not a["error"]
    }
    for tool_id in observations:
        result["quality"][tool_id]["rule_hit_percent"] = (
            100 * len(rule_hits) / len(samples) if tool_id == ref else None
        )
    cases = (
        load_json(args.workloads)
        if args.workloads
        else make_workloads(samples, rule_hits, counts, rates, spec["seed"])
    )
    by_hash = {s["sha256"]: s for s in samples}
    row_indices = {s["sha256"]: i for i, s in enumerate(samples)}
    for case in cases:
        if (
            len(case["samples"]) != case["file_count"]
            or case["samples"] != sorted(set(case["samples"]))
        ) or any(h not in by_hash for h in case["samples"]):
            raise ValueError("Saved workload differs from supplied inputs")
    if len({case["id"] for case in cases}) != len(cases):
        raise ValueError("Duplicate workload id")
    corpus.atomic_json(output / "workloads.json", cases)
    protocol = {k: v for k, v in spec.items() if k not in ("tools", "environment")}
    result["compatibility"] = dict(
        host=fingerprint(stable_host),
        protocol=fingerprint(protocol),
        corpus=fingerprint([(s["sha256"], s["size"], s["truth"]) for s in samples]),
        mappings={tool: fingerprint(mapping) for tool, mapping in mappings.items()},
        workloads=fingerprint(cases),
        hyperfine=fingerprint(hyperfine_id),
        environment=fingerprint(spec.get("environment", {})),
        settings={t["id"]: fingerprint(command_settings(t)) for t in tools},
        benchmark_code=fingerprint(
            {p.name: corpus.file_hash(p) for p in sorted(Path(__file__).parent.glob("*.py"))}
        ),
    )
    jobs = [(tool, case) for case in cases for tool in tools if tool["id"] in observations]
    random.Random(spec["seed"]).shuffle(jobs)
    for tool, case in jobs:
        tool_id = tool["id"]
        paths = ["files/" + h for h in case["samples"]]
        command = tool_command(tool, paths, lists)
        # Verify each exact workload before timing; never time a broken/early-exit path.
        actual = parse_output(
            Adapter(tool["adapter"]),
            invoke(command, env, args.timeout, cwd=output),
            paths,
            mappings[tool_id],
        )
        expected = [observations[tool_id][row_indices[h]] for h in case["samples"]]
        if actual != expected or any(row["error"] for row in actual):
            raise ValueError(f"{tool_id}: workload outputs differ from quality observations")
        filename = f"{tool_id}-{case['id']}.json"
        export = output / "raw" / filename
        invocation = [
            hyperfine,
            "--shell=none",
            "--style=none",
            "--runs",
            str(spec["runs"]),
            "--warmup",
            str(spec["warmup"]),
            "--export-json",
            str(export),
            shlex.join(command),
        ]
        invoke(invocation, env, args.timeout * (spec["runs"] + spec["warmup"]), cwd=output)
        timed = load_json(export)["results"]
        if len(timed) != 1:
            raise ValueError("Expected one Hyperfine result per workload")
        measurement = timing_summary(timed[0], case["file_count"])
        measurement.update(
            tool=tool_id,
            case=case["id"],
            file_count=case["file_count"],
            requested_rule_hit_percent=case["requested_rule_hit_percent"],
            observed_rule_hit_percent=100
            * sum(h in rule_hits for h in case["samples"])
            / case["file_count"]
            if tool_id == ref
            else None,
            raw=f"raw/{filename}",
            command=command,
            input_order_sha256=fingerprint(case["samples"]),
            distinct_files=len(set(case["samples"])),
        )
        result["measurements"].append(measurement)
        corpus.atomic_json(output / "results.json", result)
        print(f"Timed: {tool_id}, {case['id']}", flush=True)
    result["measurements"].sort(key=lambda m: (m["file_count"], m["case"], m["tool"]))
    for tool in tools:
        if tool["id"] not in result["tools"]:
            continue
        identity = result["tools"][tool["id"]]
        if corpus.file_hash(tool["command"][0]) != identity["executable_sha256"] or any(
            corpus.file_hash(path) != identity["artifacts"][role]
            for role, path in tool["artifacts"].items()
        ):
            raise ValueError("Tool artifacts changed during measurement")
    for sample in samples:
        if corpus.file_hash(sample["path"]) != sample["sha256"]:
            raise ValueError("Corpus changed during measurement")
    result["status"] = "complete" if not result["unavailable"] else "partial"
    if args.previous:
        previous = load_json(args.previous)
        result["previous"] = compare_previous(result, previous) | {
            "sha256": corpus.file_hash(args.previous)
        }
    corpus.atomic_json(output / "results.json", result)
    (output / "report.md").write_text(render(result))
    return result


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--config", type=Path)
    parser.add_argument("--inputs", type=Path)
    parser.add_argument("--files-root", type=Path)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--revision")
    parser.add_argument("--hyperfine", default="hyperfine")
    parser.add_argument("--workloads", type=Path)
    parser.add_argument("--previous", type=Path)
    parser.add_argument(
        "--reuse-quality",
        type=Path,
        help="Reuse matching raw quality stdout from an earlier run; never timing",
    )
    parser.add_argument("--timeout", type=float, default=120)
    parser.add_argument("--render", type=Path, help="Render saved JSON only; run no tools")
    args = parser.parse_args(argv)
    if args.render:
        result = load_json(args.render)
        if args.previous:
            result["previous"] = compare_previous(result, load_json(args.previous))
        args.output.write_text(render(result))
    else:
        if not args.config or not args.inputs or not args.revision:
            parser.error("--config, --inputs and --revision are required for measurements")
        run(args)


if __name__ == "__main__":
    main()
