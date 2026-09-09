# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Stage native inference libraries into the wheel's script installation directory."""

import argparse
import platform
import runpy
import shutil
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


def stage(runtime_dir, data_dir):
    suffix = {"Darwin": ".dylib", "Windows": ".dll"}.get(platform.system(), ".so")
    prefix = "" if platform.system() == "Windows" else "lib"
    cpu = runtime_dir / f"{prefix}magika_runtime_cpu{suffix}"
    if not cpu.is_file():
        raise FileNotFoundError(cpu)
    destination = data_dir / "scripts"
    destination.mkdir(parents=True, exist_ok=False)
    for backend in ("cpu", "metal", "cuda"):
        path = runtime_dir / f"{prefix}magika_runtime_{backend}{suffix}"
        if path.is_file():
            shutil.copy2(path, destination / path.name)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--runtime-dir", type=Path)
    parser.add_argument("--data-dir", type=Path, default=ROOT / "python/magika.data")
    args = parser.parse_args()
    if args.runtime_dir:
        stage(args.runtime_dir.resolve(), args.data_dir.resolve())
    else:
        build = runpy.run_path(str(ROOT / "rust/build-runtime.py"))["build"]
        with tempfile.TemporaryDirectory(prefix="magika-wheel-") as temporary:
            output = Path(temporary) / "runtime"
            build(
                ROOT,
                ROOT / "rust/target",
                output,
                "metal" if platform.system() == "Darwin" else None,
                backends_only=True,
            )
            stage(output / "lib", args.data_dir.resolve())


if __name__ == "__main__":
    main()
