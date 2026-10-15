import struct

from magika_datasets.validators.data import postgres_dump


def integer(value: int) -> bytes:
    return bytes([1 if value < 0 else 0]) + struct.pack("<I", abs(value))


def string(value: bytes | None) -> bytes:
    return integer(-1) if value is None else integer(len(value)) + value


def build(version: tuple[int, int] = (1, 14), broken_offset: bool = False) -> bytes:
    head = b"PGDMP" + bytes([version[0], version[1], 0, 4, 8, 1])
    head += (b"\0" if version >= (1, 15) else integer(0)) + b"".join(integer(0) for _ in range(7))
    head += string(b"db") + string(b"16.0") + string(b"16.0")
    entry = (
        integer(1)
        + integer(1)
        + string(b"1259")
        + string(b"16384")
        + string(b"t")
        + string(b"TABLE DATA")
        + integer(2)
    )
    entry += (
        string(b"") + string(b"") + string(b"COPY t FROM stdin;") + string(b"public") + string(b"")
    )
    if version >= (1, 14):
        entry += string(b"heap")
    if version >= (1, 16):
        entry += integer(114)
    entry += string(b"owner")
    if version < (1, 14):
        entry += string(b"false")
    entry += string(None)
    toc = integer(1) + entry
    offset_field_position = len(head) + len(toc)
    block_offset = offset_field_position + 9
    toc += bytes([2]) + struct.pack("<Q", block_offset + (5 if broken_offset else 0))
    block = bytes([1]) + integer(1) + integer(5) + b"1\t2\n\n" + integer(0)
    return head + toc + block


def test_custom_dump_passes_for_supported_versions():
    for version in ((1, 12), (1, 14), (1, 15), (1, 16)):
        result = postgres_dump.validate(build(version), frozenset())
        assert result.status == "pass", (version, result)


def test_offset_mismatch_and_truncation_fail():
    assert postgres_dump.validate(build(broken_offset=True), frozenset()).status == "fail"
    assert postgres_dump.validate(build()[:-2], frozenset()).status == "fail"
