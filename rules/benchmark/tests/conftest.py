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
    parquet_sink = pa.BufferOutputStream()
    pq.write_table(pa.table({"value": pa.array([], type=pa.int32())}), parquet_sink)
    ese = bytearray(668)
    struct.pack_into("<III", ese, 4, 0x89ABCDEF, 0x620, 0)
    fits = b"".join(
        card.ljust(80, b" ")
        for card in (
            b"SIMPLE  =                    T",
            b"BITPIX  =                    8",
            b"NAXIS   =                    0",
            b"END",
        )
    ).ljust(2880, b" ")
    shapefile = bytearray(112)
    struct.pack_into(">I", shapefile, 0, 9994)
    struct.pack_into(">I", shapefile, 24, 56)
    struct.pack_into("<II", shapefile, 28, 1000, 0)
    struct.pack_into(">II", shapefile, 100, 1, 2)
    sav = bytearray(176)
    sav[:4] = b"$FL2"
    struct.pack_into("<III", sav, 64, 2, 0, 0)
    vhd = bytearray(512)
    vhd[:8] = b"conectix"
    struct.pack_into(">II", vhd, 8, 2, 0x10000)
    struct.pack_into(">I", vhd, 60, 3)
    flac = bytearray(b"fLaC\x00\x00\x00\x22" + bytes(34))
    struct.pack_into(">HH", flac, 8, 16, 16)
    struct.pack_into(">I", flac, 18, 44100 << 12)
    jp2 = bytes.fromhex("0000000c6a5020200d0a870a") + struct.pack(
        ">I4s4sI", 16, b"ftyp", b"jp2 ", 0
    )
    pdb = b"Microsoft C/C++ MSF 7.00\r\n\x1aDS\0\0\0" + struct.pack("<6I", 4096, 1, 4, 4, 0, 3)
    stata = b"<stata_dta><header><release>118</release><byteorder>LSF</byteorder>"
    cases = [
        (
            "crx",
            b"Cr24" + struct.pack("<II", 3, 1) + b"x",
            12,
            [(4, struct.pack("<I", 1)), (8, bytes(4))],
        ),
        (
            "flac",
            bytes(flac),
            42,
            [(4, b"\x01"), (7, b"\x21"), (8, bytes(2)), (10, bytes(2)), (18, bytes(4))],
        ),
        (
            "hlp",
            struct.pack("<IIII", 0x00035F3F, 16, 0xFFFFFFFF, 16),
            16,
            [(4, bytes(4)), (12, bytes(4))],
        ),
        ("jp2", jp2, 28, [(12, struct.pack(">I", size)) for size in (0, 8, 15, 17)] + [(23, b"x")]),
        ("mscompress", b"SZDD\x88\xf0\x27\x33A\0" + bytes(4), 14, [(7, b"\0"), (8, b"Z")]),
        (
            "netcdf",
            b"CDF\x01" + bytes(28),
            32,
            [(8, struct.pack(">II", tag, 1)) for tag in (0, 9, 11, 12)],
        ),
        ("pdb", pdb, 56, [(24, bytes(8)), (32, struct.pack("<I", 1000)), (36, bytes(4))]),
        (
            "stata",
            stata,
            len(stata),
            [(stata.index(b"118"), b"x18"), (stata.index(b"LSF"), b"BAD")],
        ),
        ("ese", bytes(ese), 668, [(8, bytes(4)), (12, struct.pack("<I", 2))]),
        ("fits", fits, 2880, [(29, b"X"), (80, b"COMMENT "), (109, b"7")]),
        ("llvm_bitcode", b"BC\xc0\xde\x35\x14\x00\x00", 8, [(4, bytes(4))]),
        ("lrz", b"LRZI\x00\x06" + bytes(18), 24, [(4, b"\xff"), (5, b"\x00")]),
        (
            "postgres_dump",
            b"PGDMP\x01\x0e\x00\x04\x08\x01" + bytes(20),
            11,
            [(5, b"\x00"), (8, b"\x00"), (8, b"\x21"), (9, b"\x00"), (10, b"\xff")],
        ),
        ("shapefile", bytes(shapefile), 108, [(24, bytes(4)), (32, struct.pack("<I", 2))]),
        ("spss", bytes(sav), 176, [(64, bytes(4)), (72, struct.pack("<I", 3))]),
        (
            "vhd",
            bytes(vhd),
            512,
            [(8, bytes(4)), (12, bytes(4)), (60, struct.pack(">I", 1)), (84, b"\x02")],
        ),
        (
            "ace",
            struct.pack("<HHBH7sBBBBI8sB", 0, 27, 0, 0, b"**ACE**", 20, 20, 0, 0, 0, bytes(8), 0),
            31,
            [(2, struct.pack("<H", 26)), (4, b"\x01"), (5, b"\x01")],
        ),
        (
            "bpg",
            b"BPG\xfb\x20\x00\x01\x01\x00" + bytes(4),
            9,
            [(4, bytes([flag])) for flag in (0x07, 0x17, 0xC0, 0xFF)]
            + [(5, bytes([flag])) for flag in (0x50, 0xFF)]
            + [(6, b"\x00"), (6, b"\x80"), (7, b"\x00"), (7, b"\x80")]
            + [(4, b"\x00\x10")],
        ),
        (
            "dsstore",
            struct.pack(">I4sIII16s", 1, b"Bud1", 32, 12, 32, bytes(16)),
            36,
            [(8, bytes(4)), (8, struct.pack(">I", 31)), (12, bytes(4))],
        ),
        (
            "duckdb",
            (bytes(8) + b"DUCK" + struct.pack("<Q", 64)).ljust(4096, b"\0"),
            4096,
            [(12, bytes(8)), (16, b"\x01")],
        ),
        (
            "avro",
            b'Obj\x01\x02\x16avro.schema\x0a"int"\x00' + bytes(range(16)),
            21,
            [(4, b"\x00"), (3, b"\x02")],
        ),
        ("parquet", parquet_sink.getvalue().to_pybytes(), 12, [(3, b"0")]),
        (
            "icc",
            bytes(12) + b"mntr" + bytes(20) + b"acsp" + bytes(92),
            132,
            [(12, b"junk"), (36, b"ASCP")],
        ),
        (
            "lnk",
            bytes.fromhex("4c0000000114020000000000c000000000000046") + bytes(56),
            76,
            [(0, b"M"), (4, bytes(16))],
        ),
        ("bam", b"BAM\x01" + bytes(8), 12, [(3, b"\x02")]),
        (
            "hdf4",
            bytes.fromhex("0e031301") + struct.pack(">HIHHII", 1, 0, 1, 0, 0, 0),
            22,
            [(4, struct.pack(">H", count)) for count in (0, 0x8000, 0xFFFF)],
        ),
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
    llvm_invalid = next(row[3] for row in result if row[0] == "llvm_bitcode")
    llvm_invalid.extend(
        [
            bytes.fromhex("dec0170b") + bytes(4),
            struct.pack("<4I", 0x0B17C0DE, 0, 16, 0) + bytes(4),
            struct.pack("<4I", 0x0B17C0DE, 0, 0, 4) + bytes(4),
        ]
    )
    spss_invalid = next(row[3] for row in result if row[0] == "spss")
    spss_invalid.append(bytes.fromhex("c9c3e2c1") + bytes(459))
    crx_invalid = next(row[3] for row in result if row[0] == "crx")
    crx_invalid.extend(
        [
            b"Cr24" + struct.pack("<I", 2) + bytes(4),
            b"Cr24" + struct.pack("<III", 2, 0, 1),
            b"Cr24" + struct.pack("<III", 2, 1, 0),
        ]
    )
    # Optional LZ4 fields must be observed through the header checksum.
    lz4_invalid = next(row[3] for row in result if row[0] == "lz4")
    for flag, extra in ((0x61, 4), (0x68, 8), (0x69, 12)):
        header = bytes.fromhex("04224d18") + bytes([flag, 0x40]) + bytes(extra + 1)
        lz4_invalid.extend(header[:length] for length in range(8, len(header)))
    return result


@pytest.fixture(scope="module")
def reviewed_binary_header_variants():
    variants = []
    variants.append(("crx", b"Cr24" + struct.pack("<III", 2, 65536, 65536) + bytes(131072)))
    # Zero minimum-version and empty compatibility list are accepted by OpenJPEG.
    signature = bytes.fromhex("0000000c6a5020200d0a870a")
    for box_size in (16, 20, 24, 4100):
        variants.append(
            (
                "jp2",
                signature
                + struct.pack(">I4s4sI", box_size, b"ftyp", b"jp2 ", 1)
                + b"jp2 " * ((box_size - 16) // 4),
            )
        )
    for last in (0, 128):
        for rate in (1, 44100, 1048575):
            flac = bytearray(b"fLaC" + bytes([last, 0, 0, 34]) + bytes(34))
            struct.pack_into(">HH", flac, 8, 16, 65535)
            struct.pack_into(">I", flac, 18, rate << 12)
            variants.append(("flac", bytes(flac)))
    # Early Windows betas used mode B; missing filename characters and size can be zero.
    variants.append(("mscompress", b"SZDD\x88\xf0\x27\x33B\0" + bytes(4)))
    for version in (1, 2):
        for tag, count in ((0, 0), (10, 0), (10, 1)):
            variants.append(
                (
                    "netcdf",
                    b"CDF"
                    + bytes([version])
                    + struct.pack(">III", 0xFFFFFFFF, tag, count)
                    + bytes(16),
                )
            )
    for block in (512, 1024, 2048, 4096, 8192, 16384, 32768):
        modern = b"Microsoft C/C++ MSF 7.00\r\n\x1aDS\0\0\0" + struct.pack(
            "<6I", block, 2, 4, 4, 0, 3
        )
        variants.append(("pdb", modern))
    old_magic = b"Microsoft C/C++ program database 2.00\r\n\x1aJG\0\0"
    assert len(old_magic) == 44
    variants.append(("pdb", old_magic + struct.pack("<IHHII", 4096, 1, 4, 4, 0)))
    for version in (117, 118, 119, 120):
        for order in (b"LSF", b"MSF"):
            variants.append(
                (
                    "stata",
                    b"<stata_dta><header><release>"
                    + str(version).encode()
                    + b"</release><byteorder>"
                    + order
                    + b"</byteorder>",
                )
            )
    # The ESE streaming subtype and future revisions do not require a fixed page-size list.
    ese = bytearray(668)
    struct.pack_into("<III", ese, 4, 0x89ABCDEF, 0x620, 1)
    struct.pack_into("<II", ese, 232, 122, 0)
    variants.append(("ese", bytes(ese)))
    # Tolerate the historical third-card BITPIX layout and all defined pixel widths.
    for bits in (b"8", b"16", b"32", b"64", b"-32", b"-64"):
        cards = (
            b"SIMPLE  =                    F",
            b"COMMENT     legacy order",
            b"BITPIX  = " + bits,
            b"NAXIS   =                    0",
            b"END",
        )
        variants.append(("fits", b"".join(c.ljust(80, b" ") for c in cards).ljust(2880, b" ")))
    # LLVM can start with top-level abbreviations or records, and ignores wrapper version/CPU.
    for first in (1, 2, 3, 0x35):
        variants.append(("llvm_bitcode", b"BC\xc0\xde" + bytes([first]) + bytes(7)))
    for offset in (16, 20, 4096):
        variants.append(
            (
                "llvm_bitcode",
                struct.pack("<4I", 0x0B17C0DE, 7, offset, 8)
                + bytes(offset - 16)
                + b"BC\xc0\xde\x35\x14\x00\x00",
            )
        )
    # LRZIP 0.7 uses formerly unused flags, including encryption mode 3.
    for minor in (2, 4, 5, 6, 7):
        variants.append(("lrz", b"LRZI\x00" + bytes([minor]) + bytes(16) + b"\x03\x01"))
    for minor in (0, 1, 6, 7, 14, 15, 17):
        for fmt in (1, 3, 5):
            header = b"PGDMP\x01" + bytes([minor]) + (b"\x00" if minor else b"")
            header += b"\x04" + (b"\x08" if minor >= 7 else b"") + bytes([fmt])
            variants.append(("postgres_dump", header + bytes(20)))
    for endian in ("<", ">"):
        for magic in (b"$FL2", b"$FL3"):
            for layout in (2, 3):
                for compression in (0, 1, 2):
                    sav = bytearray(176)
                    sav[:4] = magic
                    struct.pack_into(endian + "III", sav, 64, layout, 0, compression)
                    variants.append(("spss", bytes(sav)))
    variants.append(("spss", bytes.fromhex("c9c3e2c1") + bytes(460)))
    for kind in (2, 3, 4):
        vhd = bytearray(512)
        vhd[:8] = b"conectix"
        struct.pack_into(">II", vhd, 8, 3, 0x10000)
        struct.pack_into(">I", vhd, 60, kind)
        vhd[84] = 1
        variants.append(("vhd", bytes(vhd)))
    # Do not freeze ACE creator/extractor versions or host identifiers to today's list.
    for version in (10, 11, 12, 13, 20, 22):
        variants.append(
            (
                "ace",
                struct.pack(
                    "<HHBH7sBBBBI8sB",
                    0,
                    27,
                    0,
                    0,
                    b"**ACE**",
                    version,
                    version,
                    11,
                    0,
                    0,
                    bytes(8),
                    0,
                ),
            )
        )
    # These remain header checks when optional data or allocator roots lie beyond 4 KiB.
    variants.append(("ace", struct.pack("<HHBH7s", 0, 5000, 0, 2, b"**ACE**") + bytes(4990)))
    for offset in (32, 2048, 8192):
        variants.append(
            ("dsstore", struct.pack(">I4sIII16s", 1, b"Bud1", offset, 1264, offset, bytes(16)))
        )
    # Keep historical, newer and deprecated-sentinel versions; flags may mark encryption.
    for version in (1, 4, 43, 64, 68, 69, 999):
        variants.append(
            ("duckdb", (bytes(8) + b"DUCK" + struct.pack("<QQ", version, 1)).ljust(4096, b"\0"))
        )
    # Every defined pixel format/depth, alpha combination and short/long dimension encoding.
    for pixel in range(6):
        for depth in range(7):
            variants.append(
                (
                    "bpg",
                    b"BPG\xfb"
                    + bytes([pixel * 32 + 16 + depth, 0x0F if pixel == 0 else 0x4F])
                    + b"\x01\x01\x00"
                    + bytes(8),
                )
            )
    for dimension in (b"\x01", b"\x7f", b"\x81\x00", b"\xff\xff\x7f", b"\x8f\xff\xff\xff\x7f"):
        variants.append(("bpg", b"BPG\xfb\x20\x00" + dimension * 2 + b"\x00" + bytes(4)))
    # Negative map counts carry a byte size; primitive schemas need no record/name keys.
    entry = b'\x16avro.schema\x0a"int"'
    variants.append(("avro", b"Obj\x01\x01" + bytes([len(entry) * 2]) + entry + b"\0" + bytes(16)))
    # Metadata order is arbitrary, and the schema may lie beyond the scan prefix.
    variants.append(
        ("avro", b"Obj\x01\x04\x08note\x80\x40" + b"x" * 4096 + entry + b"\0" + bytes(16))
    )
    # LittleCMS explicitly retains zero device class for profiles written by older versions.
    for device in (
        bytes(4),
        b"scnr",
        b"mntr",
        b"prtr",
        b"link",
        b"abst",
        b"spac",
        b"nmcl",
        b"cenc",
        b"mid ",
        b"mlnk",
        b"mvis",
    ):
        variants.append(("icc", bytes(12) + device + bytes(20) + b"acsp" + bytes(92)))
    # Readers tolerate arbitrary shortcut show commands and reserved header fields.
    for show_command in (0, 1, 3, 7, 0xFFFFFFFF):
        lnk = bytearray(bytes.fromhex("4c0000000114020000000000c000000000000046") + bytes(56))
        struct.pack_into("<I", lnk, 60, show_command)
        lnk[66:76] = b"\xff" * 10
        variants.append(("lnk", bytes(lnk)))
    # Raw BAM can omit its SAM text; text and reference lists may exceed the prefix.
    for text in (b"@HD\tVN:1.6\n", b"@CO\t" + b"x" * 5000 + b"\n"):
        variants.append(("bam", b"BAM\x01" + struct.pack("<I", len(text)) + text + bytes(4)))
    for count in (2, 400, 32767):
        variants.append(
            (
                "hdf4",
                bytes.fromhex("0e031301")
                + struct.pack(">HI", count, 0)
                + struct.pack(">HHII", 1, 0, 0, 0) * count,
            )
        )
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
