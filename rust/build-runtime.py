#!/usr/bin/env python3
# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Build a CLI distribution with deferred CPU and optional GPU runtime libraries."""

import argparse
import hashlib
import json
import os
import platform
import shutil
import subprocess
from pathlib import Path


def build(root, target, output, gpu, backends_only=False):
    output.mkdir(parents=True, exist_ok=False)
    (output / "lib").mkdir()
    env = dict(os.environ, CARGO_TARGET_DIR=str(target))
    windows = platform.system() == "Windows"
    suffix = ".dll" if windows else ".dylib" if platform.system() == "Darwin" else ".so"
    prefix = "" if windows else "lib"
    commands = []
    for backend in ["cpu", *([gpu] if gpu else [])]:
        command = [
            "cargo",
            "build",
            "--release",
            "--locked",
            "--manifest-path",
            str(root / "rust/runtime-plugin/Cargo.toml"),
        ]
        if backend != "cpu":
            command += ["--features", backend]
        commands.append(command)
        subprocess.run(command, cwd=root, env=env, check=True)
        shutil.copy2(
            target / f"release/{prefix}magika_runtime_plugin{suffix}",
            output / f"lib/{prefix}magika_runtime_{backend}{suffix}",
        )
    executable = "magika.exe" if windows else "magika"
    if not backends_only:
        command = [
            "cargo",
            "build",
            "--release",
            "--locked",
            "--manifest-path",
            str(root / "rust/cli/Cargo.toml"),
            "--features",
            "yara-rules",
        ]
        commands.append(command)
        subprocess.run(command, cwd=root, env=env, check=True)
        executable = "magika.exe" if windows else "magika"
        shutil.copy2(target / "release" / executable, output / executable)
    receipt = {
        "schema": 1,
        "commands": commands,
        "rustflags": env.get("RUSTFLAGS", ""),
        "source_revision": subprocess.check_output(
            ["git", "rev-parse", "HEAD"], cwd=root, text=True
        ).strip(),
        "source_diff_sha256": hashlib.sha256(
            subprocess.check_output(["git", "diff", "HEAD"], cwd=root)
        ).hexdigest(),
        "rustc": subprocess.check_output(["rustc", "-Vv"], text=True),
        "artifacts": {
            str(p.relative_to(output)): hashlib.sha256(p.read_bytes()).hexdigest()
            for p in [
                *([output / executable] if not backends_only else []),
                *sorted((output / "lib").iterdir()),
            ]
        },
    }
    (output / "build.json").write_text(json.dumps(receipt, indent=2) + "\n")


if __name__ == "__main__":
    root = Path(__file__).resolve().parent.parent
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--target-dir", type=Path, default=root / "rust/target")
    parser.add_argument("--backends-only", action="store_true")
    parser.add_argument("--gpu", choices=["metal", "cuda"])
    args = parser.parse_args()
    if args.gpu == "metal" and platform.system() != "Darwin":
        parser.error("Metal requires macOS")
    build(
        root,
        args.target_dir.resolve(),
        args.output.resolve(),
        args.gpu,
        args.backends_only,
    )
