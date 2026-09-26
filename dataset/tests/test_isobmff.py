import io
import struct

from helpers import status
from PIL import Image

from magika_datasets.validators.media import isobmff


def box(kind, payload=b""):
    return struct.pack(">I", 8 + len(payload)) + kind + payload


def ftyp(major, *compatible):
    return box(b"ftyp", major + b"\0\0\0\0" + b"".join(compatible))


def movie(brand=b"isom", *compatible, tail=b"payload"):
    return ftyp(brand, *compatible) + box(b"moov", box(b"mvhd", b"\0" * 100)) + box(b"mdat", tail)


def test_mp4_boxes_and_brands():
    observation = isobmff.validate(movie(), frozenset())
    assert (observation.status, observation.format_id) == ("pass", "mp4")
    assert isobmff.validate(movie(b"3gp4"), frozenset()).format_id == "3gp"
    assert isobmff.validate(movie(b"qt  "), frozenset()).format_id == "qt"
    assert isobmff.validate(movie(b"M4V ", b"isom"), frozenset()).format_id == "mp4"
    unknown = isobmff.validate(movie(b"zzzz"), frozenset())
    assert unknown.status == "inconclusive"
    assert status(isobmff, movie()[:-3]) == "fail"
    assert status(isobmff, movie() + b"x") == "fail"
    assert status(isobmff, ftyp(b"isom") + box(b"mdat", b"no movie")) == "fail"
    assert status(isobmff, b"\0\0\0\x0cjP  \r\n\x87\n" + b"rest") == "not_applicable"
    assert status(isobmff, b"other") == "not_applicable"


def test_open_ended_last_box_and_fragments():
    data = (
        ftyp(b"isom") + box(b"moov", box(b"mvhd")) + struct.pack(">I", 0) + b"mdat" + b"to the end"
    )
    assert status(isobmff, data) == "pass"
    fragmented = (
        ftyp(b"iso5") + box(b"moov", box(b"mvex")) + box(b"moof", box(b"traf")) + box(b"mdat", b"x")
    )
    observation = isobmff.validate(fragmented, frozenset())
    assert observation.status == "pass" and "fragmented" in observation.tags
    large = struct.pack(">I", 1) + b"mdat" + struct.pack(">Q", 16 + 3) + b"abc"
    assert status(isobmff, ftyp(b"isom") + box(b"moov", box(b"mvhd")) + large) == "pass"


def test_heif_and_avif():
    heif = (
        ftyp(b"heic", b"mif1")
        + box(b"meta", b"\0\0\0\0" + box(b"hdlr", b"\0" * 24))
        + box(b"mdat", b"x")
    )
    observation = isobmff.validate(heif, frozenset())
    assert (observation.status, observation.format_id) == ("pass", "heif")
    output = io.BytesIO()
    Image.new("RGB", (8, 8), "blue").save(output, format="AVIF")
    avif = output.getvalue()
    observation = isobmff.validate(avif, frozenset())
    assert (observation.status, observation.format_id) == ("pass", "avif")
    assert status(isobmff, avif[:-20]) == "fail"


def test_classic_quicktime_without_ftyp():
    data = box(b"moov", box(b"mvhd", b"\0" * 100)) + box(b"mdat", b"payload")
    observation = isobmff.validate(data, frozenset())
    assert (observation.status, observation.format_id) == ("pass", "qt")
    assert "no_ftyp" in observation.tags
    assert status(isobmff, box(b"wide") + data) == "pass"
    assert status(isobmff, box(b"mdat", b"only")) == "not_applicable"
    assert status(isobmff, data[:-2]) == "fail"
