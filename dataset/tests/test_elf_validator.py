import struct

from helpers import status

from magika_datasets.validators.executable import elf


def binary(
    bits=64,
    kind=2,
    osabi=0,
    sections=(b".text", b".symtab"),
    trailing=b"",
    big=False,
    segment_beyond=False,
):
    order = ">" if big else "<"
    names = b"\0" + b"\0".join(sections) + b"\0.shstrtab\0"
    text = b"\x90" * 64
    symtab = b"\0" * 48
    if bits == 64:
        ehsize, phentsize, shentsize = 64, 56, 64
    else:
        ehsize, phentsize, shentsize = 52, 32, 40
    ph_offset = ehsize
    text_offset = ph_offset + phentsize
    symtab_offset = text_offset + len(text)
    names_offset = symtab_offset + len(symtab)
    sh_offset = names_offset + len(names)
    sh_offset += -sh_offset % 8
    section_count = len(sections) + 2  # null + sections + shstrtab
    ident = b"\x7fELF" + bytes([2 if bits == 64 else 1, 2 if big else 1, 1, osabi]) + b"\0" * 8
    if bits == 64:
        header = ident + struct.pack(
            order + "HHIQQQIHHHHHH",
            kind,
            0x3E,
            1,
            0x1000,
            ph_offset,
            sh_offset,
            0,
            ehsize,
            phentsize,
            1,
            shentsize,
            section_count,
            section_count - 1,
        )
        phdr = struct.pack(
            order + "IIQQQQQQ",
            1,
            5,
            text_offset,
            0x1000,
            0x1000,
            len(text) + (999999 if segment_beyond else 0),
            len(text),
            0x1000,
        )
    else:
        header = ident + struct.pack(
            order + "HHIIIIIHHHHHH",
            kind,
            3,
            1,
            0x1000,
            ph_offset,
            sh_offset,
            0,
            ehsize,
            phentsize,
            1,
            shentsize,
            section_count,
            section_count - 1,
        )
        phdr = struct.pack(
            order + "IIIIIIII",
            1,
            text_offset,
            0x1000,
            0x1000,
            len(text) + (999999 if segment_beyond else 0),
            len(text),
            5,
            0x1000,
        )
    body = header + phdr + text + symtab + names
    body += b"\0" * (sh_offset - len(body))
    entries = [(0, 0, 0, 0)]
    name_index = 1
    offsets = {b".text": (text_offset, len(text), 1), b".symtab": (symtab_offset, len(symtab), 2)}
    for name in sections:
        offset, size, typ = offsets.get(name, (text_offset, 8, 1))
        entries.append((name_index, typ, offset, size))
        name_index += len(name) + 1
    entries.append((name_index, 3, names_offset, len(names)))
    table = b""
    for name_index, typ, offset, size in entries:
        if bits == 64:
            table += struct.pack(
                order + "IIQQQQIIQQ", name_index, typ, 0, 0, offset, size, 0, 0, 1, 0
            )
        else:
            table += struct.pack(
                order + "IIIIIIIIII", name_index, typ, 0, 0, offset, size, 0, 0, 1, 0
            )
    return body + table + trailing


def test_elf_headers_segments_and_sections():
    observation = elf.validate(binary(), frozenset())
    assert (observation.status, observation.format_id) == ("pass", "elf")
    assert {"executable", "elf64"} <= set(observation.tags) and "stripped" not in observation.tags
    small = elf.validate(binary(bits=32, kind=3, big=True), frozenset())
    assert small.status == "pass" and {"shared_object", "big_endian"} <= set(small.tags)
    stripped = elf.validate(binary(sections=(b".text",)), frozenset())
    assert "stripped" in stripped.tags
    trailing = elf.validate(binary(trailing=b"payload"), frozenset())
    assert trailing.status == "pass" and "trailing_bytes" in trailing.tags
    assert status(elf, binary()[:-8]) == "fail"
    assert status(elf, binary(segment_beyond=True)) == "fail"
    assert status(elf, b"\x7fELF\x03" + b"\0" * 60) == "fail"
    assert status(elf, b"\x7fELG" + b"\0" * 60) == "not_applicable"


def test_cuda_binaries_are_cubin():
    observation = elf.validate(binary(osabi=0x33, sections=(b".text", b".nv.info")), frozenset())
    assert (observation.status, observation.format_id) == ("pass", "cubin")
