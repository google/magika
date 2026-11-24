import struct

from helpers import status

from magika_datasets.validators.font import sfnt


def checksum(chunk):
    chunk += b"\0" * (-len(chunk) % 4)
    return sum(struct.unpack(f">{len(chunk) // 4}I", chunk)) & 0xFFFFFFFF


def head():
    return struct.pack(">IIIIHH", 0x00010000, 0, 0, 0x5F0F3CF5, 0, 16) + b"\0" * 34  # 54 bytes


def build(tables, version=b"\0\1\0\0"):
    names = sorted(tables)
    offset = 12 + 16 * len(names)
    directory, body = b"", b""
    for tag in names:
        payload = tables[tag]
        directory += tag + struct.pack(">III", checksum(payload), offset, len(payload))
        padded = payload + b"\0" * (-len(payload) % 4)
        body += padded
        offset += len(padded)
    data = bytearray(version + struct.pack(">HHHH", len(names), 0, 0, 0) + directory + body)
    if b"head" in tables:
        position = 12 + 16 * names.index(b"head")
        head_offset = struct.unpack_from(">I", data, position + 8)[0]
        adjustment = (0xB1B0AFBA - checksum(bytes(data))) & 0xFFFFFFFF
        struct.pack_into(">I", data, head_offset + 8, adjustment)
    return bytes(data)


def truetype():
    return {
        b"head": head(),
        b"hhea": b"\0\1\0\0" + b"\0" * 30 + struct.pack(">H", 1),
        b"maxp": struct.pack(">IH", 0x00005000, 1),
        b"hmtx": struct.pack(">Hh", 500, 0),
        b"cmap": struct.pack(">HH", 0, 0),
        b"loca": struct.pack(">HH", 0, 0),
        b"glyf": b"\0" * 4,
        b"name": b"\0" * 6,
    }


def test_truetype_tables_checksums_and_coverage():
    data = build(truetype())
    observation = sfnt.validate(data, frozenset())
    assert (observation.status, observation.format_id) == ("pass", "ttf")
    assert status(sfnt, data[:-3]) == "fail"  # into the last table; -2 only drops padding
    assert status(sfnt, data + b"\0\0\0\0\0") == "fail"
    corrupt = bytearray(data)
    corrupt[-3] ^= 0xFF  # inside name table
    assert status(sfnt, bytes(corrupt)) == "fail"
    tables = truetype()
    del tables[b"glyf"]
    assert status(sfnt, build(tables)) == "fail"
    assert status(sfnt, b"OTTx" + data[4:]) == "not_applicable"


def test_opentype_cff_and_adjustment_tag():
    tables = truetype()
    del tables[b"glyf"], tables[b"loca"]
    tables[b"CFF "] = b"\1\0\4\1" + b"\0" * 8
    observation = sfnt.validate(build(tables, b"OTTO"), frozenset())
    assert (observation.status, observation.format_id, observation.tags) == ("pass", "otf", ())
    data = bytearray(build(tables, b"OTTO"))
    struct.pack_into(">I", data, 12 + 16 * 1 + 8, 0)  # locate head via directory below
    observation = sfnt.validate(bytes(build(tables, b"OTTO")), frozenset())
    assert observation.status == "pass"


def test_adjustment_mismatch_is_tagged_not_failed():
    data = bytearray(build(truetype()))
    names = sorted(truetype())
    head_offset = struct.unpack_from(">I", data, 12 + 16 * names.index(b"head") + 8)[0]
    struct.pack_into(">I", data, head_offset + 8, 0x12345678)
    observation = sfnt.validate(bytes(data), frozenset())
    assert observation.status == "pass" and "checksum_adjustment_mismatch" in observation.tags


def test_collection_shares_tables():
    face = build(truetype())
    count = 2
    header = b"ttcf" + struct.pack(">II", 0x00010000, count)
    offsets = [len(header) + 4 * count] * count
    data = header + struct.pack(f">{count}I", *offsets) + face[: 12 + 16 * len(truetype())]
    shift = len(header) + 4 * count
    body = bytearray(data)
    for index in range(len(truetype())):
        position = shift + 12 + 16 * index + 8
        struct.pack_into(">I", body, position, struct.unpack_from(">I", body, position)[0] + shift)
    data = bytes(body) + face[12 + 16 * len(truetype()) :]
    observation = sfnt.validate(data, frozenset())
    assert observation.status == "pass" and "collection" in observation.tags
    assert observation.format_id == "ttf"


def woff_file(tables=None, corrupt_checksum=False, meta=b""):
    import zlib

    from magika_datasets.validators.font import woff

    tables = tables or truetype()
    names = sorted(tables)
    header_size = 44 + 20 * len(names)
    directory, body, offset, sfnt_size = b"", b"", header_size, 12 + 16 * len(names)
    for tag in names:
        payload = tables[tag]
        compressed = zlib.compress(payload, 9)
        stored = compressed if len(compressed) < len(payload) else payload
        check = checksum(payload) ^ (0xFFFF if corrupt_checksum and tag == b"name" else 0)
        directory += tag + struct.pack(">IIII", offset, len(stored), len(payload), check)
        padded = stored + b"\0" * (-len(stored) % 4)
        body += padded
        offset += len(padded)
        sfnt_size += len(payload) + (-len(payload) % 4)
    meta_block = zlib.compress(meta) if meta else b""
    meta_offset = offset if meta else 0
    total = offset + len(meta_block)
    header = (
        b"wOFF"
        + b"\0\1\0\0"
        + struct.pack(
            ">IHHIHHIIIII",
            total,
            len(names),
            0,
            sfnt_size,
            1,
            0,
            meta_offset,
            len(meta_block),
            len(meta),
            0,
            0,
        )
    )
    return header + directory + body + meta_block, woff


def test_woff_inflates_tables_and_checks_checksums():
    data, woff = woff_file()
    observation = woff.validate(data, frozenset())
    assert (observation.status, observation.format_id, observation.tags) == ("pass", "woff", ())
    with_meta, _ = woff_file(meta=b"<metadata/>")
    assert woff.validate(with_meta, frozenset()).tags == ("has_metadata",)
    assert status(woff, data[:-5]) == "fail"
    assert status(woff, data + b"\0\0\0\0") == "fail"
    assert status(woff, woff_file(corrupt_checksum=True)[0]) == "fail"
    assert status(woff, b"wOFx" + data[4:]) == "not_applicable"


def base128(value):
    out = []
    while True:
        out.append(value & 0x7F)
        value >>= 7
        if not value:
            break
    out.reverse()
    return bytes(b | 0x80 for b in out[:-1]) + bytes(out[-1:])


def woff2_file(compressed=b"\x1b" * 100, drop=0):
    from magika_datasets.validators.font import woff

    entries = [
        (1, 54),
        (2, 36),
        (4, 6),
        (3, 4),
        (0, 4),
        (11, 4),
        (10, 4),
    ]  # head hhea maxp hmtx cmap loca glyf
    directory = b""
    for index, length in entries:
        directory += bytes([index]) + base128(length)
        if index in (10, 11):
            directory += base128(length)  # glyf/loca transform length
    size = 48 + len(directory) + len(compressed)
    size += -size % 4
    header = (
        b"wOF2"
        + b"\0\1\0\0"
        + struct.pack(
            ">IHHIIHHIIIII",
            size,
            len(entries),
            0,
            12 + 16 * len(entries) + 116,
            len(compressed),
            1,
            0,
            0,
            0,
            0,
            0,
            0,
        )
    )
    data = header + directory + compressed
    data += b"\0" * (-len(data) % 4)
    return data[: len(data) - drop], woff


def test_woff2_directory_and_compressed_block_accounting():
    data, woff = woff2_file()
    observation = woff.validate(data, frozenset())
    assert (observation.status, observation.format_id) == ("pass", "woff2")
    assert status(woff, woff2_file(drop=4)[0]) == "fail"
    assert status(woff, data + b"\0\0\0\0") == "fail"


def test_sfnt_prefix_on_non_font_bytes_is_not_applicable():
    access = b"\0\1\0\0Standard Jet DB\0\1\0\0\0" + b"\0" * 200
    assert status(sfnt, access) == "not_applicable"
    hbam = (
        b"\x00\x01\x00\x00\x00\x02\x00\x01\x00\x05\x00\x02\x00\x02\xc0HBAM7\x00\x00\x10\x00"
        + b"\0" * 200
    )
    assert status(sfnt, hbam) == "not_applicable"
