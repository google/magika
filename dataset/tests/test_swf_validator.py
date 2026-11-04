import lzma
import struct
import zlib

from helpers import status

from magika_datasets.validators.media import swf


def body(tags=None):
    rect = bytes(
        [0x08, 0x00, 0x00, 0x00, 0x00]
    )  # nbits=1 -> 5 + 4 bits, padded to 5 bytes? (nbits 1 -> 9 bits -> 2 bytes)
    rect = bytes([0x08, 0x00])
    header = rect + struct.pack("<HH", 0x1800, 1)
    tags = tags if tags is not None else [(9, b"\0\0\0"), (1, b"")]
    encoded = b""
    for code, payload in tags:
        if len(payload) < 63:
            encoded += struct.pack("<H", (code << 6) | len(payload)) + payload
        else:
            encoded += struct.pack("<HI", (code << 6) | 63, len(payload)) + payload
    return header + encoded + struct.pack("<H", 0)


def movie(kind=b"FWS", content=None):
    content = body() if content is None else content
    length = 8 + len(content)
    if kind == b"CWS":
        content = zlib.compress(content)
    elif kind == b"ZWS":
        compressed = lzma.compress(content, format=lzma.FORMAT_ALONE)
        content = struct.pack("<I", len(compressed) - 13) + compressed[:5] + compressed[13:]
    return kind + bytes([10]) + struct.pack("<I", length) + content


def test_swf_signatures_lengths_and_tags():
    for kind, tag in ((b"FWS", ()), (b"CWS", ("compressed_zlib",)), (b"ZWS", ("compressed_lzma",))):
        observation = swf.validate(movie(kind), frozenset())
        assert (observation.status, observation.format_id, observation.tags) == (
            "pass",
            "swf",
            tag,
        ), kind
        assert status(swf, movie(kind)[:-2]) == "fail", kind
    assert status(swf, movie(content=body() + b"\0\0")) == "fail"
    assert status(swf, movie(content=body([(9, b"\0\0\0")])[:-2])) == "fail"  # no End tag
    assert status(swf, movie(content=body([(9, b"x" * 70)]))) == "pass"
    assert status(swf, b"FWS\x0a" + struct.pack("<I", 99) + body()) == "fail"
    assert status(swf, b"XWS" + b"\0" * 20) == "not_applicable"
