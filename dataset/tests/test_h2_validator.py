from magika_datasets.validators.data import h2


def build(page_size: int = 2048, pages: int = 4) -> bytes:
    head = h2.HEADER * 3 + page_size.to_bytes(4, "big") + bytes([3, 3])
    return head + bytes(page_size * pages - len(head))


def test_page_store_header_and_lattice():
    assert h2.validate(build(), frozenset()).status == "pass"
    assert h2.validate(build() + b"\0", frozenset()).status == "fail"
    assert h2.validate(build(page_size=1000), frozenset()).status == "fail"
