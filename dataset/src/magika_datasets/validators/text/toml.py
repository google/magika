# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""TOML syntax validation using Python's standard-library parser."""

import tomllib

from ..contract import Observation

FAMILY = "text"
FORMAT_IDS = ("toml",)
SCOPE = "Complete UTF-8 TOML parse; no application-schema validation"
CONTEXT_REQUIRED = True
REQUIRES_HINT = True


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    try:
        parsed = tomllib.loads(data.decode("utf-8"))
    except RecursionError:
        return Observation("inconclusive", "Parser recursion limit reached", "toml")
    except (UnicodeError, ValueError):
        return Observation("fail", "Invalid UTF-8 or TOML syntax", "toml")
    if not parsed:
        return Observation(
            "inconclusive", "Empty/comment-only document does not establish TOML identity", "toml"
        )
    return Observation(
        "pass", "Complete TOML document parsed; application schema not checked", "toml"
    )
