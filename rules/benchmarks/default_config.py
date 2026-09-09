# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Create the cross-tool benchmark configuration using shipping resource defaults."""

import argparse
import json
from pathlib import Path

from magika_rules_benchmark.comparison import validate_default_policy


def build(args):
    def tool(identifier, adapter, command, artifacts, settings, version_command=None):
        return {
            "id": identifier,
            "adapter": adapter,
            "command": list(map(str, command)),
            "version_command": list(map(str, version_command or [command[0], "--version"])),
            "artifacts": {k: str(v) for k, v in artifacts.items()},
            "settings": {"resource_policy": "tool defaults", **settings},
        }

    binary = args.magika2 / "magika"
    receipt = json.loads((args.magika2 / "build.json").read_text())
    libraries = {
        str(p.relative_to(args.magika2)): p for p in sorted((args.magika2 / "lib").iterdir())
    }
    libraries["build_receipt"] = args.magika2 / "build.json"
    tools = [
        tool(
            "magika1",
            "magika-jsonl",
            [args.magika1, "--jsonl"],
            {"binary": args.magika1},
            {"backend": "CPU", "release": "cli/v1.1.0", "model": "standard_v3_3"},
        )
    ]
    for backend, suffix, label in [
        ("cpu", "", "CPU"),
        ("gpu", "-gpu", "GPU (Metal)"),
        ("auto", "-auto", "Auto"),
    ]:
        for mode, name in [("off", "ml"), ("enforce", "rules")]:
            entry = tool(
                f"magika2{suffix}-{name}",
                "magika-jsonl",
                [binary, "--jsonl", f"--backend={backend}", f"--rules={mode}"],
                libraries | {"native": args.native},
                {"backend": label, "rules": mode, "model": "standard_v3_3"},
            )
            if backend in ("gpu", "auto"):
                entry["settings"].update(
                    startup_backend="CPU",
                    gpu_admission="ready" if backend == "gpu" else "ready with queued work",
                )
            entry["source_revision"] = receipt["source_revision"]
            tools.append(entry)
    entry = tool(
        "magika2-rules-only",
        "magika-jsonl",
        [binary, "--jsonl", "--rules=only"],
        libraries | {"native": args.native},
        {"backend": "CPU", "rules": "only", "model": None},
    )
    entry["source_revision"] = receipt["source_revision"]
    tools.append(entry)
    tools.append(
        tool(
            "libmagic",
            "file-mime",
            [args.file, "-0", "-0", "--mime-type", "--no-pad", "--magic-file", args.magic],
            {"definitions": args.magic},
            {"output": "NUL-framed MIME"},
        )
    )
    tools.append(
        tool(
            "trid",
            "trid",
            [args.python, args.trid, "-d", args.trid_definitions],
            {
                "script": args.trid,
                "definitions": args.trid_definitions,
                "stringzilla": args.stringzilla,
            },
            {"strings": True, "stringzilla": "5.1.2"},
            [args.python, args.trid, "-v"],
        )
    )
    spec = {
        "schema": 1,
        "file_counts": [1, 2, 5, 10, 25, 100, 1000],
        "rule_hit_percentages": [0, 20, 50, 100],
        "runs": 3,
        "warmup": 1,
        "seed": 20260908,
        "quality_chunk_files": 128,
        "rules_reference": "magika2-rules-only",
        "rules_reference_mode": "only",
        "environment": {"MAGIKA_VECTORSCAN_LIBRARY": str(args.native)},
        "tools": tools,
        "dataset": json.loads(args.dataset.read_text()),
    }
    validate_default_policy(spec)
    return spec


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    for name in [
        "magika1",
        "magika2",
        "native",
        "file",
        "magic",
        "python",
        "trid",
        "trid-definitions",
        "stringzilla",
        "dataset",
        "output",
    ]:
        parser.add_argument("--" + name, type=lambda p: Path(p).resolve(), required=True)
    args = parser.parse_args()
    args.output.write_text(json.dumps(build(args), indent=2) + "\n")
