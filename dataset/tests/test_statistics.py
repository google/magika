import struct

from helpers import status

from magika_datasets.validators.data import sas, spss, stata


def sav(cases=2, compression=0, bad=False):
    header = (
        b"$FL2"
        + b"@(#) SPSS DATA FILE test".ljust(60)
        + struct.pack("<iiiii", 2, 2, compression, 0, cases)
        + struct.pack("<d", 100.0)
        + b"creation".ljust(81)
        + b"\0" * 3
    )
    assert len(header) == 176
    records = b""
    for index in range(2):
        records += struct.pack("<iiiiii", 2, 0, 0, 0, 5, 0) + f"VAR{index}".ljust(8).encode()
    records += (
        struct.pack("<iii", 7, 3, 4) + struct.pack("<i", 8) + b"\0" * 32
    )  # extension: subtype 3, size 4, count 8
    records += struct.pack("<ii", 999, 0)
    if compression == 0:
        data = struct.pack("<dd", 1.0, 2.0) * cases
    else:
        data = bytes([253, 253, 0, 0, 0, 0, 0, 0]) + struct.pack("<dd", 1.0, 2.0)
        data = data * cases + bytes([252] + [0] * 7)
    if bad:
        data = data[:-3]
    return header + records + data


def test_spss_dictionary_and_cases():
    observation = spss.validate(sav(), frozenset())
    assert (observation.status, observation.format_id) == ("pass", "spss")
    assert status(spss, sav(compression=1)) == "pass"
    assert status(spss, sav(bad=True)) == "fail"
    assert status(spss, sav(compression=1, bad=True)) == "fail"
    assert status(spss, sav() + b"\0") == "fail"
    assert status(spss, b"$FL2" + b"\0" * 100) == "fail"
    assert status(spss, b"$FL9" + b"\0" * 200) == "not_applicable"


def sas7bdat(pages=2, page_size=1024, bad=False, big=False):
    magic = (
        b"\0" * 12
        + b"\xc2\xea\x81\x60\xb3\x14\x11\xcf\xbd\x92\x08\x00\x09\xc7\x31\x8c\x18\x1f\x10\x11"
    )
    header = bytearray(1024)
    header[:32] = magic
    header[32] = 0x22  # 32-bit alignment marker: a1 = 0
    header[35] = 0x22  # a2 = 0
    header[37] = 0 if big else 1
    order = ">" if big else "<"
    struct.pack_into(order + "I", header, 196, 1024)  # header length
    struct.pack_into(order + "I", header, 200, page_size)
    struct.pack_into(order + "I", header, 204, pages if not bad else pages + 1)
    return bytes(header) + b"\0" * page_size * pages


def test_sas_header_and_pages():
    observation = sas.validate(sas7bdat(), frozenset())
    assert (observation.status, observation.format_id) == ("pass", "sas")
    assert status(sas, sas7bdat(big=True)) == "pass"
    assert status(sas, sas7bdat(bad=True)) == "fail"
    assert status(sas, sas7bdat()[:-1]) == "fail"
    assert status(sas, b"\0" * 12 + b"\xc2\xea\x81\x61" + b"\0" * 300) == "not_applicable"


def dta(bad=False, trailing=b""):
    def tag(name, body):
        return f"<{name}>".encode() + body + f"</{name}>".encode()

    header = tag(
        "header",
        tag("release", b"118")
        + tag("byteorder", b"LSF")
        + tag("K", struct.pack("<H", 1))
        + tag("N", struct.pack("<Q", 2))
        + tag("label", struct.pack("<H", 0))
        + tag("timestamp", b"\0"),
    )
    sections = [
        ("map", None),
        ("variable_types", struct.pack("<H", 65526)),
        ("varnames", b"x".ljust(129, b"\0")),
        ("sortlist", b"\0" * 4),
        ("formats", b"%9.0g".ljust(57, b"\0")),
        ("value_label_names", b"\0" * 129),
        ("variable_labels", b"\0" * 321),
        ("characteristics", b""),
        ("data", struct.pack("<ii", 1, 2)),
        ("strls", b""),
        ("value_labels", b""),
    ]
    body = b"<stata_dta>" + header
    offsets = []
    placeholder = tag("map", b"\0" * 112)
    for name, payload in sections:
        offsets.append(len(body))
        body += placeholder if name == "map" else tag(name, payload)
    offsets.append(len(body))
    body += b"</stata_dta>"
    offsets.append(len(body))
    offsets = [0] + offsets
    if bad:
        offsets[-1] += 5
    if bad == "zero":
        offsets[-1] -= 5
        offsets[8] = 0  # unused section left at zero by the writer
    mapping = struct.pack("<14Q", *offsets)
    map_at = body.index(placeholder)
    body = body[:map_at] + tag("map", mapping) + body[map_at + len(placeholder) :]
    return body + trailing


def test_stata_118_sections_and_map():
    observation = stata.validate(dta(), frozenset())
    assert (observation.status, observation.format_id) == ("pass", "stata")
    assert status(stata, dta(bad=True)) == "fail"
    assert status(stata, dta(bad="zero")) == "pass"
    assert status(stata, dta(trailing=b"x")) == "fail"
    assert status(stata, dta()[:-5]) == "fail"
    old = bytes([115, 2, 1, 0]) + struct.pack("<HI", 1, 2) + b"\0" * 100
    assert status(stata, old) == "inconclusive"
    assert status(stata, b"<stata_dtx>") == "not_applicable"
