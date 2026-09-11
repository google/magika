# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0

"""Explicit detector description mappings; these functions do not adjudicate labels."""

import re

from .local_detector_labels import generic_continuation

WIM = re.compile(
    r"Windows imaging (?P<variant>\(WIM\) image|\(ESD\) image|"
    r"\(SWM [1-9][0-9]{0,4} of [1-9][0-9]{0,4}\) image|"
    r"\(Windows provisioning package\))"
    r"(?:, wimlib pipable format)? v(?P<version>1\.13|0\.14)"
    r"(?:, (?:[1-9][0-9]{0,9} images|bootable no\. [1-9][0-9]{0,9}|"
    r"(?:LZMS|LZX|XPRESS2?)(?: compressed)?|compressed|read only|resource only|"
    r"metadata only|reparse point fixup))*"
)

MAYA = re.compile(
    r"Alias Maya (?P<representation>Ascii|Binary) File, version "
    r"(?P<version>[0-9]{1,4}(?:\.[0-9]{1,2})?(?:ff[0-9]{2})?) scene"
    r"(?P<qualifiers>, .+)?"
)

BAM = re.compile(
    r"SAMtools BAM \(Binary Sequence Alignment/Map\)"
    r"(?:, with SAM header(?: version [0-9]+(?:\.[0-9]+)*)?)?"
    r"(?:, with [1-9][0-9]{0,9} reference sequences)?"
)

RZIP = re.compile(r"rzip compressed data - version 2\.1 \([0-9]{1,10} bytes\)")


def description_kind(stdout):
    lines = stdout.splitlines()
    if not lines:
        return None
    first = lines[0]
    if any(
        not generic_continuation(line) and not (RZIP.fullmatch(first) and line == "- " + first)
        for line in lines[1:]
    ):
        return None
    if WIM.fullmatch(first):
        return "wim"
    maya = MAYA.fullmatch(first)
    if maya and (not maya["qualifiers"] or generic_continuation("- " + maya["qualifiers"][2:])):
        return "maya"
    if first in {
        "LRZIP compressed data - version 0.6",
        "LRZIP compressed data - version 0.6, encrypted",
    }:
        return "lrz"
    if RZIP.fullmatch(first):
        return "rzip"
    if first == "Microsoft ASF":
        return "asf"
    if BAM.fullmatch(first):
        return "bam"
    return None
