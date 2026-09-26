import struct

from magika_datasets.validators.archive import iso


def record(extent: int, size: int, name: bytes) -> bytes:
    body = (
        bytes([0])
        + struct.pack("<I", extent)
        + struct.pack(">I", extent)
        + struct.pack("<I", size)
        + struct.pack(">I", size)
    )
    body += bytes(7) + b"\x02\0\0" + b"\x01\0\0\x01" + bytes([len(name)]) + name
    length = 1 + len(body) + (len(body) + 1) % 2
    return bytes([length]) + body + bytes(length - 1 - len(body))


def image(blocks: int = 24, extra: bytes = b"") -> bytes:
    data = bytearray(blocks * 2048)
    pvd = bytearray(2048)
    pvd[0:6] = b"\x01CD001"
    pvd[6] = 1
    struct.pack_into("<I", pvd, 80, blocks)
    struct.pack_into(">I", pvd, 84, blocks)
    struct.pack_into("<H", pvd, 128, 2048)
    struct.pack_into("<I", pvd, 132, 10)
    struct.pack_into("<I", pvd, 140, 18)
    pvd[156:190] = record(20, 2048, b"\0")
    data[16 * 2048 : 17 * 2048] = pvd
    data[17 * 2048 : 17 * 2048 + 6] = b"\xffCD001"
    root = record(20, 2048, b"\0") + record(20, 2048, b"\x01") + record(21, 100, b"HELLO.TXT;1")
    data[20 * 2048 : 20 * 2048 + len(root)] = root
    return bytes(data) + extra


def test_image_passes():
    result = iso.validate(image(), frozenset())
    assert result.status == "pass" and "3 root records" in result.detail


def test_truncated_image_fails_and_padding_is_inconclusive():
    assert iso.validate(image()[:-2048], frozenset()).status == "fail"
    assert iso.validate(image(extra=bytes(512)), frozenset()).status == "inconclusive"


def test_extent_outside_volume_fails():
    data = bytearray(image())
    struct.pack_into("<I", data, 20 * 2048 + 34 + 34 + 2, 500)
    assert iso.validate(bytes(data), frozenset()).status == "fail"
