# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Pickle streams walked opcode by opcode with pickletools; nothing is ever loaded."""

import io
import pickletools

from ..contract import Observation

FAMILY = "data"
FORMAT_IDS = ("pickle",)
SCOPE = "Opcode stream disassembled with pickletools.genops: PROTO version, every opcode and argument in bounds, STOP as the final opcode at EOF; callables never resolved or executed, so a hint is required to relabel"
PREFIX_ONLY = True
CONTEXT_REQUIRED = True
OPCODES = 1_000_000
EXECUTING = {"REDUCE", "BUILD", "INST", "OBJ", "NEWOBJ", "NEWOBJ_EX", "GLOBAL", "STACK_GLOBAL"}


def walk(data: bytes) -> tuple[str, str, tuple[str, ...]]:
    protocol, count, names = 0, 0, set()
    last_end = 0
    try:
        for opcode, argument, position in pickletools.genops(io.BytesIO(data)):
            count += 1
            if count > OPCODES:
                return "inconclusive", "Opcode budget exceeded", ()
            names.add(opcode.name)
            if opcode.name == "PROTO":
                protocol = argument
            if opcode.name == "STOP":
                last_end = position + 1
                break
        else:
            return "fail", "Stream ends without STOP", ()
    except (ValueError, IndexError, EOFError, TypeError) as error:
        return "fail", f"pickletools rejected the stream: {str(error)[:60]}", ()
    if last_end != len(data):
        return "fail", "Bytes after the STOP opcode", ()
    tags = [f"protocol_{protocol}"]
    if names & EXECUTING:
        tags.append("has_reduce")
    if "PERSID" in names or "BINPERSID" in names:
        tags.append("has_persistent_id")
    return "pass", f"{count} opcodes walked; protocol {protocol}", tuple(tags)


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if not data:
        return None
    if data[0] == 0x80:
        if len(data) < 2 or data[1] > 5:
            return Observation("fail", "Unknown pickle protocol", "pickle")
    elif data[0] not in b"(cS]}NIFLVUBbdegjlpqrtu0":
        return None  # protocol 0 streams start with a small set of opcodes
    status, detail, tags = walk(data)
    return Observation(status, detail, "pickle", tags)


def stream_ok(data: bytes) -> tuple[bool, str]:
    """For container probes: whether a member is a complete pickle stream."""
    status, detail, _ = walk(data)
    return status == "pass", detail
