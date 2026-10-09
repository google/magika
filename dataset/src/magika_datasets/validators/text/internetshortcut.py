# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Windows .url internet shortcuts: INI with an [InternetShortcut] section holding URL."""

import configparser

from ..contract import Observation

FAMILY = "text"
FORMAT_IDS = ("internetshortcut",)
SCOPE = "INI text parsed strictly, an [InternetShortcut] section with a URL key; other sections such as the GUID property stores allowed; nothing fetched"


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    head = data.lstrip(b"\xef\xbb\xbf\r\n ")
    if (
        not head.startswith(b"[")
        or b"[InternetShortcut]" not in head[:4096]
        and b"[internetshortcut]" not in head[:4096].lower()
    ):
        return None
    try:
        text = head.decode("utf-8")
    except UnicodeDecodeError:
        text = head.decode("latin-1")
    parser = configparser.RawConfigParser(strict=False, interpolation=None, allow_no_value=True)
    parser.optionxform = str
    try:
        parser.read_string(text)
    except configparser.Error as error:
        return Observation(
            "fail", f"configparser: {str(error).splitlines()[0][:80]}", "internetshortcut"
        )
    section = next((s for s in parser.sections() if s.lower() == "internetshortcut"), None)
    if section is None or not any(k.lower() == "url" for k in parser[section]):
        return Observation(
            "fail", "No [InternetShortcut] section with a URL key", "internetshortcut"
        )
    return Observation(
        "pass",
        f"[InternetShortcut] with URL; {len(parser.sections())} sections",
        "internetshortcut",
    )
