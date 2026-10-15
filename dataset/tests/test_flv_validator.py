import struct

from helpers import status

from magika_datasets.validators.media import flv


def tag(kind, payload, timestamp=0):
    size = len(payload)
    head = bytes([kind]) + size.to_bytes(3, "big") + (timestamp & 0xFFFFFF).to_bytes(3, "big")
    head += bytes([timestamp >> 24]) + b"\0\0\0"
    return head + payload + struct.pack(">I", 11 + size)


def movie(flags=5, tags=None):
    header = b"FLV\x01" + bytes([flags]) + struct.pack(">I", 9) + struct.pack(">I", 0)
    body = b"".join(
        tags if tags is not None else [tag(18, b"meta"), tag(9, b"vid", 40), tag(8, b"aud", 40)]
    )
    return header + body


def test_flv_tag_chain():
    observation = flv.validate(movie(), frozenset())
    assert (observation.status, observation.format_id) == ("pass", "flv")
    assert set(observation.tags) == {"has_audio", "has_video"}
    assert flv.validate(movie(flags=1), frozenset()).tags == ("has_video",)
    assert status(flv, movie()[:-3]) == "fail"
    assert status(flv, movie() + b"\0") == "fail"
    bad = movie(tags=[tag(9, b"vid")[:-4] + struct.pack(">I", 99)])
    assert status(flv, bad) == "fail"
    assert status(flv, movie(tags=[tag(7, b"x")])) == "fail"
    assert status(flv, b"FLV\x02" + movie()[4:]) == "fail"
    assert status(flv, b"FLA" + b"\0" * 20) == "not_applicable"
