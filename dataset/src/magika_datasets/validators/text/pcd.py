# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Point Cloud Data files: header fields, point count and ASCII or binary data extent."""

from ..contract import Observation

FAMILY = "text"
FORMAT_IDS = ("pcd",)
SCOPE = "# .PCD header, FIELDS/SIZE/TYPE/COUNT/WIDTH/HEIGHT/POINTS lines consistent, DATA ascii with one line per point or DATA binary with points times point step bytes; binary_compressed inconclusive"


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if not data.startswith(b"# .PCD"):
        return None
    header_end = data.find(b"\nDATA ")
    if header_end < 0:
        return Observation("fail", "Missing DATA line", "pcd")
    data_line_end = data.find(b"\n", header_end + 1)
    if data_line_end < 0:
        return Observation("fail", "DATA line unterminated", "pcd")
    fields = {}
    for line in data[:header_end].decode("latin-1").splitlines():
        if line.startswith("#") or " " not in line:
            continue
        key, value = line.split(" ", 1)
        fields[key] = value.split()
    try:
        sizes = [int(v) for v in fields["SIZE"]]
        counts = [int(v) for v in fields.get("COUNT", ["1"] * len(sizes))]
        points = int(fields["POINTS"][0])
        width, height = int(fields["WIDTH"][0]), int(fields["HEIGHT"][0])
    except (KeyError, ValueError, IndexError):
        return Observation("fail", "Header lacks SIZE, POINTS, WIDTH or HEIGHT", "pcd")
    if width * height != points:
        return Observation("fail", "WIDTH times HEIGHT differs from POINTS", "pcd")
    mode = data[header_end + 6 : data_line_end].strip()
    body = data[data_line_end + 1 :]
    if mode == b"ascii":
        lines = [line for line in body.splitlines() if line.strip()]
        if len(lines) != points:
            return Observation("fail", f"{len(lines)} data lines for {points} points", "pcd")
        return Observation("pass", f"{points} ASCII points", "pcd", ("ascii",))
    if mode == b"binary":
        step = sum(s * c for s, c in zip(sizes, counts))
        if len(body) != points * step:
            return Observation(
                "fail",
                f"Binary data is {len(body)} bytes; {points} points of {step} bytes expected",
                "pcd",
            )
        return Observation("pass", f"{points} binary points of {step} bytes", "pcd", ("binary",))
    return Observation("inconclusive", f"DATA {mode.decode('latin-1')} not sized", "pcd")
