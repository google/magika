# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""NumPy .npy arrays: header dictionary, dtype size arithmetic and payload length; no numpy import."""

import ast
import re
import struct

from ..contract import Observation

FAMILY = "data"
FORMAT_IDS = ("npy",)
SCOPE = "Magic and version, header length, header dictionary parsed with ast.literal_eval, dtype itemsize derived from the descr grammar (including structured dtypes), shape product times itemsize equal to the payload; object arrays inconclusive"
SIMPLE = re.compile(r"^[<>|=]?([?bBiufcmMSaUV])(\d*)$")


class Unsized(Exception):
    pass


def itemsize(descr) -> int:
    if isinstance(descr, str):
        match = SIMPLE.match(descr)
        if not match:
            if descr.lstrip("<>|=") == "O":
                raise Unsized("object")
            raise ValueError(f"Unknown dtype {descr!r}")
        kind, size = match.groups()
        if kind in "?b B".replace(" ", ""):
            return 1
        if kind == "U":
            return 4 * int(size or 0)
        if kind in "SaV":
            return int(size or 0)
        return int(size or {"i": 8, "u": 8, "f": 8, "c": 16, "m": 8, "M": 8}[kind])
    if isinstance(descr, list):
        total = 0
        for field in descr:
            if not isinstance(field, tuple) or len(field) not in (2, 3):
                raise ValueError("Malformed structured dtype")
            size = itemsize(field[1])
            if len(field) == 3:
                for dim in field[2]:
                    size *= int(dim)
            total += size
        return total
    raise ValueError("Unknown dtype description")


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if not data.startswith(b"\x93NUMPY"):
        return None
    if len(data) < 10:
        return Observation("fail", "Truncated header", "npy")
    major = data[6]
    if major == 1:
        length, offset = struct.unpack_from("<H", data, 8)[0], 10
    elif major in (2, 3):
        length, offset = struct.unpack_from("<I", data, 8)[0], 12
    else:
        return Observation("fail", f"Unknown npy version {major}", "npy")
    if offset + length > len(data):
        return Observation("fail", "Header length exceeds file", "npy")
    try:
        header = ast.literal_eval(
            data[offset : offset + length].decode("utf-8" if major == 3 else "latin-1").strip()
        )
        if not isinstance(header, dict) or set(header) != {"descr", "fortran_order", "shape"}:
            raise ValueError("Header keys")
        shape = header["shape"]
        if not isinstance(shape, tuple) or any(not isinstance(d, int) or d < 0 for d in shape):
            raise ValueError("Shape")
        size = itemsize(header["descr"])
    except Unsized:
        return Observation("inconclusive", "Object arrays are pickled; not sized", "npy")
    except (ValueError, SyntaxError, TypeError, KeyError, UnicodeError) as error:
        return Observation("fail", f"Invalid header: {error}", "npy")
    expected = size
    for dim in shape:
        expected *= dim
    payload = len(data) - offset - length
    if payload != expected:
        return Observation(
            "fail", f"Payload is {payload} bytes; dtype and shape need {expected}", "npy"
        )
    return Observation(
        "pass", f"npy v{major}: shape {shape} of {size}-byte items sized exactly", "npy"
    )


def payload_matches(data: bytes) -> bool:
    """For the npz probe: a member is a complete, exactly sized npy array."""
    observation = validate(data, frozenset())
    return observation is not None and observation.status == "pass"
