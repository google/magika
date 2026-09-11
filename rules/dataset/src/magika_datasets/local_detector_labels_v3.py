# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0

"""Explicit detector description mappings; these functions do not adjudicate labels."""

import re

from .local_detector_labels import generic_continuation

PG = re.compile(r"PostgreSQL custom database dump - v1\.(?:[7-9]|1[0-6])-0")

COMPRESS = re.compile(r"compress'd data(?: block compressed)? (?:9|1[0-6]) bits")


def description_kind(stdout):
    lines = stdout.splitlines()
    if not lines or any(not generic_continuation(s) for s in lines[1:]):
        return None
    first = lines[0]
    if PG.fullmatch(first):
        return "postgres_dump"
    if first == "LZ4 compressed data (v1.4+)":
        return "lz4"
    if COMPRESS.fullmatch(first):
        return "unixcompress"
    return None
