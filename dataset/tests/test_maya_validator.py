import struct

from magika_datasets.validators.geometry import maya

ASCII = b'//Maya ASCII 2022 scene\n//Name: test.ma\nrequires maya "2022";\ncurrentUnit -l centimeter -a degree -t film;\ncreateNode transform -n "pCube1";\n\tsetAttr ".t" -type "double3" 1 2 3 ;\nsetAttr -k on ".semi;colon" "a\\"b";\n'


def test_ascii_scene_passes_and_unterminated_fails():
    result = maya.validate(ASCII, frozenset())
    assert result.status == "pass" and "5 statements" in result.detail
    assert maya.validate(ASCII + b"createNode transform", frozenset()).status == "fail"


def chunk(tag: bytes, payload: bytes) -> bytes:
    body = tag + struct.pack(">I", len(payload)) + payload
    return body + bytes((-len(body)) % 4)


def test_binary_scene_chunks_tile():
    inner = chunk(b"VERS", b"2022") + chunk(b"HEAD", b"abc")
    form = b"FOR4" + struct.pack(">I", 4 + len(inner)) + b"Maya" + inner
    result = maya.validate(form, frozenset())
    assert result.status == "pass" and result.tags == ("binary",)
    assert maya.validate(form[:-2], frozenset()).status == "fail"
