import struct

import pytest
from helpers import status

from magika_datasets.validators.office import cfb

SECTOR = 512
END, FREE = 0xFFFFFFFE, 0xFFFFFFFF


def entry(
    name,
    kind,
    start=END,
    size=0,
    child=0xFFFFFFFF,
    left=0xFFFFFFFF,
    right=0xFFFFFFFF,
    clsid=b"\0" * 16,
):
    encoded = name.encode("utf-16-le") + b"\0\0"
    return (
        encoded.ljust(64, b"\0")
        + struct.pack("<HBB", len(encoded), kind, 1)
        + struct.pack("<III", left, right, child)
        + clsid
        + struct.pack("<I", 0)
        + b"\0" * 16
        + struct.pack("<IQ", start, size)
    )


def compound(streams, root_clsid=b"\0" * 16, truncate=0, bad_chain=False):
    """Minimal v3 CFB: sector 0 FAT, sector 1 directory, then one sector per stream (all >= 4096 bytes)."""
    data_sectors = []
    entries = [None]  # root placeholder
    for index, (name, payload) in enumerate(streams):
        start = 2 + len(data_sectors)
        count = -(-len(payload) // SECTOR)
        for piece in range(count):
            data_sectors.append(payload[piece * SECTOR : (piece + 1) * SECTOR].ljust(SECTOR, b"\0"))
        entries.append((name, start, len(payload), count))
    # directory: root with child = first stream; streams chained as a right-leaning tree
    directory = b""
    children = list(range(1, len(entries)))
    root_child = children[0] if children else 0xFFFFFFFF
    directory += entry("Root Entry", 5, child=root_child, clsid=root_clsid)
    for position, (name, start, size, _) in enumerate(entries[1:], 1):
        right = position + 1 if position + 1 < len(entries) else 0xFFFFFFFF
        directory += entry(name, 2, start=start, size=size, right=right)
    directory = directory.ljust(SECTOR, b"\0")
    fat = [0xFFFFFFFD, END]  # sector 0 FAT itself, sector 1 directory (single sector)
    for name, start, size, count in entries[1:]:
        for piece in range(count):
            fat.append(start + piece + 1 if piece + 1 < count else END)
    if bad_chain:
        fat[2] = 999
    fat_sector = struct.pack(f"<{len(fat)}I", *fat).ljust(SECTOR, b"\xff")
    header = (
        b"\xd0\xcf\x11\xe0\xa1\xb1\x1a\xe1"
        + b"\0" * 16
        + struct.pack("<HHHHH", 0x3E, 3, 0xFFFE, 9, 6)
        + b"\0" * 6
        + struct.pack("<IIIIIIIII", 0, 1, 1, 0, 4096, END, 0, END, 0)
    )
    header += struct.pack("<I", 0) + struct.pack("<108I", *([FREE] * 108))
    assert len(header) == 512
    data = header + fat_sector + directory + b"".join(data_sectors)
    return data[: len(data) - truncate] if truncate else data


BIG = b"x" * 5000


@pytest.mark.parametrize(
    "streams,kind",
    [
        ([("WordDocument", BIG), ("1Table", BIG)], "doc"),
        ([("Workbook", BIG)], "xls"),
        ([("Book", BIG)], "xls"),
        ([("PowerPoint Document", BIG)], "ppt"),
        ([("VisioDocument", BIG)], "visio"),
        ([("FileHeader", BIG), ("BodyText", BIG)], "hwp"),
        ([("__properties_version1.0", BIG), ("__substg1.0_0037001F", BIG)], "outlook"),
        ([("Catalog", BIG), ("1", BIG)], "thumbsdb"),
        ([("256_5317557871b4deae", BIG)], "thumbsdb"),
        ([("Data", BIG)], "ole"),
    ],
)
def test_cfb_dispatch_by_streams(streams, kind):
    observation = cfb.validate(compound(streams), frozenset())
    assert (observation.status, observation.format_id) == ("pass", kind), kind
    assert observation.generic == (kind == "ole")


def test_version_4_sectors_start_after_a_padded_header():
    data = compound([("WordDocument", BIG)])
    header = bytearray(data[:512])
    struct.pack_into("<H", header, 26, 4)  # major version 4
    struct.pack_into("<H", header, 30, 12)  # 4096-byte sectors
    body = data[512:]
    # re-lay the same three sectors at 4096-byte granularity
    sectors = [
        body[i : i + 512].ljust(4096, b"\0" if i else b"\xff") for i in range(0, len(body), 512)
    ]
    fat = (
        struct.pack("<4I", 0xFFFFFFFD, END, 3, END) + struct.pack("<I", END) * 9
    )  # the 5000-byte stream spans sectors 2 and 3
    sectors[0] = fat.ljust(4096, b"\xff")
    rebuilt = bytes(header).ljust(4096, b"\0") + b"".join(sectors)
    observation = cfb.validate(rebuilt, frozenset())
    assert (observation.status, observation.format_id) == ("pass", "doc")


def test_msi_by_root_clsid_and_integrity_failures():
    import uuid

    msi = compound(
        [("\x05SummaryInformation", BIG)],
        root_clsid=uuid.UUID("000C1084-0000-0000-C000-000000000046").bytes_le,
    )
    assert cfb.validate(msi, frozenset()).format_id == "msi"
    good = compound([("WordDocument", BIG)])
    assert status(cfb, good[:-100]) == "fail"
    assert status(cfb, compound([("WordDocument", BIG)], bad_chain=True)) == "fail"
    assert status(cfb, good + b"\0") == "fail"
    assert status(cfb, b"\xd0\xcf\x11\xe0\xa1\xb1\x1a\xe1" + b"\0" * 100) == "fail"
    assert status(cfb, b"\xd0\xcf\x11\xe1" + b"\0" * 600) == "not_applicable"
