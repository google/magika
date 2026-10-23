import struct

from magika_datasets.validators.application import hlp


def internal(data: bytes) -> bytes:
    return struct.pack("<IIB", len(data) + 9, len(data), 0) + data


def build(page_size: int = 64) -> bytes:
    files = [(b"|SYSTEM", b"s" * 10), (b"|TOPIC", b"t" * 20)]
    header_size = 16
    blobs, offsets, position = b"", [], header_size
    for _, data in files:
        blob = internal(data)
        offsets.append(position)
        blobs += blob
        position += len(blob)
    directory_start = position
    leaf = struct.pack("<HHhh", 0, len(files), -1, -1)
    for (name, _), offset in zip(files, offsets):
        leaf += name + b"\0" + struct.pack("<I", offset)
    leaf = leaf.ljust(page_size, b"\0")
    btree = (
        struct.pack("<HHH", 0x293B, 2, page_size)
        + b"Fz4".ljust(16, b"\0")
        + struct.pack("<HHhhHHI", 0, 0, 0, -1, 1, 1, len(files))
        + leaf
    )
    directory = struct.pack("<IIB", len(btree) + 9, len(btree), 4) + btree
    total = directory_start + len(directory)
    header = b"?_\x03\x00" + struct.pack("<IiI", directory_start, -1, total)
    return header + blobs + directory


def test_help_file_passes():
    result = hlp.validate(build(), frozenset())
    assert result.status == "pass" and "2 internal files" in result.detail


def test_size_mismatch_and_bad_extent_fail():
    assert hlp.validate(build() + b"\0", frozenset()).status == "fail"
    data = bytearray(build())
    struct.pack_into("<I", data, 16, 1 << 20)  # first internal file's reserved space
    assert hlp.validate(bytes(data), frozenset()).status == "fail"
