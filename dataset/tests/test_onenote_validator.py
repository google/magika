import struct
import uuid

from magika_datasets.validators.office import onenote


def build(expected: int | None = None) -> bytes:
    data = bytearray(2048)
    data[0:16] = uuid.UUID("{7B5C52E4-D88C-4DA7-AEB1-5378D02996D3}").bytes_le
    data[48:64] = onenote.FORMAT.bytes_le
    struct.pack_into("<Q", data, 196, len(data) if expected is None else expected)
    struct.pack_into("<QI", data, 172, 1024, 512)  # file node list root
    struct.pack_into("<QI", data, 160, 0xFFFFFFFFFFFFFFFF, 0)  # nil transaction log
    return bytes(data)


def test_section_passes():
    result = onenote.validate(build(), frozenset())
    assert result.status == "pass" and result.tags == ("section",)


def test_length_mismatch_and_bad_reference_fail():
    assert onenote.validate(build(expected=100), frozenset()).status == "fail"
    data = bytearray(build())
    struct.pack_into("<QI", data, 172, 4000, 512)
    assert onenote.validate(bytes(data), frozenset()).status == "fail"
