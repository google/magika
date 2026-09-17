import struct

from helpers import status

from magika_datasets.validators.executable import macho


def thin(bits=64, cputype=0x0100000C, filetype=2, signed=False, swapped=False):
    order = ">" if swapped else "<"
    magic = 0xFEEDFACF if bits == 64 else 0xFEEDFACE
    header_size = 32 if bits == 64 else 28
    segment_size = 72 if bits == 64 else 56
    commands = 2 if signed else 1
    sizeofcmds = segment_size + (16 if signed else 0)
    payload_offset = header_size + sizeofcmds
    payload = b"\xc3" * 64
    signature = b"\xfa\xde\x0c\xc0" + b"\0" * 28 if signed else b""
    header = struct.pack(order + "IiiIIII", magic, cputype, 0, filetype, commands, sizeofcmds, 0)
    if bits == 64:
        header += struct.pack(order + "I", 0)
        segment = struct.pack(
            order + "II16sQQQQiiII",
            0x19,
            segment_size,
            b"__TEXT",
            0x1000,
            0x1000,
            payload_offset,
            len(payload),
            7,
            5,
            0,
            0,
        )
    else:
        segment = struct.pack(
            order + "II16sIIIIiiII",
            0x1,
            segment_size,
            b"__TEXT",
            0x1000,
            0x1000,
            payload_offset,
            len(payload),
            7,
            5,
            0,
            0,
        )
    sig_cmd = (
        struct.pack(order + "IIII", 0x1D, 16, payload_offset + len(payload), len(signature))
        if signed
        else b""
    )
    return header + segment + sig_cmd + payload + signature


def fat(slices):
    header = struct.pack(">II", 0xCAFEBABE, len(slices))
    offset = 8 + 20 * len(slices)
    offset += -offset % 0x1000
    table, body = b"", b""
    for data in slices:
        table += struct.pack(">iiIII", 0x0100000C, 0, offset + len(body), len(data), 12)
        body += data + b"\0" * (-len(data) % 0x1000)
    return header + table + b"\0" * (offset - 8 - 20 * len(slices)) + body


def test_thin_mach_o_load_commands_and_segments():
    observation = macho.validate(thin(), frozenset())
    assert (observation.status, observation.format_id) == ("pass", "macho")
    assert {"arm64", "executable", "macho64"} <= set(observation.tags)
    signed = macho.validate(thin(signed=True), frozenset())
    assert signed.status == "pass" and "signed" in signed.tags
    dylib = macho.validate(thin(filetype=6, cputype=0x01000007), frozenset())
    assert {"dylib", "x86_64"} <= set(dylib.tags)
    small = macho.validate(thin(bits=32, cputype=7, swapped=True), frozenset())
    assert small.status == "pass" and "macho32" in small.tags
    assert status(macho, thin()[:-10]) == "fail"
    assert status(macho, thin(signed=True)[:-4]) == "fail"
    assert status(macho, b"\xcf\xfa\xed\xfe" + b"\0" * 40) == "fail"  # no load commands
    assert status(macho, b"other") == "not_applicable"


def test_fat_binaries_and_java_class_disambiguation():
    observation = macho.validate(fat([thin(), thin(bits=32, cputype=7)]), frozenset())
    assert observation.status == "pass" and "fat" in observation.tags
    assert status(macho, fat([thin()])[:-0x1000]) == "fail"  # into the slice
    java = b"\xca\xfe\xba\xbe" + struct.pack(">HH", 0, 52) + struct.pack(">H", 10) + b"\0" * 40
    assert status(macho, java) == "not_applicable"
