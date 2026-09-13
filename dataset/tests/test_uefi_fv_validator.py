import struct

from magika_datasets.validators.system import uefi_fv


def ffs(payload: bytes, large: bool = False) -> bytes:
    width = uefi_fv.LARGE_HEADER if large else uefi_fv.FFS_HEADER
    size = width + len(payload)
    header = bytearray(b"\x11" * 16 + bytes([0, 0xAA, 0x07, uefi_fv.LARGE_FILE if large else 0]))
    header += (0).to_bytes(3, "little") if large else size.to_bytes(3, "little")
    header += b"\xf8"  # state
    if large:
        header += struct.pack("<Q", size)
    checked = bytearray(header)
    checked[17] = checked[23] = 0
    header[16] = (-sum(checked)) & 0xFF
    body = bytes(header) + payload
    return body + b"\xff" * (-len(body) % 8)


def volume(contents: bytes, blocks: int = 2, block: int = 256) -> bytes:
    length = blocks * block
    header = bytearray(b"\0" * 16 + b"\x22" * 16 + struct.pack("<Q", length) + uefi_fv.SIGNATURE)
    header += struct.pack("<IHHHBB", 0x0004FEFF, 72, 0, 0, 0, 2)
    header += struct.pack("<IIII", blocks, block, 0, 0)
    struct.pack_into("<H", header, 50, (-sum(struct.unpack("<36H", header))) & 0xFFFF)
    return (bytes(header) + contents).ljust(length, b"\xff")


def test_a_volume_with_files_passes():
    data = volume(ffs(b"driver") + ffs(b"x" * 20, large=True))
    result = uefi_fv.validate(data, frozenset())
    assert (
        result.status == "pass" and result.detail == "1 firmware volumes and 2 FFS files verified"
    )


def test_consecutive_volumes_and_padding_pass():
    data = volume(ffs(b"a")) + volume(ffs(b"b")) + b"\xff" * 64
    assert uefi_fv.validate(data, frozenset()).detail.startswith("2 firmware volumes")


def test_a_header_checksum_mismatch_fails():
    data = bytearray(volume(ffs(b"a")))
    data[44] ^= 1
    assert "checksum" in uefi_fv.validate(bytes(data), frozenset()).detail


def test_a_corrupt_file_header_fails():
    data = bytearray(volume(ffs(b"driver")))
    data[72 + 18] ^= 1
    assert "FFS file header checksum" in uefi_fv.validate(bytes(data), frozenset()).detail


def test_a_truncated_volume_fails():
    assert uefi_fv.validate(volume(ffs(b"a"))[:-10], frozenset()).status == "fail"
