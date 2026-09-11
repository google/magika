# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0

"""Hand-built archives and PE images, ported from the Rust preprocessor tests.

These are the builders of `rust/lib/src/rules/preprocess/{zip,pe}.rs` (`archive`, `stored`,
`pad_directory`, `Image`, `pe32`, `pe32plus`) byte for byte, so the Python and native
tests can assert the same facts and view bytes on the same inputs.
"""

import struct
from dataclasses import dataclass, field

from magika_rules_benchmark.preprocess import PE32_MAGIC, PE32PLUS_MAGIC

EOCD_LEN = 22


def put_u16(buffer, at, value):
    struct.pack_into("<H", buffer, at, value)


def put_u32(buffer, at, value):
    struct.pack_into("<I", buffer, at, value)


def u16(buffer, at):
    return struct.unpack_from("<H", buffer, at)[0]


def u32(buffer, at):
    return struct.unpack_from("<I", buffer, at)[0]


def stored(name, data):
    """A stored (method 0) entry."""
    return (bytes(name), bytes(data), 0)


def archive(entries, comment=b""):
    """A single-disk archive: local entries, the central directory, the EOCD and comment."""
    out = bytearray()
    offsets = []
    for name, data, method in entries:
        offsets.append(len(out))
        out += b"PK\x03\x04" + bytes([20, 0, 0, 0])  # version, flags
        out += struct.pack("<H", method) + bytes(8)  # method; time, date, crc
        out += struct.pack("<IIHH", len(data), len(data), len(name), 0)  # sizes, lengths
        out += name + data
    cd_offset = len(out)
    for (name, data, method), offset in zip(entries, offsets, strict=True):
        out += b"PK\x01\x02" + bytes([20, 0, 20, 0, 0, 0])  # made by, needed, flags
        out += struct.pack("<H", method) + bytes(8)  # method; time, date, crc
        out += struct.pack("<IIHHH", len(data), len(data), len(name), 0, 0)  # sizes, lengths
        out += bytes(8) + struct.pack("<I", offset) + name  # disk, attributes, offset
    cd_size = len(out) - cd_offset
    out += b"PK\x05\x06" + bytes(4)  # disk, central directory disk
    out += struct.pack("<HHIIH", len(entries), len(entries), cd_size, cd_offset, len(comment))
    return bytes(out + comment)


def docx_like(content_types=b"[Content_Types].xml"):
    """An archive listing the members a docx rule tests for, `content_types` renamable."""
    return archive([stored(content_types, b"<Types/>"), stored(b"word/document.xml", b"<w/>")])


# A hundred 60-byte names: 67 fit the 4096-byte view, the 68th does not.
LONG_NAMES = [f"{i:0>60}".encode() for i in range(100)]
LONG_NAMES_HELD = 67


def overflowing_archive():
    """An archive whose names overflow the view: `LONG_NAMES_HELD` of them are written."""
    return archive([stored(name, b"") for name in LONG_NAMES])


def eocd_at(data, comment_len):
    """Offset of the EOCD record of an archive built by `archive` with `comment_len` bytes."""
    return len(data) - EOCD_LEN - comment_len


def pad_directory(data, pad):
    """Grow the single entry's extra field of a comment-less archive by `pad` bytes.

    `data` is a `bytearray`; the EOCD is kept consistent.
    """
    eocd = eocd_at(data, 0)
    cd_size = u32(data, eocd + 12)
    cd = eocd - cd_size
    assert u16(data, eocd + 10) == 1, "single entry"
    assert u16(data, cd + 30) == 0, "no extra field yet"
    put_u16(data, cd + 30, pad)
    data[eocd:eocd] = bytes(pad)
    put_u32(data, eocd + pad + 12, cd_size + pad)


@dataclass
class Image:
    """A hand-built image: DOS header, `PE\\0\\0`, COFF header, optional header, sections.

    Fields default to a consistent PE32 console executable with one section.
    """

    e_lfanew: int = 0x40
    machine: int = 0x14C
    characteristics: int = 0x0102
    magic: int = PE32_MAGIC
    subsystem: int = 3
    dll_characteristics: int = 0x8140
    # The data directories in index order, `(rva, size)`.
    directories: list = field(default_factory=lambda: [(0, 0)] * 16)
    # The declared directory count, `len(directories)` by default.
    directory_count: int | None = None
    # The optional header size, both declared and emitted: the emitted fields are cut or
    # zero-padded to it. The natural size of the fields by default.
    size_of_optional_header: int | None = None
    # The sections, `(pointer_to_raw_data, size_of_raw_data)`.
    sections: list = field(default_factory=lambda: [(0x200, 0x200)])
    # The declared section count, `len(sections)` by default.
    section_count: int | None = None
    # The file length, the end of the furthest section's raw data by default.
    length: int | None = None

    def build(self):
        """The bytes of the image; an `e_lfanew` below 64 overlaps the DOS header."""
        out = bytearray(self.e_lfanew)
        fixed = 112 if self.magic == PE32PLUS_MAGIC else 96
        optional = bytearray(fixed + 8 * len(self.directories))
        put_u16(optional, 0, self.magic)
        put_u16(optional, 68, self.subsystem)
        put_u16(optional, 70, self.dll_characteristics)
        count = len(self.directories) if self.directory_count is None else self.directory_count
        put_u32(optional, fixed - 4, count)
        for index, (rva, size) in enumerate(self.directories):
            put_u32(optional, fixed + 8 * index, rva)
            put_u32(optional, fixed + 8 * index + 4, size)
        # The declared size is the layout: the section table follows it directly.
        soh = (
            len(optional) if self.size_of_optional_header is None else self.size_of_optional_header
        )
        optional = optional[:soh].ljust(soh, b"\0")
        out += b"PE\0\0"
        coff = bytearray(20)
        put_u16(coff, 0, self.machine)
        sections = len(self.sections) if self.section_count is None else self.section_count
        put_u16(coff, 2, sections)
        put_u16(coff, 16, soh)
        put_u16(coff, 18, self.characteristics)
        out += coff + optional
        for index, (pointer, size) in enumerate(self.sections):
            header = bytearray(40)
            put_u32(header, 8, size)
            put_u32(header, 12, 0x1000 * (index + 1))
            put_u32(header, 16, size)
            put_u32(header, 20, pointer)
            out += header
        length = self.length
        if length is None:
            length = max(len(out), max((p + s for p, s in self.sections), default=0))
        target = max(length, 0x40)
        out = out[:target].ljust(target, b"\xcc")
        out[:2] = b"MZ"
        put_u32(out, 0x3C, self.e_lfanew)
        return bytes(out[: max(length, 2)])


def pe32(**overrides):
    return Image(**overrides)


def pe32plus(**overrides):
    return Image(**{"machine": 0x8664, "magic": PE32PLUS_MAGIC, **overrides})


# Fact and view conditions with the payloads they decide, `name: (condition, payloads,
# expected)`: `test_facts_oracle.py` evaluates them through YARA-X given the Python facts,
# and `test_parity.py` checks that the product and that oracle agree on every one of them.
FACTS_CASES = {
    "zip-names-membership": (
        'zip_valid == 1 and zip_names contains "\\n[Content_Types].xml\\n"',
        [
            docx_like(),
            docx_like(b"[Content_Types].xm"),
            docx_like(b"x[Content_Types].xml"),
            docx_like(b"[Content_Types].xml.bak"),
            b"\n[Content_Types].xml\n" + bytes(64),
        ],
        [True, False, False, False, False],
    ),
    "zip-names-startswith": (
        'zip_names startswith "\\nmimetype"',
        [
            archive([stored(b"mimetype", b"application/epub+zip"), stored(b"OEBPS/x", b"")]),
            archive([stored(b"OEBPS/x", b""), stored(b"mimetype", b"application/epub+zip")]),
            archive([stored(b"mimetypes", b"")]),
            archive([]),
        ],
        [True, False, True, False],
    ),
    "pe-machine": (
        "pe_valid == 1 and pe_machine == 0x8664",
        [pe32plus().build(), pe32().build(), b"MZ" + bytes(0x80), pe32plus().build()[:0x50]],
        [True, False, False, False],
    ),
    "zip-names-last-held": (
        f"zip_flags == 16 and zip_names_entries == {LONG_NAMES_HELD} and zip_names contains "
        f'"\\n{LONG_NAMES[LONG_NAMES_HELD - 1].decode()}\\n"',
        [overflowing_archive()],
        [True],
    ),
    "zip-names-first-dropped": (
        f'zip_valid == 1 and zip_names contains "\\n{LONG_NAMES[LONG_NAMES_HELD].decode()}\\n"',
        [overflowing_archive()],
        [False],
    ),
}
