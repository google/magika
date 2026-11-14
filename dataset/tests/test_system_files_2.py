import struct

from helpers import status

from magika_datasets.validators.executable import beam
from magika_datasets.validators.system import applesingle, intelhex, palmos, uf2


def apple(magic=0x00051600, entries=2, bad=False):
    header = struct.pack(">IIQQH", magic, 0x00020000, 0, 0, entries)
    offset = 26 + 12 * entries
    table, body = b"", b""
    for index in range(entries):
        payload = bytes([index + 1]) * 8
        table += struct.pack(
            ">III", index + 1, 0xFFFFFF if bad and index == 1 else offset + len(body), len(payload)
        )
        body += payload
    return header + table + body


def test_applesingle_and_appledouble_entries():
    single = applesingle.validate(apple(), frozenset())
    assert (single.status, single.format_id) == ("pass", "applesingle")
    double = applesingle.validate(apple(0x00051607), frozenset())
    assert double.format_id == "appledouble"
    assert status(applesingle, apple(bad=True)) == "fail"
    assert status(applesingle, apple()[:-2]) == "fail"
    assert status(applesingle, apple(0x00051609)) == "not_applicable"


def palm(records=2, bad=False):
    header = b"MyApp".ljust(32, b"\0") + struct.pack(
        ">HHIIIIIIIIHH", 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
    )
    header = header[:60] + b"DATAappl" + struct.pack(">IIH", 0, 0, records)
    assert len(header) == 78
    offset = 78 + 8 * records + 2
    table, body = b"", b""
    for index in range(records):
        payload = b"r" * 10
        table += struct.pack(
            ">IBBH", 0xFFFFFF if bad and index == 1 else offset + len(body), 0, 0, index
        )
        body += payload
    return header + table + b"\0\0" + body


def prc(resources=2):
    header = b"MyApp".ljust(32, b"\0") + struct.pack(">HH", 0x0001, 0) + b"\0" * 24
    header = header[:60] + b"applMyAp" + struct.pack(">IIH", 0, 0, resources)
    offset = 78 + 10 * resources + 2
    table, body = b"", b""
    for index in range(resources):
        payload = b"r" * 6
        table += struct.pack(">4sHI", b"code", index, offset + len(body))
        body += payload
    return header + table + b"\0\0" + body


def test_palmos_record_list():
    observation = palmos.validate(palm(), frozenset())
    assert (observation.status, observation.format_id) == ("pass", "palmos")
    assert "pdb" in observation.tags
    resources = palmos.validate(prc(), frozenset())
    assert resources.status == "pass" and "prc" in resources.tags
    assert status(palmos, palm(bad=True)) == "fail"
    assert (
        status(palmos, palm()[:80]) == "fail"
    )  # into the record list; the last record runs to EOF
    assert status(palmos, b"\0" * 100) == "not_applicable"


def beam_file(chunks=None, trailing=b""):
    chunks = (
        chunks
        if chunks is not None
        else [(b"AtU8", b"\0\0\0\1\4main"), (b"Code", b"\0" * 20), (b"StrT", b"")]
    )
    body = b"BEAM"
    for kind, payload in chunks:
        body += kind + struct.pack(">I", len(payload)) + payload + b"\0" * (-len(payload) % 4)
    return b"FOR1" + struct.pack(">I", len(body)) + body + trailing


def test_beam_chunks():
    observation = beam.validate(beam_file(), frozenset())
    assert (observation.status, observation.format_id) == ("pass", "beam")
    assert status(beam, beam_file()[:-2]) == "fail"
    assert status(beam, beam_file(trailing=b"\0")) == "fail"
    assert status(beam, beam_file(chunks=[(b"StrT", b"")])) == "fail"
    assert status(beam, b"FOR1\0\0\0\x08BEAX" + b"\0" * 4) == "not_applicable"


def uf2_file(blocks=2, corrupt=None):
    out = b""
    for index in range(blocks):
        payload = bytes([index]) * 256
        block = struct.pack(
            "<IIIIIIII", 0x0A324655, 0x9E5D5157, 0, 0x2000 + 256 * index, 256, index, blocks, 0
        )
        block += (
            payload
            + b"\0" * (476 - 256)
            + struct.pack("<I", 0x0AB16F30 if corrupt != "final" else 0)
        )
        out += block
    if corrupt == "count":
        out = out[:-512]
    return out


def test_uf2_blocks():
    observation = uf2.validate(uf2_file(), frozenset())
    assert (observation.status, observation.format_id) == ("pass", "uf2")
    assert status(uf2, uf2_file(corrupt="final")) == "fail"
    assert status(uf2, uf2_file(corrupt="count")) == "fail"
    assert status(uf2, uf2_file() + b"\0") == "fail"
    assert status(uf2, b"UF2\n" + b"\0" * 100) == "not_applicable"


def hex_line(address, kind, payload):
    body = bytes([len(payload)]) + struct.pack(">H", address) + bytes([kind]) + payload
    checksum = (-sum(body)) & 0xFF
    return b":" + body.hex().upper().encode() + f"{checksum:02X}".encode() + b"\r\n"


def test_intel_hex_records():
    good = hex_line(0x100, 0, b"\x01\x02\x03") + hex_line(0, 4, b"\x00\x01") + hex_line(0, 1, b"")
    observation = intelhex.validate(good, frozenset())
    assert (observation.status, observation.format_id) == ("pass", "intelhex")
    broken = good.replace(b"010203", b"010204", 1)
    assert status(intelhex, broken) == "fail"
    assert status(intelhex, good + hex_line(0, 0, b"\x00")) == "fail"  # data after EOF
    assert status(intelhex, good[:-15]) == "fail"
    assert status(intelhex, b":00000001FF") == "pass"
    assert status(intelhex, good.replace(b"\r\n", b"\r\nxyz\r\n", 1)) == "fail"
    assert status(intelhex, b"not hex") == "not_applicable"
