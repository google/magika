# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0

import hashlib
import json
import os
import plistlib
import struct
from pathlib import Path

import pyarrow as pa
import pyarrow.parquet as pq
import pytest
from magika_rules_benchmark.corpus import file_hash


@pytest.fixture(scope="module")
def reviewed_binary_headers():
    """Header structures from the format readers, plus independently invalid fields."""
    dex = bytearray(112)
    dex[:8] = b"dex\n035\0"
    struct.pack_into("<III", dex, 32, 112, 112, 0x12345678)
    uf2 = bytearray(512)
    struct.pack_into("<8I", uf2, 0, 0x0A324655, 0x9E5D5157, 0, 0, 256, 0, 1, 0)
    struct.pack_into("<I", uf2, 508, 0x0AB16F30)
    mat = b"MATLAB 5.0 MAT-file".ljust(124, b" ") + b"\x00\x01IM"
    cases = [
        (
            "lz4",
            bytes.fromhex("04224d18604082") + bytes(4),
            8,
            [(4, bytes([flag])) for flag in (0, 0x20, 0x80, 0xC0, 0x62)]
            + [(5, bytes([block])) for block in (0, 0x30, 0x41, 0x80, 0xC0)],
        ),
        (
            "zst",
            bytes.fromhex("28b52ffd2000010000"),
            9,
            [(4, b"\x28"), (4, b"\xff")],
        ),
        ("sketchup", b"\x0eSketchUp Model\x08", 16, [(2, b"?")]),
        ("applebplist", plistlib.dumps({}, fmt=plistlib.FMT_BINARY), 41, [(6, b"??")]),
        ("appledouble", bytes.fromhex("0005160700020000") + bytes(18), 26, [(4, bytes(4))]),
        ("applesingle", bytes.fromhex("0005160000020000") + bytes(18), 26, [(4, bytes(4))]),
        ("uf2", bytes(uf2), 512, [(4, bytes(4)), (508, bytes(4)), (16, struct.pack("<I", 477))]),
        (
            "xcf",
            b"gimp xcf v011\0" + struct.pack(">3I", 1, 1, 0),
            26,
            [(9, b"junk"), (12, b"x"), (13, b"!"), (22, struct.pack(">I", 3))],
        ),
        ("rar", b"Rar!\x1a\x07\x01\0" + bytes(8), 8, [(4, bytes(4)), (5, b"x"), (6, b"\x02")]),
        ("mat", mat, 128, [(124, bytes(2)), (126, b"XX")]),
        (
            "gguf",
            b"GGUF" + struct.pack("<IQQ", 3, 0, 0),
            24,
            [(4, bytes(4)), (4, struct.pack("<I", 0x01000003))],
        ),
        ("wad", struct.pack("<4sII", b"IWAD", 0, 12), 12, []),
        ("cram", b"CRAM\x03\0" + bytes(20), 26, [(4, b"\0"), (5, b"\xff")]),
        ("dex", bytes(dex), 112, [(7, b"!"), (6, b"x"), (36, bytes(4)), (40, bytes(4))]),
        ("redis_rdb", b"REDIS0009\xff" + bytes(8), 9, [(5, b"x009"), (5, b"0000")]),
        ("lz", b"LZIP\x01\xce" + bytes(30), 8, [(4, b"\x02"), (5, b"\0"), (5, b"\xfe")]),
        ("rzip", b"RZIP\x02\x01" + bytes(18), 24, [(4, b"\0"), (14, b"\x01")]),
        (
            "xar",
            struct.pack(">4sHHQQI", b"xar!", 28, 1, 8, 8, 0) + bytes(8),
            28,
            [(8, bytes(8)), (16, bytes(8))],
        ),
        (
            "spirv",
            struct.pack("<5I", 0x07230203, 0x00010000, 0, 1, 0),
            20,
            [(4, bytes(4)), (12, bytes(4)), (16, b"\x01")],
        ),
        (
            "icns",
            struct.pack(">4sI4sI", b"icns", 16, b"TOC ", 8),
            16,
            [(4, bytes(4)), (12, (7).to_bytes(4, "big"))],
        ),
    ]
    result = []
    for label, header, minimum, mutations in cases:
        invalid = []
        for offset, value in mutations:
            changed = bytearray(header)
            changed[offset : offset + len(value)] = value
            invalid.append(bytes(changed))
        result.append((label, header, minimum, invalid))
    # Optional LZ4 fields must be observed through the header checksum.
    lz4_invalid = next(row[3] for row in result if row[0] == "lz4")
    for flag, extra in ((0x61, 4), (0x68, 8), (0x69, 12)):
        header = bytes.fromhex("04224d18") + bytes([flag, 0x40]) + bytes(extra + 1)
        lz4_invalid.extend(header[:length] for length in range(8, len(header)))
    return result


@pytest.fixture(scope="module")
def reviewed_binary_header_variants():
    variants = []
    for flag, extra in ((0x40, 0), (0x61, 4), (0x78, 8), (0x7D, 12)):
        for block in (0x40, 0x50, 0x60, 0x70):
            header = bytes.fromhex("04224d18") + bytes([flag, block]) + bytes(extra + 5)
            variants.append(("lz4", header))
    for magic in ("02214c18", "03214c18"):
        variants.append(("lz4", bytes.fromhex(magic) + bytes(4)))
    for version in range(0x22, 0x28):
        variants.append(("zst", bytes([version]) + bytes.fromhex("b52ffd") + bytes(4)))
    # The Zstandard unused bit is explicitly ignored by conforming decoders.
    variants.append(("zst", bytes.fromhex("28b52ffd3000010000")))
    for endian in ("<", ">"):
        for version, size in ((b"035", 112), (b"041", 120)):
            dex = bytearray(size)
            dex[:8] = b"dex\n" + version + b"\0"
            struct.pack_into(endian + "III", dex, 32, size, size, 0x12345678)
            variants.append(("dex", bytes(dex)))
        variants.append(("spirv", struct.pack(endian + "5I", 0x07230203, 0x00010600, 0, 1, 0)))
    variants.extend(
        [
            ("sketchup", b"\xff\xfe\xff\x0e" + "SketchUp Model".encode("utf-16le")),
            ("applebplist", b"bplist0?" + bytes(34)),
            *[
                ("applebplist", b"bplist" + suffix)
                for suffix in (b"10", b"15", b"16", b"\0\0", b"\0\1", b"@\0")
            ],
            ("appledouble", bytes.fromhex("0005160700010000") + b"Macintosh       " + bytes(2)),
            ("applesingle", bytes.fromhex("0005160000010000") + bytes(18)),
            ("xcf", b"gimp xcf file\0" + struct.pack(">3I", 0, 0, 2)),
            ("rar", b"Rar!\x1a\x07\0" + bytes(9)),
            ("rar", b"RE~^" + bytes(12)),
            ("mat", b"MATLAB 5.0 MAT-file".ljust(124, b" ") + b"\x01\0MI"),
            ("gguf", b"GGUF" + struct.pack("<3I", 1, 0, 0)),
            ("gguf", b"GGUF" + struct.pack("<IQQ", 2, 0, 0)),
            ("gguf", b"GGUF" + struct.pack(">IQQ", 3, 0, 0)),
            ("wad", struct.pack("<4sII", b"PWAD", 0, 12)),
            ("xar", struct.pack(">4sHHQQI", b"xar!", 1, 28, 8, 8, 0) + bytes(8)),
            ("xar", struct.pack(">4sHHQQI", b"xar!", 32, 1, 8, 8, 1) + bytes(12)),
            ("icns", struct.pack(">4sI", b"icns", 8)),
            ("lz", b"LZIP\x00\x0c" + bytes(30)),
        ]
    )
    return variants


def pytest_addoption(parser):
    parser.addoption(
        "--run-native", action="store_true", help="Run actual Magika integration tests"
    )
    parser.addoption(
        "--run-packaging", action="store_true", help="Build staged Rust source packages"
    )


def pytest_collection_modifyitems(config, items):
    for marker, option in (("native", "--run-native"), ("packaging", "--run-packaging")):
        if not config.getoption(option):
            for item in items:
                if marker in item.keywords:
                    item.add_marker(pytest.mark.skip(reason=f"requires explicit {option}"))
    if config.getoption("--run-native"):
        for variable in ("MAGIKA_TEST_BINARY", "MAGIKA_VECTORSCAN_LIBRARY"):
            if not os.environ.get(variable) or not Path(os.environ[variable]).is_file():
                raise pytest.UsageError(f"Native suite requires an existing {variable}")


@pytest.fixture
def corpus_factory(tmp_path):
    def build(contents=(b"PNG sample", b"GIF sample"), labels=("png", "gif"), ambiguous=False):
        root = tmp_path / "dataset"
        (root / "shards").mkdir(parents=True, exist_ok=True)
        classes = [
            dict(
                format_id=name,
                name=name.upper(),
                categories=["image"],
                extensions=[name],
                metadata_json=json.dumps(dict(magika=dict(output_labels=[name], kb_labels=[name]))),
            )
            for name in ("png", "gif", "extra")
        ]
        pq.write_table(pa.Table.from_pylist(classes), root / "classes.parquet")
        rows = []
        for content, label in zip(contents, labels, strict=True):
            rows.append(
                dict(
                    sha256=hashlib.sha256(content).digest(),
                    size=len(content),
                    format_id=label,
                    origins=["generated test fixture"],
                    annotation_json=json.dumps(
                        dict(
                            format_ids=[label],
                            ambiguous=ambiguous,
                            label_status="accepted",
                            properties=dict(
                                artifact_layout=dict(state="known", value="single_file")
                            ),
                        )
                    ),
                    content=content,
                )
            )
        pq.write_table(pa.Table.from_pylist(rows), root / "shards/000.parquet")
        metadata = hashlib.sha256()
        for row in rows:
            encoded = {
                k: v.hex() if isinstance(v, bytes) else v for k, v in row.items() if k != "content"
            }
            metadata.update(
                (json.dumps(encoded, sort_keys=True, separators=(",", ":")) + "\n").encode()
            )
        manifest = dict(
            format="parquet-whole-file-v1",
            samples=len(rows),
            original_bytes=sum(len(c) for c in contents),
            metadata_rows_sha256=metadata.hexdigest(),
            source_metadata_receipt=dict(
                files={"classes.parquet": dict(sha256=file_hash(root / "classes.parquet"))}
            ),
            shards=[
                dict(
                    path="shards/000.parquet",
                    samples=len(rows),
                    sha256=file_hash(root / "shards/000.parquet"),
                )
            ],
        )
        (root / "manifest.json").write_text(json.dumps(manifest))
        return root

    return build


@pytest.fixture
def observations():
    def row(truth, prediction, hit):
        return dict(
            truth=truth,
            ambiguous=False,
            raw_matches=["png_rule"] if hit else [],
            rule_prediction="png" if hit else None,
            ml_prediction=prediction,
            hybrid_prediction="png" if hit else prediction,
            ml_score=0.9,
            hybrid_score=1.0 if hit else 0.9,
            reference_error=None,
            reference_mismatch=False,
            conflict=False,
        )

    return dict(
        samples=[
            row("png", "png", True),
            row("png", "gif", False),
            row("gif", "gif", False),
            row("extra", "extra", True),
        ],
        rules={"png_rule": dict(format_id="png", enforced=True)},
        classes={
            name: dict(categories=["image"], extensions=[name]) for name in ("png", "gif", "extra")
        },
        supported=["png", "gif"],
    )
