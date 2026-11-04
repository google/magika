from magika_datasets.validators.application import catalog


def der(tag: int, content: bytes) -> bytes:
    if len(content) < 0x80:
        return bytes([tag, len(content)]) + content
    length = len(content).to_bytes(2, "big")
    return bytes([tag, 0x82]) + length + content


def build(content_oid: bytes = catalog.CTL) -> bytes:
    signed = (
        der(0x02, b"\x01") + der(0x31, b"") + der(0x30, content_oid + der(0xA0, der(0x30, b"")))
    )
    return der(0x30, catalog.SIGNED_DATA + der(0xA0, der(0x30, signed)))


def test_catalog_passes():
    assert catalog.validate(build(), frozenset()).status == "pass"


def test_other_signed_data_is_inconclusive_and_truncation_fails():
    other = b"\x06\x09\x2a\x86\x48\x86\xf7\x0d\x01\x07\x01"
    assert catalog.validate(build(content_oid=other), frozenset()).status == "inconclusive"
    assert catalog.validate(build()[:-1], frozenset()).status == "fail"
