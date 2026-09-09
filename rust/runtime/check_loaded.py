# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Observe actual dyld loads; these diagnostic executions are never timed."""

import argparse
import json
import os
import subprocess
import tempfile
from pathlib import Path


def check(distribution, destination):
    destination.mkdir(parents=True, exist_ok=True)
    rows = []
    for mode, flags in [
        ("rules", ["--rules=only"]),
        ("cpu", ["--rules=off", "--backend=cpu"]),
        ("gpu", ["--rules=off", "--backend=gpu"]),
    ]:
        command = [
            str(distribution / "magika"),
            *flags,
            "--batch-size=8",
            "--backend-info",
        ]
        # No backend path override: verify the relocatable packaged layout itself.
        env = {
            k: v
            for k, v in os.environ.items()
            if k not in ["MAGIKA_RUNTIME_DIR", "MAGIKA_STARTUP_TRACE"]
        }
        env["DYLD_PRINT_LIBRARIES"] = "1"
        env["DYLD_PRINT_INITIALIZERS"] = "1"
        result = subprocess.run(command, env=env, capture_output=True, check=True)
        text = result.stderr.decode()
        (destination / (mode + "-dyld.log")).write_text(text)
        cpu = "libmagika_runtime_cpu.dylib" in text
        gpu = "libmagika_runtime_metal.dylib" in text
        # dyld also lists shared-cache entries that it immediately marks delayed.
        # Track transitions instead of treating every printed image as active.
        metal = False
        for line in text.splitlines():
            if "Metal.framework/Versions/A/Metal" in line:
                metal = True
            if line.endswith("move loaded to delayed: Metal"):
                metal = False
            if line.endswith("move delayed to loaded: Metal"):
                metal = True
        assert (cpu, gpu, metal) == {
            "rules": (False, False, False),
            "cpu": (True, False, False),
            "gpu": (False, True, True),
        }[mode]
        rows.append(
            {
                "mode": mode,
                "command": command,
                "stdout": result.stdout.decode(),
                "cpu_library_loaded": cpu,
                "metal_library_loaded": gpu,
                "metal_framework_active": metal,
                "dyld_log": mode + "-dyld.log",
            }
        )
    (destination / "loaded.json").write_text(json.dumps(rows, indent=2) + "\n")
    with tempfile.TemporaryDirectory(prefix="magika-abi-") as temporary:
        directory = Path(temporary)
        (directory / "libmagika_runtime_cpu.dylib").symlink_to(
            distribution / "lib/libmagika_runtime_cpu.dylib"
        )
        source = directory / "bad.c"
        source.write_text(
            "#include <stdint.h>\nstatic uint32_t header[] = {99, 8};\nconst void *magika_runtime_v1(void) { return header; }\n"
        )
        subprocess.run(
            [
                "cc",
                "-dynamiclib",
                str(source),
                "-o",
                str(directory / "libmagika_runtime_metal.dylib"),
            ],
            check=True,
        )
        results = []
        for backend in ["cpu", "auto", "gpu"]:
            command = [
                str(distribution / "magika"),
                "--backend=" + backend,
                "--rules=off",
                "--batch-size=8",
                "--backend-info",
            ]
            env = dict(os.environ, MAGIKA_RUNTIME_DIR=str(directory))
            result = subprocess.run(command, env=env, capture_output=True, check=False)
            if backend == "gpu":
                assert (
                    result.returncode != 0
                    and b"incompatible inference backend ABI" in result.stderr
                )
            else:
                assert result.returncode == 0 and b"tract-cpu" in result.stdout
            results.append(
                {
                    "backend": backend,
                    "returncode": result.returncode,
                    "stdout": result.stdout.decode(),
                    "stderr": result.stderr.decode(),
                }
            )
        (destination / "abi-rejection.json").write_text(
            json.dumps(results, indent=2) + "\n"
        )


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("distribution", type=Path)
    parser.add_argument("destination", type=Path)
    args = parser.parse_args()
    check(args.distribution.resolve(), args.destination.resolve())
