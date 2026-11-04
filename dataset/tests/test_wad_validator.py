import struct

from magika_datasets.validators.application import wad


def build(kind: bytes = b"PWAD", extra: bytes = b"") -> bytes:
    lumps = [(b"MAP01", b""), (b"THINGS", b"t" * 20), (b"VERTEXES", b"v" * 8)]
    body = b""
    directory = b""
    for name, data in lumps:
        directory += struct.pack("<II", 12 + len(body), len(data)) + name.ljust(8, b"\0")
        body += data
    header = kind + struct.pack("<II", len(lumps), 12 + len(body))
    return header + body + directory + extra


def test_wad_passes():
    result = wad.validate(build(), frozenset())
    assert result.status == "pass" and result.tags == ("pwad",)


def test_trailing_bytes_and_truncation_fail():
    assert wad.validate(build(extra=b"zz"), frozenset()).status == "fail"
    assert wad.validate(build()[:-4], frozenset()).status == "fail"
