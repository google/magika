# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Filesystem and runtime identity helpers shared by command entry points."""

import json
import subprocess
from pathlib import Path


def private_output(path: Path) -> Path:
    """Refuse generated dataset metadata in a repository unless it is ignored."""
    path = path.resolve()
    existing = path
    while not existing.exists():
        existing = existing.parent
    cwd = existing if existing.is_dir() else existing.parent
    probe = subprocess.run(
        ["git", "rev-parse", "--show-toplevel"], cwd=cwd, capture_output=True, text=True
    )
    if probe.returncode == 0:
        root = Path(probe.stdout.strip())
        ignored = subprocess.run(["git", "check-ignore", "--quiet", str(path)], cwd=root)
        if ignored.returncode:
            raise ValueError(
                "generated outputs inside a repository must be gitignored; use dataset/local/"
            )
    return path


def read(path: Path):
    return json.loads(Path(path).read_text())


def write(path: Path, value) -> None:
    """Write JSON atomically, and leave an unchanged file untouched so its mtime means something."""
    path = Path(path)
    path.parent.mkdir(parents=True, exist_ok=True)
    content = json.dumps(value, sort_keys=True) + "\n"
    if path.exists() and path.read_text() == content:
        return
    temporary = path.with_suffix(path.suffix + ".next")
    temporary.write_text(content)
    temporary.replace(path)
