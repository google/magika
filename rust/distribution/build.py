# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Build the complete native payload expected by cargo-dist's installers."""

import json
import os
import platform
import re
import runpy
import shutil
import subprocess
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


def main():
    metadata = json.loads(
        subprocess.check_output(
            [
                "cargo",
                "metadata",
                "--no-deps",
                "--format-version=1",
                "--manifest-path",
                str(ROOT / "rust/cli/Cargo.toml"),
            ],
            text=True,
        )
    )
    version = next(
        p["version"] for p in metadata["packages"] if p["name"] == "magika-cli"
    )
    declared = re.search(
        r'^version = "([^"]+)"$',
        Path(__file__).with_name("dist.toml").read_text(),
        re.MULTILINE,
    )
    if declared is None or declared[1] != version:
        raise ValueError(f"dist.toml must match magika-cli version {version}")
    host = next(
        line.removeprefix("host: ")
        for line in subprocess.check_output(["rustc", "-vV"], text=True).splitlines()
        if line.startswith("host: ")
    )
    target = os.environ["CARGO_DIST_TARGET"]
    if target != host:
        raise ValueError(
            f"Release runtime builds require a native runner: {host} != {target}"
        )
    build = runpy.run_path(str(ROOT / "rust/build-runtime.py"))["build"]
    windows = platform.system() == "Windows"
    suffix = ".dll" if windows else ".dylib" if platform.system() == "Darwin" else ".so"
    prefix = "" if windows else "lib"
    gpu = "metal" if platform.system() == "Darwin" else "cuda"
    with tempfile.TemporaryDirectory(prefix="magika-dist-") as temporary:
        output = Path(temporary) / "distribution"
        build(ROOT, ROOT / "rust/target", output, gpu)
        executable = "magika.exe" if windows else "magika"
        shutil.copy2(output / executable, Path.cwd() / executable)
        for source, name in [("cpu", "cpu"), (gpu, "gpu")]:
            shutil.copy2(
                output / f"lib/{prefix}magika_runtime_{source}{suffix}",
                Path.cwd() / f"{prefix}magika_runtime_{name}{suffix}",
            )


if __name__ == "__main__":
    main()
