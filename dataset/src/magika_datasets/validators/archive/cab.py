# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Microsoft Cabinet: header, folders, file table and CFDATA blocks with checksums."""

import struct

from ..contract import Observation

FAMILY = "archive"
FORMAT_IDS = ("cab",)
SCOPE = "CFHEADER size against the file (plus an Authenticode signature described by the header reserve), CFFOLDER and CFFILE tables, every folder's CFDATA chain with block checksums, file extents inside their folder's uncompressed size, blocks tiling the file; compressed payloads not expanded"
MAGIC = b"MSCF"
BLOCKS = 65536
COMPRESSION = {0: "none", 1: "mszip", 2: "quantum", 3: "lzx"}


class Malformed(Exception):
    pass


def checksum(block: bytes, seed: int) -> int:
    words = len(block) // 4
    total = seed
    for value in struct.unpack_from(f"<{words}I", block):
        total ^= value
    rest = block[words * 4 :]
    tail = 0
    if len(rest) == 3:
        tail = rest[0] << 16 | rest[1] << 8 | rest[2]
    elif len(rest) == 2:
        tail = rest[0] << 8 | rest[1]
    elif len(rest) == 1:
        tail = rest[0]
    return total ^ tail


def cstring(data: bytes, offset: int) -> int:
    end = data.find(b"\0", offset)
    if end < 0:
        raise Malformed("Unterminated string")
    return end + 1


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if not data.startswith(MAGIC) or len(data) < 36:
        return None
    size, files_offset = struct.unpack_from("<I", data, 8)[0], struct.unpack_from("<I", data, 16)[0]
    minor, major, folders, files, flags = struct.unpack_from("<BBHHH", data, 24)
    signature = 0
    if (
        flags & 4
        and len(data) >= 60
        and data[36:38] == b"\x14\0"
        and struct.unpack_from("<HH", data, 40) == (0, 0x10)
    ):
        # Authenticode: reserve holds the PKCS#7 blob's offset and size past cbCabinet
        signature_offset, signature = struct.unpack_from("<II", data, 44)
        if signature_offset != size or size + signature != len(data):
            return Observation("fail", "Signature offset or size disagrees with the file", "cab")
        if data[size] != 0x30:
            return Observation("fail", "Signature is not a DER SEQUENCE", "cab")
        tags = ["signed"]
    if size + signature != len(data):
        return Observation("fail", f"Header declares {size} bytes, file has {len(data)}", "cab")
    if (major, minor) != (1, 3):
        return Observation("fail", "Unsupported cabinet version", "cab")
    tags = tags if signature else []
    try:
        offset = 36
        header_reserve = folder_reserve = data_reserve = 0
        if flags & 4:
            header_reserve, folder_reserve, data_reserve = struct.unpack_from("<HBB", data, 36)
            offset = 40 + header_reserve
        if flags & 1:
            tags.append("multi_part")
            offset = cstring(data, cstring(data, offset))
        if flags & 2:
            tags.append("multi_part")
            offset = cstring(data, cstring(data, offset))
        folder_table = []
        for _ in range(folders):
            start, count, kind = struct.unpack_from("<IHH", data, offset)
            folder_table.append((start, count, kind))
            offset += 8 + folder_reserve
        if offset > files_offset or files_offset > len(data):
            raise Malformed("File table offset outside header")
        if offset != files_offset:
            raise Malformed("Gap between folder table and file table")
        offset = files_offset
        extents = []
        for _ in range(files):
            length, start, folder = struct.unpack_from("<IIH", data, offset)
            offset = cstring(data, offset + 16)
            extents.append((folder, start, length))
        covered = offset
        expanded = {}
        for index, (start, count, kind) in enumerate(folder_table):
            if start != covered:
                raise Malformed("CFDATA blocks do not follow the previous table or folder")
            if count > BLOCKS:
                raise Malformed("CFDATA budget exceeded")
            position = start
            total = 0
            for _ in range(count):
                stored, compressed, uncompressed = struct.unpack_from("<IHH", data, position)
                body = data[position + 8 + data_reserve : position + 8 + data_reserve + compressed]
                if len(body) != compressed:
                    raise Malformed("CFDATA block outside file")
                if (
                    stored
                    and checksum(
                        data[position + 4 : position + 8 + data_reserve], checksum(body, 0)
                    )
                    != stored
                ):
                    raise Malformed("CFDATA checksum mismatch")
                position += 8 + data_reserve + compressed
                total += uncompressed
            expanded[index] = total
            covered = position
            method = COMPRESSION.get(kind & 0xF, "unknown")
            if method not in tags:
                tags.append(method)
        for folder, start, length in extents:
            if folder >= 0xFFFD:  # continued from or into another cabinet
                continue
            if folder not in expanded or start + length > expanded[folder]:
                raise Malformed("File extent outside its folder's uncompressed data")
    except (struct.error, Malformed) as error:
        return Observation(
            "fail", str(error) if isinstance(error, Malformed) else "Truncated table", "cab"
        )
    if covered != size:
        return Observation("fail", "Bytes after the last CFDATA block", "cab")
    return Observation(
        "pass",
        f"{folders} folders, {files} files; tables, CFDATA chains and block checksums verified",
        "cab",
        tuple(sorted(set(tags))),
    )
