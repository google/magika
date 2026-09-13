import gzip
import json

from magika_datasets.validators.geometry import pmtiles


def varint(value):
    out = bytearray()
    while True:
        byte = value & 0x7F
        value >>= 7
        out.append(byte | (0x80 if value else 0))
        if not value:
            return bytes(out)


def directory(entries):
    """entries: (tile_id, run_length, length, offset) with offsets already +1 encoded."""
    fields = [len(entries)] + [e[0] for e in entries] + [e[1] for e in entries]
    fields += [e[2] for e in entries] + [e[3] for e in entries]
    return b"".join(varint(v) for v in fields)


def archive(entries=((0, 1, 5, 1), (1, 1, 7, 0)), tiles=b"x" * 12, bounds=(-10, -10, 10, 10)):
    root = gzip.compress(directory(entries), mtime=0)
    meta = gzip.compress(json.dumps({"name": "test"}).encode(), mtime=0)
    root_at = pmtiles.HEADER.size
    meta_at = root_at + len(root)
    tiles_at = meta_at + len(meta)
    header = pmtiles.HEADER.pack(
        pmtiles.MAGIC,
        root_at,
        len(root),
        meta_at,
        len(meta),
        tiles_at,
        0,
        tiles_at,
        len(tiles),
        2,
        2,
        2,
        1,
        pmtiles.GZIP,
        pmtiles.GZIP,
        1,
        0,
        1,
        bounds[0],
        bounds[1],
        bounds[2],
        bounds[3],
        0,
        0,
        0,
    )
    return header + root + meta + tiles


def test_sections_and_root_directory_verify():
    result = pmtiles.validate(archive(), frozenset())
    assert result.status == "pass" and "2 entries" in result.detail


def test_an_entry_past_tile_data_fails():
    assert "past its section" in pmtiles.validate(archive(tiles=b"x" * 11), frozenset()).detail


def test_truncation_fails():
    assert pmtiles.validate(archive()[:-4], frozenset()).status == "fail"


def test_bounds_out_of_range_fail():
    data = archive(bounds=(-1900000000, 0, 0, 0))
    assert "Bounds" in pmtiles.validate(data, frozenset()).detail
