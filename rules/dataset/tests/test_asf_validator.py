import struct
import uuid

from helpers import status

from magika_datasets.validators.media import asf


def guid(text):
    return uuid.UUID(text).bytes_le


def obj(kind, payload):
    return guid(kind) + struct.pack("<Q", 24 + len(payload)) + payload


def movie(sub=2, trailing=b""):
    props = obj("8CABDCA1-A947-11CF-8EE4-00C00C205365", b"\0" * 80)
    stream = obj("B7DC0791-A9B7-11CF-8EE6-00C00C205365", b"\0" * 54)
    subs = [props, stream][:sub]
    header = (
        guid(asf.HEADER)
        + struct.pack("<Q", 30 + sum(len(s) for s in subs))
        + struct.pack("<IBB", len(subs), 1, 2)
        + b"".join(subs)
    )
    data = obj(asf.DATA, b"\0" * 26 + b"packet")
    return header + data + trailing


def test_asf_objects():
    observation = asf.validate(movie(), frozenset())
    assert (observation.status, observation.format_id) == ("pass", "asf")
    assert status(asf, movie()[:-2]) == "fail"
    assert status(asf, movie(trailing=b"\0")) == "fail"
    index = obj("D6E229D3-35DA-11D1-9034-00A0C90349BE", b"\0" * 8)
    assert status(asf, movie(trailing=index)) == "pass"
    assert status(asf, movie(sub=0)) == "fail"
    assert asf.validate(movie(sub=1), frozenset()).tags == ("no_stream_properties",)
    assert status(asf, b"\0" * 40) == "not_applicable"
