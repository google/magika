from magika_datasets.validators.archive import zstd


def frame(blocks: bytes, checksum: bool = False, single: bool = True) -> bytes:
    descriptor = (0x20 if single else 0) | (0x04 if checksum else 0)
    header = b"\x28\xb5\x2f\xfd" + bytes([descriptor]) + (b"\x05" if single else b"\x10")
    return header + blocks + (b"\0\0\0\0" if checksum else b"")


def block(payload: bytes, last: bool, kind: int = 0) -> bytes:
    header = (len(payload) << 3) | (kind << 1) | int(last)
    return header.to_bytes(3, "little") + payload


def test_frames_and_skippable_frames_tile_the_file():
    data = frame(block(b"abc", False) + block(b"z", True, kind=1), checksum=True)
    data += b"\x50\x2a\x4d\x18" + (2).to_bytes(4, "little") + b"hi"
    data += frame(block(b"", True), single=False)
    result = zstd.validate(data, frozenset())
    assert result.status == "pass" and result.generic
    assert result.tags == ("content_checksum", "skippable_frame")


def test_truncated_block_and_trailing_garbage_fail():
    data = frame(block(b"abcdef", True))
    assert zstd.validate(data[:-2], frozenset()).status == "fail"
    assert zstd.validate(data + b"junk", frozenset()).status == "fail"


def test_reserved_block_type_fails():
    assert zstd.validate(frame(block(b"abc", True, kind=3)), frozenset()).status == "fail"
