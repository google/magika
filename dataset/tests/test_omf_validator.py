import struct

from magika_datasets.validators.executable import omf


def record(kind: int, body: bytes, checksum: bool = True) -> bytes:
    head = bytes([kind]) + struct.pack("<H", len(body) + 1) + body
    return head + bytes([(-sum(head)) & 0xFF if checksum else 0])


def module(name: bytes = b"hello.c") -> bytes:
    return (
        record(0x80, bytes([len(name)]) + name)
        + record(0x96, b"\x04CODE")
        + record(0xA0, b"\x01\x00\x00\x90\xc3", checksum=False)
        + record(0x8A, b"\x00")
    )


def test_modules_of_checksummed_records_pass():
    result = omf.validate(module() + module(b"b.c") + b"\0" * 7, frozenset())
    assert result.status == "pass" and result.detail == "2 modules of 8 checksummed records"


def test_a_bad_checksum_fails():
    data = bytearray(module())
    data[16] ^= 1  # inside the LNAMES body
    assert "checksum" in omf.validate(bytes(data), frozenset()).detail


def test_a_module_without_modend_fails():
    assert "MODEND" in omf.validate(module()[:-5], frozenset()).detail


def test_an_unknown_record_type_fails():
    data = module()[:-5] + record(0x42, b"\x00") + record(0x8A, b"\x00")
    assert "Unknown record type" in omf.validate(data, frozenset()).detail
