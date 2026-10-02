import struct

from magika_datasets.validators.data import filemaker

PAGE = filemaker.PAGE


def page(previous: int, following: int) -> bytes:
    return (b"\0" * 4 + struct.pack(">II", previous, following)).ljust(PAGE, b"\x5a")


def fmp12(order=(2, 4, 3)) -> bytes:
    """Header page, page 1 naming the last page, then a chain through `order`."""
    pages = len(order) + 2
    links = {}
    for index, number in enumerate(order):
        previous = order[index - 1] if index else 0
        following = order[index + 1] if index + 1 < len(order) else 0
        links[number] = page(previous, following)
    header = (filemaker.SIGNATURE + filemaker.MARKER + b"Pro 12.0").ljust(PAGE, b"\0")
    return header + page(0, pages - 1) + b"".join(links[n] for n in range(2, pages))


def test_a_chain_through_every_page_passes():
    result = filemaker.validate(fmp12(), frozenset())
    assert result.status == "pass" and result.detail.startswith("5 pages")


def test_a_broken_back_link_fails():
    data = bytearray(fmp12())
    data[3 * PAGE + 7] = 9  # page 3's previous no longer names page 4
    assert "breaks the chain" in filemaker.validate(bytes(data), frozenset()).detail


def test_an_orphan_chain_fails():
    data = fmp12(order=(2, 3)) + page(0, 0)
    data = data[:PAGE] + page(0, 4) + data[2 * PAGE :]
    assert "2 page chains" in filemaker.validate(data, frozenset()).detail


def test_a_partial_page_fails():
    assert filemaker.validate(fmp12()[:-1], frozenset()).status == "fail"


def test_the_signature_without_the_marker_fails():
    data = fmp12().replace(filemaker.MARKER, b"XXXXX", 1)
    assert filemaker.validate(data, frozenset()).status == "fail"
