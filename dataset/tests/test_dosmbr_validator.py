import struct

from magika_datasets.validators.system import dosmbr


def sector(entries: list[tuple[int, int, int, int]]) -> bytes:
    data = bytearray(512)
    data[0:3] = b"\x33\xc0\x8e"
    for index, (boot, kind, start, size) in enumerate(entries):
        struct.pack_into(
            "<B3sB3sII", data, 446 + 16 * index, boot, b"\0\0\0", kind, b"\0\0\0", start, size
        )
    data[510:512] = b"\x55\xaa"
    return bytes(data)


def test_partition_table_checks():
    assert (
        dosmbr.validate(
            sector([(0x80, 0x83, 2048, 1000), (0, 0x82, 4096, 500)]), frozenset()
        ).status
        == "pass"
    )
    assert (
        dosmbr.validate(
            sector([(0x80, 0x83, 2048, 1000), (0, 0x82, 2500, 500)]), frozenset()
        ).status
        == "fail"
    )
    assert (
        dosmbr.validate(sector([(0x80, 0x83, 1, 1), (0x80, 0x82, 5, 1)]), frozenset()).status
        == "fail"
    )
    assert dosmbr.validate(sector([]) + b"\0", frozenset()) is None
