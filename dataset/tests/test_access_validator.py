from magika_datasets.validators.data import access


def build(
    engine: bytes = b"Jet DB\0", version: int = 1, pages: int = 4, page_type: int = 1
) -> bytes:
    size = 4096 if version or engine == b"ACE DB\0" else 2048
    head = bytearray(size)
    head[0:13] = b"\x00\x01\x00\x00Standard "
    head[13:20] = engine
    head[20] = version
    body = b""
    for _ in range(pages - 1):
        page = bytearray(size)
        page[0] = page_type
        body += bytes(page)
    return bytes(head) + body


def test_databases_pass():
    assert "Jet 4" in access.validate(build(), frozenset()).detail
    assert "Jet 3" in access.validate(build(version=0), frozenset()).detail
    assert (
        "ACE"
        in access.validate(build(engine=b"ACE DB\0", version=2, page_type=9), frozenset()).detail
    )


def test_bad_page_type_and_partial_page_fail():
    assert access.validate(build(page_type=90), frozenset()).status == "fail"
    assert access.validate(build() + b"\0", frozenset()).status == "fail"
