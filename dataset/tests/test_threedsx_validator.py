import struct

from magika_datasets.validators.executable import threedsx


def build(with_smdh: bool = True, with_romfs: bool = False) -> bytes:
    code, rodata, data, bss = b"c" * 16, b"r" * 8, b"d" * 12, 4
    relocs = [(1, 0), (0, 1), (0, 0)]
    header_size = 44
    header = b"3DSX" + struct.pack(
        "<HHIIIIII", header_size, 8, 0, 0, len(code), len(rodata), len(data) + bss, bss
    )
    body = (
        b"".join(struct.pack("<II", a, r) for a, r in relocs)
        + code
        + rodata
        + data
        + b"\0\0\0\0" * 2
    )
    smdh = b"SMDH" + bytes(12) if with_smdh else b""
    romfs = struct.pack("<I", 0x28) + bytes(36) if with_romfs else b""
    smdh_offset = header_size + len(body) if with_smdh else 0
    romfs_offset = smdh_offset + len(smdh) if with_romfs else 0
    header += struct.pack("<III", smdh_offset, len(smdh), romfs_offset)
    return header + body + smdh + romfs


def test_layouts_pass():
    assert threedsx.validate(build(), frozenset()).tags == ("smdh",)
    assert threedsx.validate(build(with_romfs=True), frozenset()).tags == ("smdh", "romfs")
    assert threedsx.validate(build(with_smdh=False), frozenset()).status == "pass"


def test_trailing_and_truncation_fail():
    assert threedsx.validate(build() + b"x", frozenset()).status == "fail"
    assert threedsx.validate(build()[:-2], frozenset()).status == "fail"
