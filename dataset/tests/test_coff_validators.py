import struct

from helpers import status

from magika_datasets.validators.executable import coff, xcoff


def coff_object(sections=2, relocs=1, symbols=2, bigobj=False, bad=None):
    header_size = 56 if bigobj else 20
    table = header_size + 40 * sections
    raw, body = [], b""
    for index in range(sections):
        data = bytes([index % 255 + 1]) * 16
        rel = struct.pack("<IIH", 0, 1, 6) * relocs
        raw.append((table + len(body), len(data), table + len(body) + len(data), relocs))
        body += data + rel
    symtab = table + len(body)
    strings = b"\0" * 8
    symbols_blob = b"".join(
        (b"_main\0\0\0" + struct.pack("<IhHBB", 0, 1, 0, 2, 0))
        if not bigobj
        else (b"_main\0\0\0" + struct.pack("<IIHBB", 0, 1, 0, 2, 0))
        for _ in range(symbols)
    )
    strings_blob = struct.pack("<I", 4 + len(strings)) + strings
    if bigobj:
        header = (
            struct.pack("<HHHHI", 0, 0xFFFF, 2, 0x8664, 0)
            + b"\0" * 16
            + struct.pack("<IIII", 0, 0, 0, 0)
            + struct.pack("<III", sections, symtab, symbols)
        )
        assert len(header) == 56
    else:
        header = struct.pack(
            "<HHIIIHH", 0x8664, sections, 0, symtab if bad != "symtab" else 0xFFFFFF, symbols, 0, 0
        )
    table_blob = b""
    for index, (ptr, size, relptr, nreloc) in enumerate(raw):
        table_blob += struct.pack(
            "<8sIIIIIIHHI",
            b".text" if index == 0 else b".data",
            0,
            0,
            size,
            ptr if bad != "section" or index else 0xFFFFFF,
            relptr,
            0,
            nreloc,
            0,
            0x60000020,
        )
    return header + table_blob + body + symbols_blob + strings_blob


def test_coff_objects():
    observation = coff.validate(coff_object(), frozenset({"coff"}))
    assert (observation.status, observation.format_id) == ("pass", "coff")
    big = coff.validate(coff_object(bigobj=True), frozenset({"coff"}))
    assert big.status == "pass" and "bigobj" in big.tags
    assert status(coff, coff_object(bad="section"), frozenset({"coff"})) == "fail"
    assert status(coff, coff_object(bad="symtab"), frozenset({"coff"})) == "fail"
    assert status(coff, coff_object()[:-6], frozenset({"coff"})) == "fail"
    assert status(coff, b"\x99\x99" + coff_object()[2:], frozenset({"coff"})) == "not_applicable"
    assert coff.CONTEXT_REQUIRED and coff.PREFIX_ONLY
    many = coff.validate(coff_object(sections=300), frozenset({"coff"}))
    assert many.status == "pass"
    empty_strings = coff_object()[:-12] + struct.pack("<I", 0)  # zero-sized string table marker
    assert status(coff, empty_strings, frozenset({"coff"})) == "pass"


def xcoff_object(bits=32, sections=1, bad=False):
    header_size = 20 if bits == 32 else 24
    section_size = 40 if bits == 32 else 72
    table = header_size + section_size * sections
    data = b"\x01" * 16
    symtab = table + len(data) * sections
    symbols = b"".join(b"main\0\0\0\0" + struct.pack(">IhHBB", 0, 1, 0, 2, 0) for _ in range(2))
    strings = struct.pack(">I", 4)
    if bits == 32:
        header = struct.pack(
            ">HHIIIHH", 0x01DF, sections, 0, symtab if not bad else 0xFFFFFF, 2, 0, 0
        )
    else:
        header = struct.pack(
            ">HHIQHHI", 0x01F7, sections, 0, symtab if not bad else 0xFFFFFF, 0, 0, 2
        )
    table_blob = b""
    for index in range(sections):
        ptr = table + len(data) * index
        if bits == 32:
            table_blob += struct.pack(
                ">8sIIIIIIHHI", b".text", 0, 0, len(data), ptr, 0, 0, 0, 0, 0x20
            )
        else:
            table_blob += struct.pack(
                ">8sQQQQQQIIIxxxx", b".text", 0, 0, len(data), ptr, 0, 0, 0, 0, 0x20
            )
    return header + table_blob + data * sections + symbols + strings


def test_xcoff_objects():
    observation = xcoff.validate(xcoff_object(), frozenset())
    assert (observation.status, observation.format_id) == ("pass", "xcoff")
    assert "xcoff64" in xcoff.validate(xcoff_object(bits=64), frozenset()).tags
    assert status(xcoff, xcoff_object(bad=True)) == "fail"
    assert status(xcoff, xcoff_object()[:-3]) == "fail"
    assert status(xcoff, b"\x01\xdc" + b"\0" * 40) == "not_applicable"
