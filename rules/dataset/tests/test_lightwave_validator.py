import struct

from magika_datasets.validators.geometry import lightwave

SCENE = b"LWSC\r\n1\r\n\r\nFirstFrame 1\r\nLastFrame 60\r\nFrameStep 1\r\nAddNullObject Null\r\nObjectMotion\r\n{\r\n  Channel 0\r\n  { Envelope\r\n    1\r\n    Key 0 0 0 0 0 0 0 0 0\r\n  }\r\n}\r\nPlugin ItemMotionHandler 1 Foo\r\nMeme-X Lock&Key v1.0\r\n1\r\nEndPlugin\r\n"


def test_scene_grammar():
    result = lightwave.validate(SCENE, frozenset())
    assert result.status == "pass" and result.tags == ("scene",)
    assert lightwave.validate(SCENE + b"{\r\n", frozenset()).status == "fail"
    assert lightwave.validate(SCENE + b"*bad\r\n", frozenset()).status == "fail"


def chunk(tag: bytes, payload: bytes) -> bytes:
    return tag + struct.pack(">I", len(payload)) + payload + (b"\0" if len(payload) & 1 else b"")


def test_object_chunks_tile_the_form():
    inner = chunk(b"TAGS", b"Default\0") + chunk(b"LAYR", bytes(18)) + chunk(b"PNTS", bytes(24))
    form = b"FORM" + struct.pack(">I", 4 + len(inner)) + b"LWO2" + inner
    result = lightwave.validate(form, frozenset())
    assert result.status == "pass" and result.tags == ("object",)
    assert lightwave.validate(form + b"\0\0", frozenset()).status == "fail"
