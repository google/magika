# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""INI files parsed strictly with configparser; hint-gated because the grammar is permissive."""

import configparser

from ..contract import Observation

FAMILY = "text"
FORMAT_IDS = ("ini",)
SCOPE = "UTF-8 text, configparser strict parse with sections, duplicate sections or options fail, no NUL bytes; a sectionless file is inconclusive; hint required"
REQUIRES_HINT = True
CONTEXT_REQUIRED = True


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if b"\0" in data:
        return Observation("fail", "NUL bytes in text", "ini")
    try:
        text = data.decode("utf-8-sig")
    except UnicodeDecodeError:
        try:
            text = data.decode("latin-1")
        except UnicodeDecodeError:
            return Observation("fail", "Undecodable text", "ini")
    parser = configparser.RawConfigParser(
        strict=True,
        allow_no_value=True,
        interpolation=None,
        delimiters=("=", ":"),
        comment_prefixes=("#", ";"),
        inline_comment_prefixes=None,
    )
    try:
        parser.read_string(text)
    except configparser.MissingSectionHeaderError:
        return Observation("inconclusive", "Key/value lines without a section header", "ini")
    except configparser.Error as error:
        return Observation("fail", f"configparser: {str(error).splitlines()[0][:80]}", "ini")
    except (
        AttributeError,
        ValueError,
        TypeError,
    ) as error:  # configparser itself trips on continuation after a no-value option
        return Observation("fail", f"configparser crashed: {type(error).__name__}", "ini")
    if not parser.sections():
        return Observation("inconclusive", "No sections", "ini")
    return Observation("pass", f"{len(parser.sections())} sections parsed strictly", "ini")
