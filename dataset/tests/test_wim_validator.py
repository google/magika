import struct

from magika_datasets.validators.archive import wim


def reshdr(offset: int, size: int, flags: int = 0) -> bytes:
    return struct.pack("<QQ", size | flags << 56, offset) + struct.pack("<Q", size)


def image(compressed_xml: bool = False, extra: bytes = b"") -> bytes:
    xml_data = '<WIM><TOTALBYTES>1</TOTALBYTES><IMAGE INDEX="1"/></WIM>'.encode("utf-16-le")
    blob = b"resource-bytes"
    header_size = 208
    table_offset = header_size + len(blob)
    xml_offset = table_offset + 50
    header = b"MSWIM\0\0\0" + struct.pack("<IIII", header_size, 0x10D00, 2, 32768) + bytes(16)
    header += struct.pack("<HHI", 1, 1, 1)
    header += reshdr(table_offset, 50) + reshdr(
        xml_offset, len(xml_data), 4 if compressed_xml else 0
    )
    header += reshdr(0, 0) + struct.pack("<I", 0) + reshdr(0, 0) + bytes(60)
    assert len(header) == header_size
    entry = reshdr(header_size, len(blob)) + struct.pack("<HI", 1, 1) + bytes(20)
    return header + blob + entry + xml_data + extra


def test_wim_passes():
    result = wim.validate(image(), frozenset())
    assert result.status == "pass" and result.tags == ("compressed",)


def test_wim_trailing_bytes_fail():
    assert wim.validate(image(extra=b"\0\0"), frozenset()).status == "fail"


def test_wim_compressed_xml_is_inconclusive():
    assert wim.validate(image(compressed_xml=True), frozenset()).status == "inconclusive"


def test_wim_resource_outside_file_fails():
    data = bytearray(image())
    struct.pack_into("<Q", data, 208 + 14 + 8, 1 << 30)
    assert wim.validate(bytes(data), frozenset()).status == "fail"


def test_wim_solid_entries_are_not_file_offsets():
    data = bytearray(image())
    entry = 208 + 14
    struct.pack_into("<QQ", data, entry, 1000 | 0x10 << 56, 1 << 40)  # solid, far beyond the file
    result = wim.validate(bytes(data), frozenset())
    assert result.status == "pass" and "solid" in result.tags


def test_wim_trailing_authenticode_blob_is_accounted():
    signature = b"\x30\x06\x02\x01\x05\x04\x01x"

    result = wim.validate(image(extra=signature), frozenset())
    assert result.status == "pass" and "signed" in result.tags
    assert wim.validate(image(extra=signature + b"\0"), frozenset()).status == "fail"
