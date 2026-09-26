# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Extended M3U playlists: header, EXTINF entries followed by locations, HLS tags."""

from ..contract import Observation

FAMILY = "text"
FORMAT_IDS = ("m3u",)
SCOPE = "#EXTM3U header, every #EXTINF followed by a non-tag entry line, other lines either #EXT tags, comments or entries; URLs not fetched"


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    head = data.lstrip(b"\xef\xbb\xbf")
    if not head.startswith(b"#EXTM3U"):
        return None
    try:
        text = head.decode("utf-8")
    except UnicodeDecodeError:
        text = head.decode("latin-1")
    pending, entries, tags = False, 0, 0
    for line in text.splitlines()[1:]:
        stripped = line.strip()
        if not stripped:
            continue
        if stripped.startswith("#EXTINF"):
            if pending:
                return Observation("fail", "Two #EXTINF lines without an entry between them", "m3u")
            pending = True
            tags += 1
        elif stripped.startswith("#"):
            tags += stripped.startswith("#EXT")
        else:
            entries += 1
            pending = False
    if pending:
        return Observation("fail", "#EXTINF without a following entry", "m3u")
    return Observation(
        "pass",
        f"{entries} entries and {tags} extended tags",
        "m3u",
        ("hls",) if "#EXT-X-" in text else (),
    )
