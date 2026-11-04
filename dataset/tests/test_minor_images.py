import struct

from helpers import status

from magika_datasets.validators.image import bpg, jxl, xcf


def ue7(value):
    groups = []
    while True:
        groups.append(value & 0x7F)
        value >>= 7
        if not value:
            break
    groups.reverse()
    return bytes(g | 0x80 for g in groups[:-1]) + bytes(groups[-1:])


def bpg_file(length=None, extension=False):
    payload = b"\x40\x01\x0c" * 40
    length = len(payload) if length is None else length
    flags = 0x08 if extension else 0x00  # extension_present_flag
    head = b"BPG\xfb" + bytes([0x10, flags]) + ue7(200) + ue7(200) + ue7(length)
    if extension:
        head += ue7(3) + b"ext"
    return head + payload


def test_bpg_header_and_picture_length():
    assert status(bpg, bpg_file()) == "pass"
    assert status(bpg, bpg_file(extension=True)) == "pass"
    assert status(bpg, bpg_file(length=0)) == "pass"
    assert status(bpg, bpg_file()[:-1]) == "fail"
    assert status(bpg, bpg_file() + b"x") == "fail"
    assert status(bpg, b"BPG\xfb" + ue7(1)) == "fail"
    assert status(bpg, b"other") == "not_applicable"


def box(kind, payload=b""):
    return struct.pack(">I", 8 + len(payload)) + kind + payload


def jxl_file(codestream=True):
    data = jxl.SIGNATURE + box(b"ftyp", b"jxl \0\0\0\0jxl ")
    if codestream:
        data += box(b"jxlc", b"\xff\x0a" + b"\0" * 20)
    return data


def test_jxl_container_and_raw_codestream():
    observation = jxl.validate(jxl_file(), frozenset())
    assert (observation.status, observation.format_id) == ("pass", "jxl")
    assert status(jxl, jxl_file(codestream=False)) == "fail"
    assert status(jxl, jxl_file()[:-3]) == "fail"
    assert status(jxl, b"\xff\x0a" + b"\0" * 10) == "inconclusive"
    assert status(jxl, b"\0\0\0\x0cjP  \r\n\x87\n") == "not_applicable"


def xcf_with_channel_and_unused_level():
    def prop_end():
        return struct.pack(">II", 0, 0)

    header = b"gimp xcf v011\0" + struct.pack(">IIII", 4, 4, 0, 150) + prop_end()
    fixed_len = len(header) + 8 + 8 + 8 + 8  # layer ptr, terminator, channel ptr, terminator
    layer_at = fixed_len
    name = b"L"
    layer_head = struct.pack(">III", 4, 4, 0) + struct.pack(">I", 2) + name + b"\0" + prop_end()
    hier_at = layer_at + len(layer_head) + 16
    layer = layer_head + struct.pack(">QQ", hier_at, 0)
    level0_at = hier_at + 12 + 8 + 8 + 8  # w h bpp, two level ptrs, terminator
    level1_at = level0_at + 8 + 8 + 8  # w h, tile ptr, terminator
    hierarchy = struct.pack(">III", 4, 4, 3) + struct.pack(">QQQ", level0_at, level1_at, 0)
    tile_at = level1_at + 8 + 4  # unused level: w h + 32-bit zero
    level0 = struct.pack(">II", 4, 4) + struct.pack(">QQ", tile_at, 0)
    level1 = struct.pack(">II", 2, 2) + struct.pack(">I", 0)
    tile = b"\x7f" * 48
    channel_at = tile_at + len(tile)
    fixed = header + struct.pack(">QQQQ", layer_at, 0, channel_at, 0)
    chan_head = struct.pack(">II", 4, 4) + struct.pack(">I", 2) + b"C\0" + prop_end()
    chan_hier_at = channel_at + len(chan_head) + 8
    channel = chan_head + struct.pack(">Q", chan_hier_at)
    chan_level_at = chan_hier_at + 12 + 8 + 8
    chan_hierarchy = struct.pack(">III", 4, 4, 1) + struct.pack(">QQ", chan_level_at, 0)
    chan_tile_at = chan_level_at + 8 + 8 + 8
    chan_level = struct.pack(">II", 4, 4) + struct.pack(">QQ", chan_tile_at, 0)
    return (
        fixed
        + layer
        + hierarchy
        + level0
        + level1
        + tile
        + channel
        + chan_hierarchy
        + chan_level
        + b"\x10" * 16
    )


def test_xcf_v11_channels_and_unused_levels():
    data = xcf_with_channel_and_unused_level()
    observation = xcf.validate(data, frozenset())
    assert (observation.status, observation.detail) == (
        "pass",
        "XCF v11: 1 layers, 1 channels resolved",
    )
    assert status(xcf, data[:-20]) == "fail"


def xcf_file(bad_offset=False):
    def prop_end():
        return struct.pack(">II", 0, 0)

    header = b"gimp xcf v001\0" + struct.pack(">III", 4, 4, 0) + prop_end()
    layer_offset = len(header) + 4 + 4 + 4  # layer pointer, layer terminator, channel terminator
    fixed = (
        header
        + struct.pack(">I", 0xFFFFFF if bad_offset else layer_offset)
        + struct.pack(">I", 0)
        + struct.pack(">I", 0)
    )
    layer_start = len(fixed)
    name = b"Background"
    layer_head = (
        struct.pack(">III", 4, 4, 0) + struct.pack(">I", len(name) + 1) + name + b"\0" + prop_end()
    )
    hierarchy_offset = layer_start + len(layer_head) + 8
    layer = layer_head + struct.pack(">II", hierarchy_offset, 0)
    level_offset = hierarchy_offset + 12 + 4 + 4
    hierarchy = struct.pack(">III", 4, 4, 3) + struct.pack(">II", level_offset, 0)
    tile_offset = level_offset + 8 + 4 + 4
    level = struct.pack(">II", 4, 4) + struct.pack(">II", tile_offset, 0)
    tile = b"\x7f" * 48
    return fixed + layer + hierarchy + level + tile


def test_xcf_layer_hierarchy_offsets():
    assert status(xcf, xcf_file()) == "pass"
    assert status(xcf, xcf_file(bad_offset=True)) == "fail"
    assert status(xcf, xcf_file()[:-60]) == "fail"
    assert status(xcf, b"gimp xcf zzzz\0" + b"\0" * 20) == "fail"
    assert status(xcf, b"other") == "not_applicable"
