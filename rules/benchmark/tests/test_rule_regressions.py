# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0

"""Exercise maintained signatures directly, including adversarial format lookalikes."""

from pathlib import Path

import pytest
import yara_x

ROOT = Path(__file__).resolve().parents[3]
NEGATIVES = sorted(p for p in (ROOT / "tests_data/rules_negative").iterdir() if p.suffix != ".md")


@pytest.fixture(scope="module")
def scan_rules():
    compiler = yara_x.Compiler(includes_enabled=False)
    compiler.define_global("original_size", 0)
    compiler.define_global("prefix_size", 0)
    for path in sorted((ROOT / "rules/rulesets").rglob("*.yar")):
        compiler.add_source(path.read_text())
    scanner = yara_x.Scanner(compiler.build())

    def scan(content):
        prefix = bytes(content[:4096])
        scanner.set_global("original_size", len(content))
        scanner.set_global("prefix_size", len(prefix))
        return {
            dict(rule.metadata)["label"]
            for rule in scanner.scan(prefix).matching_rules
            if dict(rule.metadata).get("enforced", False)
        }

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
]


@pytest.mark.parametrize("label,relative", POSITIVE_FILES)
def test_real_positive_fixtures(scan_rules, label, relative):
    assert scan_rules((ROOT / "tests_data" / relative).read_bytes()) == {label}


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
def test_epub_encrypted_mimetype_is_not_accepted(scan_rules, flags):
    content = bytearray((ROOT / "tests_data/basic/epub/doc.epub").read_bytes())
    content[6:8] = flags.to_bytes(2, "little")
    assert "epub" not in scan_rules(content)


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
