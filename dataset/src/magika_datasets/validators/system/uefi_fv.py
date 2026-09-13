# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""UEFI firmware volumes: volume header checksum, block map and the FFS files inside."""

import struct

from ..contract import Observation

FAMILY = "system"
FORMAT_IDS = ("uefi_fv",)
SCOPE = "EFI_FIRMWARE_VOLUME_HEADER at offset 0: _FVH signature, 16-bit header checksum, revision 1 or 2, block map summing to FvLength inside the file; then FFS files walked on 8-byte alignment with header checksums (FFSv2 and large-file FFSv3 headers) and sizes inside the volume until erased space; further volumes in the file walked the same way; section and compression contents not decoded"
SIGNATURE = b"_FVH"
FFS_HEADER, LARGE_HEADER = 24, 32
LARGE_FILE = 0x01
VOLUMES, FILES = 64, 100_000


def volume(data: bytes, start: int) -> tuple[int, int]:
    """(volume length, files walked) for the volume at start; raises ValueError."""
    if data[start + 40 : start + 44] != SIGNATURE:
        raise ValueError(f"No _FVH signature at offset {start}")
    (length,) = struct.unpack_from("<Q", data, start + 32)
    header_length, _, extension, _, revision = struct.unpack_from("<HHHBB", data, start + 48)
    if revision not in (1, 2) or header_length < 56 or header_length % 2:
        raise ValueError("Volume header revision or length invalid")
    if length < header_length or start + length > len(data):
        raise ValueError("Volume extends past the end of the file")
    if sum(struct.unpack_from(f"<{header_length // 2}H", data, start)) & 0xFFFF:
        raise ValueError("Volume header checksum mismatch")
    offset, mapped = start + 56, 0
    while True:
        if offset + 8 > start + header_length:
            raise ValueError("Block map is not terminated inside the header")
        count, size = struct.unpack_from("<II", data, offset)
        offset += 8
        if not count and not size:
            break
        mapped += count * size
    if mapped != length:
        raise ValueError("Block map does not add up to the volume length")
    first = start + header_length
    if extension:
        if extension < header_length or start + extension + 20 > start + length:
            raise ValueError("Extended header outside the volume")
        (extended_size,) = struct.unpack_from("<I", data, start + extension + 16)
        first = start + extension + extended_size
    return length, files(data, first, start + length)


def files(data: bytes, offset: int, end: int) -> int:
    count = 0
    while True:
        offset = (offset + 7) & ~7
        if offset + FFS_HEADER > end:
            return count
        header = data[offset : offset + FFS_HEADER]
        if header == b"\xff" * FFS_HEADER or header == b"\x00" * FFS_HEADER:
            return count  # erased space ends the file list
        attributes = header[19]
        size = int.from_bytes(header[20:23], "little")
        width = FFS_HEADER
        if attributes & LARGE_FILE and size == 0:
            if offset + LARGE_HEADER > end:
                raise ValueError("Large file header truncated")
            (size,) = struct.unpack_from("<Q", data, offset + FFS_HEADER)
            width = LARGE_HEADER
        whole = bytearray(data[offset : offset + width])
        whole[17] = 0  # IntegrityCheck.Checksum.File
        whole[23] = 0  # State
        if sum(whole) & 0xFF:
            raise ValueError(f"FFS file header checksum mismatch at offset {offset}")
        if size < width or offset + size > end:
            raise ValueError(f"FFS file at offset {offset} extends past its volume")
        offset += size
        count += 1
        if count > FILES:
            raise OverflowError("File budget exceeded")


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if len(data) < 56 or data[40:44] != SIGNATURE:
        return None
    try:
        offset, volumes, walked = 0, 0, 0
        while offset < len(data):
            if data[offset + 40 : offset + 44] != SIGNATURE:
                rest = data[offset:]
                if rest.strip(b"\xff") and rest.strip(b"\x00"):
                    raise ValueError(
                        f"Bytes after volume {volumes} are neither a volume nor padding"
                    )
                break
            length, count = volume(data, offset)
            walked += count
            volumes += 1
            if volumes > VOLUMES:
                return Observation("inconclusive", "Volume budget exceeded", "uefi_fv")
            offset += length
    except OverflowError as error:
        return Observation("inconclusive", str(error), "uefi_fv")
    except (ValueError, struct.error) as error:
        return Observation("fail", str(error) or "Truncated header", "uefi_fv")
    return Observation(
        "pass", f"{volumes} firmware volumes and {walked} FFS files verified", "uefi_fv"
    )
