# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Advanced Systems Format, and the WMA and WMV files it carries, named by stream codec."""

import struct
import uuid

from ..contract import Observation

FAMILY = "media"
FORMAT_IDS = ("asf", "wma", "wmv")
SCOPE = "Header object GUID and size, sub-object count and bounds, file properties present (stream properties tagged when absent), data object and trailing index objects tiling the file; named wmv when a video stream uses a Windows Media Video codec (WMV1-3, WMVA, WVP2, WMVP, WVC1, MSS1-2), wma when every stream is audio in a Windows Media Audio codec (0x0160-0x0163, 0x000A), asf otherwise; packets not decoded"
HEADER = "75B22630-668E-11CF-A6D9-00AA0062CE6C"
DATA = "75B22636-668E-11CF-A6D9-00AA0062CE6C"
FILE_PROPERTIES = "8CABDCA1-A947-11CF-8EE4-00C00C205365"
STREAM_PROPERTIES = "B7DC0791-A9B7-11CF-8EE6-00C00C205365"
AUDIO = "F8699E40-5B4D-11CF-A8FD-00805F5C442B"
VIDEO = "BC19EFC0-5B4D-11CF-A8FD-00805F5C442B"
WMA_CODECS = {0x0160, 0x0161, 0x0162, 0x0163, 0x000A}
WMV_CODECS = {b"WMV1", b"WMV2", b"WMV3", b"WMVA", b"WVP2", b"WMVP", b"WVC1", b"MSS1", b"MSS2"}
OBJECTS = 4096


def objects(data: bytes, start: int, end: int):
    offset, count = start, 0
    while offset < end:
        count += 1
        if count > OBJECTS:
            raise ValueError("Object budget exceeded")
        if offset + 24 > end:
            raise ValueError("Truncated object header")
        kind = uuid.UUID(bytes_le=data[offset : offset + 16])
        size = struct.unpack_from("<Q", data, offset + 16)[0]
        if size < 24 or offset + size > end:
            raise ValueError("Object size outside its container")
        yield kind, offset, size
        offset += size


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if len(data) < 30 or data[:16] != uuid.UUID(HEADER).bytes_le:
        return None
    try:
        walked = list(objects(data, 0, len(data)))
        _, offset, size = walked[0]
        count = struct.unpack_from("<I", data, offset + 24)[0]
        subs = list(objects(data, offset + 30, offset + size))
        if len(subs) != count:
            raise ValueError("Header sub-object count differs from declared")
        kinds = {str(k).upper() for k, _, _ in subs}
        if FILE_PROPERTIES not in kinds:
            raise ValueError("Missing file properties object")
        if len(walked) < 2 or str(walked[1][0]).upper() != DATA:
            raise ValueError("Data object does not follow the header")
    except ValueError as error:
        status = "inconclusive" if "budget" in str(error) else "fail"
        return Observation(status, str(error), "asf")
    tags = () if STREAM_PROPERTIES in kinds else ("no_stream_properties",)
    streams = [codec(data, o, s) for k, o, s in subs if str(k).upper() == STREAM_PROPERTIES]
    detail = f"{len(walked)} top-level ASF objects and {count} header objects bounded"
    if any(kind == "video" and name in WMV_CODECS for kind, name in streams):
        return Observation("pass", f"{detail}; Windows Media Video stream", "wmv", tags)
    if streams and all(kind == "audio" and name in WMA_CODECS for kind, name in streams):
        return Observation("pass", f"{detail}; Windows Media Audio streams only", "wma", tags)
    # The container is proven, but other codecs are also carried in .wma and .wmv files.
    return Observation("pass", detail, "asf", tags, generic=True)


def codec(data: bytes, offset: int, size: int) -> tuple[str, object]:
    """(stream kind, codec) from a stream properties object; unreadable fields give None."""
    kind = str(uuid.UUID(bytes_le=data[offset + 24 : offset + 40])).upper()
    specific = offset + 78  # after type GUIDs, time offset, lengths, flags and reserved
    end = offset + size
    if kind == AUDIO and specific + 2 <= end:
        return "audio", struct.unpack_from("<H", data, specific)[0]  # WAVEFORMATEX tag
    if kind == VIDEO and specific + 31 <= end:
        return "video", data[specific + 27 : specific + 31]  # BITMAPINFOHEADER compression
    return "other", None
