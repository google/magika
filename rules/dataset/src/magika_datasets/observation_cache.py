# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0

"""Reuse detector observations only when tool and artifact identities match."""

import json
from pathlib import Path

from .unattended import write
from .verdicts import tool_identity


def seed_journal(paths, output, records, tools):
    """Reuse only observations with exactly matching tool/runtime/signature identity."""
    if output.exists():
        return
    wanted = {r["sha256"] for r in records}
    identities = dict(tool_identity(t) for t in tools)
    latest = {}
    for path in paths:
        with Path(path).open() as stream:
            for line in stream:
                if not line.endswith("\n"):
                    break
                record = json.loads(line)
                if (
                    record.get("sha256") not in wanted
                    or record.get("tool_identity") not in identities
                ):
                    continue
                key = record["sha256"], record["tool_identity"]
                if key not in latest or record.get("observed_at", 0) >= latest[key].get(
                    "observed_at", 0
                ):
                    latest[key] = record
    temporary = output.with_suffix(".seed")
    temporary.write_text(
        "".join(json.dumps(r, sort_keys=True) + "\n" for _, r in sorted(latest.items()))
    )
    write(output.with_suffix(".tools.json"), identities)
    temporary.replace(output)
