"""Regression test for the web demo's Magika dependency (issue #1613).

A caret on a 0.x version never picks up 1.x, so the demo can silently stay on
an ancient release forever. Keep the declared range and the lockfile honest.
"""

import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


def _version_tuple(value: str) -> tuple[int, int, int]:
    parts = [int(part) for part in re.findall(r"\d+", value)[:3]]
    return tuple(parts + [0, 0, 0])[:3]  # type: ignore[return-value]


def test_website_demo_declares_magika_1_or_later():
    package = json.loads((ROOT / "website-ng" / "package.json").read_text(encoding="utf-8"))

    declared = package["dependencies"]["magika"]

    assert _version_tuple(declared) >= (1, 0, 0), (
        f"website-ng must declare magika >=1.0.0, got {declared!r}"
    )


def test_website_demo_lockfile_resolves_magika_1_or_later():
    lock = json.loads((ROOT / "website-ng" / "package-lock.json").read_text(encoding="utf-8"))

    declared = lock["packages"][""]["dependencies"]["magika"]
    resolved = lock["packages"]["node_modules/magika"]["version"]

    assert _version_tuple(declared) >= (1, 0, 0), declared
    assert _version_tuple(resolved) >= (1, 0, 0), resolved
