import struct

from helpers import status

from magika_datasets.validators.data import icc
from magika_datasets.validators.media import au, midi


def track(events=b"\x00\x90\x3c\x40\x60\x80\x3c\x40", end=True):
    body = events + (b"\x00\xff\x2f\x00" if end else b"")
    return b"MTrk" + struct.pack(">I", len(body)) + body


def song(tracks=2, declared=None):
    declared = tracks if declared is None else declared
    return (
        b"MThd"
        + struct.pack(">IHHH", 6, 1, declared, 96)
        + b"".join(track() for _ in range(tracks))
    )


def test_midi_tracks_and_end_of_track():
    assert status(midi, song()) == "pass"
    assert status(midi, song(declared=3)) == "fail"
    assert status(midi, song()[:-2]) == "fail"
    assert status(midi, song() + b"\0") == "fail"
    assert status(midi, b"MThd" + struct.pack(">IHHH", 6, 0, 1, 96) + track(end=False)) == "fail"
    sysex = (
        b"MThd"
        + struct.pack(">IHHH", 6, 0, 1, 96)
        + track(b"\x00\xf0\x03\x01\x02\xf7\x00\xff\x51\x03\x07\xa1\x20")
    )
    assert status(midi, sysex) == "pass"
    assert status(midi, b"RIFF" + b"\0" * 20) == "not_applicable"


def test_au_header_and_data_size():
    payload = b"\0" * 100
    header = b".snd" + struct.pack(">IIIII", 24, len(payload), 3, 8000, 1)
    assert status(au, header + payload) == "pass"
    assert status(au, header + payload[:-1]) == "fail"
    unknown = b".snd" + struct.pack(">IIIII", 24, 0xFFFFFFFF, 3, 8000, 1) + payload
    assert status(au, unknown) == "pass"
    assert status(au, b".snd" + struct.pack(">IIIII", 24, 100, 99, 8000, 1) + payload) == "fail"
    assert status(au, b".sne" + b"\0" * 40) == "not_applicable"


def profile(tags=2, size=None, bad=False):
    table = struct.pack(">I", tags)
    header_size = 128 + 4 + 12 * tags
    entries, body = b"", b""
    for index in range(tags):
        payload = b"desc" + b"\0" * 12
        offset = header_size + len(body)
        entries += struct.pack(
            ">4sII",
            b"desc" if index == 0 else b"wtpt",
            0xFFFFFF if bad and index == 1 else offset,
            len(payload),
        )
        body += payload
    total = header_size + len(body) if size is None else size
    header = struct.pack(">I", total) + b"\0" * 32 + b"acsp" + b"\0" * 88
    return header + table + entries + body


def test_icc_profile_size_and_tag_table():
    observation = icc.validate(profile(), frozenset())
    assert (observation.status, observation.format_id) == ("pass", "icc")
    assert status(icc, profile()[:-3]) == "fail"
    assert status(icc, profile() + b"\0\0\0\0") == "fail"
    assert status(icc, profile(bad=True)) == "fail"
    assert status(icc, profile(size=1)) == "fail"
    assert status(icc, b"\0\0\0\x80" + b"\0" * 32 + b"acsq" + b"\0" * 100) == "not_applicable"
