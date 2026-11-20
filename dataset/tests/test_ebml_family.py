from helpers import status

from magika_datasets.validators.media import ebml


def size(value, width=None):
    if width is None:
        width = 1
        while value >= (1 << (7 * width)) - 1:
            width += 1
    marker = 1 << (8 * width - width)
    return (marker | value).to_bytes(width, "big")


def element(identifier, payload=b"", unknown=False):
    length = b"\x01\xff\xff\xff\xff\xff\xff\xff" if unknown else size(len(payload))
    return identifier + length + payload


def header(doctype=b"matroska"):
    return element(
        b"\x1a\x45\xdf\xa3", element(b"\x42\x86", b"\x01") + element(b"\x42\x82", doctype)
    )


def segment(children=None, unknown=False):
    body = b"".join(children) if children is not None else info() + tracks()
    return element(b"\x18\x53\x80\x67", body, unknown=unknown)


def info():
    return element(b"\x15\x49\xa9\x66", element(b"\x2a\xd7\xb1", b"\x0f\x42\x40"))


def tracks():
    return element(b"\x16\x54\xae\x6b", element(b"\xae", element(b"\xd7", b"\x01")))


def cluster(unknown=False):
    return element(
        b"\x1f\x43\xb6\x75",
        element(b"\xe7", b"\x00") + element(b"\xa3", b"\x81\x00\x00\x80data"),
        unknown=unknown,
    )


def test_matroska_and_webm_doctypes():
    observation = ebml.validate(header() + segment(), frozenset())
    assert (observation.status, observation.format_id, observation.tags) == ("pass", "mkv", ())
    assert ebml.validate(header(b"webm") + segment(), frozenset()).format_id == "webm"
    assert status(ebml, header(b"other") + segment()) == "not_applicable"
    assert status(ebml, b"not ebml") == "not_applicable"


def test_unknown_sizes_truncation_and_required_children():
    data = header() + segment([info(), tracks(), cluster(unknown=True)], unknown=True)
    observation = ebml.validate(data, frozenset())
    assert observation.status == "pass" and "unknown_size" in observation.tags
    assert status(ebml, (header() + segment())[:-3]) == "fail"
    assert status(ebml, header() + segment() + b"\0") == "fail"
    assert status(ebml, header() + segment([info()])) == "fail"
    assert (
        status(ebml, header() + segment([tracks(), element(b"\xec", b"", unknown=True)])) == "fail"
    )


def test_junk_after_header_is_judged_by_doctype():
    junk = b"\x88\xee\x25\xe8" * 40
    assert status(ebml, header(b"other") + junk) == "not_applicable"
    assert status(ebml, header() + junk) == "fail"
    assert status(ebml, b"\x1a\x45\xdf\xa3\x80" + junk) == "not_applicable"  # no DocType
