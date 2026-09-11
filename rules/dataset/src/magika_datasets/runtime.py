# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Filesystem and runtime identity helpers shared by command entry points."""

import hashlib
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
                "generated outputs inside a repository must be gitignored; use rules/dataset/.local/"
            )
    return path


def implementation_hashes() -> dict[str, str]:
    """Freeze every runtime module, including discovery and observation policies."""
    return {
        str(path.relative_to(Path(__file__).parent)): hashlib.sha256(path.read_bytes()).hexdigest()
        for path in sorted(Path(__file__).parent.rglob("*.py"))
    }
