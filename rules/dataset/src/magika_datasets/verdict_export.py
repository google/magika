# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0

"""Attach independent detector observations when constructing a public matrix."""

import hashlib
import json
import re
from pathlib import Path


class VerdictIndex:
    """Index completed journal lines; the last journal defines each tool's version."""

    def __init__(self, paths):
        self.tools, self.records, self.identities = {}, {}, {}
        for path in map(Path, paths):
            identities = json.loads(path.with_suffix(".tools.json").read_text())
            self.identities.update(identities)
            for key, tool in identities.items():
                self.tools[tool["id"]] = key
            with path.open("rb") as stream:
                for line in stream:
                    if not line.endswith(b"\n"):
                        break  # Concurrent collector may be appending its final line.
                    record = json.loads(line)
                    if record["tool_identity"] not in identities:
                        continue  # Earlier configuration retained in the append-only journal.
                    key = record["sha256"], record["tool_identity"]
                    previous = self.records.get(key)
                    if previous is None or record.get("observed_at", 0) >= previous.get(
                        "observed_at", 0
                    ):
                        self.records[key] = record

    def observations(self, sample):
        result = {}
        for tool, identity_key in self.tools.items():
            identity = self.identities[identity_key]
            record = self.records.get((sample["sha256"], identity_key))
            value = {
                "basis": "local_bytes",
                "tool_identity": identity_key,
                "version": identity.get("version"),
                "status": "pending",
            }
            if record is not None:
                value.update(
                    {
                        k: record[k]
                        for k in (
                            "status",
                            "basis",
                            "elapsed_ms",
                            "exit_code",
                            "observed_at",
                            "output_truncated",
                        )
                        if k in record
                    }
                )
                for field in ("stdout", "stderr", "error"):
                    if field in record:
                        value[field] = re.sub(
                            r'/[^\s"\'<>]*?/objects/[0-9a-f]{2}/[0-9a-f]{64}',
                            "sha256:" + sample["sha256"],
                            record[field],
                        )
                value["raw_record_sha256"] = hashlib.sha256(
                    json.dumps(record, sort_keys=True).encode()
                ).hexdigest()
            result[tool] = value
        for field in ("magika", "magic", "trid", "ssdeep"):
            raw = sample.get("vt_markings", {}).get(field)
            if raw is not None:
                result["vt." + field] = {
                    "basis": "stored_vt_metadata",
                    "status": "recorded",
                    "version": None,
                    "raw": raw,
                }
        return result

    def public_tools(self):
        return {
            key: {
                **{
                    k: self.identities[key][k]
                    for k in (
                        "id",
                        "version",
                        "license",
                        "source_revision",
                        "scope",
                        "runner_sha256",
                    )
                    if k in self.identities[key]
                },
                "artifact_sha256": [
                    {"name": Path(path).name, "sha256": sha}
                    for path, sha in self.identities[key].get("artifact_sha256", {}).items()
                ],
            }
            for key in self.tools.values()
        }
