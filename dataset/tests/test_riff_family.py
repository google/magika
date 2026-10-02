import io
import struct
import wave

from helpers import status

from magika_datasets.validators.media import riff


def chunk(kind, body):
    return kind + struct.pack("<I", len(body)) + body + (b"\0" if len(body) % 2 else b"")


def lst(form, body):
    return chunk(b"LIST", form + body)


def container(form, body):
    return chunk(b"RIFF", form + body)


def audio():
    stream = io.BytesIO()
    with wave.open(stream, "wb") as output:
        output.setparams((2, 2, 8000, 0, "NONE", "not compressed"))
        output.writeframes(b"\0" * 16)
    return stream.getvalue()


def test_pcm_and_truncation():
    data = audio()
    assert riff.validate(data, frozenset()).format_id == "wav"
    assert status(riff, data) == "pass"
    assert status(riff, data[:-1]) == "fail"
    assert status(riff, data + b"x") == "fail"
    assert status(riff, b"other format") == "not_applicable"
    assert status(riff, container(b"XXXX", chunk(b"abcd", b"1"))) == "not_applicable"


def test_float_and_extensible_formats():
    data = bytearray(audio())
    struct.pack_into("<H", data, 20, 3)  # IEEE float needs 32 or 64 bits
    assert status(riff, bytes(data)) == "fail"
    struct.pack_into("<HHIIHH", data, 20, 3, 1, 8000, 32000, 4, 32)
    assert status(riff, bytes(data)) == "pass"
    fmt = (
        struct.pack("<HHIIHHHHI", 0xFFFE, 2, 8000, 32000, 4, 16, 22, 16, 3)
        + struct.pack("<H", 1)
        + b"\0" * 14
    )
    body = b"WAVE" + chunk(b"fmt ", fmt) + chunk(b"data", b"\0" * 16)
    extensible = riff.validate(container(b"WAVE", body[4:]), frozenset())
    assert (extensible.status, extensible.tags) == ("pass", ("extensible",))


def test_invalid_alignment_and_unsupported_codec():
    data = bytearray(audio())
    struct.pack_into("<H", data, 32, 1)
    assert status(riff, bytes(data)) == "fail"
    data = bytearray(audio())
    struct.pack_into("<H", data, 20, 0x11)  # IMA ADPCM
    assert status(riff, bytes(data)) == "inconclusive"


def avi(index=True, index_offset=4):
    header = lst(b"hdrl", chunk(b"avih", b"\0" * 56))
    movie = lst(b"movi", chunk(b"00dc", b"frame"))
    idx = chunk(b"idx1", b"00dc" + struct.pack("<III", 16, index_offset, 5)) if index else b""
    return container(b"AVI ", header + movie + idx)


def test_avi_lists_and_index():
    observation = riff.validate(avi(), frozenset())
    assert (observation.status, observation.format_id, observation.tags) == (
        "pass",
        "avi",
        ("has_index",),
    )
    assert riff.validate(avi(index=False), frozenset()).tags == ()
    assert status(riff, avi(index_offset=0xFFFFFF)) == "fail"
    assert status(riff, container(b"AVI ", lst(b"hdrl", chunk(b"avih", b"\0" * 56)))) == "fail"
    assert (
        status(
            riff, container(b"AVI ", lst(b"hdrl", chunk(b"avih", b"\0" * 8)) + lst(b"movi", b""))
        )
        == "fail"
    )


def icon():
    return (
        b"\0\0\1\0" + b"\1\0" + b"\x10\x10\0\0\1\0\x20\0" + struct.pack("<II", 4, 22) + b"\0\0\0\0"
    )


def test_ani_frames_match_header():
    anih = chunk(b"anih", struct.pack("<9I", 36, 2, 2, 0, 0, 32, 1, 10, 1))
    good = container(b"ACON", anih + lst(b"fram", chunk(b"icon", icon()) + chunk(b"icon", icon())))
    observation = riff.validate(good, frozenset())
    assert (observation.status, observation.format_id) == ("pass", "ani")
    short = container(b"ACON", anih + lst(b"fram", chunk(b"icon", icon())))
    assert status(riff, short) == "fail"
    bogus = container(b"ACON", anih + lst(b"fram", chunk(b"icon", b"nope") * 2))
    assert status(riff, bogus) == "fail"


def test_webp_framing_is_explicitly_not_codec_validation():
    body = b"WEBPVP8L" + (1).to_bytes(4, "little") + b"x\0"
    data = b"RIFF" + len(body).to_bytes(4, "little") + body
    assert status(riff, data) == "fail"
    assert status(riff, data[:-1]) == "fail"
    assert status(riff, data[:-1] + b"x") == "fail"
    assert riff.validate(data, frozenset()).format_id == "webp"


def test_odd_final_chunk_without_pad_inside_its_list():
    header = lst(b"hdrl", chunk(b"avih", b"\0" * 56))
    inner = b"00dc" + struct.pack("<I", 5) + b"frame"  # odd length, no pad byte
    movie = (
        b"LIST" + struct.pack("<I", 4 + len(inner)) + b"movi" + inner + b"\0"
    )  # pad belongs to LIST
    data = container(b"AVI ", header + movie)
    assert status(riff, data) == "pass"
