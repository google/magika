# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""NIfTI-1 and NIfTI-2 images: header magic, dimensions, datatype and data extent."""

import struct

from ..contract import Observation

FAMILY = "data"
FORMAT_IDS = ("nifti",)
SCOPE = "sizeof_hdr and magic for NIfTI-1 (348) or NIfTI-2 (540), dim and bitpix, vox_offset, image data extent equal to the file for single-file images or header-only for paired images; voxels not interpreted"
BITS = {
    2: 8,
    4: 16,
    8: 32,
    16: 32,
    32: 64,
    64: 64,
    128: 24,
    256: 8,
    512: 16,
    768: 32,
    1024: 64,
    1280: 64,
    1536: 128,
    1792: 128,
    2048: 256,
    2304: 32,
}


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if len(data) < 348:
        return None
    if data[344:348] in (b"n+1\0", b"ni1\0"):
        size, magic, order = 348, data[344:348], "<"
        if struct.unpack_from("<i", data)[0] != 348:
            order = ">"
        if struct.unpack_from(order + "i", data)[0] != 348:
            return Observation("fail", "sizeof_hdr is not 348", "nifti")
        dims = struct.unpack_from(order + "8h", data, 40)
        datatype, bitpix = struct.unpack_from(order + "hh", data, 70)
        offset = int(struct.unpack_from(order + "f", data, 108)[0])
    elif len(data) >= 540 and data[4:8] in (b"n+2\0", b"ni2\0"):
        size, magic, order = 540, data[4:8], "<"
        if struct.unpack_from("<i", data)[0] != 540:
            order = ">"
        if struct.unpack_from(order + "i", data)[0] != 540:
            return Observation("fail", "sizeof_hdr is not 540", "nifti")
        datatype, bitpix = struct.unpack_from(order + "hh", data, 12)
        dims = struct.unpack_from(order + "8q", data, 16)
        offset = struct.unpack_from(order + "q", data, 168)[0]
    else:
        return None
    rank = dims[0]
    if not 1 <= rank <= 7 or any(d < 1 for d in dims[1 : rank + 1]):
        return Observation("fail", "Invalid dim array", "nifti")
    if datatype in BITS and BITS[datatype] != bitpix:
        return Observation("fail", "bitpix disagrees with datatype", "nifti")
    if bitpix <= 0 or bitpix % 8:
        return Observation("fail", "Invalid bitpix", "nifti")
    if magic in (b"ni1\0", b"ni2\0"):
        if len(data) != size and len(data) != size + 4:
            return Observation("fail", "Header-only image with extra bytes", "nifti")
        return Observation(
            "pass", f"NIfTI-{1 if size == 348 else 2} paired header", "nifti", ("paired",)
        )
    voxels = 1
    for dim in dims[1 : rank + 1]:
        voxels *= dim
    extent = voxels * bitpix // 8
    if offset < size + 4 or offset + extent != len(data):
        return Observation(
            "fail", f"vox_offset {offset} plus {extent} bytes of voxels differs from file", "nifti"
        )
    return Observation(
        "pass", f"NIfTI-{1 if size == 348 else 2}: {rank}-D image data sized exactly", "nifti"
    )
