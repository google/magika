import struct

from magika_datasets.validators.archive import cab


def cabinet(payload: bytes = b"hello cab", bad_checksum: bool = False, extra: bytes = b"") -> bytes:
    body = payload
    header_size = 36
    folder = struct.pack("<IHH", 0, 1, 0)
    name = b"a.txt\0"
    file = struct.pack("<IIHHHH", len(payload), 0, 0, 0, 0, 0x20) + name
    files_offset = header_size + len(folder)
    data_start = files_offset + len(file)
    csum = cab.checksum(struct.pack("<HH", len(body), len(body)), cab.checksum(body, 0))
    if bad_checksum:
        csum ^= 1
    block = struct.pack("<IHH", csum, len(body), len(body)) + body
    folder = struct.pack("<IHH", data_start, 1, 0)
    total = data_start + len(block) + len(extra)
    header = b"MSCF" + struct.pack(
        "<IIIIIBBHHHHH", 0, total, 0, files_offset, 0, 3, 1, 1, 1, 0, 1234, 0
    )
    return header + folder + file + block + extra


def test_well_formed_cabinet_passes():
    result = cab.validate(cabinet(), frozenset())
    assert result.status == "pass" and "none" in result.tags


def test_checksum_mismatch_fails():
    assert cab.validate(cabinet(bad_checksum=True), frozenset()).status == "fail"


def test_size_mismatch_fails():
    data = cabinet()
    assert cab.validate(data + b"x", frozenset()).status == "fail"


def test_bytes_after_last_block_fail():
    assert cab.validate(cabinet(extra=b"\0\0"), frozenset()).status == "fail"


def test_file_extent_beyond_folder_fails():
    data = bytearray(cabinet())
    files_offset = struct.unpack_from("<I", data, 16)[0]
    struct.pack_into("<I", data, files_offset, 999)
    assert cab.validate(bytes(data), frozenset()).status == "fail"


def test_authenticode_signature_after_the_cabinet_is_accounted():
    plain = cabinet()
    signature = b"\x30\x82\x00\x04abcd"
    reserve = struct.pack("<HHIIII", 0, 0x10, len(plain) + 24, len(signature), 0, 0)
    data = bytearray(plain[:36] + struct.pack("<HBB", 20, 0, 0) + reserve + plain[36:] + signature)
    struct.pack_into("<I", data, 8, len(plain) + 24)
    struct.pack_into("<I", data, 16, struct.unpack_from("<I", data, 16)[0] + 24)
    struct.pack_into("<H", data, 30, 4)
    struct.pack_into("<I", data, 60, struct.unpack_from("<I", data, 60)[0] + 24)  # folder start
    result = cab.validate(bytes(data), frozenset())
    assert result.status == "pass" and "signed" in result.tags
    data[-1] ^= 1
    assert cab.validate(bytes(data), frozenset()).status == "pass"  # blob contents are not verified
    struct.pack_into("<I", data, 48, len(signature) + 1)
    assert cab.validate(bytes(data), frozenset()).status == "fail"
