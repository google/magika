import struct
import zlib

from magika_datasets.validators.archive import mscompress


def lzss_literals(payload: bytes) -> bytes:
    out = b""
    for index in range(0, len(payload), 8):
        chunk = payload[index : index + 8]
        out += bytes([(1 << len(chunk)) - 1]) + chunk
    return out


def szdd(payload: bytes = b"hello, expanded world", declared: int | None = None) -> bytes:
    declared = len(payload) if declared is None else declared
    return b"SZDD\x88\xf0\x27\x33A\x00" + struct.pack("<I", declared) + lzss_literals(payload)


def test_szdd_expands_to_the_declared_length():
    assert mscompress.validate(szdd(), frozenset()).status == "pass"
    assert mscompress.validate(szdd(declared=5), frozenset()).status == "fail"


def test_szdd_match_references_expand():
    stream = b"\x0fabcd" + b"\xf0\xf0"  # 4 literals then a match of 3 at window offset 4080
    data = b"SZDD\x88\xf0\x27\x33A\x00" + struct.pack("<I", 7) + stream
    assert mscompress.validate(data, frozenset()).status == "pass"


def test_kwaj_mszip_and_stored():
    payload = b"kwaj payload " * 10
    deflated = zlib.compress(payload)[2:-4]
    block = struct.pack("<H", len(deflated) + 2) + b"CK" + deflated
    header = (
        b"KWAJ\x88\xf0\x27\xd1" + struct.pack("<HHH", 4, 18, 1) + struct.pack("<I", len(payload))
    )
    result = mscompress.validate(header + block, frozenset())
    assert result.status == "pass" and result.tags == ("kwaj",)
    stored = b"KWAJ\x88\xf0\x27\xd1" + struct.pack("<HHH", 0, 14, 0) + b"raw"
    assert mscompress.validate(stored, frozenset()).status == "pass"
    huffman = b"KWAJ\x88\xf0\x27\xd1" + struct.pack("<HHH", 3, 14, 0) + b"???"
    assert mscompress.validate(huffman, frozenset()).status == "inconclusive"
