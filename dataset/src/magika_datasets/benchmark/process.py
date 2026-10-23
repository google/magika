# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Run tools as subprocesses, order samples reproducibly and describe the host."""

import hashlib
import os
import platform
import plistlib
import random
import re
import signal
import subprocess
import sys
import time


def trial_seed(seed, *parts):
    text = ":".join(map(str, (seed, *parts))).encode()
    return int(hashlib.sha256(text).hexdigest()[:16], 16)


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


def run_cli(command, env, timeout, cwd=None):
    """(timing, stdout) for one command, killing its whole process group on timeout."""
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
        cwd=cwd,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        start_new_session=os.name == "posix",
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
        cpu_seconds=cpu_end.ru_utime + cpu_end.ru_stime - cpu.ru_utime - cpu.ru_stime
        if cpu
        else None,
        peak_rss_bytes=int(rss[1]) * scale if rss else None,
    )
    return timing, stdout or b""


def host_info(root):
    def probe(command):
        try:
            output = subprocess.check_output(command, stderr=subprocess.DEVNULL, timeout=5)
            return output.decode().strip()
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
    return info
