# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0

"""Run the real product and controlled disk workloads through one subprocess path."""

import hashlib
import json
import os
import platform
import plistlib
import random
import re
import signal
import subprocess
import sys
import time
from pathlib import Path

from .corpus import atomic_json, file_hash

DEFAULT_COUNTS = [*range(1, 51), *range(60, 101, 10), 500, 1000]


def command(binary, pack, mode, paths, backend="cpu", workers=4, batch_size=8):
    args = [
        str(binary),
        "--jsonl",
        f"--backend={backend}",
        f"--batch-size={batch_size}",
        f"--readers={workers}",
        f"--threads={workers}",
        "--rules=enforce" if mode == "hybrid" else "--rules=off",
    ]
    if mode == "hybrid":
        args += ["--rules-file", str(pack)]
    return args + ["--", *map(str, paths)]


def observe(binary, pack, records, canonical, env, backend="cpu", timeout=60):
    import yara_x

    compiler = yara_x.Compiler(includes_enabled=False)
    compiler.define_global("original_size", 0)
    compiler.define_global("prefix_size", 0)
    compiler.add_source(pack.read_text())
    compiled = compiler.build()
    rules = {}
    for rule in compiled:
        meta = dict(rule.metadata)
        if "label" in meta:
            rules[rule.identifier] = meta | {
                "format_id": canonical.get(meta["label"], meta["label"])
            }
    scanner = yara_x.Scanner(compiled)
    scanner.set_timeout(timeout)
    rows = []
    for start in range(0, len(records), 128):
        batch = records[start : start + 128]
        paths = [row["path"] for row in batch]
        outputs = {}
        for mode in ("ml", "hybrid"):
            _, raw = run_cli(command(binary, pack, mode, paths, backend), env, timeout)
            outputs[mode] = predictions(raw, paths)
        for original, ml, hybrid in zip(batch, outputs["ml"], outputs["hybrid"], strict=True):
            row = original.copy()
            with Path(row["path"]).open("rb") as stream:
                prefix = stream.read(4096)
            scanner.set_global("original_size", row["size"])
            scanner.set_global("prefix_size", len(prefix))
            matches, error = [], None
            try:
                matches = [
                    r.identifier
                    for r in scanner.scan(prefix).matching_rules
                    if r.identifier in rules
                ]
            except (yara_x.ScanError, yara_x.TimeoutError) as failure:
                error = type(failure).__name__
            labels = {
                rules[name]["format_id"]
                for name in matches
                if rules[name].get("enforced", rules[name].get("enabled", False))
            }
            predicted = next(iter(labels)) if len(labels) == 1 and error is None else None
            row.update(
                raw_matches=matches,
                rule_prediction=predicted,
                conflict=len(labels) > 1,
                reference_error=error,
                product_rule=deterministic(hybrid) and not deterministic(ml),
            )
            for mode, value in (("ml", ml), ("hybrid", hybrid)):
                row[f"{mode}_prediction"] = canonical.get(
                    value["output"]["label"], value["output"]["label"]
                )
                row[f"{mode}_score"] = value["score"]
            row["reference_mismatch"] = error is None and (
                (row["hybrid_prediction"] != predicted or not deterministic(hybrid))
                if predicted
                else without_scores(ml) != without_scores(hybrid)
            )
            row["ml_deterministic"] = deterministic(ml)
            rows.append(row)
        print(f"Classified {len(rows)}/{len(records)} files", flush=True)
    return rows, rules


def matrix(
    binary,
    pack,
    rows,
    output,
    env,
    *,
    counts,
    rates,
    workers,
    backends,
    repeats=3,
    seed=20260906,
    timeout=60,
    resume=False,
):
    for row in rows:
        if file_hash(Path(row["path"])) != row["sha256"]:
            raise ValueError("Materialized input changed since correctness evaluation")
    fingerprint = dict(
        binary=file_hash(binary),
        rules=file_hash(pack),
        code=file_hash(Path(__file__)),
        inputs=digest(
            json.dumps(
                [(r["sha256"], r["product_rule"], r["ml_deterministic"]) for r in rows]
            ).encode()
        ),
    )
    native = env.get("MAGIKA_VECTORSCAN_LIBRARY")
    fingerprint["native"] = file_hash(native) if native else "system-discovered"
    config = dict(
        counts=counts, rates=rates, workers=workers, backends=backends, repeats=repeats, seed=seed
    )
    host = host_info(Path(rows[0]["path"]).parent)
    report = dict(
        config=config,
        fingerprints=fingerprint,
        host=host,
        measurements=[],
        skipped=[],
        status="in_progress",
        read_baseline=read_baseline([Path(r["path"]) for r in rows]),
    )
    if resume and output.exists():
        prior = json.loads(output.read_text())

        def stable_host(value):
            return {k: v for k, v in value.items() if k != "filesystem_available_bytes"}

        if (
            prior["config"] != config
            or prior["fingerprints"] != fingerprint
            or stable_host(prior["host"]) != stable_host(host)
        ):
            raise ValueError("Resume configuration, inputs, code or hardware changed")
        report = prior
    hits = [r["path"] for r in rows if r["product_rule"] and not r["ml_deterministic"]]
    misses = [r["path"] for r in rows if not r["product_rule"] and not r["ml_deterministic"]]
    report["pools"] = dict(
        hits=len(hits), misses=len(misses), excluded=len(rows) - len(hits) - len(misses)
    )
    for backend in backends:
        warmup = (hits[:8] + misses[:8]) or [rows[0]["path"]]
        for mode in ("ml", "hybrid"):
            _, raw = run_cli(
                command(binary, pack, mode, warmup, backend, min(workers)), env, timeout
            )
            predictions(raw, warmup)
        for count in counts:
            for rate in rates:
                if count * rate % 100 or (rate and not hits) or (rate < 100 and not misses):
                    skipped = dict(backend=backend, files=count, hit_percent=rate)
                    if skipped not in report["skipped"]:
                        report["skipped"].append(skipped)
                    continue
                if any(
                    r["backend"] == backend and r["files"] == count and r["hit_percent"] == rate
                    for r in report["measurements"]
                ):
                    continue
                cells = {
                    (n, mode): dict(
                        backend=backend,
                        files=count,
                        hit_percent=rate,
                        workers=n,
                        mode=mode,
                        trials=[],
                    )
                    for n in workers
                    for mode in ("ml", "hybrid")
                }
                for trial in range(repeats):
                    selected = exact_mix(hits, misses, count, rate, trial_seed(seed, trial))
                    order = list(cells)
                    random.Random(trial_seed(seed, backend, count, rate, trial // 2)).shuffle(order)
                    if trial % 2:
                        order.reverse()
                    expected = {}
                    for n, mode in order:
                        timing, raw = run_cli(
                            command(binary, pack, mode, selected, backend, n), env, timeout
                        )
                        values = predictions(raw, selected)
                        routed = sum(deterministic(value) for value in values)
                        if routed != (count * rate // 100 if mode == "hybrid" else 0):
                            raise RuntimeError(
                                "Observed rule-hit mixture differs from requested mixture"
                            )
                        normalized = without_scores(values)
                        if expected.setdefault(mode, normalized) != normalized:
                            raise RuntimeError("Predictions differ across worker counts")
                        cells[n, mode]["trials"].append(
                            timing
                            | dict(
                                trial=trial,
                                distinct_files=len(set(selected)),
                                order_sha256=digest(
                                    "\n".join(Path(p).name for p in selected).encode()
                                ),
                                observed_rule_hits=routed,
                            )
                        )
                report["measurements"].extend(cells.values())
                atomic_json(output, report)
                print(f"{backend}: {count} files, {rate}% hits completed", flush=True)
    report["status"] = "complete"
    atomic_json(output, report)
    return report


def digest(data):
    return hashlib.sha256(data).hexdigest()


def trial_seed(seed, *parts):
    return int(digest(":".join(map(str, (seed, *parts))).encode())[:16], 16)


def sample_order(files, count, seed):
    """Shuffle complete cycles, so repeats cannot masquerade as distinct samples."""
    rng = random.Random(seed)
    selected = []
    if count and not files:
        raise ValueError("Cannot sample an empty pool")
    while len(selected) < count:
        cycle = list(files)
        rng.shuffle(cycle)
        selected.extend(cycle[: count - len(selected)])
    return selected


def run_cli(command, env, timeout, cpus=None):
    if sys.platform == "darwin":
        prefix, rss_pattern, scale = (
            ["/usr/bin/time", "-l"],
            r"(\d+)\s+maximum resident set size",
            1,
        )
    elif sys.platform.startswith("linux"):
        prefix, rss_pattern, scale = (
            ["/usr/bin/time", "-v"],
            r"Maximum resident set size \(kbytes\):\s*(\d+)",
            1024,
        )
    else:
        prefix, rss_pattern, scale = [], None, 1
    try:
        import resource
    except ImportError:
        resource = None
    cpu = resource.getrusage(resource.RUSAGE_CHILDREN) if resource else None
    start = time.perf_counter()
    child = subprocess.Popen(
        prefix + command,
        env=env,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        start_new_session=os.name == "posix",
        preexec_fn=(lambda: os.sched_setaffinity(0, cpus)) if cpus else None,
    )
    try:
        stdout, stderr = child.communicate(timeout=timeout)
    except subprocess.TimeoutExpired:
        if os.name == "posix":
            os.killpg(child.pid, signal.SIGKILL)
        else:
            child.kill()
        child.communicate()
        raise RuntimeError(f"CLI exceeded {timeout}s") from None
    elapsed = time.perf_counter() - start
    cpu_end = resource.getrusage(resource.RUSAGE_CHILDREN) if resource else None
    diagnostic = stderr.decode(errors="replace")
    if child.returncode:
        raise RuntimeError(f"CLI exited {child.returncode}: {diagnostic[-2000:]}")
    rss = re.search(rss_pattern, diagnostic) if rss_pattern else None
    timing = dict(
        seconds=elapsed,
        cpu_seconds=(cpu_end.ru_utime + cpu_end.ru_stime - cpu.ru_utime - cpu.ru_stime)
        if cpu
        else None,
        peak_rss_bytes=int(rss[1]) * scale if rss else None,
    )
    return timing, stdout or b""


def predictions(output, paths):
    rows = [json.loads(line) for line in output.splitlines()]
    if [r["path"] for r in rows] != list(map(str, paths)):
        raise RuntimeError("Missing or reordered CLI output")
    if any(r["result"]["status"] != "ok" for r in rows):
        raise RuntimeError("File classification failed")
    return [r["result"]["value"] for r in rows]


def deterministic(row):
    return row["dl"]["label"] == "undefined"


def exact_mix(hits, misses, count, rate, seed):
    """None means fractional files: never round a requested percentage."""
    if count * rate % 100:
        return None
    n = count * rate // 100
    selected = sample_order(hits, n, trial_seed(seed, "hits")) + sample_order(
        misses, count - n, trial_seed(seed, "misses")
    )
    random.Random(trial_seed(seed, count, rate, "merge")).shuffle(selected)
    return selected


def host_info(root):
    def probe(command):
        try:
            return (
                subprocess.check_output(command, stderr=subprocess.DEVNULL, timeout=5)
                .decode()
                .strip()
            )
        except (OSError, subprocess.SubprocessError):
            return None

    info = dict(
        platform=platform.platform(),
        architecture=platform.machine(),
        logical_cpus=os.cpu_count(),
        python=platform.python_version(),
    )
    if sys.platform == "darwin":
        for key, field in [
            ("machdep.cpu.brand_string", "cpu"),
            ("hw.memsize", "ram_bytes"),
            ("hw.physicalcpu", "physical_cores"),
            ("hw.perflevel0.physicalcpu", "performance_cores"),
            ("hw.perflevel1.physicalcpu", "efficiency_cores"),
        ]:
            value = probe(["sysctl", "-n", key])
            info[field] = int(value) if value and value.isdigit() else value
        df = probe(["df", "-P", str(root)])
        device = df.splitlines()[-1].split()[0] if df else ""
        disk = probe(["diskutil", "info", "-plist", device])
        if disk:
            data = plistlib.loads(disk.encode())
            info["storage"] = {
                k: data.get(k)
                for k in ("FilesystemType", "BusProtocol", "SolidState", "MediaName", "TotalSize")
            }
    elif sys.platform.startswith("linux"):
        info["ram_bytes"] = os.sysconf("SC_PHYS_PAGES") * os.sysconf("SC_PAGE_SIZE")
        info["cpu_topology"] = probe(["lscpu", "-J"])
        info["storage"] = probe(["findmnt", "-J", "-T", str(root), "-o", "SOURCE,FSTYPE,OPTIONS"])
        info["block_devices"] = probe(["lsblk", "-J", "-o", "NAME,TYPE,SIZE,ROTA,TRAN,MODEL"])
    if hasattr(os, "statvfs"):
        fs = os.statvfs(root)
        info["filesystem_capacity_bytes"] = fs.f_blocks * fs.f_frsize
        info["filesystem_available_bytes"] = fs.f_bavail * fs.f_frsize
    info["nice"] = os.getpriority(os.PRIO_PROCESS, 0) if hasattr(os, "getpriority") else None
    info["allowed_cpus"] = (
        sorted(os.sched_getaffinity(0)) if hasattr(os, "sched_getaffinity") else None
    )
    return info


def read_baseline(files, limit=64 * 2**20):
    """Bounded read-only baseline. Page-cache effects are explicitly part of this metric."""
    trials = []
    for _ in range(3):
        start, total, opened = time.perf_counter(), 0, 0
        for path in files:
            with path.open("rb") as stream:
                opened += 1
                while total < limit:
                    chunk = stream.read(min(2**20, limit - total))
                    total += len(chunk)
                    if not chunk:
                        break
            if total >= limit:
                break
        seconds = time.perf_counter() - start
        trials.append(
            dict(bytes=total, files=opened, seconds=seconds, mib_per_second=total / 2**20 / seconds)
        )
    return dict(
        scope="Sequential reads of up to 64 MiB of actual corpus files, three passes after hashing. "
        "Includes open/read/close and page cache; not raw-device or cold-disk speed.",
        trials=trials,
    )


def without_scores(value):
    if isinstance(value, dict):
        return {k: without_scores(v) for k, v in value.items() if k != "score"}
    if isinstance(value, list):
        return [without_scores(v) for v in value]
    return value
