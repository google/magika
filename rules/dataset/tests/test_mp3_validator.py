import struct

from helpers import status

from magika_datasets.validators.media import mp3


def frame(bitrate_index=9, padding=0):
    header = bytes(
        [0xFF, 0xFB, (bitrate_index << 4) | (0 << 2) | (padding << 1), 0x00]
    )  # MPEG1 L3 44.1k
    length = 144 * mp3.BITRATES[(1, 3)][bitrate_index] * 1000 // 44100 + padding
    return header + b"\0" * (length - 4)


def id3v2(size=100, footer=False):
    flags = 0x10 if footer else 0
    syncsafe = bytes([(size >> 21) & 0x7F, (size >> 14) & 0x7F, (size >> 7) & 0x7F, size & 0x7F])
    data = b"ID3\x04\x00" + bytes([flags]) + syncsafe + b"\0" * size
    if footer:
        data += b"3DI\x04\x00" + bytes([flags]) + syncsafe
    return data


def test_mp3_frames_with_id3_and_ape_tails():
    audio = frame() * 3
    observation = mp3.validate(audio, frozenset())
    assert (observation.status, observation.format_id) == ("pass", "mp3")
    assert status(mp3, id3v2() + audio) == "pass"
    assert status(mp3, id3v2(footer=True) + audio) == "pass"
    assert status(mp3, id3v2() + id3v2(20) + audio) == "pass"  # chained tags
    assert status(mp3, audio + b"TAG" + b"\0" * 125) == "pass"
    items = b"\0" * 8
    ape = items + b"APETAGEX" + struct.pack("<IIII", 2000, len(items) + 32, 1, 0) + b"\0" * 8
    assert status(mp3, audio + ape) == "pass"
    assert status(mp3, audio[:-10]) == "fail"
    assert status(mp3, audio + b"junk") == "fail"
    assert status(mp3, frame(padding=1) + frame()) == "pass"
    assert status(mp3, id3v2() + b"not audio at all" * 5) == "fail"
    assert status(mp3, frame(bitrate_index=0)) == "inconclusive"
    assert status(mp3, b"\xff\xfe" + b"\0" * 100) == "not_applicable"
    assert status(mp3, b"other") == "not_applicable"
