import struct

from magika_datasets.validators.geometry import flatgeobuf


def header(features: int, node_size: int | None = 16, geometry: int = 1) -> bytes:
    slots = [0] * 10
    slots[2], slots[8] = 4, 8
    if node_size is not None:
        slots[9] = 16
    vtable = struct.pack("<HH10H", 24, 18, *slots)
    table = struct.pack("<iBxxxQH", 24, geometry, features, node_size or 0)
    return struct.pack("<I", 28) + vtable + table


FEATURE = struct.pack("<IHHi", 8, 4, 4, 4)  # empty Feature table


def fgb(features: int = 3, node_size: int | None = 16, held: int | None = None) -> bytes:
    head = header(features, node_size)
    body = flatgeobuf.MAGIC + b"\x01" + struct.pack("<I", len(head)) + head
    body += b"\0" * flatgeobuf.index_size(features, 16 if node_size is None else node_size)
    count = features if held is None else held
    return body + b"".join(struct.pack("<I", len(FEATURE)) + FEATURE for _ in range(count))


def test_the_index_size_matches_the_reference_tree():
    assert flatgeobuf.index_size(1, 16) == 2 * 40  # a root above the single leaf
    assert flatgeobuf.index_size(17, 16) == (17 + 2 + 1) * 40
    assert flatgeobuf.index_size(5, 0) == 0


def test_header_index_and_features_tile_the_file():
    result = flatgeobuf.validate(fgb(), frozenset())
    assert result.status == "pass" and "3 features" in result.detail


def test_a_file_without_an_index_passes():
    assert flatgeobuf.validate(fgb(node_size=0), frozenset()).status == "pass"


def test_a_feature_count_mismatch_fails():
    assert "declares 3" in flatgeobuf.validate(fgb(held=2), frozenset()).detail


def test_truncation_fails():
    assert flatgeobuf.validate(fgb()[:-3], frozenset()).status == "fail"


def test_an_unknown_geometry_type_fails():
    data = fgb().replace(header(3), header(3, geometry=40))
    assert "geometry type" in flatgeobuf.validate(data, frozenset()).detail
