# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Pass cargo-dist's target matrix as arguments, never as expanded shell code."""

import json
import os
import re
import subprocess


def command(targets, tag):
    if (
        not isinstance(targets, list)
        or not targets
        or not all(
            isinstance(target, str) and re.fullmatch(r"[a-z0-9_-]+", target)
            for target in targets
        )
    ):
        raise ValueError("expected a nonempty array of Rust target names")
    args = [
        "dist",
        "build",
        "--artifacts=local",
        "--print=linkage",
        "--output-format=json",
    ]
    args += [f"--target={target}" for target in targets]
    if tag:
        args.append(f"--tag={tag}")
    return args


if __name__ == "__main__":
    args = command(
        json.loads(os.environ["DIST_TARGETS"]), os.environ.get("DIST_TAG", "")
    )
    with open("dist-manifest.json", "w") as output:
        subprocess.run(args, stdout=output, check=True)
