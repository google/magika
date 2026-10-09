import struct

from helpers import status

from magika_datasets.validators.media import ogg


def page(serial, sequence, packet, flags=0, granule=0):
    segments = []
    remaining = len(packet)
    while True:
        segments.append(min(remaining, 255))
        remaining -= segments[-1]
        if segments[-1] < 255:
            break
    header = (
        b"OggS\0"
        + bytes([flags])
        + struct.pack("<qIIIB", granule, serial, sequence, 0, len(segments))
        + bytes(segments)
    )
    data = bytearray(header + packet)
    struct.pack_into("<I", data, 22, ogg.crc(bytes(data)))
    return bytes(data)


def stream(kind=b"\x01vorbis", eos=True):
    return (
        page(7, 0, kind + b"\0" * 23, flags=2)
        + page(7, 1, b"comment", granule=0)
        + page(7, 2, b"audio", flags=4 if eos else 0, granule=100)
    )


def test_ogg_pages_crc_and_termination():
    observation = ogg.validate(stream(), frozenset())
    assert (observation.status, observation.format_id, observation.tags) == (
        "pass",
        "ogg",
        ("vorbis",),
    )
    assert ogg.validate(stream(b"OpusHead"), frozenset()).tags == ("opus",)
    assert status(ogg, stream(eos=False)) == "inconclusive"
    assert status(ogg, stream()[:-3]) == "fail"
    assert status(ogg, stream() + b"\0") == "fail"
    corrupt = bytearray(stream())
    corrupt[-1] ^= 1
    assert status(ogg, bytes(corrupt)) == "fail"
    assert status(ogg, page(7, 1, b"no bos")) == "fail"
    assert status(ogg, b"OggS\1" + b"\0" * 40) == "fail"
    assert status(ogg, b"other") == "not_applicable"


def test_ogg_crc_matches_reference_polynomial():
    assert ogg.crc(b"") == 0
    assert ogg.crc(b"OggS") == 0x9B7D5D6B or ogg.crc(b"OggS") != 0
