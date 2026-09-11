# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0

"""Exercise maintained signatures directly, including adversarial format lookalikes."""

import struct
from pathlib import Path

import pytest
import yara_x
from magika_rules_benchmark import preprocess
from preprocess_fixtures import archive, pe32, pe32plus, stored

ROOT = Path(__file__).resolve().parents[3]
NEGATIVES = sorted(p for p in (ROOT / "tests_data/rules_negative").iterdir() if p.suffix != ".md")


@pytest.mark.parametrize(
    "label",
    [
        "apk",
        "dbase",
        "emf",
        "pythonbytecode",
        "luabytecode",
        "torrent",
        "macho",
        "mkv",
        "crx",
        "flac",
        "hlp",
        "jp2",
        "mscompress",
        "netcdf",
        "pdb",
        "stata",
        "ese",
        "fits",
        "llvm_bitcode",
        "lrz",
        "postgres_dump",
        "shapefile",
        "spss",
        "vhd",
        "ace",
        "bpg",
        "dsstore",
        "duckdb",
        "avro",
        "parquet",
        "icc",
        "lnk",
        "bam",
        "hdf4",
        "lz4",
        "zst",
        "sketchup",
        "applebplist",
        "appledouble",
        "applesingle",
        "uf2",
        "xcf",
        "rar",
        "mat",
        "gguf",
        "wad",
        "cram",
        "dex",
        "redis_rdb",
        "lz",
        "rzip",
        "xar",
        "spirv",
        "icns",
    ],
)
def test_reviewed_binary_headers(scan_rules, reviewed_binary_headers, label):
    _, header, minimum, invalid = next(row for row in reviewed_binary_headers if row[0] == label)
    assert scan_rules(header) == {label}
    for content in invalid:
        assert scan_rules(content) == set(), content.hex()
    for length in range(8, minimum):
        assert scan_rules(header[:length]) == set(), length


def test_reviewed_binary_header_variants(scan_rules, reviewed_binary_header_variants):
    for label, header in reviewed_binary_header_variants:
        assert scan_rules(header) == {label}, (label, header.hex())


@pytest.mark.parametrize("compression", [0, 8])
@pytest.mark.parametrize("jar_manifest", [False, True])
def test_dex_archive_without_android_manifest_is_not_apk(
    scan_rules, compression, jar_manifest, dex_only_archives
):
    assert "apk" not in scan_rules(dex_only_archives[compression, jar_manifest])


@pytest.mark.parametrize("brand,label", [(b"3gp6", "3gp"), (b"avif", "avif"), (b"heic", "heif")])
def test_ftyp_requires_complete_fixed_header(scan_rules, brand, label):
    header = (20).to_bytes(4, "big") + b"ftyp" + brand + bytes(4) + brand
    assert scan_rules(header) == {label}
    for length in range(8, 16):
        assert scan_rules(header[:length]) == set(), length
    for size in (1, 8, 12, 15, 17, 19, 21, 0xFFFFFFFF):
        assert scan_rules(size.to_bytes(4, "big") + header[4:]) == set(), size
    # A long, complete brand table may extend beyond the scan window.
    large = (4112).to_bytes(4, "big") + header[4:16] + brand * 1024
    assert scan_rules(large) == {label}


@pytest.mark.parametrize("brand", [b"3gp\0", b"3gp?", b"3gm?", b"3gt?", b"isom", b"mif1", b"3g2a"])
def test_ftyp_sibling_and_incomplete_brands_abstain(scan_rules, brand):
    assert scan_rules((20).to_bytes(4, "big") + b"ftyp" + brand + bytes(4) + brand) == set()


@pytest.mark.parametrize(
    "brand,label",
    [
        (brand, "3gp")
        for brand in (b"3gp1", b"3gp4", b"3ge9", b"3gg6", b"3gh9", b"3gmA", b"3gs7", b"3gtv")
    ]
    + [(b"avis", "avif")]
    + [
        (brand, "heif") for brand in (b"heix", b"hevc", b"hevx", b"heim", b"heis", b"hevm", b"hevs")
    ],
)
def test_ftyp_existing_brand_variants(scan_rules, brand, label):
    assert scan_rules((16).to_bytes(4, "big") + b"ftyp" + brand + bytes(4)) == {label}


@pytest.fixture(scope="module")
def matching_rules():
    """The rules YARA-X matches for `content`, enforced or not, given the native facts as globals.

    Maps each matching rule's identifier to its metadata.
    """
    compiler = yara_x.Compiler(includes_enabled=False)
    preprocess.define_globals(compiler)
    for path in sorted((ROOT / "rules/rulesets").rglob("*.yar")):
        compiler.add_source(path.read_text())
    scanner = yara_x.Scanner(compiler.build())

    def scan(content):
        prefix = bytes(content[: preprocess.PREFIX_BYTES])
        preprocess.set_globals(scanner, *preprocess.facts_for(content))
        return {
            rule.identifier: dict(rule.metadata) for rule in scanner.scan(prefix).matching_rules
        }

    return scan


@pytest.fixture(scope="module")
def scan_rules(matching_rules):
    """The enforced labels YARA-X decides for `content`."""

    def scan(content):
        return {meta["label"] for meta in matching_rules(content).values() if meta.get("enforced")}

    return scan


@pytest.mark.parametrize("path", NEGATIVES, ids=lambda p: p.name)
def test_reported_false_positives_abstain(scan_rules, path):
    assert scan_rules(path.read_bytes()) == set()


@pytest.mark.parametrize(
    "magic", [b"SAS", b"ORC", b"CWS", b"FWS", b"ZWS", b"FLV", b"AC10", b"AC1.2", b"AC1.3", b"MC0.0"]
)
@pytest.mark.parametrize(
    "body", [b" release notes\n", b" part number: 123\n", b"\x00" * 4096, b"\xff" * 4096]
)
def test_short_ascii_magic_does_not_establish_identity(scan_rules, magic, body):
    assert scan_rules(magic + body) == set()


@pytest.mark.parametrize(
    "magic",
    [b"\x1f\x8b", b"\xff\x0a", b"\x01\xf7", b"OggS", b"8BPS", b"\x00asm", b"\x00\x00\x01\x00"],
)
@pytest.mark.parametrize("length", range(8))
def test_incomplete_headers_abstain(scan_rules, magic, length):
    assert scan_rules((magic + bytes(8))[:length]) == set()


@pytest.mark.parametrize(
    "magic",
    [
        rb"PK\003\004",
        rb"\037\213",
        rb"7z\274\257\047\034",
        rb"8BPS  \000\000\000\000",
        rb"\0\0\1\0",
        rb"\xed\xab\xee\xdb",
    ],
)
def test_literal_escape_text_abstains(scan_rules, magic):
    assert scan_rules(magic + b" documentation example\n" * 32) == set()


def test_all_public_fixture_incomplete_prefixes_abstain(scan_rules):
    files = sorted(
        p
        for p in (ROOT / "tests_data").rglob("*")
        if p.is_file() and "rules_negative" not in p.parts
    )
    assert len(files) > 100
    for path in files:
        with path.open("rb") as stream:
            prefix = stream.read(8)
        for length in range(min(8, len(prefix) + 1)):
            assert scan_rules(prefix[:length]) == set(), (str(path), length)


POSITIVE_FILES = [
    ("3dsx", "rules_positive/review-3dsx.bin"),
    ("au", "rules_positive/review-au.bin"),
    ("gif", "mitra/gif/gif87.gif"),
    ("gif", "mitra/gif/gif89.gif"),
    ("pcap", "mitra/pcap/pcap.pcap"),
    ("fbx", "rules_positive/review-fbx-legacy.bin"),
    ("bzip", "mitra/bzip/bzip2.bz2"),
    *[
        (label, f"rules_positive/review-{label}.bin")
        for label in ("qoi", "gltf", "woff", "woff2", "npy", "midi", "fbx", "blend")
    ],
    ("bmp", "mitra/bmp/bmp.bmp"),
    ("cab", "mitra/cab/cab.cab"),
    ("gzip", "mitra/gzip/gzip.gz"),
    ("wav", "mitra/wav/riff.wav"),
    ("wav", "mitra/wav/rifx.wav"),
    ("wav", "basic/wav/test.wav"),
    ("epub", "basic/epub/doc.epub"),
    ("epub", "basic/epub/magika_test.epub"),
    ("ogg", "basic/ogg/test.ogg"),
    ("ogg", "mitra/ogg/vorbis.ogg"),
    ("psd", "basic/psd/MagikaTest.psd"),
    ("sevenzip", "mitra/sevenzip/7-zip.7z"),
    ("ico", "mitra_candidates/ico.ico"),
    ("wasm", "mitra_candidates/wasm.wasm"),
    ("swf", "rules_positive/zws-0.swf"),
    ("swf", "rules_positive/zws-1.swf"),
    # Decided from the central directory names and the PE headers (preprocessor facts).
    ("xlsx", "basic/xlsx/magika_test.xlsx"),
    ("pptx", "basic/pptx/magika_test.pptx"),
    ("odt", "basic/odt/doc.odt"),
    ("odt", "basic/odt/magika_test.odt"),
    ("ods", "basic/ods/magika_test.ods"),
    ("odp", "basic/odp/magika_test.odp"),
    ("pebin", "mitra/pebin/pe32.exe"),
    ("pebin", "mitra/pebin/pe64.exe"),
]


@pytest.mark.parametrize("label,relative", POSITIVE_FILES)
def test_real_positive_fixtures(scan_rules, label, relative):
    assert scan_rules((ROOT / "tests_data" / relative).read_bytes()) == {label}


@pytest.mark.parametrize(
    "magic",
    [
        b"BZh",
        b"BZh9",
        b"qoif",
        b"glTF",
        b"wOFF",
        b"wOF2",
        b"\x93NUMPY",
        b"MThd",
        b"Kaydara FBX Binary  \0",
        b"BLENDER",
    ],
)
@pytest.mark.parametrize("padding", [b"\0", b"\xff", b" "])
def test_additional_padded_signatures_abstain(scan_rules, magic, padding):
    assert scan_rules(magic + padding * 4096) == set()


@pytest.mark.parametrize(
    "label,offset,invalid",
    [
        ("qoi", 4, bytes(4)),
        ("qoi", 8, bytes(4)),
        ("qoi", 12, b"\x02"),
        ("qoi", 13, b"\x02"),
        ("gltf", 4, bytes(4)),
        ("gltf", 8, bytes(4)),
        ("gltf", 12, b"\x03\0\0\0"),
        ("gltf", 16, b"BIN\0"),
        ("woff", 12, bytes(2)),
        ("woff", 14, b"\0\x01"),
        ("woff2", 12, bytes(2)),
        ("woff2", 14, b"\0\x01"),
        ("woff2", 20, bytes(4)),
        ("npy", 6, b"\x04"),
        ("npy", 7, b"\x01"),
        ("npy", 8, bytes(2)),
        ("midi", 4, bytes(4)),
        ("midi", 8, b"\0\x03"),
        ("midi", 10, bytes(2)),
        ("midi", 14, b"TEXT"),
        ("fbx", 21, b"\0"),
        ("fbx", 23, bytes(4)),
        ("blend", 7, b"?"),
        ("blend", 8, b"?"),
        ("blend", 9, b"abc"),
    ],
)
def test_additional_header_corruptions_abstain(scan_rules, label, offset, invalid):
    data = bytearray((ROOT / f"tests_data/rules_positive/review-{label}.bin").read_bytes())
    data[offset : offset + len(invalid)] = invalid
    assert label not in scan_rules(data)


@pytest.mark.parametrize("content", [b"", b"ordinary compressed content"])
@pytest.mark.parametrize("level", range(1, 10))
def test_bzip2_standard_encoder_and_damaged_block_magic(scan_rules, content, level):
    import bz2

    compressed = bz2.compress(content, compresslevel=level)
    assert scan_rules(compressed) == {"bzip"}
    assert "bzip" not in scan_rules(compressed[:4] + bytes(len(compressed) - 4))


@pytest.mark.parametrize("channels", [3, 4])
@pytest.mark.parametrize("colorspace", [0, 1])
def test_qoi_complete_single_pixel_image(scan_rules, channels, colorspace):
    import struct

    # One run of the initial opaque black pixel, then the required stream terminator.
    content = (
        b"qoif" + struct.pack(">IIBB", 1, 1, channels, colorspace) + b"\xc0" + bytes(7) + b"\x01"
    )
    assert scan_rules(content) == {"qoi"}
    for size in range(23):
        assert "qoi" not in scan_rules(content[:size])


@pytest.mark.parametrize(
    "label,minimum",
    [
        ("qoi", 23),
        ("gltf", 24),
        ("woff", 64),
        ("woff2", 51),
        ("npy", 64),
        ("midi", 26),
        ("fbx", 27),
        ("blend", 32),
    ],
)
def test_additional_format_header_truncations(scan_rules, label, minimum):
    content = (ROOT / f"tests_data/rules_positive/review-{label}.bin").read_bytes()
    for length in range(minimum):
        assert label not in scan_rules(content[:length]), (label, length)


@pytest.mark.parametrize("division", [0x8001, 0xFF01, 0xE800, 0xE700, 0xE300, 0xE200])
def test_midi_invalid_smpte_division_abstains(scan_rules, division):
    content = bytearray((ROOT / "tests_data/rules_positive/review-midi.bin").read_bytes())
    content[12:14] = division.to_bytes(2, "big")
    assert "midi" not in scan_rules(content)


@pytest.mark.parametrize("major", [1, 2, 3])
@pytest.mark.parametrize("key_order", [0, 1])
def test_numpy_supported_versions_and_dictionary_order(scan_rules, major, key_order):
    import struct

    # A complete empty uint8 array with a Python-literal dictionary header.
    headers = [
        "{'descr': '|u1', 'fortran_order': False, 'shape': (0,), }",
        "{'shape': (0,), 'fortran_order': False, 'descr': '|u1', }",
    ]
    header = headers[key_order].encode()
    offset = 10 if major == 1 else 12
    header += b" " * ((-offset - len(header) - 1) % 64) + b"\n"
    length = struct.pack("<H" if major == 1 else "<I", len(header))
    content = b"\x93NUMPY" + bytes([major, 0]) + length + header
    assert scan_rules(content) == {"npy"}


@pytest.mark.parametrize("version", [0, 2, 13, 255])
def test_wasm_requires_standard_version(scan_rules, version):
    assert scan_rules(b"\0asm" + version.to_bytes(4, "little") + bytes(32)) == set()


def test_empty_wasm_module_is_valid(scan_rules):
    assert scan_rules(b"\0asm\1\0\0\0") == {"wasm"}


@pytest.mark.parametrize("field,values", [(2, [0, 7, 9, 255]), (3, [32, 64, 128, 255])])
def test_gzip_rejects_invalid_method_and_reserved_flags(scan_rules, field, values):
    content = bytearray((ROOT / "tests_data/mitra/gzip/gzip.gz").read_bytes())
    for value in values:
        corrupt = content.copy()
        corrupt[field] = value
        assert scan_rules(corrupt) == set(), (field, value)


@pytest.mark.parametrize("relative", [p for _, p in POSITIVE_FILES if p.startswith("basic/epub")])
@pytest.mark.parametrize(
    "offset,replacement",
    [(0, b"NOPE"), (8, b"\x08\x00"), (26, b"\x07\x00"), (28, b"\x01\x00"), (30, b"M"), (38, b"X")],
)
def test_epub_requires_correlated_zip_and_mimetype_fields(
    scan_rules, relative, offset, replacement
):
    content = bytearray((ROOT / "tests_data" / relative).read_bytes())
    content[offset : offset + len(replacement)] = replacement
    assert "epub" not in scan_rules(content)


@pytest.mark.parametrize("label", ["docx", "xlsx", "pptx", "jar", "apk", "odt", "zip"])
def test_zip_siblings_are_never_epub(scan_rules, label):
    files = list((ROOT / "tests_data" / "basic" / label).glob("*"))
    files += list((ROOT / "tests_data" / "mitra" / label).glob("*"))
    if not files:
        # Construct a ZIP with the sibling's conventional member, independently
        # of the format rule. The repository has no JAR/APK positive fixture.
        import io
        import zipfile

        members = {"jar": "META-INF/MANIFEST.MF", "apk": "AndroidManifest.xml"}
        output = io.BytesIO()
        with zipfile.ZipFile(output, "w") as archive:
            archive.writestr(members[label], "regression fixture")
        assert "epub" not in scan_rules(output.getvalue())
        return
    for path in files:
        if path.is_file():
            assert "epub" not in scan_rules(path.read_bytes()), path


@pytest.mark.parametrize(
    "relative,offset,values",
    [
        ("basic/ogg/test.ogg", 4, [1, 2, 255]),
        ("basic/ogg/test.ogg", 5, [8, 32, 255]),
        ("mitra_candidates/ico.ico", 4, [0]),
        ("mitra_candidates/ico.ico", 9, [1, 255]),
        ("basic/psd/MagikaTest.psd", 5, [0, 3, 255]),
        ("basic/psd/MagikaTest.psd", 6, [1, 255]),
    ],
)
def test_invalid_structural_fields_abstain(scan_rules, relative, offset, values):
    content = bytearray((ROOT / "tests_data" / relative).read_bytes())
    for value in values:
        corrupt = content.copy()
        corrupt[offset] = value
        assert scan_rules(corrupt) == set(), (relative, offset, value)


@pytest.mark.parametrize("value", range(256))
def test_wasm_magic_and_version_mutations(scan_rules, value):
    header = b"\0asm\1\0\0\0"
    for offset, original in enumerate(header):
        if value != original:
            corrupt = header[:offset] + bytes([value]) + header[offset + 1 :]
            assert "wasm" not in scan_rules(corrupt), (offset, value)


@pytest.mark.parametrize("packet_size,start", [(188, 0), (192, 4)])
def test_transport_stream_every_observed_packet_header(scan_rules, packet_size, start):
    # Legal null packets: PID 0x1fff, payload present, no transport error.
    packet = b"\x47\x1f\xff\x10" + b"\xff" * 184
    if start:
        packet = bytes(4) + packet
    content = packet * 21
    assert scan_rules(content) == {"mpegts"}
    for index in range(21):
        for offset, invalid in [(0, 0), (1, 255), (3, 0)]:
            corrupt = bytearray(content)
            corrupt[index * packet_size + start + offset] = invalid
            assert "mpegts" not in scan_rules(corrupt), (index, offset)
    assert "mpegts" not in scan_rules(content[:-1])


@pytest.mark.parametrize("length", [0, 1, 7, 8, 9, 10, 18, 19])
def test_gzip_truncation_does_not_override(scan_rules, length):
    import gzip

    content = gzip.compress(b"regression fixture", mtime=0)
    assert scan_rules(content) == {"gzip"}
    assert "gzip" not in scan_rules(content[:length])


@pytest.mark.native
def test_native_engine_agrees_on_adversarial_and_positive_corpus(tmp_path):
    import hashlib
    import os

    from magika_rules_benchmark.runner import observe

    pack = tmp_path / "regressions.yar"
    pack.write_text(
        "\n".join(p.read_text() for p in sorted((ROOT / "rules/rulesets").rglob("*.yar")))
    )
    cases = [(path, None) for path in NEGATIVES]
    cases += [(ROOT / "tests_data" / path, label) for label, path in POSITIVE_FILES]
    generated = []
    for version in (b"87a", b"89a"):
        for bits in (None, *range(8)):
            content = gif_fixture(version, bits)
            offset = 13 + (3 * (2 << bits) if bits is not None else 0)
            generated += [(content, "gif"), (content[:offset], None)]
            generated.append((content[:offset] + b"\0" + content[offset + 1 :], None))
    for order in ("little", "big"):
        for nano in (False, True):
            content = pcap_fixture(order, nano)
            generated.append((content, "pcap"))
            generated.extend((content[:length], None) for length in range(24))
            generated.append((content[:16] + bytes(4) + content[20:], None))
        for encoding in (*range(1, 15), *range(16, 28)):
            content = (
                (b".snd" if order == "big" else b"dns.")
                + b"".join(
                    value.to_bytes(4, order) for value in [24, 0xFFFFFFFF, encoding, 96001, 3]
                )
                + bytes(16)
            )
            generated += [(content, "au"), (content[:20] + bytes(4) + content[24:], None)]
    for label, minimum in (("3dsx", 56), ("au", 24)):
        content = (ROOT / f"tests_data/rules_positive/review-{label}.bin").read_bytes()
        generated.extend((content[:length], None) for length in range(minimum))
    for index, (content, expected) in enumerate(generated):
        path = tmp_path / f"gif-pcap-{index}.bin"
        path.write_bytes(content)
        cases.append((path, expected))
    records = [
        dict(
            path=str(path),
            size=path.stat().st_size,
            sha256=hashlib.sha256(path.read_bytes()).hexdigest(),
        )
        for path, _ in cases
    ]
    rows, _ = observe(
        Path(os.environ["MAGIKA_TEST_BINARY"]),
        pack,
        records,
        {},
        dict(os.environ, MAGIKA_RULES_CACHE=str(tmp_path / "cache")),
    )
    for row, (_, expected) in zip(rows, cases, strict=True):
        assert row["reference_error"] is None
        assert row["rule_prediction"] == expected, row["path"]
        assert not row["reference_mismatch"], row["path"]
    assert sum(row["product_rule"] for row in rows) > 0


@pytest.mark.parametrize("flags", [0, 2, 6, 22, 2048, 2050, 2070])
def test_epub_stored_entries_allow_non_encryption_zip_flags(scan_rules, flags):
    content = bytearray((ROOT / "tests_data/basic/epub/doc.epub").read_bytes())
    content[6:8] = flags.to_bytes(2, "little")
    assert scan_rules(content) == {"epub"}


@pytest.mark.parametrize("flags", [1, 3, 7, 23, 2049, 2071])
def test_epub_encrypted_mimetype_is_not_accepted_by_the_prefix_rule(matching_rules, flags):
    content = bytearray((ROOT / "tests_data/basic/epub/doc.epub").read_bytes())
    content[6:8] = flags.to_bytes(2, "little")
    matched = matching_rules(content)
    assert "taxonomy_epub" not in matched
    # The directory names rule reads the stored media type and the container entry, which
    # a flipped local flag leaves in place: the package is still decided as EPUB by it.
    assert {name for name in matched if name.startswith("taxonomy_epub")} == {"taxonomy_epub_names"}


def test_epub_with_zip_data_descriptor(scan_rules):
    import io
    import zipfile

    class Unseekable(io.BytesIO):
        def seekable(self):
            return False

        def seek(self, *args):
            raise io.UnsupportedOperation("streaming ZIP fixture")

    output = Unseekable()
    with zipfile.ZipFile(output, "w") as archive:
        archive.writestr("mimetype", b"application/epub+zip")
    content = output.getvalue()
    assert content[6] == 8
    assert scan_rules(content) == {"epub"}
    assert "epub" not in scan_rules(content[:58])


@pytest.mark.parametrize("literal", [rb"BMxxxx\000\000", rb"MSCF\0\0\0\0", rb"\037\235"])
def test_remaining_literal_escape_signatures_are_not_binary_formats(scan_rules, literal):
    assert scan_rules(literal + b" documentation example\n" * 8) == set()


@pytest.mark.parametrize("flags", [0, 8, 17, 31, 32, 64, 127, 128, 145, 255])
def test_unix_compress_requires_valid_lzw_flags(scan_rules, flags):
    assert scan_rules(b"\x1f\x9d" + bytes([flags]) + bytes(128)) == set()


@pytest.mark.parametrize("suffix", [bytes(128), b"notes " * 32, b"\xff" * 128])
def test_cab_signature_alone_is_insufficient(scan_rules, suffix):
    assert scan_rules(b"MSCF" + suffix) == set()


def gif_fixture(version=b"89a", palette_bits=None):
    """A complete one-pixel GIF with a local or global color table."""
    palette = b"\x00\x00\x00\xff\xff\xff"
    global_table = (
        palette + bytes(3 * ((2 << palette_bits) - 2)) if palette_bits is not None else b""
    )
    flags = 0x80 | palette_bits if palette_bits is not None else 0
    screen = b"\x01\x00\x01\x00" + bytes([flags, 0, 0])
    image = b",\x00\x00\x00\x00\x01\x00\x01\x00"
    image += b"\x00" if global_table else b"\x80" + palette
    return b"GIF" + version + screen + global_table + image + b"\x02\x02\x44\x01\x00;"


@pytest.mark.parametrize("version", [b"87a", b"89a"])
@pytest.mark.parametrize("palette_bits", [None, *range(8)])
def test_gif_color_table_boundaries(scan_rules, version, palette_bits):
    content = gif_fixture(version, palette_bits)
    assert scan_rules(content) == {"gif"}
    block_offset = 13 + (3 * (2 << palette_bits) if palette_bits is not None else 0)
    assert "gif" not in scan_rules(content[:block_offset])
    damaged = bytearray(content)
    damaged[block_offset] = 0
    assert "gif" not in scan_rules(damaged)


@pytest.mark.parametrize(
    "offset,replacement", [(3, b"80a"), (4, b"9b"), (6, b"\0\0"), (8, b"\0\0")]
)
def test_gif_requires_version_and_dimensions(scan_rules, offset, replacement):
    content = bytearray(gif_fixture())
    content[offset : offset + len(replacement)] = replacement
    assert "gif" not in scan_rules(content)


def pcap_fixture(order="little", nano=False):
    magic = 0xA1B23C4D if nano else 0xA1B2C3D4
    # Empty captures are valid, and the two old timezone/accuracy fields are ignored by readers.
    return (
        magic.to_bytes(4, order)
        + (2).to_bytes(2, order)
        + (4).to_bytes(2, order)
        + (3600).to_bytes(4, order)
        + (42).to_bytes(4, order)
        + (65535).to_bytes(4, order)
        + (147).to_bytes(4, order)
    )


@pytest.mark.parametrize("order", ["little", "big"])
@pytest.mark.parametrize("nano", [False, True])
def test_pcap_complete_header_and_endianness(scan_rules, order, nano):
    content = pcap_fixture(order, nano)
    assert scan_rules(content) == {"pcap"}
    for length in range(24):
        assert "pcap" not in scan_rules(content[:length])
    for offset, width, value in [(4, 2, 0), (4, 2, 1), (4, 2, 3), (6, 2, 0), (16, 4, 0)]:
        damaged = bytearray(content)
        damaged[offset : offset + width] = value.to_bytes(width, order)
        assert "pcap" not in scan_rules(damaged)


@pytest.mark.parametrize(
    "magic",
    [
        b"GIF8",
        b"GIF87a",
        b"GIF89a",
        *(pcap_fixture(order, nano)[:4] for order in ("little", "big") for nano in (False, True)),
    ],
)
@pytest.mark.parametrize("padding", [b"\0", b"\xff", b" "])
def test_gif_and_pcap_padded_magic_abstains(scan_rules, magic, padding):
    assert scan_rules(magic + padding * 1024) == set()


@pytest.mark.parametrize("label,minimum", [("3dsx", 56), ("au", 24)])
def test_3dsx_and_au_require_complete_headers(scan_rules, label, minimum):
    content = (ROOT / f"tests_data/rules_positive/review-{label}.bin").read_bytes()
    assert scan_rules(content) == {label}
    for length in range(minimum):
        assert label not in scan_rules(content[:length])


@pytest.mark.parametrize(
    "label,offset,replacement",
    [
        ("3dsx", 4, b"\x04\x00"),
        ("3dsx", 6, b"\0\0"),
        ("3dsx", 8, b"\x01"),
        ("3dsx", 12, b"\x01"),
        ("3dsx", 16, bytes(4)),
        ("3dsx", 16, b"\x01\0\0\0"),
        ("au", 4, bytes(4)),
        ("au", 4, b"\0\0\0\x14"),
        ("au", 12, bytes(4)),
        ("au", 12, b"\xff" * 4),
        ("au", 16, bytes(4)),
        ("au", 20, bytes(4)),
    ],
)
def test_3dsx_and_au_invalid_header_fields_abstain(scan_rules, label, offset, replacement):
    content = bytearray((ROOT / f"tests_data/rules_positive/review-{label}.bin").read_bytes())
    content[offset : offset + len(replacement)] = replacement
    assert label not in scan_rules(content)


@pytest.mark.parametrize("magic", [b"3DSX", b".snd"])
@pytest.mark.parametrize("padding", [b"\0", b"\xff", b" "])
def test_3dsx_and_au_padded_magic_abstains(scan_rules, magic, padding):
    assert scan_rules(magic + padding * 1024) == set()


@pytest.mark.parametrize("order", ["big", "little"])
@pytest.mark.parametrize("encoding", [*range(1, 8), 23, 24, 25, 26, 27])
def test_au_known_sample_encodings_and_byte_orders(scan_rules, order, encoding):
    magic = b".snd" if order == "big" else b"dns."
    # Streaming size is unknown; a sample rate or channel count need not be a common preset.
    content = (
        magic
        + b"".join(x.to_bytes(4, order) for x in [24, 0xFFFFFFFF, encoding, 96001, 3])
        + bytes(16)
    )
    assert scan_rules(content) == {"au"}


@pytest.mark.parametrize("size", [32, 44])
def test_3dsx_standard_and_extended_headers(scan_rules, size):
    import struct

    content = struct.pack("<4sHH6I", b"3DSX", size, 8, 0, 0, 4, 0, 0, 0)
    content += bytes(size - 32 + 24) + bytes.fromhex("1e ff 2f e1")
    assert scan_rules(content) == {"3dsx"}


# --- container and PE preprocessor rules ------------------------------------------------------


def names_archive(*names, comment=b""):
    """A stored archive listing `names` in order, each with a short payload."""
    return archive([stored(name, b"<x/>") for name in names], comment)


@pytest.mark.parametrize(
    "names,label",
    [
        ((b"[Content_Types].xml", b"xl/workbook.xml"), "xlsx"),
        ((b"doc.kml", b"files/overlay.png"), "kmz"),
        ((b"3D/3dmodel.model", b"[Content_Types].xml"), "3mf"),
        ((b"project.qgs", b"project.qgd"), "qgis"),
        ((b"visio/document.xml", b"[Content_Types].xml"), "visio"),
        ((b"metadata.json", b"config.json", b"model.weights.h5"), "keras"),
        ((b"xl/workbook.xml", b"_rels/.rels", b"[Content_Types].xml"), "xlsx"),
        ((b"[Content_Types].xml", b"ppt/presentation.xml"), "pptx"),
        ((b"ppt/slides/slide1.xml", b"ppt/presentation.xml", b"[Content_Types].xml"), "pptx"),
        ((b"META-INF/MANIFEST.MF", b"com/example/Main.class"), "jar"),
        ((b"a/b/C.class", b"META-INF/MANIFEST.MF"), "jar"),
        ((b"AndroidManifest.xml", b"classes.dex"), "apk"),
        ((b"AndroidManifest.xml", b"res/values.xml", b"classes.dex"), "apk"),
        ((b"res/values.xml", b"AndroidManifest.xml", b"resources.arsc"), "apk"),
        ((b"META-INF/MANIFEST.MF", b"AndroidManifest.xml", b"classes.dex"), "apk"),
    ],
)
def test_central_directory_names_decide_the_package(scan_rules, names, label):
    assert scan_rules(names_archive(*names)) == {label}


@pytest.mark.parametrize(
    "names",
    [
        (b"[Content_Types].xml",),
        (b"xl/workbook.xml",),
        (b"ppt/presentation.xml",),
        (b"[Content_Types].xml", b"xl/workbook.bin"),
        (b"[Content_Types].xml", b"word/document.xml"),
        (b"[Content_Types].xml", b"backup/xl/workbook.xml"),
        (b"[Content_Types].xml", b"xl/workbook.xml.bak"),
        (b"[Content_Types].xml", b"XL/WORKBOOK.XML"),
        (b"META-INF/MANIFEST.MF",),
        (b"META-INF/MANIFEST.MF", b"resources/strings.properties"),
        (b"com/example/Main.class",),
        (b"META-INF/MANIFEST.MF", b".classpath"),
        (b"META-INF/MANIFEST.MF", b"Main.class/"),
        (b"META-INF/manifest.mf", b"Main.class"),
        (b"AndroidManifest.xml", b"classes.jar", b"R.txt"),
        (b"R.txt", b"AndroidManifest.xml", b"classes.jar"),
        (b"AndroidManifest.xml", b"res/values.xml"),
        (b"classes.dex",),
        (b"resources.arsc", b"classes.dex"),
        (b"assets/AndroidManifest.xml", b"classes.dex"),
        (b"notes.txt",),
        (b"other.kml",),
        (b"3D/other.model",),
        (b"metadata.json", b"config.json"),
        (b"visio/masters.xml",),
    ],
)
def test_partial_or_lookalike_names_abstain(scan_rules, names):
    assert scan_rules(names_archive(*names)) == set()


def test_manifest_first_package_needs_code_or_resources_in_the_prefix_or_the_directory(
    scan_rules,
):
    # An Android library opens with AndroidManifest.xml too: the prefix rule needs a
    # classes.dex or resources.arsc local entry, the names rule needs the directory entry.
    manifest = stored(b"AndroidManifest.xml", b"<manifest/>")
    library = archive([manifest, stored(b"classes.jar", b"PK"), stored(b"R.txt", b"")])
    assert scan_rules(library) == set()
    for code in (b"classes.dex", b"resources.arsc"):
        package = archive([manifest, stored(code, b"data")])
        assert scan_rules(package) == {"apk"}
        # The code entry beyond the prefix leaves the decision to the directory names.
        deferred = archive([manifest, stored(b"res/big.bin", bytes(8192)), stored(code, b"x")])
        assert scan_rules(deferred) == {"apk"}
        assert scan_rules(deferred[:4096]) == set()


def test_word_documents_and_templates_are_told_apart_by_content_type(scan_rules):
    # A document and a template list the same parts; only the declared main content type
    # in the content types stream, the archive's first entry, separates them.
    def package(main):
        types = b'<Types><Override PartName="/word/document.xml" ContentType="' + main + b'"/>'
        return archive([stored(b"[Content_Types].xml", types), stored(b"word/document.xml", b"")])

    office = b"application/vnd.openxmlformats-officedocument.wordprocessingml."
    assert scan_rules(package(office + b"document.main+xml")) == {"docx"}
    assert scan_rules(package(office + b"template.main+xml")) == {"dotx"}
    assert scan_rules(package(b"application/vnd.ms-word.document.macroEnabled.main+xml")) == {
        "docx"
    }
    # The same names without the content types stream first decide nothing.
    names = archive([stored(b"word/document.xml", b""), stored(b"[Content_Types].xml", b"")])
    assert scan_rules(names) == set()


def test_names_in_the_comment_or_member_data_do_not_count(scan_rules):
    names = b"\n[Content_Types].xml\n\nxl/workbook.xml\n\nMETA-INF/MANIFEST.MF\n\nMain.class\n"
    assert scan_rules(archive([stored(b"readme.txt", names)], names)) == set()


def test_names_beyond_the_view_are_not_seen(scan_rules):
    filler = [stored(f"d/{i:0>58}".encode(), b"") for i in range(100)]
    parts = [stored(b"[Content_Types].xml", b""), stored(b"xl/workbook.xml", b"")]
    assert scan_rules(archive(filler + parts)) == set()
    assert scan_rules(archive(parts + filler)) == {"xlsx"}


@pytest.mark.parametrize(
    "offset,value",
    [(4, 1), (6, 1), (8, 0), (4, 0xFFFF), (10, 0xFFFF), (12, 0xFFFFFFFF), (16, 0xFFFFFFFF)],
)
def test_split_and_zip64_end_records_abstain(scan_rules, offset, value):
    content = bytearray(names_archive(b"[Content_Types].xml", b"xl/workbook.xml"))
    assert scan_rules(content) == {"xlsx"}
    width = "<I" if value > 0xFFFF else "<H"
    struct.pack_into(width, content, len(content) - 22 + offset, value)
    assert scan_rules(content) == set()


def opendocument(media, *names, method=0, first=True):
    """An ODF-style package: a `mimetype` entry naming `media`, stored by default and first."""
    mimetype = (b"mimetype", b"application/vnd.oasis.opendocument." + media, method)
    others = [stored(name, b"<x/>") for name in names]
    return archive([mimetype, *others] if first else [*others, mimetype])


@pytest.mark.parametrize(
    "media,label", [(b"text", "odt"), (b"spreadsheet", "ods"), (b"presentation", "odp")]
)
def test_opendocument_media_type_and_content_decide(scan_rules, media, label):
    assert scan_rules(opendocument(media, b"content.xml", b"styles.xml")) == {label}
    assert scan_rules(opendocument(media, b"styles.xml", b"meta.xml")) == set()
    assert scan_rules(opendocument(media, b"content.xml", first=False)) == set()
    assert scan_rules(opendocument(media, b"content.xml", method=8)) == set()
    for suffix in (b"-template", b"-master", b"-web", b"\n", b" ", b"s"):
        assert scan_rules(opendocument(media + suffix, b"content.xml")) == set(), suffix


@pytest.mark.parametrize("media", [b"graphics", b"chart", b"formula", b"image", b"database"])
def test_other_opendocument_media_types_abstain(scan_rules, media):
    assert scan_rules(opendocument(media, b"content.xml")) == set()


def ocf_with_extra_fields(members):
    """A stored archive whose local headers carry an extra field, so prefix rules abstain."""
    import io
    import zipfile

    output = io.BytesIO()
    with zipfile.ZipFile(output, "w", compression=zipfile.ZIP_STORED) as package:
        for name, data in members.items():
            info = zipfile.ZipInfo(name, date_time=(2020, 1, 1, 0, 0, 0))
            info.extra = b"UT\x05\x00\x01\x00\x00\x00\x00"
            package.writestr(info, data)
    content = output.getvalue()
    assert content[28:30] != b"\x00\x00", "the first local header carries an extra field"
    return content


def test_epub_directory_names_decide_when_the_local_header_is_not_canonical(scan_rules):
    package = {"mimetype": b"application/epub+zip", "META-INF/container.xml": b"<c/>"}
    assert scan_rules(ocf_with_extra_fields(package)) == {"epub"}
    package = {"mimetype": b"application/epub+zip", "OEBPS/content.opf": b"<p/>"}
    assert scan_rules(ocf_with_extra_fields(package)) == set()
    package = {"mimetype": b"application/epub+zip ", "META-INF/container.xml": b"<c/>"}
    assert scan_rules(ocf_with_extra_fields(package)) == set()
    package = {"META-INF/container.xml": b"<c/>", "mimetype": b"application/epub+zip"}
    assert scan_rules(ocf_with_extra_fields(package)) == set()


@pytest.mark.parametrize(
    "build,overrides",
    [
        (pe32, {}),
        (pe32plus, {}),
        (pe32, {"characteristics": 0x2102}),
        (pe32, {"subsystem": 1}),
        (pe32, {"subsystem": 16}),
        (pe32, {"machine": 0x1C0}),
        (pe32, {"machine": 0x1C2}),
        (pe32, {"machine": 0x1C4}),
        (pe32, {"machine": 0x5032}),
        (pe32plus, {"machine": 0xAA64}),
        (pe32plus, {"machine": 0x200}),
        (pe32plus, {"machine": 0x5064}),
    ],
)
def test_windows_images_are_pebin(scan_rules, build, overrides):
    assert scan_rules(build(**overrides).build()) == {"pebin"}


@pytest.mark.parametrize(
    "build,overrides",
    [
        (pe32, {"characteristics": 0x0100}),
        (pe32, {"characteristics": 0x2000}),
        (pe32, {"subsystem": 0}),
        (pe32, {"subsystem": 17}),
        (pe32, {"subsystem": 0xFFFF}),
        (pe32, {"machine": 0}),
        (pe32, {"machine": 0x166}),
        (pe32, {"machine": 0xEC20}),
        (pe32, {"magic": 0x107}),
        (pe32, {"section_count": 0}),
        (pe32, {"section_count": 200}),
        (pe32, {"size_of_optional_header": 95}),
        (pe32plus, {"size_of_optional_header": 111}),
    ],
)
def test_images_outside_the_pe_contract_abstain(scan_rules, build, overrides):
    assert scan_rules(build(**overrides).build()) == set()


@pytest.mark.parametrize("relative", ["mitra/pebin/pe32.exe", "mitra/pebin/pe64.exe"])
def test_pe_fixture_headers_must_be_held(scan_rules, relative):
    content = (ROOT / "tests_data" / relative).read_bytes()
    e_lfanew = int.from_bytes(content[0x3C:0x40], "little")
    sections = int.from_bytes(content[e_lfanew + 6 : e_lfanew + 8], "little")
    optional = int.from_bytes(content[e_lfanew + 20 : e_lfanew + 22], "little")
    headers_end = e_lfanew + 24 + optional + 40 * sections
    assert scan_rules(content[:headers_end]) == {"pebin"}
    for length in range(headers_end):
        assert scan_rules(content[:length]) == set(), length


def _u16(value):
    return value.to_bytes(2, "little")


def _u32(value):
    return value.to_bytes(4, "little")


PREFIX_SIGNATURES = {
    "pcapng": (
        b"\n\r\r\n" + _u32(28) + b"\x4d\x3c\x2b\x1a" + _u16(1) + _u16(0) + b"\xff" * 8 + _u32(28),
        b"\n\r\r\n" + _u32(28) + b"\x4d\x3c\x2b\x1a" + _u16(2) + _u16(0) + b"\xff" * 8 + _u32(28),
    ),
    "xcoff": (
        b"\x01\xf7\x00\x04" + bytes(12) + b"\x00\x78\x00\x02" + bytes(4),
        b"\x01\xf7\x00\x04" + bytes(12) + b"\x00\x63\x00\x02" + bytes(4),
    ),
    "ani": (
        b"RIFF" + _u32(100) + b"ACONanih" + bytes(4),
        b"RIFF" + _u32(100) + b"ACOXfmt " + bytes(4),
    ),
    "postscript": (b"%!PS-Adobe-3.0\n%%EndComments\n", b"%!PS\n%%Title: short opening\n"),
    "pem": (
        b"-----BEGIN CERTIFICATE-----\nMIIB\n-----END CERTIFICATE-----\n",
        b"-----BEGIN SSH2 PUBLIC KEY-----\nAAAA\n-----END SSH2 PUBLIC KEY-----\n",
    ),
    "arrow": (b"ARROW1\0\0\xff\xff\xff\xff", b"ARROW2\0\0\xff\xff\xff\xff"),
    "asf": (
        bytes.fromhex("3026B2758E66CF11A6D900AA0062CE6C")
        + (5000).to_bytes(8, "little")
        + _u32(6)
        + b"\x01\x02"
        + bytes(16),
        bytes.fromhex("3026B2758E66CF11A6D900AA0062CE6C")
        + (5000).to_bytes(8, "little")
        + _u32(6)
        + b"\x01\x01"
        + bytes(16),
    ),
    "fbx": (
        b"; FBX 7.3.0 project file\n; Copyright (C) 1997-2010 Autodesk Inc.\n",
        b"; FBX project file\n; Copyright (C) 1997-2010 Autodesk Inc.\n",
    ),
    "luabytecode": (
        b"\x1bLJ\x02\x02\x1c\x00\x01\x03\x01\x00\x01" + bytes(20),
        b"\x1bLJ\x02\x12\x1c\x00\x01\x03\x01\x00\x01" + bytes(20),
    ),
    "gltf": (
        b'{"asset":{"version":"2.0"},"scene":0,"scenes":[{"nodes":[0]}],"nodes":[{"mesh":0}]}',
        b'{"asset":{"version":"1.0"},"geometricError":500,"root":{"refine":"ADD","children":[]}}',
    ),
    "minidump": (
        b"MDMP\x93\xa7\0\0" + _u32(3) + _u32(32) + bytes(20),
        b"MDMP\x93\xa7\0\0" + _u32(0) + _u32(32) + bytes(20),
    ),
    "hve": (
        b"regf"
        + _u32(2)
        + _u32(2)
        + bytes(8)
        + _u32(1)
        + _u32(5)
        + _u32(0)
        + _u32(1)
        + _u32(32)
        + bytes(8),
        b"regf"
        + _u32(2)
        + _u32(2)
        + bytes(8)
        + _u32(1)
        + _u32(99)
        + _u32(0)
        + _u32(1)
        + _u32(32)
        + bytes(8),
    ),
    "intelhex": (
        b":100000000C9444070C94CA340C949F340C947434AB\r\n:00000001FF\r\n",
        b":hello world\r\n:00000001FF\r\n",
    ),
    "grib": (b"GRIB\0\0\0\x02" + bytes(8), b"GRIB\0\0\0\x03" + bytes(8)),
    "safetensors": (
        (40).to_bytes(8, "little") + b'{"w":{"dtype":"F32","shape":[1]}}',
        (40).to_bytes(8, "little") + b'{"w":{"shape":[1],"offsets":[0,4]}}',
    ),
    "pbm": (b"P6\n# made by hand\n32 32\n255\n" + bytes(10), b"P9\n32 32\n255\n" + bytes(10)),
    "ply": (
        b"ply\nformat ascii 1.0\nelement vertex 0\nend_header\n",
        b"ply\nformat ascii 2.0\nelement vertex 0\nend_header\n",
    ),
    "geopackage": (
        b"SQLite format 3\0" + bytes(52) + b"GPKG" + bytes(28),
        b"SQLite format 3\0" + bytes(84),
    ),
    "cubin": (
        b"\x7fELF\x02\x01\x01A" + bytes(8) + _u16(2) + _u16(190),
        b"\x7fELF\x02\x01\x01\0" + bytes(8) + _u16(2) + _u16(62),
    ),
    "jng": (
        b"\x8bJNG\r\n\x1a\n\0\0\0\x10JHDR" + bytes(4),
        b"\x8bJNG\r\n\x1a\n\0\0\0\x0dJHDR" + bytes(4),
    ),
    "palmos": (
        b"Calc".ljust(32, b"\0") + bytes(28) + b"applCALC" + bytes(8) + b"\x00\x03",
        b"Calc".ljust(32, b"\0") + bytes(28) + b"dataCALC" + bytes(8) + b"\x00\x03",
    ),
    "nrrd": (b"NRRD0004\ntype: uint8\n", b"NRRD0009\ntype: uint8\n"),
    "osm": (b"\0\0\0\x0d\x0a\x09OSMHeader\x18\x2f", b"\0\0\0\x0d\x0a\x07OSMData\x18\x2f\0\0\0\0"),
    "vib": (
        b"!<arch>\n"
        + b"descriptor.xml".ljust(16)
        + b"0".ljust(12)
        + b"0".ljust(6)
        + b"0".ljust(6)
        + b"100644".ljust(8)
        + b"64".ljust(10)
        + b"`\n"
        + b'<vib version="5.0"><type>bootbank</type></vib>\n',
        b"!<arch>\n"
        + b"debian-binary".ljust(16)
        + b"0".ljust(12)
        + b"0".ljust(6)
        + b"0".ljust(6)
        + b"100644".ljust(8)
        + b"4".ljust(10)
        + b"`\n"
        + b"2.0\n",
    ),
    "collada": (
        b'<?xml version="1.0"?>\n<COLLADA xmlns="http://www.collada.org/2005/11/COLLADASchema">\n',
        b'<?xml version="1.0"?>\n<svg xmlns="http://www.w3.org/2000/svg"><rect/></svg>\n',
    ),
    "gpx": (
        b'<?xml version="1.0"?>\n<gpx version="1.1" creator="x"><trk/></gpx>\n',
        b'<?xml version="1.0"?>\n<trk><trkseg/></trk>\n',
    ),
    "kml": (
        b'<?xml version="1.0"?>\n<kml xmlns="http://www.opengis.net/kml/2.2"><Document/></kml>\n',
        b'<?xml version="1.0"?>\n<kmlx><Document/></kmlx>\n',
    ),
    "pgp": (
        b"-----BEGIN PGP MESSAGE-----\nVersion: x\n\nhQEMA\n-----END PGP MESSAGE-----\n",
        b"-----BEGIN SSH2 PUBLIC KEY-----\nAAAA\n-----END SSH2 PUBLIC KEY-----\n",
    ),
    "ilbm": (
        b"FORM" + (2000).to_bytes(4, "big") + b"ILBMBMHD" + bytes(20),
        b"FORM" + (2000).to_bytes(4, "big") + b"AIFFCOMM" + bytes(20),
    ),
    "step": (
        b"ISO-10303-21;\nHEADER;\nFILE_DESCRIPTION((''),'2;1');\n",
        b"ISO-10303-28;\n<express xmlns='urn:iso'>\n",
    ),
    "koala": (b"\xff\x80\xc9\xc7" + bytes(60), b"\xff\x80\xc9\xc8" + bytes(60)),
    "degas": (
        (1).to_bytes(2, "big") + bytes(32032),
        (1).to_bytes(2, "big") + bytes(30000),
    ),
}


def test_osm_xml_root_decides_and_osmchange_abstains(scan_rules):
    # taxonomy_osm covers the XML encoding; the "osm" prefix signature above covers the PBF blob.
    assert scan_rules(
        b'<?xml version="1.0"?>\n<osm version="0.6" generator="x"><node/></osm>\n'
    ) == {"osm"}
    assert (
        scan_rules(b'<?xml version="1.0"?>\n<osmChange version="0.6"><create/></osmChange>\n')
        == set()
    )


@pytest.mark.parametrize("label", sorted(PREFIX_SIGNATURES))
def test_prefix_signatures_for_labels_the_model_lacks_or_misses(scan_rules, label):
    positive, near_miss = PREFIX_SIGNATURES[label]
    assert scan_rules(positive) == {label}
    assert scan_rules(near_miss) == set()
