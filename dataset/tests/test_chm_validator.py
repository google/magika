import struct

from magika_datasets.validators.application import chm


def encint(value: int) -> bytes:
    out = bytes([value & 0x7F])
    value >>= 7
    while value:
        out = bytes([0x80 | value & 0x7F]) + out
        value >>= 7
    return out


def build(chunk_size: int = 256, bad_entry: bool = False) -> bytes:
    entries = [
        (b"/", 0, 0, 0),
        (b"/index.htm", 0, 0, 40 if not bad_entry else 4000),
        (b"::DataSpace/Storage/MSCompressed/Content", 0, 40, 24),
    ]
    listing = b""
    for name, section, offset, size in entries:
        listing += encint(len(name)) + name + encint(section) + encint(offset) + encint(size)
    free = chunk_size - 0x14 - len(listing)
    pmgl = b"PMGL" + struct.pack("<IIii", free, 0, -1, -1) + listing + bytes(free)
    itsp = bytearray(0x54)
    itsp[0:4] = b"ITSP"
    struct.pack_into("<III", itsp, 4, 1, 0x54, 10)
    struct.pack_into("<I", itsp, 0x10, chunk_size)
    struct.pack_into("<I", itsp, 0x2C, 1)
    directory = bytes(itsp) + pmgl
    section0 = struct.pack("<IIQQ", 0x1FE, 0, 0, 0)  # file size patched below
    header = bytearray(0x60)
    header[0:4] = b"ITSF"
    struct.pack_into("<II", header, 4, 3, 0x60)
    content_offset = 0x60 + len(section0) + len(directory)
    struct.pack_into(
        "<QQQQ", header, 0x38, 0x60, len(section0), 0x60 + len(section0), len(directory)
    )
    struct.pack_into("<Q", header, 0x58, content_offset)
    content = bytes(64)
    total = content_offset + len(content)
    section0 = struct.pack("<IIQQ", 0x1FE, 0, total, 0)
    return bytes(header) + section0 + directory + content


def test_help_archive_passes():
    result = chm.validate(build(), frozenset())
    assert result.status == "pass" and "3 directory entries" in result.detail


def test_size_mismatch_and_bad_entry_fail():
    assert chm.validate(build() + b"\0", frozenset()).status == "fail"
    assert chm.validate(build(bad_entry=True), frozenset()).status == "fail"
