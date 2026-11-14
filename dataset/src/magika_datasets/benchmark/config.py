# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Write the benchmark configuration: Magika 1, Magika 2's modes, libmagic and TrID at defaults."""

import argparse
import json
from pathlib import Path

from .tools import validate_default_policy

FILE_COUNTS = [1, 2, 5, 10, 25, 100, 1000]
SEED = 20260908


def tool(identifier, adapter, command, artifacts, settings, version_command=None) -> dict:
    return {
        "id": identifier,
        "adapter": adapter,
        "command": list(map(str, command)),
        "version_command": list(map(str, version_command or [command[0], "--version"])),
        "artifacts": {k: str(v) for k, v in artifacts.items()},
        "settings": {"resource_policy": "tool defaults", **settings},
    }


def build(args) -> dict:
    binary = args.magika2 / "magika"
    libraries = {
        str(p.relative_to(args.magika2)): p for p in sorted((args.magika2 / "lib").iterdir())
    }
    tools = [
        tool(
            "magika1",
            "magika-jsonl",
            [args.magika1, "--jsonl"],
            {"binary": args.magika1},
            {"backend": "CPU"},
        ),
    ]
    for backend, suffix, label in [
        ("cpu", "", "CPU"),
        ("gpu", "-gpu", "GPU"),
        ("auto", "-auto", "Auto"),
    ]:
        for mode, name in [("off", "ml"), ("enforce", "rules")]:
            entry = tool(
                f"magika2{suffix}-{name}",
                "magika-jsonl",
                [binary, "--jsonl", f"--backend={backend}", f"--rules={mode}"],
                libraries | {"native": args.native},
                {"backend": label, "rules": mode},
            )
            if backend != "cpu":
                entry["settings"]["startup_backend"] = "CPU"
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
            {"script": args.trid, "definitions": args.trid_definitions},
            {},
            [args.python, args.trid, "-v"],
        )
    )
    spec = {
        "schema": 1,
        "file_counts": FILE_COUNTS,
        "runs": 3,
        "warmup": 1,
        "seed": SEED,
        "environment": {"MAGIKA_VECTORSCAN_LIBRARY": str(args.native)},
        "tools": tools,
    }
    validate_default_policy(spec)
    return spec


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__, prog="magika-datasets benchmark-config")
    for name in [
        "magika1",
        "magika2",
        "native",
        "file",
        "magic",
        "python",
        "trid",
        "trid-definitions",
        "output",
    ]:
        parser.add_argument("--" + name, type=lambda p: Path(p).absolute(), required=True)
    args = parser.parse_args(argv)
    args.output.write_text(json.dumps(build(args), indent=2) + "\n")
