import struct

from magika_datasets.validators.system import hfs

BLOCK, TOTAL = 4096, 4


def volume_header(signature=b"H+", version=4, catalog=(1, 1), free=1):
    header = bytearray(512)
    header[:2] = signature
    struct.pack_into(">H", header, 2, version)
    struct.pack_into(">III", header, 40, BLOCK, TOTAL, free)
    struct.pack_into(">QII", header, 272, BLOCK, BLOCK, catalog[1])
    struct.pack_into(">II", header, 272 + 16, *catalog)
    return bytes(header)


def image(node_kind=1, **options):
    data = bytearray(BLOCK * TOTAL)
    header = volume_header(**options)
    data[1024:1536] = header
    data[BLOCK + 8] = node_kind
    data[BLOCK * TOTAL - 1024 : BLOCK * TOTAL - 512] = header
    return bytes(data)


def test_an_hfs_plus_volume_passes():
    result = hfs.validate(image(), frozenset())
    assert result.status == "pass" and result.detail.startswith("HFS+ volume of 4 blocks")


def test_a_missing_alternate_header_fails():
    data = image()[: BLOCK * 3]
    assert hfs.validate(data, frozenset()).status == "fail"


def test_a_catalog_outside_the_volume_fails():
    assert "outside" in hfs.validate(image(catalog=(3, 2)), frozenset()).detail


def test_a_catalog_without_a_header_node_fails():
    assert "B-tree header" in hfs.validate(image(node_kind=0), frozenset()).detail


def test_a_mismatched_version_fails():
    assert hfs.validate(image(signature=b"HX", version=4), frozenset()).status == "fail"


def test_classic_hfs_needs_a_hint():
    mdb = bytearray(BLOCK * TOTAL)
    mdb[1024:1026] = b"BD"
    struct.pack_into(">HIIH", mdb, 1024 + 18, 6, 1024, 4096, 16)
    assert hfs.validate(bytes(mdb), frozenset()) is None
    assert hfs.validate(bytes(mdb), frozenset({"hfs"})).status == "pass"
