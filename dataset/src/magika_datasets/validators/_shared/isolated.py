# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Run a validator's heavy inspection in a subprocess with a time limit."""

import json
import subprocess
import sys

SECONDS = 10


def run(module: str, kind: str, data: bytes, seconds: int = SECONDS) -> tuple[str, str]:
    """Execute `python -m module kind` with data on stdin; expects a JSON [status, detail]."""
    try:
        result = subprocess.run(
            [sys.executable, "-m", module, kind],
            input=data,
            stdout=subprocess.PIPE,
            stderr=subprocess.DEVNULL,
            timeout=seconds,
        )
    except subprocess.TimeoutExpired:
        return "inconclusive", f"Inspection exceeded {seconds}-second limit"
    if result.returncode:
        return "inconclusive", "Inspection process failed"
    try:
        status, detail = json.loads(result.stdout)
        if status not in {"pass", "fail", "inconclusive"}:
            raise ValueError("Unknown status")
        return status, detail
    except (ValueError, TypeError):
        return "inconclusive", "Inspection returned an invalid observation"
