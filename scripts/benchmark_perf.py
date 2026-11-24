#!/usr/bin/env python3
# Copyright 2026 Google LLC
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     https://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.
"""Cross-platform performance benchmark for Magika (Python library & Rust CLI).

Collects system info and measures cold startup, warm single-file in-memory
latency (ML vs binary/resolver), batched throughput, and Python peak RSS.
Zero third-party dependencies (Python 3.8+ standard library only).

Usage:
    python3 scripts/benchmark_perf.py
    python3 scripts/benchmark_perf.py --trials 3 --output perf.json
    python3 scripts/benchmark_perf.py --python /path/to/venv/bin/python
    python3 scripts/benchmark_perf.py --only cli --cli-backend cpu
"""

from __future__ import annotations

import argparse
import ctypes
import importlib
import io
import json
import os
import platform
import shutil
import statistics
import subprocess
import sys
import sysconfig
import tempfile
import time
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Callable, Sequence

DEFAULT_REPO_CORPUS = Path(__file__).resolve().parent.parent / "tests_data" / "basic"

# Fixed extension buckets so Magika 1.x and 2.x benchmark identical file sets.
CODE_ML_EXTS = frozenset(
    ".asm .awk .bib .c .clj .cpp .cs .css .csv .dart .erl .exs .f90 .go "
    ".gradle .groovy .hcl .ini .j2 .java .js .lua .md .proto .py .rb .rs "
    ".sh .sql .swift .toml .ts .txt .yaml .yar .yml .zig".split()
)
BINARY_RES_EXTS = frozenset(
    ".bmp .docx .elf .epub .flac .gif .gz .jar .jpeg .jpg .mp3 .mp4 .msg "
    ".npy .npz .odp .ods .odt .ogg .parquet .pdf .png .pptx .psd .pth "
    ".rtf .sqlite .tar .wasm .wav .webp .xlsx .zip".split()
)
_PNG_1X1 = (
    b"\x89PNG\r\n\x1a\n\x00\x00\x00\rIHDR\x00\x00\x00\x01\x00\x00\x00\x01"
    b"\x08\x02\x00\x00\x00\x90wS\xde\x00\x00\x00\x0cIDATx\x9cc\xf8\xcf\xc0"
    b"\x00\x00\x03\x01\x01\x00\x18\xdd\x8d\xb0\x00\x00\x00\x00IEND\xaeB`\x82"
)

# Imports only `time` before t0 so benchmark_perf.py's stdlib imports do not skew
# cold import timing or initial peak RSS.
_COLD_SNIPPET = """
import sys, time
out, sample = sys.argv[1], sys.argv[2]
t0 = time.perf_counter()
try:
    import magika
except ModuleNotFoundError as exc:
    if exc.name == "magika":
        with open(out, "w", encoding="utf-8") as f: f.write('{"skipped":"magika not installed"}')
        raise SystemExit(2)
    raise
m = magika.Magika()
t_init = time.perf_counter()
rss = None
try:
    if sys.platform == "win32":
        import ctypes, ctypes.wintypes as w
        class _P(ctypes.Structure):
            _fields_ = [("cb", w.DWORD), ("pf", w.DWORD), ("peak", ctypes.c_size_t)] + [(f"x{i}", ctypes.c_size_t) for i in range(7)]
        k, p = ctypes.WinDLL("kernel32"), ctypes.WinDLL("psapi")
        k.GetCurrentProcess.restype, p.GetProcessMemoryInfo.restype = w.HANDLE, w.BOOL
        p.GetProcessMemoryInfo.argtypes = [w.HANDLE, ctypes.POINTER(_P), w.DWORD]
        c = _P(cb=ctypes.sizeof(_P))
        if p.GetProcessMemoryInfo(k.GetCurrentProcess(), ctypes.byref(c), c.cb):
            rss = round(c.peak / 1048576.0, 2)
    else:
        import resource
        rss = round(resource.getrusage(resource.RUSAGE_SELF).ru_maxrss / (1048576.0 if sys.platform == "darwin" else 1024.0), 2)
except Exception:
    pass
t_call, first_ms = time.perf_counter(), 0.0
if sample:
    res = m.identify_path(sample)
    first_ms = (time.perf_counter() - t_call) * 1000.0
    if not res.ok:
        raise RuntimeError(f"cold identify_path failed: {res.status}")
import json
init_ms = (t_init - t0) * 1000.0
with open(out, "w", encoding="utf-8") as f:
    json.dump({"import_init_ms": round(init_ms, 3), "total_ms": round(init_ms + first_ms, 3), "peak_rss_init_mib": rss}, f)
"""


def _collapse(msg: str, limit: int = 200) -> str:
    s = " ".join(msg.splitlines()).strip()
    return (s[: limit - 3] + "...") if len(s) > limit else s


def _run(
    cmd: Sequence[str], timeout_s: int = 600, discard_stdout: bool = False
) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        list(cmd),
        stdin=subprocess.DEVNULL,
        stdout=subprocess.DEVNULL if discard_stdout else subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        encoding="utf-8",
        errors="replace",
        timeout=timeout_s,
        check=False,
    )


def _stats(
    label: str,
    kind: str,
    samples_ms: Sequence[float],
    files: int = 0,
    ml_inferred: int | None = None,
    non_ml: int | None = None,
) -> dict[str, Any]:
    med, lo, hi = statistics.median(samples_ms), min(samples_ms), max(samples_ms)
    per_file = (med / files) if files > 0 else 0.0
    fps = (
        (files / (med / 1000.0)) if (files > 0 and med > 0 and kind != "cold") else 0.0
    )
    return {
        "label": label,
        "kind": kind,
        "median_ms": round(med, 4),
        "min_ms": round(lo, 4),
        "max_ms": round(hi, 4),
        "files": files,
        "ms_per_file": round(per_file, 5),
        "files_per_sec": round(fps, 1),
        "ml_inferred": ml_inferred,
        "non_ml": non_ml,
    }


def _get_cpu_name() -> str:
    if sys.platform == "win32":
        try:
            import winreg

            with winreg.OpenKey(
                winreg.HKEY_LOCAL_MACHINE,
                r"HARDWARE\DESCRIPTION\System\CentralProcessor\0",
            ) as key:
                val, _ = winreg.QueryValueEx(key, "ProcessorNameString")
                if isinstance(val, str) and val.strip():
                    return " ".join(val.strip().split())
        except OSError:
            pass
    elif sys.platform == "darwin":
        try:
            p = _run(["sysctl", "-n", "machdep.cpu.brand_string"], timeout_s=5)
            if p.returncode == 0 and p.stdout.strip():
                return p.stdout.strip()
        except (OSError, subprocess.SubprocessError):
            pass
    elif sys.platform.startswith("linux"):
        try:
            txt = Path("/proc/cpuinfo").read_text(encoding="utf-8", errors="ignore")
            for key in ("model name", "Model", "Hardware", "uarch"):
                for line in txt.splitlines():
                    if ":" in line and line.split(":", 1)[0].strip() == key:
                        val = line.split(":", 1)[1].strip()
                        if val:
                            return val
        except OSError:
            pass
    return platform.processor() or platform.machine() or "unknown"


def _get_total_ram_gib() -> float | None:
    if sys.platform == "win32":
        try:
            import ctypes.wintypes as w

            class _Mem(ctypes.Structure):
                _fields_ = [
                    ("len", w.DWORD),
                    ("load", w.DWORD),
                    ("phys", ctypes.c_ulonglong),
                ] + [(f"f{i}", ctypes.c_ulonglong) for i in range(6)]

            k32 = ctypes.WinDLL("kernel32")
            k32.GlobalMemoryStatusEx.argtypes = [ctypes.POINTER(_Mem)]
            k32.GlobalMemoryStatusEx.restype = w.BOOL
            m = _Mem(len=ctypes.sizeof(_Mem))
            if k32.GlobalMemoryStatusEx(ctypes.byref(m)):
                return round(m.phys / (1024**3), 1)
        except Exception:
            return None
    elif sys.platform == "darwin":
        try:
            p = _run(["sysctl", "-n", "hw.memsize"], timeout_s=5)
            if p.returncode == 0 and p.stdout.strip().isdigit():
                return round(int(p.stdout.strip()) / (1024**3), 1)
        except (OSError, subprocess.SubprocessError):
            return None
    elif sys.platform.startswith("linux"):
        try:
            for line in Path("/proc/meminfo").read_text(encoding="utf-8").splitlines():
                if line.startswith("MemTotal:"):
                    parts = line.split()
                    if len(parts) >= 2 and parts[1].isdigit():
                        return round(int(parts[1]) / (1024**2), 1)
        except OSError:
            return None
    return None


def _get_peak_rss_mib() -> float | None:
    if sys.platform == "win32":
        try:
            import ctypes.wintypes as w

            class _PMC(ctypes.Structure):
                _fields_ = [
                    ("cb", w.DWORD),
                    ("pf", w.DWORD),
                    ("peak", ctypes.c_size_t),
                ] + [(f"f{i}", ctypes.c_size_t) for i in range(7)]

            k32, psapi = ctypes.WinDLL("kernel32"), ctypes.WinDLL("psapi")
            k32.GetCurrentProcess.restype, psapi.GetProcessMemoryInfo.restype = (
                w.HANDLE,
                w.BOOL,
            )
            psapi.GetProcessMemoryInfo.argtypes = [
                w.HANDLE,
                ctypes.POINTER(_PMC),
                w.DWORD,
            ]
            pmc = _PMC(cb=ctypes.sizeof(_PMC))
            if psapi.GetProcessMemoryInfo(
                k32.GetCurrentProcess(), ctypes.byref(pmc), pmc.cb
            ):
                return round(pmc.peak / 1048576.0, 2)
        except Exception:
            return None
        return None
    try:
        import resource

        div = 1048576.0 if sys.platform == "darwin" else 1024.0
        return round(resource.getrusage(resource.RUSAGE_SELF).ru_maxrss / div, 2)
    except Exception:
        return None


def collect_system_info() -> dict[str, Any]:
    """Collect OS, architecture, CPU model, logical core count, and RAM."""
    return {
        "os": platform.platform(),
        "arch": platform.machine(),
        "cpu": _get_cpu_name(),
        "cpu_logical_cores": os.cpu_count(),
        "ram_gib": _get_total_ram_gib(),
        "timestamp_utc": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
    }


def _prepare_corpus(
    corpus_arg: Path | None, batch_files: int, tmp_dir: Path
) -> tuple[str, list[Path], list[Path], list[Path], list[Path], Path]:
    if corpus_arg is not None:
        base_files = sorted(
            (p for p in corpus_arg.rglob("*") if not p.is_symlink() and p.is_file()),
            key=lambda p: p.relative_to(corpus_arg).as_posix(),
        )
        source = corpus_arg.name or str(corpus_arg)
    elif DEFAULT_REPO_CORPUS.is_dir():
        base_files = sorted(
            (
                p
                for p in DEFAULT_REPO_CORPUS.rglob("*")
                if not p.is_symlink() and p.is_file()
            ),
            key=lambda p: p.relative_to(DEFAULT_REPO_CORPUS).as_posix(),
        )
        source = "tests_data/basic"
    else:
        # Tiny standalone fallback when run outside a repository checkout.
        syn = tmp_dir / "synthetic"
        syn.mkdir()
        py_src = "def avg(xs):\n    return sum(xs) / len(xs)\n" + "\n".join(
            f"W_{i} = {i}.5" for i in range(25)
        )
        (syn / "sample.py").write_bytes((py_src + "\n").encode("utf-8"))
        (syn / "sample.png").write_bytes(_PNG_1X1)
        base_files, source = sorted(syn.iterdir()), "built-in synthetic"

    ml_all, bin_all = [], []
    for p in base_files:
        try:
            sz = p.stat().st_size
        except OSError:
            continue
        ext = p.suffix.lower()
        if ext in CODE_ML_EXTS and sz >= 64:
            ml_all.append(p)
        elif ext in BINARY_RES_EXTS and sz > 0:
            bin_all.append(p)

    def _stride(items: Sequence[Path], n: int) -> list[Path]:
        if len(items) <= n:
            return list(items)
        return [items[int(i * len(items) / n)] for i in range(n)]

    def _copy_batch(files: Sequence[Path], dest: Path, count: int) -> list[Path]:
        if not files or count <= 0:
            return []
        dest.mkdir(parents=True, exist_ok=True)
        chosen = (
            _stride(files, count)
            if len(files) >= count
            else [files[i % len(files)] for i in range(count)]
        )
        out: list[Path] = []
        for i, src in enumerate(chosen):
            dst = dest / f"{i:05d}_{src.stem[:60]}{src.suffix[:20]}"
            shutil.copyfile(src, dst)
            out.append(dst)
        return out

    batch_dir = tmp_dir / "batch_mixed"
    _copy_batch(base_files, batch_dir, batch_files)
    batch_ml = (
        _copy_batch(ml_all, tmp_dir / "batch_ml", max(1, batch_files // 2))
        if ml_all
        else []
    )
    return (
        source,
        base_files,
        _stride(ml_all, 40),
        _stride(bin_all, 40),
        batch_ml,
        batch_dir,
    )


def _measure(fn: Callable[[], Any], trials: int) -> tuple[list[float], Any]:
    warm_res = fn()
    samples = []
    for _ in range(trials):
        t0 = time.perf_counter()
        fn()
        samples.append((time.perf_counter() - t0) * 1000.0)
    return samples, warm_res


def _count_dl(results: Sequence[Any]) -> tuple[int, int]:
    ml, skip, errs = 0, 0, []
    for r in results:
        if not getattr(r, "ok", False):
            raw = getattr(r, "path", None)
            errs.append(
                f"{Path(str(raw)).name if raw is not None else '?'}: {getattr(r, 'status', 'err')}"
            )
        elif str(r.dl.label) == "undefined":
            skip += 1
        else:
            ml += 1
    if errs:
        raise RuntimeError(f"{len(errs)} identification error(s), e.g. {errs[0]}")
    return ml, skip


def _run_py_worker(spec_path: Path, result_path: Path) -> int:
    try:
        magika = importlib.import_module("magika")
    except ModuleNotFoundError as exc:
        if exc.name == "magika":
            result_path.write_text(
                '{"skipped":"magika not installed"}', encoding="utf-8"
            )
            return 2
        raise

    spec = json.loads(spec_path.read_text(encoding="utf-8"))
    trials = int(spec["trials"])
    m = magika.Magika()
    meta = {
        "python_version": platform.python_version(),
        "module_version": m.get_module_version(),
        "model_name": m.get_model_name(),
        "output_content_types": len(m.get_output_content_types()),
    }
    timings: dict[str, dict[str, Any]] = {}

    # Run identify_paths() first so peak_rss_batch_mib excludes read_bytes() buffers.
    for key, s_key, adj in (
        ("py_batch_ml", "batch_ml", "ML"),
        ("py_batch_mixed", "batch_mixed", "mixed"),
    ):
        paths = [Path(p) for p in spec[s_key]]
        if paths:
            s, res = _measure(lambda ps=paths: m.identify_paths(ps), trials)
            ml_cnt, non_ml_cnt = _count_dl(res)
            w = "file" if len(paths) == 1 else "files"
            lbl = (
                f"Python identify_paths ({len(paths)} mixed: {ml_cnt} ML, {non_ml_cnt} non-ML)"
                if key == "py_batch_mixed"
                else f"Python identify_paths ({len(paths)} {adj} {w})"
            )
            timings[key] = _stats(lbl, "batch", s, len(paths), ml_cnt, non_ml_cnt)
    peak_batch_mib = _get_peak_rss_mib()

    for key, s_key, lbl in (
        ("py_bytes_ml", "ml_single", "Python identify_bytes (ML, no disk I/O)"),
        (
            "py_bytes_resolver",
            "bin_single",
            "Python identify_bytes (binary/resolver, no disk I/O)",
        ),
    ):
        bufs = []
        for rp in spec[s_key]:
            p = Path(rp)
            try:
                if p.stat().st_size <= 1024 * 1024:
                    bufs.append(p.read_bytes())
            except OSError:
                continue
        if bufs:
            s, res = _measure(lambda bs=bufs: [m.identify_bytes(b) for b in bs], trials)
            ml_cnt, non_ml_cnt = _count_dl(res)
            timings[key] = _stats(lbl, "per_file", s, len(bufs), ml_cnt, non_ml_cnt)

    result_path.write_text(
        json.dumps(
            {"meta": meta, "timings": timings, "peak_rss_batch_mib": peak_batch_mib}
        ),
        encoding="utf-8",
    )
    return 0


def _run_json_worker(
    cmd: Sequence[str], out_file: Path, timeout_s: int = 600
) -> tuple[int, dict[str, Any] | None, str]:
    if out_file.exists():
        out_file.unlink()
    try:
        p = _run(cmd, timeout_s=timeout_s)
    except (OSError, subprocess.SubprocessError) as exc:
        return -1, None, _collapse(str(exc))
    data = None
    if out_file.is_file():
        try:
            data = json.loads(out_file.read_text(encoding="utf-8"))
        except (OSError, json.JSONDecodeError) as exc:
            return p.returncode, None, f"invalid worker JSON: {exc}"
    return p.returncode, data, _collapse(p.stderr or p.stdout or f"exit {p.returncode}")


def benchmark_python(
    py_exe: str,
    explicit_py: bool,
    ml_single: Sequence[Path],
    bin_single: Sequence[Path],
    batch_ml: Sequence[Path],
    batch_mixed: Sequence[Path],
    trials: int,
    work_dir: Path,
) -> dict[str, Any]:
    """Run cold-start and warm Python library benchmarks."""
    cold_out = work_dir / "cold.json"
    sample = str(ml_single[0]) if ml_single else ""
    cold_cmd = [py_exe, "-c", _COLD_SNIPPET, str(cold_out), sample]

    rc, row, err = _run_json_worker(cold_cmd, cold_out, timeout_s=30)
    if rc == 2 and row and "skipped" in row:
        return {
            "status": "error" if explicit_py else "skipped",
            "reason": str(row["skipped"]),
        }
    if rc != 0 or row is None:
        return {"status": "error", "reason": err}

    init_ms, total_ms, init_rss = [], [], []
    for _ in range(trials):
        rc, row, err = _run_json_worker(cold_cmd, cold_out, timeout_s=30)
        if rc != 0 or row is None:
            return {"status": "error", "reason": f"cold trial failed: {err}"}
        init_ms.append(float(row["import_init_ms"]))
        total_ms.append(float(row["total_ms"]))
        if row.get("peak_rss_init_mib") is not None:
            init_rss.append(float(row["peak_rss_init_mib"]))

    timings: dict[str, dict[str, Any]] = {
        "py_cold_import_init": _stats("Python cold import + Magika()", "cold", init_ms)
    }
    if sample:
        timings["py_cold_first_ml"] = _stats(
            "Python cold -> 1st ML file", "cold", total_ms, 1
        )

    spec_file, warm_out = work_dir / "spec.json", work_dir / "warm.json"
    spec_file.write_text(
        json.dumps(
            {
                "trials": trials,
                "ml_single": [str(p) for p in ml_single],
                "bin_single": [str(p) for p in bin_single],
                "batch_ml": [str(p) for p in batch_ml],
                "batch_mixed": [str(p) for p in batch_mixed],
            }
        ),
        encoding="utf-8",
    )
    rc, warm, err = _run_json_worker(
        [
            py_exe,
            str(Path(__file__).resolve()),
            "--_py-worker",
            str(spec_file),
            str(warm_out),
        ],
        warm_out,
    )
    if rc != 0 or warm is None:
        return {"status": "error", "reason": f"warm worker failed: {err}"}

    for k in ("py_bytes_ml", "py_bytes_resolver", "py_batch_ml", "py_batch_mixed"):
        if k in warm["timings"]:
            timings[k] = warm["timings"][k]
    return {
        "status": "ok",
        "meta": warm["meta"],
        "timings": timings,
        "memory_mib": {
            "peak_rss_init_mib": round(statistics.median(init_rss), 2)
            if init_rss
            else None,
            "peak_rss_batch_mib": warm.get("peak_rss_batch_mib"),
        },
    }


def resolve_cli_binary(
    explicit_cli: str | None, py_exe: str, py_explicit: bool
) -> str | None:
    """Locate magika CLI without following venv symlinks out of the target env."""
    if explicit_cli:
        return explicit_cli
    is_win = sys.platform == "win32"
    exe = "magika.exe" if is_win else "magika"
    resolved = (
        shutil.which(py_exe) if (os.sep not in py_exe and "/" not in py_exe) else py_exe
    )
    if resolved:
        py_dir = Path(os.path.abspath(resolved)).parent
        for cand in (py_dir / exe, py_dir / ("Scripts" if is_win else "bin") / exe):
            if cand.is_file():
                return str(cand)
    if py_explicit:
        return None
    sdir = sysconfig.get_path("scripts")
    if sdir and (Path(sdir) / exe).is_file():
        return str(Path(sdir) / exe)
    return shutil.which(exe)


def benchmark_cli(
    cli_exe: str | None,
    explicit_cli: bool,
    ml_sample: Path | None,
    bin_sample: Path | None,
    batch_dir: Path,
    batch_count: int,
    trials: int,
    backend: str | None,
) -> dict[str, Any]:
    """Benchmark the Rust CLI (cold single-file and recursive batch throughput)."""
    if not cli_exe:
        return {"status": "skipped", "reason": "magika CLI binary not found"}
    name = Path(cli_exe).name
    try:
        ver = _run([cli_exe, "--version"], timeout_s=30)
    except (OSError, subprocess.SubprocessError) as exc:
        return {
            "status": "error" if explicit_cli else "skipped",
            "reason": f"failed to execute {name}: {_collapse(str(exc))}",
        }
    if ver.returncode != 0:
        return {
            "status": "error" if explicit_cli else "skipped",
            "reason": f"`{name} --version` failed: {_collapse(ver.stderr or ver.stdout)}",
        }

    has_no_res = False
    try:
        hp = _run([cli_exe, "--help"], timeout_s=30)
        has_no_res = "--no-resolvers" in (hp.stdout + hp.stderr)
    except (OSError, subprocess.SubprocessError):
        pass

    backend_args = ["--backend", backend] if backend else []
    backend_info = None
    try:
        bp = _run([cli_exe, *backend_args, "--backend-info"], timeout_s=30)
        if bp.returncode == 0 and bp.stdout.strip():
            backend_info = bp.stdout.strip().splitlines()[0]
    except (OSError, subprocess.SubprocessError):
        pass

    vlines = [ln.strip() for ln in ver.stdout.splitlines() if ln.strip()]
    meta = {"cli_version": vlines[0] if vlines else name, "backend_info": backend_info}
    base = [cli_exe, "--no-colors", *backend_args]

    def _exec(cmd: Sequence[str]) -> None:
        p = _run(cmd, discard_stdout=True)
        if p.returncode != 0:
            raise RuntimeError(
                f"CLI failed (exit {p.returncode}): {_collapse(p.stderr)}"
            )

    timings: dict[str, dict[str, Any]] = {}
    try:
        for key, smp, lbl in (
            ("cli_single_ml", ml_sample, "CLI 1 file (cold - ML)"),
            ("cli_single_resolver", bin_sample, "CLI 1 file (cold - binary/resolver)"),
        ):
            if smp is not None:
                c = [*base, "--", str(smp)]
                s, _ = _measure(lambda cmd=c: _exec(cmd), trials)
                timings[key] = _stats(lbl, "cold", s, 1)
        if batch_count > 0:
            w = "file" if batch_count == 1 else "files"
            c_mix = [*base, "-r", "--", str(batch_dir)]
            s, _ = _measure(lambda: _exec(c_mix), trials)
            timings["cli_batch_mixed"] = _stats(
                f"CLI -r ({batch_count} mixed {w})", "batch", s, batch_count
            )
            if has_no_res:
                c_nr = [*base, "--no-resolvers", "-r", "--", str(batch_dir)]
                s, _ = _measure(lambda: _exec(c_nr), trials)
                timings["cli_batch_no_resolvers"] = _stats(
                    f"CLI -r --no-resolvers ({batch_count} {w})",
                    "batch",
                    s,
                    batch_count,
                )
    except Exception as exc:
        return {"status": "error", "meta": meta, "reason": _collapse(str(exc))}
    return {"status": "ok", "meta": meta, "timings": timings}


def _fmt_ms(v: float) -> str:
    return f"{v:.3f} ms" if v < 0.1 else (f"{v:.2f} ms" if v < 10.0 else f"{v:,.1f} ms")


def format_markdown(rep: dict[str, Any]) -> str:
    """Render an ASCII-safe Markdown summary ready to paste into a GitHub issue."""
    s, corpus = rep["system"], rep["corpus"]
    cores = s.get("cpu_logical_cores")
    ram = f"{s['ram_gib']} GiB RAM" if s.get("ram_gib") else "RAM unknown"
    lines = [
        "### Magika Performance Benchmark",
        "",
        f"- **System:** {s['os']} | `{s['arch']}` | {s['cpu']} ({cores if cores is not None else '?'} logical cores, {ram})",
    ]
    py = rep.get("python") or {}
    if py.get("status") == "ok":
        pm, mem = py["meta"], py.get("memory_mib") or {}
        mb = [
            f"{t} {mem[k]:,.1f} MiB"
            for t, k in (("init", "peak_rss_init_mib"), ("batch", "peak_rss_batch_mib"))
            if mem.get(k) is not None
        ]
        rss = f", peak RSS: {' / '.join(mb)}" if mb else ""
        lines.append(
            f"- **Python:** `magika {pm['module_version']}` (model `{pm['model_name']}`, {pm['output_content_types']} types, Python {pm['python_version']}{rss})"
        )
    elif py.get("status"):
        lines.append(
            f"- **Python:** {py['status']} ({_collapse(str(py.get('reason', 'n/a')))})"
        )

    cli = rep.get("cli") or {}
    if cli.get("status") == "ok":
        cm = cli["meta"]
        b = f" (backend `{cm['backend_info']}`)" if cm.get("backend_info") else ""
        lines.append(f"- **CLI:** `{cm['cli_version']}`{b}")
    elif cli.get("status"):
        lines.append(
            f"- **CLI:** {cli['status']} ({_collapse(str(cli.get('reason', 'n/a')))})"
        )

    lines.append(
        f"- **Corpus:** `{corpus['source']}` ({corpus['base_files']} base, "
        f"{corpus['batch_files']} batch files, {corpus['batch_size_mib']:.1f} MiB; "
        f"{rep['trials']} trials + 1 warmup)"
    )

    rows = [
        st
        for sec in ("python", "cli")
        if isinstance(rep.get(sec), dict) and rep[sec].get("status") == "ok"
        for st in (rep[sec].get("timings") or {}).values()
    ]
    if rows:
        lines.extend(
            [
                "",
                "| Benchmark | Median | Trial Min-Max | Throughput |",
                "|---|---:|---:|---:|",
            ]
        )
        noisy = 0
        for st in rows:
            kind = st["kind"]
            is_noisy = st["min_ms"] >= 5.0 and st["max_ms"] / st["min_ms"] > 1.25
            noisy += int(is_noisy)
            mark = " ~" if is_noisy else ""
            if kind == "per_file" and st["files"] > 0:
                med_s = f"{_fmt_ms(st['ms_per_file'])}/file{mark}"
                rng_s = f"{_fmt_ms(st['min_ms'] / st['files'])}-{_fmt_ms(st['max_ms'] / st['files'])}/file"
            else:
                med_s, rng_s = (
                    f"{_fmt_ms(st['median_ms'])}{mark}",
                    f"{_fmt_ms(st['min_ms'])}-{_fmt_ms(st['max_ms'])}",
                )
            fps_s = (
                f"{st['files_per_sec']:,.0f} files/s"
                if (kind != "cold" and st["files_per_sec"] > 0)
                else "-"
            )
            lines.append(f"| {st['label']} | **{med_s}** | {rng_s} | {fps_s} |")
        if noisy:
            lines.append(
                f"\n> Note: `~` marks >25% trial spread ({noisy} metric(s)); re-run on an idle machine."
            )
    return "\n".join(lines)


def main(argv: Sequence[str] | None = None) -> int:
    """CLI entry point."""
    if isinstance(sys.stdout, io.TextIOWrapper):
        try:
            sys.stdout.reconfigure(encoding="utf-8", errors="replace")
        except Exception:
            pass

    p = argparse.ArgumentParser(
        description="End-to-end performance benchmark for Magika (Python & Rust CLI)."
    )
    p.add_argument(
        "--only",
        choices=("all", "python", "cli"),
        default="all",
        help="Targets to benchmark (default: all).",
    )
    p.add_argument(
        "--python",
        dest="python_exe",
        default=None,
        help="Python interpreter to benchmark (default: current).",
    )
    p.add_argument(
        "--cli",
        dest="cli_exe",
        default=None,
        help="Path to magika CLI binary (default: auto-detect).",
    )
    p.add_argument(
        "--cli-backend",
        choices=("auto", "cpu", "gpu"),
        default=None,
        help="Optional --backend flag for 2.x CLI.",
    )
    p.add_argument(
        "--corpus",
        type=Path,
        default=None,
        help="Sample directory (default: tests_data/basic, else tiny synthetic).",
    )
    p.add_argument(
        "--batch-files",
        type=int,
        default=500,
        help="Files in materialized batch corpus (default: 500).",
    )
    p.add_argument(
        "--trials",
        type=int,
        default=5,
        help="Measured trials after 1 warmup (default: 5).",
    )
    p.add_argument(
        "--json",
        dest="as_json",
        action="store_true",
        help="Print JSON instead of Markdown.",
    )
    p.add_argument(
        "--output",
        type=Path,
        default=None,
        help="Also save structured JSON report to this path.",
    )
    p.add_argument("--_py-worker", nargs=2, default=None, help=argparse.SUPPRESS)
    args = p.parse_args(argv)

    if args._py_worker:
        return _run_py_worker(Path(args._py_worker[0]), Path(args._py_worker[1]))
    if args.trials < 1 or args.batch_files < 1:
        p.error("--trials and --batch-files must be >= 1")
    if (args.python_exe is not None and not args.python_exe.strip()) or (
        args.cli_exe is not None and not args.cli_exe.strip()
    ):
        p.error("--python and --cli must not be empty")
    if args.corpus is not None and not args.corpus.is_dir():
        p.error(f"--corpus not found: {args.corpus}")
    if args.output is not None and args.output.is_dir():
        p.error(f"--output must be a file path, not a directory: {args.output}")

    py_explicit = args.python_exe is not None
    py_exe = args.python_exe if py_explicit else sys.executable

    with tempfile.TemporaryDirectory(prefix="magika_benchmark_") as tmp:
        tmp_dir = Path(tmp)
        source, base_files, ml_single, bin_single, batch_ml, batch_dir = (
            _prepare_corpus(args.corpus, args.batch_files, tmp_dir)
        )
        if not base_files:
            p.error(f"No regular files in corpus ({source})")
        batch_mixed = sorted(batch_dir.iterdir())
        batch_bytes = sum(f.stat().st_size for f in batch_mixed)

        report: dict[str, Any] = {
            "system": collect_system_info(),
            "trials": args.trials,
            "corpus": {
                "source": source,
                "base_files": len(base_files),
                "batch_files": len(batch_mixed),
                "batch_size_mib": round(batch_bytes / 1048576.0, 2),
            },
            "python": None,
            "cli": None,
        }
        if args.only in ("all", "python"):
            report["python"] = benchmark_python(
                py_exe,
                py_explicit,
                ml_single,
                bin_single,
                batch_ml,
                batch_mixed,
                args.trials,
                tmp_dir,
            )
        if args.only in ("all", "cli"):
            cli_exe = resolve_cli_binary(args.cli_exe, py_exe, py_explicit)
            report["cli"] = benchmark_cli(
                cli_exe,
                args.cli_exe is not None,
                ml_single[0] if ml_single else None,
                bin_single[0] if bin_single else None,
                batch_dir,
                len(batch_mixed),
                args.trials,
                args.cli_backend,
            )

    print(json.dumps(report, indent=2) if args.as_json else format_markdown(report))
    if args.output is not None:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")

    statuses = [
        (report[k] or {}).get("status")
        for k in ("python", "cli")
        if report[k] is not None
    ]
    return (
        1
        if (any(s == "error" for s in statuses) or not any(s == "ok" for s in statuses))
        else 0
    )


if __name__ == "__main__":
    raise SystemExit(main())
