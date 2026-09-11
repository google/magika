# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Bounded Pillow decoder subprocess, isolated from the dataset writer."""

import io
import json
import struct
import subprocess
import sys
import warnings

SCOPE = "Pillow 12.3.0 verify and all-frame decoding; max 256 frames, 32M pixels/frame, 256M total pixels, ten seconds; PSD composite only; decoder acceptance, not all metadata semantics"
FRAMES = 256
PIXELS = 32_000_000
TOTAL = 256_000_000
SECONDS = 10


def decode(data: bytes, kind: str) -> tuple[str, str]:
    try:
        result = subprocess.run(
            [sys.executable, "-m", __name__, kind],
            input=data,
            stdout=subprocess.PIPE,
            stderr=subprocess.DEVNULL,
            timeout=SECONDS,
        )
    except subprocess.TimeoutExpired:
        return "inconclusive", f"Decoder exceeded {SECONDS}-second limit"
    if result.returncode:
        return "inconclusive", "Decoder process failed"
    try:
        status, detail = json.loads(result.stdout)
        if status not in {"pass", "fail", "inconclusive"}:
            raise ValueError("Unknown decoder status")
        return status, detail
    except (ValueError, TypeError):
        return "inconclusive", "Decoder returned an invalid observation"


def inspect(data: bytes, kind: str) -> tuple[str, str]:
    from PIL import Image, ImageFile, features

    codec = {"JPEG2000": "jpg_2000", "AVIF": "avif", "WEBP": "webp"}.get(kind)
    if codec and not features.check(codec):
        return "inconclusive", f"Required codec unavailable: {codec}"
    Image.MAX_IMAGE_PIXELS = PIXELS
    ImageFile.LOAD_TRUNCATED_IMAGES = False
    warnings.simplefilter("error", Image.DecompressionBombWarning)
    try:
        with Image.open(io.BytesIO(data), formats=[kind]) as image:
            image.verify()
        if kind in {"ICO", "CUR"}:
            count = int.from_bytes(data[4:6], "little")
            if not count or 6 + 16 * count > len(data):
                return "fail", "Invalid icon directory"
            if count > 16:
                return "inconclusive", "Icon directory exceeds 16 images"
            for index in range(count):
                entry = data[6 + 16 * index : 22 + 16 * index]
                size, offset = struct.unpack_from("<II", entry, 8)
                if not size or offset < 6 + 16 * count or offset + size > len(data):
                    return "fail", "Icon image outside file bounds"
                # Decode each entry in isolation, including duplicate dimensions and
                # cursor entries; Pillow otherwise selects only one image per size.
                single = (
                    data[:4]
                    + b"\1\0"
                    + entry[:12]
                    + struct.pack("<I", 22)
                    + data[offset : offset + size]
                )
                with Image.open(io.BytesIO(single), formats=[kind]) as frame:
                    if frame.width * frame.height > 4_000_000:
                        return "inconclusive", "Icon pixel budget exceeded"
                    frame.load()
            return "pass", f"{count} icon images verified and decoded by Pillow 12.3.0"
        with Image.open(io.BytesIO(data), formats=[kind]) as image:
            total = 0
            limit = 1 if kind == "PSD" else FRAMES  # PSD layers may use codecs Pillow lacks
            for frame in range(limit + 1):
                if frame == limit:
                    if kind == "PSD":
                        return (
                            "pass",
                            "Composite image verified and decoded by Pillow 12.3.0; layers not decoded",
                        )
                    return "inconclusive", "Frame budget exceeded"
                if frame:  # frame 0 is always decoded; layerless PSDs cannot seek at all
                    try:
                        image.seek(frame)
                    except EOFError:
                        return "pass", f"{frame} frames verified and decoded by Pillow 12.3.0"
                pixels = image.width * image.height
                total += pixels
                if pixels > PIXELS or total > TOTAL:
                    return "inconclusive", "Pixel budget exceeded"
                image.load()
    except (Image.DecompressionBombError, Image.DecompressionBombWarning):
        return "inconclusive", "Pixel budget exceeded"
    except (OSError, ValueError, SyntaxError, EOFError):
        return "fail", "Pillow rejected image structure or compressed pixels"
    return "inconclusive", "Decoder did not establish completion"


if __name__ == "__main__":
    data = sys.stdin.buffer.read(16 * 1024 * 1024 + 1)
    result = (
        ("inconclusive", "Input limit exceeded")
        if len(data) > 16 * 1024 * 1024
        else inspect(data, sys.argv[1])
    )
    print(json.dumps(result))
