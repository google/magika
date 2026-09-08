import struct
import zlib

from magika_datasets.validators.geometry import dwg


def r2000(trailing: bytes = b"") -> bytes:
    head = bytearray(0x19)
    head[0:6] = b"AC1015"
    struct.pack_into("<I", head, 0x15, 2)
    records_end = 0x19 + 9 * 2 + 2 + 16
    sections = [(0, records_end, 100), (1, records_end + 100, 50)]
    for index, (number, start, size) in enumerate(sections):
        head += struct.pack("<BII", number, start, size)
    head += struct.pack("<H", dwg.crc16(bytes(head), 0xC0C1)) + dwg.SENTINEL
    return bytes(head) + bytes(150) + trailing


def test_r2000_passes_and_crc_and_tail_are_checked():
    assert dwg.validate(r2000(), frozenset()).tags == ("r2000_layout",)
    assert dwg.validate(r2000(trailing=b"x"), frozenset()).status == "fail"
    data = bytearray(r2000())
    data[0x19] ^= 1
    assert dwg.validate(bytes(data), frozenset()).status == "fail"


def stored(payload: bytes) -> bytes:
    """R2004 LZ77 stream: one initial literal run (short or extended form) then the 0x11 terminator."""
    assert 3 <= len(payload) < 0x0F + 3 + 0xFF
    if len(payload) <= 0x0F + 3:
        return bytes([len(payload) - 3]) + payload + b"\x11"
    return b"\x00" + bytes([len(payload) - 3 - 0x0F]) + payload + b"\x11"


def test_r2004_decompressor_round_trips_literals():
    payload = bytes(range(40))
    assert dwg.Decompressor(stored(payload)).run(len(payload)) == payload


def system_page(payload: bytes, kind: int = dwg.SYSTEM_PAGE) -> bytes:
    compressed = stored(payload)
    header = struct.pack("<IIII", kind, len(payload), len(compressed), 2)
    checksum = zlib.adler32(header + b"\0\0\0\0" + compressed, 0)
    return header + struct.pack("<I", checksum) + compressed


def r2004() -> bytes:
    data = bytearray(0x100)
    data[0:6] = b"AC1024"
    section_map = (
        struct.pack("<I", 1)
        + bytes(16)
        + bytes(8)
        + struct.pack("<I", 1)
        + bytes(84)
        + struct.pack("<IIQ", 3, 8, 0)
    )
    section_page = system_page(section_map, dwg.SECTION_MAP)
    data_page = bytes(64)
    page_map_payload = (
        struct.pack("<iI", 3, len(data_page))
        + struct.pack("<iI", 2, len(section_page) + 20)
        + struct.pack("<iI", 1, 200)
    )
    map_page = system_page(page_map_payload)
    body = data_page + section_page + bytes(20) + map_page
    body += bytes(200 - len(map_page))
    header = bytearray(0x6C)
    header[0:12] = b"AcFssFcAJMB\0"
    struct.pack_into("<I", header, 0x50, 1)
    struct.pack_into("<Q", header, 0x54, len(data_page) + len(section_page) + 20)
    struct.pack_into("<I", header, 0x5C, 2)
    struct.pack_into("<I", header, 0x68, zlib.crc32(bytes(header[:0x68]) + b"\0\0\0\0"))
    data[0x80 : 0x80 + 0x6C] = bytes(a ^ b for a, b in zip(header, dwg.magic_sequence(0x6C)))
    return bytes(data) + body


def test_r2004_layout_passes_and_checksum_is_verified():
    result = dwg.validate(r2004(), frozenset())
    assert result.status == "pass", result
    data = bytearray(r2004())
    data[0x100 + 64 + 30] ^= 1  # inside the section map page
    assert dwg.validate(bytes(data), frozenset()).status == "fail"
