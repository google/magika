# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Strict UTF-8 JSON container validation using the Python parser."""

import json

from ..contract import Observation

FAMILY = "text"
FORMAT_IDS = ("json",)
PREFIX_ONLY = True  # applicability rests on a short prefix; failures need a hint
SCOPE = "Complete UTF-8 JSON object/array parse; duplicate keys inconclusive; no application-schema validation"
CONTEXT_REQUIRED = True


class DuplicateKey(ValueError):
    pass


def pairs(items):
    result = {}
    for key, value in items:
        if key in result:
            raise DuplicateKey(key)
        result[key] = value
    return result


def reject_constant(value):
    raise ValueError("Non-JSON constant")


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if not data.lstrip().startswith((b"{", b"[")):
        return None
    try:
        json.loads(data.decode("utf-8"), object_pairs_hook=pairs, parse_constant=reject_constant)
    except DuplicateKey:
        return Observation(
            "inconclusive", "Duplicate object keys have ambiguous interpretation", "json"
        )
    except RecursionError:
        return Observation("inconclusive", "Parser recursion limit reached", "json")
    except (UnicodeError, ValueError):
        return Observation("fail", "Invalid UTF-8 or JSON syntax", "json")
    return Observation(
        "pass", "Complete JSON object/array parsed; application type not established", "json"
    )
