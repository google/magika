# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0

"""Cross-check the Python preprocessors against the native contract.

The archives and PE images built here are ports of the Rust test helpers in
`rust/lib/src/rules/preprocess/{zip,pe}.rs`, and the facts and view bytes asserted are
the ones the Rust tests assert: a disagreement is a finding against the contract, not
against either implementation alone.

`golden/preprocess.json`, when present, pins facts produced by the native engine:
a list of `{"path": <repository-relative fixture>, "facts": {<name>: <int>}, "view_sha256":
<hex digest of the whole 4096-byte zip_names view>}` records.
"""

import hashlib
import io
import json
import re
import zipfile
from pathlib import Path

import pytest
import yara_x
from magika_rules_benchmark import preprocess
from magika_rules_benchmark.preprocess import (
    FACT_NAMES,
    FACTS_BYTES,
    PE32_MAGIC,
    PE32PLUS_MAGIC,
    PREFIX_BYTES,
    ZIP_NAMES_BYTES,
    ZIP_TAIL_BYTES,
    facts_for,
    pe_facts,
    prepare,
    read_tail,
    tail_window,
    zip_analysis,
)
from preprocess_fixtures import (
    EOCD_LEN,
    archive,
    docx_like,
    eocd_at,
    pad_directory,
    pe32,
    pe32plus,
    put_u16,
    put_u32,
    stored,
    u32,
)

ROOT = Path(__file__).resolve().parents[3]
GOLDEN = Path(__file__).with_name("golden") / "preprocess.json"
NATIVE = ROOT / "rust/lib/src/rules/preprocess/mod.rs"

ZIP64, MULTIDISK, CD_NOT_HELD, MALFORMED, NAMES_TRUNCATED, PREPENDED = (1 << n for n in range(6))
SECTIONS_NOT_HELD, CERTIFICATE_OUT_OF_FILE, UNKNOWN_MAGIC, SECTION_BEYOND_FILE = (
    1 << n for n in range(4)
)


# --- zip helpers ----------------------------------------------------------------------------


def zip_expected(**facts):
    expected = dict.fromkeys(preprocess.ZIP_FACT_NAMES, 0)
    for name, value in facts.items():
        expected[f"zip_{name}"] = int(value)
    return expected


def analyze(first, size, tail):
    """Both zip outputs, checked for the invariants every analysis upholds."""
    facts, view = zip_analysis(first, size, tail)
    assert set(facts) == set(preprocess.ZIP_FACT_NAMES)
    assert facts["zip_names_len"] == len(view) <= ZIP_NAMES_BYTES
    assert facts["zip_names_entries"] <= min(facts["zip_entries"], 96)
    if view:
        assert view[0] == view[-1] == 0x0A
    if facts["zip_valid"]:
        assert facts["zip_flags"] & 0b1111 == 0
    return facts, view


def small(data):
    """Analyze a small input held entirely in one block, as `first` and `tail` alike."""
    return analyze(bytes(data), len(data), bytes(data))


def blocks(data):
    """Analyze with the bounded blocks the pipeline holds: 4 KiB at 0, 16 KiB at the end."""
    data = bytes(data)
    return analyze(data[:4096], len(data), data[max(0, len(data) - ZIP_TAIL_BYTES) :])


# --- PE helpers -----------------------------------------------------------------------------


def facts_of(data):
    facts = pe_facts(data[: min(len(data), 4096)], len(data))
    assert set(facts) == set(preprocess.PE_FACT_NAMES)
    return facts


def pe_expected(**facts):
    """The facts of a valid `pe32()` image, with `facts` overriding by short name."""
    expected = dict(
        valid=1,
        flags=0,
        machine=0x14C,
        characteristics=0x0102,
        subsystem=3,
        dll_characteristics=0x8140,
        sections=1,
        magic=PE32_MAGIC,
        clr=0,
        signed=0,
        overlay=0,
        is_dll=0,
        is_executable_image=1,
    )
    expected.update(facts)
    return {f"pe_{name}": int(value) for name, value in expected.items()}


def pe_zero():
    return dict.fromkeys(preprocess.PE_FACT_NAMES, 0)


# The end of the headers of a `pe32()` image with one section.
HEADERS_END = 0x40 + 4 + 20 + 224 + 40


def pe_check(facts, size):
    """The invariants every PE analysis upholds, whatever the input."""
    if facts["pe_valid"]:
        assert facts["pe_flags"] & (SECTIONS_NOT_HELD | UNKNOWN_MAGIC) == 0, facts
        assert 1 <= facts["pe_sections"] <= 96, facts
        assert facts["pe_magic"] in (PE32_MAGIC, PE32PLUS_MAGIC), facts
    else:
        assert facts["pe_overlay"] == 0, facts
    assert facts["pe_overlay"] <= size, facts
    assert facts["pe_is_dll"] == int(facts["pe_characteristics"] & 0x2000 != 0), facts
    assert facts["pe_is_executable_image"] == int(facts["pe_characteristics"] & 0x0002 != 0)
    assert not (facts["pe_signed"] and facts["pe_flags"] & CERTIFICATE_OUT_OF_FILE), facts


# --- the facts table and the tail window ----------------------------------------------------


def native_constant(source, name):
    """The value of `pub(crate) const <name>: usize = <expression>;` in the native module."""
    expression = re.search(rf"const {name}: usize = ([\d* ]+);", source)[1]
    factors = [int(x) for x in expression.split("*")]
    return factors[0] * factors[1] if len(factors) == 2 else factors[0]


def test_fact_table_matches_the_native_header_layout():
    # The table is read from the native `FACTS` entries, so a field added or moved on
    # either side is a failure here, not a silent drift.
    source = NATIVE.read_text()
    table = source[source.index("const FACTS: &[Fact] = &[") :]
    table = table[: table.index("];")]
    entries = re.findall(r'fact_at\("(\w+)", (\d+), (\d+)\)', table)
    native = tuple((name, int(offset), int(width)) for name, offset, width in entries)
    assert len(native) == 22
    assert preprocess.FACTS == native
    assert FACT_NAMES == tuple(name for name, _, _ in preprocess.FACTS)
    assert preprocess.ZIP_FACT_NAMES == FACT_NAMES[2:9]
    assert preprocess.PE_FACT_NAMES == FACT_NAMES[9:]
    assert preprocess.VIEWS == (("zip_names", FACTS_BYTES, ZIP_NAMES_BYTES),)
    assert 'View { name: "zip_names", offset: FACTS_BYTES, size: ZIP_NAMES_BYTES }' in source
    assert FACTS_BYTES == native_constant(source, "FACTS_BYTES") == 64
    assert ZIP_NAMES_BYTES == native_constant(source, "ZIP_NAMES_BYTES") == 4096
    assert ZIP_TAIL_BYTES == native_constant(source, "ZIP_TAIL_BYTES") == 16 * 1024


def encode_facts(facts):
    """The 64-byte big-endian facts header, saturating each value to its field's maximum.

    A port of the native `write_fact` for asserting the facts land at the native offsets.
    """
    fields = {name: (offset, width) for name, offset, width in preprocess.FACTS}
    header = bytearray(FACTS_BYTES)
    for name, value in facts.items():
        offset, width = fields[name]
        header[offset : offset + width] = min(value, (1 << (8 * width)) - 1).to_bytes(width, "big")
    return bytes(header)


def test_encode_facts_uses_big_endian_field_widths_and_saturates():
    header = encode_facts({"pe_machine": 0x8664, "zip_valid": 1, "pe_overlay": 0x0102030405060708})
    assert len(header) == FACTS_BYTES
    assert header[34:36] == b"\x86\x64"
    assert header[16] == 1
    assert header[48:56] == bytes([1, 2, 3, 4, 5, 6, 7, 8])
    assert not any(header[:16])
    saturated = encode_facts({"zip_valid": 300, "zip_entries": 70_000, "pe_overlay": 2**64 - 1})
    assert saturated[16] == 255
    assert saturated[18:20] == b"\xff\xff"
    assert saturated[48:56] == b"\xff" * 8
    assert saturated[17] == 0 and saturated[20:22] == b"\0\0" and saturated[56] == 0
    with pytest.raises(KeyError):
        encode_facts({"filesize": 1})


def test_only_zip_signatures_want_a_tail():
    assert preprocess.wants_tail(b"PK\x03\x04\x14\x00")
    assert preprocess.wants_tail(b"PK\x05\x06")
    for prefix in (b"", b"PK", b"PK\x03", b"PK\x01\x02", b"PK\x07\x08", b"\x89PNG"):
        assert not preprocess.wants_tail(prefix), prefix


def test_tail_window_reads_one_bounded_window_only_for_unheld_zips():
    for size, expected in [
        (100_000, (83_616, 100_000)),
        (16_384, (0, 16_384)),
        (10_000, (0, 10_000)),
        (4097, (0, 4097)),
        (4096, None),
        (4000, None),
        (4, None),
    ]:
        prefix = (b"PK\x03\x04" + b"A" * 4096)[: min(size, 4096)]
        assert tail_window(size, prefix) == expected, size
    assert tail_window(100_000, b"A" * 4096) is None


class CountingStream(io.BytesIO):
    """A stream counting its reads: the tail costs one read, and only when wanted."""

    def __init__(self, data):
        super().__init__(data)
        self.reads = 0

    def read(self, size=-1):
        self.reads += 1
        return super().read(size)


def test_read_tail_reads_one_bounded_window_only_for_unheld_zips():
    for data, expected in [
        (b"PK\x03\x04" + bytes(100_000 - 4), 16_384),
        (b"PK\x03\x04" + bytes(16_384 - 4), 16_384),
        (b"PK\x03\x04" + bytes(10_000 - 4), 10_000),
        (b"PK\x03\x04" + bytes(4097 - 4), 4097),
        (b"PK\x05\x06" + bytes(4096 - 4), None),
        (b"PK\x03\x04" + bytes(4000 - 4), None),
        (b"\x89PNG" + bytes(100_000 - 4), None),
    ]:
        stream = CountingStream(data)
        first = stream.read(PREFIX_BYTES)
        tail = read_tail(stream, len(data), first)
        if expected is None:
            assert tail is None and stream.reads == 1, len(data)
        else:
            assert tail == data[-expected:] and stream.reads == 2, len(data)


def test_prepare_derives_stream_b_from_the_held_blocks():
    data = docx_like()
    facts, view = prepare(data, len(data), data)
    assert facts["zip_valid"] == 1 and facts["original_size"] == len(data)
    assert facts["prefix_size"] == len(data) and len(view) == ZIP_NAMES_BYTES
    assert view.startswith(b"\n[Content_Types].xml\nword/document.xml\n")
    assert (facts, view) == facts_for(data)
    # No tail read: the prefix stands in when it holds the whole input, and only the first
    # block is known otherwise, so the directory is not found but a mimetype line still is.
    assert prepare(data, len(data), None) == (facts, view)
    epub = archive([stored(b"mimetype", b"application/epub+zip")])
    facts, view = prepare(epub, len(epub) + 1, None)
    assert facts["zip_valid"] == 0 and facts["original_size"] == len(epub) + 1
    assert view.startswith(b"\nmimetype=application/epub+zip\n\0")
    # The other family, and the size facts alone.
    image = pe32().build()
    facts, view = prepare(image, len(image), None)
    assert facts["pe_valid"] == 1 and view == bytes(ZIP_NAMES_BYTES)
    facts, view = prepare(b"plain", 5, None)
    assert facts == dict.fromkeys(FACT_NAMES, 0) | {"original_size": 5, "prefix_size": 5}
    assert view == bytes(ZIP_NAMES_BYTES)


def test_is_active_only_when_a_preprocessor_produced_something():
    # The size facts alone never activate stream B: stream A already carries them.
    facts, view = prepare(b"x", (1 << 63) - 1, None)
    assert not preprocess.is_active(facts, view)
    assert preprocess.is_active(facts | {"pe_is_executable_image": 1}, view)
    assert not preprocess.is_active(facts, b"\0\n" + view[2:])
    assert preprocess.is_active(facts, b"\n" + view[1:])
    assert not preprocess.is_active(*facts_for(b"MZ" + bytes(0x80)))
    assert preprocess.is_active(*facts_for(pe32(magic=0x107).build()))


# --- zip facts and the zip_names view -------------------------------------------------------


def test_minimal_archive_yields_exact_facts_and_view():
    data = archive([stored(b"a.txt", b"hello"), stored(b"dir/b.bin", b"")])
    facts, view = small(data)
    assert facts == zip_expected(
        valid=1, entries=2, names_entries=2, names_len=17, cd_size=2 * 46 + 5 + 9
    )
    assert view == b"\na.txt\ndir/b.bin\n"


def test_archive_comment_is_measured_and_eocd_found_before_it():
    facts, view = small(archive([stored(b"x", b"1")], b"a comment"))
    assert facts["zip_valid"] == 1
    assert facts["zip_comment_len"] == 9
    assert facts["zip_entries"] == 1
    assert view == b"\nx\n"


def test_empty_archive_is_valid_with_an_empty_view():
    facts, view = small(archive([]))
    assert facts == zip_expected(valid=1)
    assert view == b""


def test_stored_mimetype_first_entry_adds_a_line_before_the_names():
    data = archive([stored(b"mimetype", b"application/epub+zip"), stored(b"OEBPS/x", b"")])
    facts, view = small(data)
    expected = b"\nmimetype=application/epub+zip\nmimetype\nOEBPS/x\n"
    assert facts["zip_valid"] == 1
    assert facts["zip_names_entries"] == 2
    assert facts["zip_names_len"] == len(expected)
    assert view == expected
    # The line is derived from the first block alone: a truncated archive keeps it.
    facts, view = small(data[:60])
    assert facts == zip_expected(names_len=31)
    assert view == b"\nmimetype=application/epub+zip\n"
    # Only a stored, 1..=128 byte, in-block payload qualifies.
    deflated = bytearray(data)
    put_u16(deflated, 8, 8)
    assert small(deflated)[1] == b"\nmimetype\nOEBPS/x\n"
    empty = bytearray(data)
    put_u32(empty, 22, 0)
    assert small(empty)[1] == b"\nmimetype\nOEBPS/x\n"
    huge = bytearray(data)
    put_u32(huge, 22, 129)
    assert small(huge)[1] == b"\nmimetype\nOEBPS/x\n"
    beyond = bytearray(data)
    put_u16(beyond, 28, 4096)
    assert small(beyond)[1] == b"\nmimetype\nOEBPS/x\n"
    other = bytearray(data)
    other[30:38] = b"mimetypX"
    # Only the local header was renamed: the directory still lists `mimetype`.
    assert small(other)[1] == b"\nmimetype\nOEBPS/x\n"
    sanitized = bytearray(data)
    sanitized[38] = 0x0A
    sanitized[39] = 0
    facts, view = small(sanitized)
    assert len(view) == len(expected) and view[:12] == b"\nmimetype=\x01\x01"


def test_eocd_with_a_mismatched_comment_length_is_rejected():
    data = bytearray(archive([stored(b"a", b"1")]))
    put_u16(data, eocd_at(data, 0) + 20, 5)
    assert small(data) == (zip_expected(), b"")
    # A shorter comment than declared is rejected too; a longer one is not an EOCD.
    data = bytearray(archive([stored(b"a", b"1")], b"abc"))
    put_u16(data, eocd_at(data, 3) + 20, 2)
    assert small(data)[0] == zip_expected()


def test_eocd_at_the_very_start_of_the_tail_uses_the_first_block_directory():
    data = archive([stored(b"a", b"1"), stored(b"bb", b"22")])
    tail = data[len(data) - EOCD_LEN :]
    facts, view = analyze(data, len(data), tail)
    assert facts["zip_valid"] == 1, facts
    assert facts["zip_names_entries"] == 2
    assert view == b"\na\nbb\n"
    # Without the first block, the directory is held by neither block.
    facts, view = analyze(data[:4], len(data), tail)
    assert facts == zip_expected(flags=CD_NOT_HELD, entries=2, cd_size=2 * 46 + 3)
    assert view == b""


def test_directory_filling_the_tail_window_exactly_is_held():
    held = bytearray(archive([stored(b"a", b"1")]))
    pad_directory(held, ZIP_TAIL_BYTES - EOCD_LEN - 47)
    assert len(held) > ZIP_TAIL_BYTES
    facts, view = blocks(held)
    assert facts == zip_expected(
        valid=1, entries=1, names_entries=1, names_len=3, cd_size=ZIP_TAIL_BYTES - EOCD_LEN
    )
    assert view == b"\na\n"
    unheld = bytearray(archive([stored(b"a", b"1")]))
    pad_directory(unheld, ZIP_TAIL_BYTES - EOCD_LEN - 47 + 1)
    facts, view = blocks(unheld)
    assert facts == zip_expected(
        flags=CD_NOT_HELD, entries=1, cd_size=ZIP_TAIL_BYTES - EOCD_LEN + 1
    )
    assert view == b""


def test_long_comment_keeps_the_eocd_and_directory_in_the_first_block():
    entries = [stored(bytes([name]), b"1") for name in range(ord("a"), ord("h") + 1)]
    data = archive(entries, b"c" * 16_000)
    eocd_abs = 8 * 32 + 8 * 47
    assert len(data) == eocd_abs + EOCD_LEN + 16_000
    assert len(data) > ZIP_TAIL_BYTES and eocd_abs < 4096
    facts, view = blocks(data)
    assert facts == zip_expected(
        valid=1, entries=8, names_entries=8, names_len=17, comment_len=16_000, cd_size=8 * 47
    )
    assert view == b"\na\nb\nc\nd\ne\nf\ng\nh\n"


def test_directory_larger_than_the_tail_window_is_not_held():
    data = archive([stored(f"{i:0>40}".encode(), b"") for i in range(400)])
    facts, view = blocks(data)
    assert facts == zip_expected(flags=CD_NOT_HELD, entries=400, cd_size=400 * 86)
    assert view == b""


def test_directory_size_beyond_the_file_is_malformed():
    data = bytearray(archive([stored(b"a", b"1")]))
    eocd = eocd_at(data, 0)
    put_u32(data, eocd + 12, 0xFFFF_0000)
    facts, view = small(data)
    assert facts == zip_expected(flags=MALFORMED, entries=1, cd_size=0xFFFF_0000)
    assert view == b""
    # One byte too many is malformed as well; the exact size is fine.
    data = bytearray(archive([stored(b"a", b"1")]))
    put_u32(data, eocd + 12, eocd + 1)
    assert small(data)[0]["zip_flags"] == MALFORMED
    put_u32(data, eocd + 12, eocd)
    assert small(data)[0]["zip_flags"] == MALFORMED, "a directory spanning the entries"


def test_directory_offset_overflow_does_not_disturb_the_walk():
    data = bytearray(archive([stored(b"a", b"1")]))
    put_u32(data, eocd_at(data, 0) + 16, 0xFFFF_FFFE)
    facts, view = small(data)
    assert (facts["zip_flags"], facts["zip_valid"]) == (0, 1)
    assert view == b"\na\n"


def test_name_running_past_the_directory_end_is_malformed():
    entries = [stored(b"first", b"1"), stored(b"second", b"2")]
    data = bytearray(archive(entries))
    cd = eocd_at(data, 0) - (2 * 46 + 5 + 6)
    put_u16(data, cd + 46 + 5 + 28, 0xFFFF)
    facts, view = small(data)
    assert facts == zip_expected(
        flags=MALFORMED, entries=2, names_entries=1, names_len=7, cd_size=2 * 46 + 5 + 6
    )
    assert view == b"\nfirst\n"
    # A bad signature stops the walk the same way.
    data = bytearray(archive(entries))
    data[cd + 46 + 5] = ord("Q")
    facts, view = small(data)
    assert facts["zip_flags"] == MALFORMED
    assert view == b"\nfirst\n"
    # An extra or comment length past the end drops that entry without writing it.
    data = bytearray(archive(entries))
    put_u16(data, cd + 46 + 5 + 32, 1)
    facts, view = small(data)
    assert (facts["zip_flags"], facts["zip_names_entries"]) == (MALFORMED, 1)
    assert view == b"\nfirst\n"


def test_names_are_sanitized_of_newlines_and_nuls():
    facts, view = small(archive([stored(b"a\nb\0c", b""), stored(b"\n", b"")]))
    assert facts["zip_valid"] == 1
    assert view == b"\na\x01b\x01c\n\x01\n"


def test_entries_are_capped_and_truncation_only_flags_a_full_view():
    data = bytearray(archive([stored(f"n{i}".encode(), b"") for i in range(100)]))
    eocd = eocd_at(data, 0)
    put_u16(data, eocd + 8, 0xFFFE)
    put_u16(data, eocd + 10, 0xFFFE)
    facts, view = small(data)
    assert (facts["zip_valid"], facts["zip_flags"]) == (1, 0), facts
    assert facts["zip_entries"] == 0xFFFE
    assert facts["zip_names_entries"] == 96
    expected = b"".join(f"\nn{i}".encode() for i in range(96)) + b"\n"
    assert facts["zip_names_len"] == len(expected)
    assert view == expected
    # Long names fill the view: whole names are dropped and the truncation flagged.
    facts, view = small(archive([stored(f"{i:0>60}".encode(), b"") for i in range(100)]))
    assert facts["zip_valid"] == 1
    assert facts["zip_flags"] == NAMES_TRUNCATED
    assert facts["zip_names_entries"] == 67
    assert facts["zip_names_len"] == 67 * 61 + 1
    assert view == b"".join(f"\n{i:0>60}".encode() for i in range(67)) + b"\n"
    # A name filling the view exactly, terminator included, still fits.
    name = b"z" * (ZIP_NAMES_BYTES - 2)
    facts, view = small(archive([stored(name, b""), stored(b"q", b"")]))
    assert (facts["zip_flags"], facts["zip_names_entries"]) == (NAMES_TRUNCATED, 1)
    assert facts["zip_names_len"] == ZIP_NAMES_BYTES
    assert view == b"\n" + name + b"\n"


def test_zip64_sentinels_and_locator_report_a_plain_invalid_zip():
    base = archive([stored(b"a", b"1")])
    eocd = eocd_at(base, 0)

    def expected(entries):
        return zip_expected(flags=ZIP64, entries=entries)

    total = bytearray(base)
    put_u16(total, eocd + 8, 0xFFFF)
    put_u16(total, eocd + 10, 0xFFFF)
    assert small(total) == (expected(0xFFFF), b"")
    size = bytearray(base)
    put_u32(size, eocd + 12, 0xFFFF_FFFF)
    assert small(size)[0] == expected(1)
    offset = bytearray(base)
    put_u32(offset, eocd + 16, 0xFFFF_FFFF)
    assert small(offset)[0] == expected(1)
    disk = bytearray(base)
    put_u16(disk, eocd + 4, 0xFFFF)
    assert small(disk)[0]["zip_flags"] & ZIP64 == ZIP64
    # A zip64 locator immediately before a plain EOCD marks zip64 as well.
    located = base[:eocd] + b"PK\x06\x07" + bytes(16) + base[eocd:]
    assert small(located)[0] == expected(1)


def test_multi_disk_archives_are_invalid():
    base = archive([stored(b"a", b"1")])
    eocd = eocd_at(base, 0)
    expected = zip_expected(flags=MULTIDISK, entries=1)
    disk = bytearray(base)
    put_u16(disk, eocd + 4, 1)
    assert small(disk) == (expected, b"")
    cd_disk = bytearray(base)
    put_u16(cd_disk, eocd + 6, 1)
    assert small(cd_disk)[0] == expected
    split = bytearray(base)
    put_u16(split, eocd + 8, 0)
    assert small(split)[0] == expected


def test_prepended_data_is_flagged_and_still_walked():
    data = b"M" * 100 + archive([stored(b"payload.bin", b"xyz")])
    facts, view = small(data)
    assert facts == zip_expected(
        valid=1, flags=PREPENDED, entries=1, names_entries=1, names_len=13, cd_size=46 + 11
    )
    assert view == b"\npayload.bin\n"
    # The same archive behind a large stub, with only the bounded blocks held.
    data = b"M" * 100_000 + archive([stored(b"payload.bin", b"xyz")])
    facts, view = blocks(data)
    assert (facts["zip_valid"], facts["zip_flags"]) == (1, PREPENDED)
    assert view == b"\npayload.bin\n"


def test_short_or_foreign_tails_yield_nothing():
    data = archive([stored(b"a", b"1")])
    for tail in (b"", b"PK\x05\x06", data[len(data) - 21 :]):
        assert analyze(data, len(data), tail) == (zip_expected(), b""), tail
    # A tail longer than the file cannot be placed and yields nothing.
    assert analyze(data, 10, data) == (zip_expected(), b"")
    assert small(b"PK\x03\x04 not an archive")[0] == zip_expected()


@pytest.mark.parametrize(
    "name", ["simple.zip", "zip64.zip", "volumecomment.zip", "filecomment.zip"]
)
def test_every_byte_mutation_of_the_zip_fixtures_is_survived(name):
    original = (ROOT / "tests_data/mitra/zip" / name).read_bytes()
    assert blocks(original)[0]["zip_entries"] > 0, name
    for at in range(len(original)):
        for value in (0x00, 0xFF, 0x50, original[at] ^ 0x01, original[at] ^ 0x80):
            mutated = bytearray(original)
            mutated[at] = value
            mutated = bytes(mutated)
            blocks(mutated)
            small(mutated)
            # Truncations and the two-block split are reachable states as well.
            analyze(mutated[:at], len(mutated), mutated[at:])


# --- PE facts -------------------------------------------------------------------------------


def test_plain_dos_executables_yield_nothing():
    dos = bytearray(0x80)
    dos[:2] = b"MZ"
    assert facts_of(bytes(dos)) == pe_zero()
    # `e_lfanew` pointing at bytes that are not a PE signature.
    put_u32(dos, 0x3C, 0x40)
    assert facts_of(bytes(dos)) == pe_zero()
    dos[0x40:0x44] = b"PE\0\x01"
    assert facts_of(bytes(dos)) == pe_zero()
    # Too short to hold `e_lfanew` at all, and not starting with `MZ`.
    assert facts_of(b"MZ") == pe_zero()
    assert facts_of(b"M" * 0x3F) == pe_zero()
    foreign = bytearray(pe32().build())
    foreign[1] = ord("z")
    assert facts_of(bytes(foreign)) == pe_zero()


@pytest.mark.parametrize("magic", [0, 0x107, 0x10A, 0x20A, 0xFFFF])
def test_unknown_optional_magic_is_flagged_and_invalid(magic):
    image = pe32(magic=magic)
    # Directories are not read behind an unknown magic, whatever they hold.
    image.directories[4] = (0x200, 0x100)
    image.directories[14] = (0x2000, 72)
    assert facts_of(image.build()) == pe_expected(valid=0, flags=UNKNOWN_MAGIC, magic=magic)


def test_hostile_e_lfanew_values_yield_nothing():
    data = pe32().build()
    length = len(data)
    for e_lfanew in (0, 1, 3, 0x3C, 0x41, length - 24 + 1, length - 1):
        mutated = bytearray(data)
        put_u32(mutated, 0x3C, e_lfanew)
        assert facts_of(bytes(mutated)) == pe_zero(), hex(e_lfanew)
    # An 8 KiB file: these offsets lie inside it, but the headers they address do not lie
    # inside its 4 KiB first block, the only bytes the analysis sees.
    for e_lfanew in (0xFFFF_FFFF, 0xFFFF_FFE8, 0x8000_0000, 4096 - 23):
        mutated = bytearray(data).ljust(8192, b"\0")
        put_u32(mutated, 0x3C, e_lfanew)
        assert facts_of(bytes(mutated)) == pe_zero(), hex(e_lfanew)
    # Headers past the first block but inside the file are not held: nothing is known.
    unheld = pe32(e_lfanew=5000).build()
    assert len(unheld) > 5000
    assert facts_of(unheld) == pe_zero()
    # Headers ending exactly at the block's end are held; nothing after them is.
    facts = facts_of(pe32(e_lfanew=4096 - 24, directories=[]).build())
    assert facts["pe_machine"] == 0x14C, facts
    assert facts["pe_flags"] == SECTIONS_NOT_HELD | UNKNOWN_MAGIC, facts
    assert facts["pe_valid"] == 0, facts


def test_e_lfanew_inside_the_dos_header_is_parsed_from_the_overlap():
    data = pe32(e_lfanew=0x10).build()
    assert data[:2] == b"MZ" and data[0x10:0x14] == b"PE\0\0" and u32(data, 0x3C) == 0x10
    assert facts_of(data) == pe_expected()
    # Four is the least offset that keeps `MZ` out of the signature.
    assert facts_of(pe32(e_lfanew=4).build()) == pe_expected()


def test_hostile_optional_header_sizes_are_invalid():
    # Without any optional header the section table's first bytes read as the magic.
    facts = facts_of(pe32(size_of_optional_header=0).build())
    assert facts["pe_flags"] == UNKNOWN_MAGIC, facts
    assert (facts["pe_valid"], facts["pe_magic"], facts["pe_overlay"]) == (0, 0, 0), facts
    # One byte short of the fixed fields is invalid, the fixed fields alone are valid.
    for image, short, enough in [(pe32, 95, 96), (pe32plus, 111, 112)]:
        facts = facts_of(image(size_of_optional_header=short).build())
        assert (facts["pe_valid"], facts["pe_flags"], facts["pe_overlay"]) == (0, 0, 0), facts
        assert facts["pe_magic"] == image().magic
        facts = facts_of(image(size_of_optional_header=enough).build())
        assert facts["pe_valid"] == 1 and facts["pe_flags"] == 0, facts
    # A huge optional header pushes the section table out of the block.
    huge = pe32(size_of_optional_header=0xFFFF).build()
    assert len(huge) > 0xFFFF
    facts = facts_of(huge)
    assert facts["pe_flags"] == SECTIONS_NOT_HELD, facts
    assert (facts["pe_valid"], facts["pe_magic"], facts["pe_overlay"]) == (0, PE32_MAGIC, 0)


def test_hostile_section_counts_are_invalid():
    def summary(facts):
        return facts["pe_valid"], facts["pe_flags"], facts["pe_sections"], facts["pe_overlay"]

    assert summary(facts_of(pe32(section_count=0).build())) == (0, 0, 0, 0)
    # The most sections allowed, all held in the block, are valid.
    full = pe32(sections=[(0x1000, 0x10)] * 96, directories=[])
    assert summary(facts_of(full.build())) == (1, 0, 96, 0)
    # One more, still held in the block, is not.
    over = pe32(sections=[(0x1000, 0x10)] * 97, directories=[])
    assert 0x40 + 24 + 96 + 40 * 97 == 4064
    assert summary(facts_of(over.build())) == (0, 0, 97, 0)
    # A count beyond the block is not held.
    facts = facts_of(pe32(section_count=0xFFFF).build())
    assert facts["pe_flags"] == SECTIONS_NOT_HELD, facts
    assert (facts["pe_valid"], facts["pe_sections"], facts["pe_overlay"]) == (0, 0xFFFF, 0)


def test_hostile_directory_counts_bound_the_entries_read():
    image = pe32(length=0x600)
    image.directories[4] = (0x400, 0x200)
    image.directories[14] = (0x2000, 72)
    image.directory_count = 0
    assert facts_of(image.build()) == pe_expected(overlay=0x200)
    image.directory_count = 0xFFFF_FFFF
    assert facts_of(image.build()) == pe_expected(overlay=0x200, signed=1, clr=1)


def test_section_table_cut_by_the_block_is_not_held():
    # The optional header is held whole, the section table lies just beyond the block.
    data = pe32(e_lfanew=4096 - 24 - 224).build()
    assert len(data) > 4096
    expected = pe_expected(valid=0, flags=SECTIONS_NOT_HELD)
    assert facts_of(data) == expected
    # With the table one byte short of held, the same; one byte earlier, held.
    table = 4096 - 24 - 224 - 40
    assert facts_of(pe32(e_lfanew=table + 1).build()) == expected
    # The table ends with the block, which is the whole file: nothing is overlay.
    held = pe32(e_lfanew=table).build()
    assert len(held) == 4096
    assert facts_of(held) == pe_expected()
    # Being cut by a short file rather than the block is the same condition.
    assert facts_of(pe32().build()[: 0x40 + 24 + 224 + 39]) == expected


def test_sections_beyond_the_file_are_flagged_and_contribute_nothing():
    image = pe32(length=0x400, sections=[(0x200, 0x200), (0x10000, 0x10)])
    expected = pe_expected(flags=SECTION_BEYOND_FILE, sections=2)
    assert facts_of(image.build()) == expected
    # The extents are measured without wrapping: the largest u32 pair does not overflow.
    image.sections[1] = (0xFFFF_FFFF, 0xFFFF_FFFF)
    assert facts_of(image.build()) == expected
    # Ending exactly at the file's end is inside it.
    image.sections[1] = (0x3F0, 0x10)
    assert facts_of(image.build()) == pe_expected(sections=2)
    # With every section beyond the file, everything after the headers is overlay.
    image.sections = [(0x10000, 0x10)]
    facts = facts_of(image.build())
    assert (facts["pe_valid"], facts["pe_flags"], facts["pe_overlay"]) == (
        1,
        SECTION_BEYOND_FILE,
        0x400 - HEADERS_END,
    )


def test_headers_are_never_overlay():
    # Sections without raw data leave the headers as the only data: the overlay follows them.
    image = pe32(length=0x400, sections=[(0, 0)])
    assert facts_of(image.build()) == pe_expected(overlay=0x400 - HEADERS_END)
    # A second empty section lengthens the table by 40 bytes.
    image.sections = [(0, 0), (0, 0)]
    expected = pe_expected(sections=2, overlay=0x400 - HEADERS_END - 40)
    assert facts_of(image.build()) == expected
    # A section whose raw data lies inside the headers does not shorten them.
    image.sections = [(0x10, 0x10), (0, 0)]
    assert facts_of(image.build()) == expected
    # A file ending exactly at the headers has no overlay at all.
    image.length = HEADERS_END + 40
    assert facts_of(image.build()) == pe_expected(sections=2)


def test_overlay_measures_the_bytes_after_the_last_section():
    assert facts_of(pe32().build() + b"!" * 1234) == pe_expected(overlay=1234)
    # Sections are not necessarily in file order: the furthest extent counts.
    image = pe32plus(sections=[(0x400, 0x100), (0x200, 0x200)], length=0x500 + 7)
    expected = pe_expected(machine=0x8664, magic=PE32PLUS_MAGIC, sections=2, overlay=7)
    assert facts_of(image.build()) == expected
    # The size is the input's, not the block's: a large payload is measured whole.
    large = pe32().build().ljust(1 << 20, b"\0")
    assert facts_of(large) == pe_expected(overlay=(1 << 20) - 0x400)


def test_minimal_pe32_yields_exact_facts():
    data = pe32().build()
    assert len(data) == 0x400
    assert facts_of(data) == pe_expected()


def test_minimal_pe32plus_yields_exact_facts():
    data = pe32plus(sections=[(0x200, 0x200), (0x400, 0x100)]).build()
    assert len(data) == 0x500
    assert facts_of(data) == pe_expected(machine=0x8664, magic=PE32PLUS_MAGIC, sections=2)
    # The optional header is 16 bytes longer, so the section table moved.
    end = 0x40 + 4 + 20 + 240 + 80
    assert data[end - 40 : end - 40 + 8] == bytes(8)
    assert u32(data, end - 40 + 20) == 0x400


def test_fixture_executables_yield_exact_facts():
    expected = pe_expected(dll_characteristics=0)
    pe32_bytes = (ROOT / "tests_data/mitra/pebin/pe32.exe").read_bytes()
    assert len(pe32_bytes) == 1024
    assert facts_of(pe32_bytes) == expected
    pe64_bytes = (ROOT / "tests_data/mitra/pebin/pe64.exe").read_bytes()
    assert len(pe64_bytes) == 1024
    assert facts_of(pe64_bytes) == pe_expected(
        dll_characteristics=0, machine=0x8664, magic=PE32PLUS_MAGIC
    )


@pytest.mark.parametrize(
    "characteristics,is_dll,is_executable_image",
    [(0x0000, 0, 0), (0x0002, 0, 1), (0x2000, 1, 0), (0x2102, 1, 1), (0xFFFD, 1, 0)],
)
def test_dll_and_executable_image_bits_are_exposed(characteristics, is_dll, is_executable_image):
    facts = facts_of(pe32(characteristics=characteristics).build())
    assert facts["pe_characteristics"] == characteristics
    assert (facts["pe_is_dll"], facts["pe_is_executable_image"]) == (is_dll, is_executable_image)
    assert facts["pe_valid"] == 1, "the bits do not affect validity"


def test_clr_directory_marks_managed_images():
    image = pe32()
    image.directories[14] = (0x2000, 72)
    assert facts_of(image.build()) == pe_expected(clr=1)
    # Either half being zero means no runtime header.
    image.directories[14] = (0x2000, 0)
    assert facts_of(image.build()) == pe_expected()
    image.directories[14] = (0, 72)
    assert facts_of(image.build()) == pe_expected()
    # The entry counts only when the declared count reaches it.
    image.directories[14] = (0x2000, 72)
    image.directory_count = 14
    assert facts_of(image.build()) == pe_expected()
    image.directory_count = 15
    assert facts_of(image.build()) == pe_expected(clr=1)
    # ...and when the declared optional header size covers it: cut to 14 entries, the
    # bytes where entry 14 would be are the first section's name, whatever they hold.
    image.directory_count = None
    image.size_of_optional_header = 96 + 8 * 14
    data = bytearray(image.build())
    name = 0x40 + 4 + 20 + 96 + 8 * 14
    put_u32(data, name, 0x2000)
    put_u32(data, name + 4, 72)
    assert facts_of(bytes(data)) == pe_expected()
    image.size_of_optional_header = 96 + 8 * 15
    assert facts_of(image.build()) == pe_expected(clr=1)
    # A PE32+ image keeps the same directory 16 bytes further along.
    plus = pe32plus()
    plus.directories[14] = (0x2000, 72)
    facts = facts_of(plus.build())
    assert facts["pe_clr"] == 1 and facts["pe_valid"] == 1, facts


def test_certificate_directory_marks_signed_images_when_inside_the_file():
    image = pe32(length=0x600)
    image.directories[4] = (0x400, 0x200)
    signed = pe_expected(signed=1, overlay=0x200)
    assert facts_of(image.build()) == signed
    # Ending exactly at the end of the file is inside it; one byte more is not.
    image.directories[4] = (0x400, 0x201)
    out = pe_expected(flags=CERTIFICATE_OUT_OF_FILE, overlay=0x200)
    assert facts_of(image.build()) == out
    image.directories[4] = (0xFFFF_FFFF, 0xFFFF_FFFF)
    assert facts_of(image.build()) == out
    # An empty table is not a certificate, wherever it claims to be.
    image.directories[4] = (0xFFFF_FFFF, 0)
    assert facts_of(image.build()) == pe_expected(overlay=0x200)
    # Nor is one at offset zero, the DOS header: both halves must be set, as for CLR.
    image.directories[4] = (0, 0x200)
    assert facts_of(image.build()) == pe_expected(overlay=0x200)
    # The directory is only read when the count and the optional header reach it.
    image.directories[4] = (0x400, 0x200)
    image.directory_count = 4
    assert facts_of(image.build()) == pe_expected(overlay=0x200)
    image.directory_count = None
    image.size_of_optional_header = 96 + 8 * 4
    data = bytearray(image.build())
    name = 0x40 + 4 + 20 + 96 + 8 * 4
    put_u32(data, name, 0x400)
    put_u32(data, name + 4, 0x200)
    assert facts_of(bytes(data)) == pe_expected(overlay=0x200)
    # A PE32+ image keeps the same directory 16 bytes further along.
    plus = pe32plus(length=0x600)
    plus.directories[4] = (0x400, 0x200)
    facts = facts_of(plus.build())
    assert (facts["pe_signed"], facts["pe_valid"], facts["pe_overlay"]) == (1, 1, 0x200), facts


@pytest.mark.parametrize("fixture", ["pe32", "pe32plus", "pe32.exe", "pe64.exe"])
def test_every_byte_mutation_of_the_pe_fixtures_is_survived(fixture):
    if fixture.endswith(".exe"):
        original = (ROOT / "tests_data/mitra/pebin" / fixture).read_bytes()
    else:
        original = {"pe32": pe32, "pe32plus": pe32plus}[fixture]().build()
    assert facts_of(original)["pe_valid"] == 1
    held = min(len(original), 4096)
    for at in range(held):
        for value in (0x00, 0xFF, 0x50, original[at] ^ 0x01, original[at] ^ 0x80):
            mutated = bytearray(original)
            mutated[at] = value
            mutated = bytes(mutated)
            pe_check(pe_facts(mutated[:held], len(mutated)), len(mutated))
            # Truncations, and sizes disagreeing with the block, are reachable too.
            pe_check(pe_facts(mutated[:at], at), at)
            pe_check(pe_facts(mutated[:held], 1 << 40), 1 << 40)
            pe_check(pe_facts(mutated[:held], 0), 0)


# --- yara-x's own PE module, an independent decoder of the fixtures -------------------------

PE_MACHINES = {"MACHINE_I386": 0x14C, "MACHINE_AMD64": 0x8664}
PE_SUBSYSTEMS = {"SUBSYSTEM_WINDOWS_CUI": 3, "SUBSYSTEM_WINDOWS_GUI": 2}
PE_MAGICS = {"IMAGE_NT_OPTIONAL_HDR32_MAGIC": 0x10B, "IMAGE_NT_OPTIONAL_HDR64_MAGIC": 0x20B}


PE_CROSS_CHECK = {
    "pe32.exe": lambda: (ROOT / "tests_data/mitra/pebin/pe32.exe").read_bytes(),
    "pe64.exe": lambda: (ROOT / "tests_data/mitra/pebin/pe64.exe").read_bytes(),
    "pe32": lambda: pe32().build(),
    "pe32plus-two-sections": lambda: pe32plus(sections=[(0x200, 0x200), (0x400, 0x100)]).build(),
    "dll": lambda: pe32(characteristics=0x2102).build(),
    "gui-with-overlay": lambda: pe32(subsystem=2, length=0x400 + 100).build(),
    "certificate-in-file": lambda: pe32(
        directories=[(0, 0)] * 4 + [(0x300, 0x100)] + [(0, 0)] * 11
    ).build(),
}


@pytest.mark.parametrize("name", PE_CROSS_CHECK)
def test_pe_facts_agree_with_the_yara_x_pe_module(name):
    data = PE_CROSS_CHECK[name]()
    scanner = yara_x.Scanner(yara_x.compile('import "pe" rule pe_module { condition: true }'))
    module = scanner.scan(data).module_outputs["pe"]
    assert module["is_pe"]
    facts = facts_of(data)
    assert facts["pe_valid"] == 1
    assert facts["pe_machine"] == PE_MACHINES[module["machine"]]
    assert facts["pe_characteristics"] == module["characteristics"]
    assert facts["pe_subsystem"] == PE_SUBSYSTEMS[module["subsystem"]]
    assert facts["pe_dll_characteristics"] == module["dll_characteristics"]
    assert facts["pe_sections"] == module["number_of_sections"]
    assert facts["pe_magic"] == PE_MAGICS[module["opthdr_magic"]]
    # `pe_signed` is the certificate table entry lying inside the file, which the module
    # exposes as data directory 4; its `is_signed` further needs a parsed signature.
    certificate = module["data_directories"][4]
    offset, length = certificate["virtual_address"], certificate["size"]
    assert facts["pe_signed"] == int(0 < length and offset != 0 and offset + length <= len(data))
    assert facts["pe_signed"] >= int(module["is_signed"])
    assert facts["pe_overlay"] == module["overlay"]["size"]
    assert facts["pe_is_dll"] == int(module["characteristics"] & 0x2000 != 0)
    assert facts["pe_is_executable_image"] == int(module["characteristics"] & 0x0002 != 0)


# --- facts_for over the repository's fixtures -----------------------------------------------


def zipfile_names(path):
    """The raw central directory names, as `zipfile` decodes and this test re-encodes them."""
    with zipfile.ZipFile(path) as opened:
        return [
            info.orig_filename.encode("utf-8" if info.flag_bits & 0x800 else "cp437")
            for info in opened.infolist()
        ]


# The zip-based fixtures under `tests_data/basic`, with the stored `mimetype` payload that
# opens the view of those that carry one first.
OFFICE_FIXTURES = {
    "basic/docx/doc.docx": None,
    "basic/docx/magika_test.docx": None,
    "basic/xlsx/magika_test.xlsx": None,
    "basic/pptx/magika_test.pptx": None,
    "basic/zip/magika_test.zip": None,
    "basic/odt/doc.odt": b"application/vnd.oasis.opendocument.text",
    "basic/odt/magika_test.odt": b"application/vnd.oasis.opendocument.text",
    "basic/ods/magika_test.ods": b"application/vnd.oasis.opendocument.spreadsheet",
    "basic/odp/magika_test.odp": b"application/vnd.oasis.opendocument.presentation",
    "basic/epub/doc.epub": b"application/epub+zip",
    "basic/epub/magika_test.epub": b"application/epub+zip",
}


@pytest.mark.parametrize("relative", OFFICE_FIXTURES)
def test_office_fixtures_list_their_members(relative):
    path = ROOT / "tests_data" / relative
    facts, view = facts_for(path)
    assert len(view) == ZIP_NAMES_BYTES
    assert facts["zip_valid"] == 1, facts
    assert facts["zip_flags"] == 0, facts
    assert facts["original_size"] == path.stat().st_size
    assert facts["prefix_size"] == min(4096, path.stat().st_size)
    names = zipfile_names(path)
    assert facts["zip_entries"] == len(names)
    assert facts["zip_names_entries"] == len(names)
    expected = b"".join(b"\n" + name for name in names) + b"\n"
    mimetype = OFFICE_FIXTURES[relative]
    if mimetype is not None:
        assert names[0] == b"mimetype"
        expected = b"\nmimetype=" + mimetype + expected
    assert view[: len(expected)] == expected
    assert not any(view[len(expected) :])
    assert facts["zip_names_len"] == len(expected)
    if relative.endswith((".docx", ".xlsx", ".pptx")):
        assert b"\n[Content_Types].xml\n" in view
    if relative.endswith(".docx"):
        assert b"\nword/document.xml\n" in view
    if relative.endswith(".epub"):
        assert b"\nMETA-INF/container.xml\n" in view
    if mimetype is not None:
        assert view.startswith(b"\nmimetype=" + mimetype + b"\nmimetype\n")


def test_facts_for_zip_fixtures_match_zipfile_where_the_contract_reaches():
    root = ROOT / "tests_data"
    paths = sorted(p for p in root.rglob("*") if p.is_file() and p.suffix in {".zip", ".jar"})
    assert paths, "no zip fixtures"
    seen = 0
    for path in paths:
        facts, view = facts_for(path)
        if not facts["zip_valid"] or facts["zip_flags"] & NAMES_TRUNCATED:
            continue
        try:
            names = zipfile_names(path)
        except (zipfile.BadZipFile, NotImplementedError, UnicodeEncodeError):
            continue
        if len(names) > 96:
            continue
        seen += 1
        sanitized = [name.replace(b"\0", b"\x01").replace(b"\n", b"\x01") for name in names]
        expected = b"".join(b"\n" + name for name in sanitized) + (b"\n" if sanitized else b"")
        assert facts["zip_names_entries"] == len(names), path
        assert view[: len(expected)] == expected, path
    assert seen >= 20


def test_facts_for_reads_a_pe_fixture_and_pads_nothing_into_the_view():
    path = ROOT / "tests_data/mitra/pebin/pe64.exe"
    facts, view = facts_for(path)
    assert set(facts) == set(FACT_NAMES)
    assert facts["pe_valid"] == 1 and facts["pe_machine"] == 0x8664
    assert all(facts[name] == 0 for name in preprocess.ZIP_FACT_NAMES)
    assert facts["original_size"] == facts["prefix_size"] == 1024
    assert view == bytes(ZIP_NAMES_BYTES)
    assert facts_for(path.read_bytes()) == (facts, view)


def test_facts_for_accepts_bytes_and_mirrors_the_held_blocks():
    data = archive([stored(b"word/document.xml", b"<w/>"), stored(b"x", b"")], b"!!")
    facts, view = facts_for(data)
    expected = zip_expected(
        valid=1, entries=2, names_entries=2, names_len=21, comment_len=2, cd_size=2 * 46 + 18
    )
    assert {name: facts[name] for name in preprocess.ZIP_FACT_NAMES} == expected
    assert facts["original_size"] == len(data) and facts["prefix_size"] == len(data)
    assert view == b"\nword/document.xml\nx\n".ljust(ZIP_NAMES_BYTES, b"\0")
    # The same archive behind a stub, seen through the bounded blocks.
    big = b"PK\x03\x04" + bytes(100_000 - 4) + data
    facts, view = facts_for(big)
    assert {name: facts[name] for name in preprocess.ZIP_FACT_NAMES} == expected | {
        "zip_flags": PREPENDED
    }
    assert facts["original_size"] == len(big) and facts["prefix_size"] == 4096
    assert view[:21] == b"\nword/document.xml\nx\n"
    # A prefix without the zip signature never takes the zip path, whatever the tail holds.
    foreign = b"\x89PNG" + big[4:]
    facts, view = facts_for(foreign)
    assert all(facts[name] == 0 for name in FACT_NAMES[2:])
    assert view == bytes(ZIP_NAMES_BYTES)
    # An archive prefix takes the zip path only: a PE header behind it is not read.
    zipped = b"PK\x03\x04" + pe32().build()[4:]
    facts, _ = facts_for(zipped)
    assert all(facts[name] == 0 for name in FACT_NAMES[2:])
    assert facts_for(b"")[0] == dict.fromkeys(FACT_NAMES, 0)


def test_facts_for_output_encodes_into_the_native_header_layout():
    facts, _ = facts_for(b"")
    assert facts["original_size"] == 0
    header = encode_facts(facts_for(pe32().build() + b"!" * 1234)[0])
    assert header[:8] == (0x400 + 1234).to_bytes(8, "big")
    assert header[8:16] == (0x400 + 1234).to_bytes(8, "big")
    assert header[32] == 1 and header[48:56] == (1234).to_bytes(8, "big")


@pytest.mark.skipif(not GOLDEN.is_file(), reason="no native golden facts recorded yet")
def test_native_golden_facts_are_reproduced():
    records = json.loads(GOLDEN.read_text())
    assert records, "empty golden file"
    for record in records:
        facts, view = facts_for(ROOT / record["path"])
        assert facts == record["facts"], record["path"]
        assert hashlib.sha256(view).hexdigest() == record["view_sha256"], record["path"]
