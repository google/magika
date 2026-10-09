import struct
import zlib

from helpers import status

from magika_datasets.validators.data import osm_pbf


def varint(value):
    out = b""
    while True:
        byte = value & 0x7F
        value >>= 7
        out += bytes([byte | (0x80 if value else 0)])
        if not value:
            return out


def field(number, payload, wire=2):
    key = varint((number << 3) | wire)
    return key + (varint(len(payload)) + payload if wire == 2 else payload)


def blob(kind, raw):
    zipped = zlib.compress(raw)
    body = field(2, varint(len(raw)), wire=0) + field(3, zipped)
    header = field(1, kind) + field(3, varint(len(body)), wire=0)
    return struct.pack(">I", len(header)) + header + body


def pbf(bad=False, trailing=b""):
    data = blob(b"OSMHeader", b"\x22\x0dOsmSchema-V0.6") + blob(b"OSMData", b"\x0a\x00" * 4)
    if bad:
        data = data[:-3]
    return data + trailing


def test_osm_pbf_blobs():
    observation = osm_pbf.validate(pbf(), frozenset())
    assert (observation.status, observation.format_id) == ("pass", "osm")
    assert status(osm_pbf, pbf(bad=True)) == "fail"
    assert status(osm_pbf, pbf(trailing=b"\0")) == "fail"
    assert status(osm_pbf, struct.pack(">I", 9) + b"\x0a\x03Foo" + b"\0" * 20) == "not_applicable"
    assert (
        status(osm_pbf, b"\0\0\0\x0b\n\tOSMHeader" + b"\0" * 8) == "fail"
    )  # header without datasize
