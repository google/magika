import struct

from helpers import status

from magika_datasets.validators.executable import pe


def image(
    plus=False,
    sections=2,
    overlay=b"",
    signed=False,
    characteristics=0x0102,
    subsystem=2,
    clr=False,
    checksum=None,
    machine=0x14C,
):
    e_lfanew = 0x80
    dos = b"MZ" + b"\0" * (0x3C - 2) + struct.pack("<I", e_lfanew) + b"\0" * (e_lfanew - 0x40)
    optional_size = 0xF0 if plus else 0xE0
    headers_size = e_lfanew + 4 + 20 + optional_size + 40 * sections
    headers_size += -headers_size % 0x200
    raw = []
    offset = headers_size
    for index in range(sections):
        raw.append((offset, 0x200))
        offset += 0x200
    body = b"".join(bytes([index + 1]) * size for index, (_, size) in enumerate(raw))
    security_offset = offset
    signature = b"\x08\0\0\0\0\2\2\0" + b"S" * 8 if signed else b""
    coff = struct.pack("<HHIIIHH", machine, sections, 0, 0, 0, optional_size, characteristics)
    magic = 0x20B if plus else 0x10B
    size_of_image = 0x1000 * (sections + 1)
    directories = [(0, 0)] * 16
    if signed:
        directories[4] = (security_offset, len(signature))
    if clr:
        directories[14] = (0x1000, 72)
    if plus:
        opt = struct.pack(
            "<HBBIIIIIQ", magic, 14, 0, 0x200 * sections, 0, 0, 0x1000, 0x1000, 0x400000
        )
        opt += struct.pack(
            "<IIHHHHHHIIIIHHQQQQII",
            0x1000,
            0x200,
            6,
            0,
            0,
            0,
            6,
            0,
            0,
            size_of_image,
            headers_size,
            checksum or 0,
            subsystem,
            0,
            0,
            0,
            0,
            0,
            0,
            16,
        )
    else:
        opt = struct.pack(
            "<HBBIIIIIII", magic, 14, 0, 0x200 * sections, 0, 0, 0x1000, 0x1000, 0x2000, 0x400000
        )
        opt += struct.pack(
            "<IIHHHHHHIIIIHHIIIIII",
            0x1000,
            0x200,
            6,
            0,
            0,
            0,
            6,
            0,
            0,
            size_of_image,
            headers_size,
            checksum or 0,
            subsystem,
            0,
            0,
            0,
            0,
            0,
            0,
            16,
        )
    opt += b"".join(struct.pack("<II", rva, size) for rva, size in directories)
    assert len(opt) == optional_size
    table = b""
    for index, (pointer, size) in enumerate(raw):
        table += struct.pack(
            "<8sIIIIIIHHI",
            b".sec%d" % index,
            0x1000,
            0x1000 * (index + 1),
            size,
            pointer,
            0,
            0,
            0,
            0,
            0x60000020,
        )
    header = dos + b"PE\0\0" + coff + opt + table
    header += b"\0" * (headers_size - len(header))
    return header + body + signature + overlay


def test_pe32_and_pe32plus_sections_and_tags():
    observation = pe.validate(image(), frozenset())
    assert (observation.status, observation.format_id) == ("pass", "pe")
    assert set(observation.tags) == {"machine_x86", "executable", "pe32"}
    plus = pe.validate(image(plus=True, machine=0x8664), frozenset())
    assert plus.status == "pass" and {"pe32plus", "machine_x64"} <= set(plus.tags)
    dll = pe.validate(image(characteristics=0x2102), frozenset())
    assert "dll" in dll.tags
    driver = pe.validate(image(subsystem=1), frozenset())
    assert "driver" in driver.tags
    dotnet = pe.validate(image(clr=True), frozenset())
    assert "dotnet" in dotnet.tags


def test_pe_overlay_signature_and_bounds():
    plain = pe.validate(image(overlay=b"installer payload"), frozenset())
    assert plain.status == "pass" and "overlay" in plain.tags
    signed = pe.validate(image(signed=True), frozenset())
    assert signed.status == "pass" and "signed" in signed.tags
    assert status(pe, image(signed=True)[:-2]) == "fail"
    after = pe.validate(image(signed=True) + b"payload after signature", frozenset())
    assert after.status == "pass" and {"signed", "overlay"} <= set(after.tags)
    assert status(pe, image()[:-0x100]) == "fail"
    assert status(pe, b"MZ" + b"\0" * 100) == "not_applicable"  # DOS-only program
    assert status(pe, b"ZM" + image()[2:]) == "not_applicable"


def test_pe_checksum_is_verified_when_present():
    data = image()
    computed = pe.checksum(data)
    stamped = pe.validate(image(checksum=computed), frozenset())
    assert stamped.status == "pass" and "checksum_mismatch" not in stamped.tags
    wrong = pe.validate(image(checksum=computed ^ 1), frozenset())
    assert wrong.status == "pass" and "checksum_mismatch" in wrong.tags
