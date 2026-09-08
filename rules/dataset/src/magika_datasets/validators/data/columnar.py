# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Parquet, ORC and Arrow IPC files inspected with pyarrow metadata readers in a subprocess."""

import io
import json
import sys

from .._shared import isolated
from ..contract import Observation

FAMILY = "data"
FORMAT_IDS = ("parquet", "orc", "arrow")
SCOPE = "Parquet: PAR1 magics, footer length, metadata parsed by pyarrow, every column chunk inside the file. ORC: postscript and footer parsed by pyarrow, stripes inside the file. Arrow IPC: ARROW1 magics, footer parsed, every record batch read. Ten-second subprocess; values not interpreted"


def kind_of(data: bytes) -> str | None:
    if data[:4] == b"PAR1":
        return "parquet"
    if data[:6] == b"ARROW1":
        return "arrow"
    if data[:3] == b"ORC":
        return "orc"
    return None


def tail_ok(data: bytes, kind: str) -> bool:
    if kind == "parquet":
        return data[-4:] == b"PAR1"
    if kind == "arrow":
        return data.rstrip(b"\0")[-6:] == b"ARROW1"
    return len(data) > 4 and data[-4:-1] == b"ORC"  # postscript magic then its length byte


def inspect(data: bytes, kind: str) -> tuple[str, str]:
    import pyarrow as pa

    stream = io.BytesIO(data)
    try:
        if kind == "parquet":
            import pyarrow.parquet as pq

            footer = int.from_bytes(data[-8:-4], "little")
            if footer + 8 > len(data):
                return "fail", "Footer length exceeds file"
            metadata = pq.read_metadata(stream)
            codecs = set()
            for group in range(metadata.num_row_groups):
                row_group = metadata.row_group(group)
                for column in range(row_group.num_columns):
                    chunk = row_group.column(column)
                    start = min(
                        o
                        for o in (chunk.dictionary_page_offset, chunk.data_page_offset)
                        if o is not None
                    )
                    if start + chunk.total_compressed_size > len(data) - footer - 8:
                        return "fail", f"Row group {group} column {column} outside data area"
                    codecs.add(chunk.compression.lower())
            return (
                "pass",
                f"{metadata.num_row_groups} row groups and {metadata.num_columns} columns bounded; codecs {','.join(sorted(codecs)) or 'none'}",
            )
        if kind == "orc":
            import pyarrow.orc as orc

            reader = orc.ORCFile(stream)
            if (
                reader.content_length
                + reader.file_footer_length
                + reader.file_postscript_length
                + 1
                > len(data)
            ):
                return "fail", "Footer and content lengths exceed file"
            return (
                "pass",
                f"{reader.nstripes} stripes and {reader.nrows} rows; compression {reader.compression.lower()}",
            )
        reader = pa.ipc.open_file(stream)
        for index in range(reader.num_record_batches):
            reader.get_batch(index)
        return (
            "pass",
            f"{reader.num_record_batches} record batches read; {len(reader.schema)} fields",
        )
    except (pa.ArrowInvalid, pa.ArrowIOError, OSError, ValueError, IndexError) as error:
        return "fail", f"pyarrow rejected the file: {str(error)[:80]}"


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    kind = kind_of(data)
    if kind is None:
        return None
    if not tail_ok(data, kind):
        if kind == "orc" and "orc" not in hints:
            return None  # three letters also open ordinary text; without the postscript magic there is no ORC evidence
        return Observation("fail", "Trailing magic missing: truncated or trailing bytes", kind)
    status, detail = isolated.run(__name__, kind, data)
    tags = ()
    if status == "pass" and "codecs " in detail:
        tags = tuple(
            sorted(
                c
                for c in detail.rsplit("codecs ", 1)[1].split(",")
                if c not in ("none", "uncompressed")
            )
        )
    elif status == "pass" and "compression " in detail:
        codec = detail.rsplit("compression ", 1)[1]
        tags = (codec,) if codec not in ("uncompressed", "none") else ()
    return Observation(status, detail, kind, tags)


if __name__ == "__main__":
    payload = sys.stdin.buffer.read(16 * 1024 * 1024 + 1)
    print(json.dumps(inspect(payload, sys.argv[1])))
