# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Windows shell links: header, ID list, link info, string data and extra data blocks."""

import struct

from ..contract import Observation

FAMILY = "application"
FORMAT_IDS = ("lnk",)
SCOPE = "ShellLinkHeader size and CLSID, LinkTargetIDList item bounds, LinkInfo offsets, counted StringData per LinkFlags, ExtraData block sizes and terminal block at EOF; targets never resolved"
CLSID = b"\x01\x14\x02\x00\x00\x00\x00\x00\xc0\x00\x00\x00\x00\x00\x00\x46"
ENVIRONMENT = 0xA0000001


class Malformed(Exception):
    pass


def idlist(data: bytes, offset: int) -> int:
    size = struct.unpack_from("<H", data, offset)[0]
    end = offset + 2 + size
    if end > len(data):
        raise Malformed("IDList outside file")
    position = offset + 2
    for _ in range(4096):
        if position + 2 > end:
            raise Malformed("IDList missing terminator")
        item = struct.unpack_from("<H", data, position)[0]
        if item == 0:
            if position + 2 != end:
                raise Malformed("IDList size differs from items")
            return end
        if item < 2 or position + item > end:
            raise Malformed("ItemID outside IDList")
        position += item
    raise Malformed("ItemID budget exceeded")


def linkinfo(data: bytes, offset: int) -> tuple[int, bool]:
    size, header, flags = struct.unpack_from("<III", data, offset)
    if size < 0x1C or header < 0x1C or header > size or offset + size > len(data):
        raise Malformed("LinkInfo outside file")
    offsets = struct.unpack_from("<IIII", data, offset + 12)
    if any(value and value >= size for value in offsets):
        raise Malformed("LinkInfo offset outside block")
    return offset + size, bool(flags & 2)


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if len(data) < 0x4C or data[:4] != b"L\0\0\0" or data[4:20] != CLSID:
        return None
    flags = struct.unpack_from("<I", data, 20)[0]
    unicode = bool(flags & 0x80)
    tags = []
    try:
        offset = 0x4C
        if flags & 0x01:
            offset = idlist(data, offset)
        if flags & 0x02:
            offset, network = linkinfo(data, offset)
            if network:
                tags.append("network_target")
        for bit, tag in (
            (0x04, None),
            (0x08, None),
            (0x10, None),
            (0x20, "has_arguments"),
            (0x40, "has_icon_location"),
        ):
            if flags & bit:
                count = struct.unpack_from("<H", data, offset)[0]
                offset += 2 + count * (2 if unicode else 1)
                if offset > len(data):
                    raise Malformed("StringData outside file")
                if tag:
                    tags.append(tag)
        for _ in range(4096):
            if offset + 4 > len(data):
                raise Malformed("Missing terminal block")
            size = struct.unpack_from("<I", data, offset)[0]
            if size < 4:
                offset += 4
                break
            if size < 8 or offset + size > len(data):
                raise Malformed("ExtraData block outside file")
            if struct.unpack_from("<I", data, offset + 4)[0] == ENVIRONMENT:
                tags.append("has_environment")
            offset += size
        else:
            raise Malformed("ExtraData block budget exceeded")
        if offset != len(data):
            raise Malformed("Trailing bytes after terminal block")
    except Malformed as error:
        return Observation("fail", str(error), "lnk")
    except struct.error:
        return Observation("fail", "Truncated structure", "lnk")
    if unicode:
        tags.append("unicode")
    return Observation(
        "pass",
        "Shell link header, lists, strings and extra data blocks bounded",
        "lnk",
        tuple(tags),
    )
