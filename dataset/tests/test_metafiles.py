import struct

from helpers import status

from magika_datasets.validators.image import emf, wmf


def emf_file(records=1, eof=True):
    body = struct.pack("<II", 0x18, 16) + b"\0" * 8 if records else b""  # EMR_SETMAPMODE-ish
    tail = struct.pack("<IIII", 14, 20, 0, 16) + struct.pack("<I", 20) if eof else b""
    total = 88 + len(body) + len(tail)
    header = (
        struct.pack("<II", 1, 88)
        + b"\0" * 32
        + b" EMF"
        + struct.pack(
            "<IIIHHIIIIIII",
            0x10000,
            total,
            records + (1 if eof else 0) + 1,
            1,
            0,
            0,
            0,
            0,
            100,
            100,
            26,
            26,
        )
    )
    return header + body + tail


def test_emf_records_and_eof():
    assert status(emf, emf_file()) == "pass"
    assert status(emf, emf_file(records=0)) == "pass"
    assert status(emf, emf_file(eof=False)) == "fail"
    assert status(emf, emf_file()[:-4]) == "fail"
    assert status(emf, emf_file() + b"\0\0\0\0") == "fail"
    assert status(emf, b"\1\0\0\0" + b"x" * 90) == "not_applicable"


def wmf_file(placeable=True, eof=True):
    records = struct.pack("<IH", 4, 0x0103) + struct.pack("<H", 8)  # META_SETMAPMODE
    if eof:
        records += struct.pack("<IH", 3, 0)
    words = (18 + len(records)) // 2
    header = struct.pack("<HHHIHIH", 1, 9, 0x0300, words, 0, 4, 0)
    data = header + records
    if placeable:
        head = struct.pack("<IH4hHI", 0x9AC6CDD7, 0, 0, 0, 100, 100, 96, 0)
        words16 = struct.unpack("<10H", head)
        checksum = 0
        for word in words16:
            checksum ^= word
        data = head + struct.pack("<H", checksum) + data
    return data


def test_wmf_records_placeable_and_eof():
    observation = wmf.validate(wmf_file(), frozenset())
    assert (observation.status, observation.format_id, observation.tags) == (
        "pass",
        "wmf",
        ("placeable",),
    )
    assert wmf.validate(wmf_file(placeable=False), frozenset()).tags == ()
    assert status(wmf, wmf_file(eof=False)) == "fail"
    assert status(wmf, wmf_file()[:-2]) == "fail"
    broken = bytearray(wmf_file())
    broken[20] ^= 1  # placeable checksum
    assert status(wmf, bytes(broken)) == "fail"
    assert status(wmf, b"\1\0\x09\0\0\3" + b"x" * 14) == "fail"
    assert status(wmf, b"other bytes here") == "not_applicable"
