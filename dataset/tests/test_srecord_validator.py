from magika_datasets.validators.system import srecord


def record(kind: int, address: bytes, payload: bytes = b"") -> bytes:
    body = bytes([len(address) + len(payload) + 1]) + address + payload
    checksum = 0xFF - (sum(body) & 0xFF)
    return b"S%d" % kind + (body + bytes([checksum])).hex().upper().encode()


def image(*extra: bytes) -> bytes:
    lines = [
        record(0, b"\0\0", b"hdr"),
        record(1, b"\x10\x00", b"\x01\x02\x03"),
        record(1, b"\x10\x03", b"\x04\x05"),
        record(5, b"\x00\x02"),
        record(9, b"\x10\x00"),
        *extra,
    ]
    return b"\r\n".join(lines) + b"\r\n"


def test_records_with_checksums_and_count_pass():
    result = srecord.validate(image(), frozenset())
    assert result.status == "pass" and result.detail.startswith("5 S-records")


def test_a_checksum_mismatch_fails():
    data = image().replace(b"S1061000010203E3", b"S1061000010203E4")
    assert "checksum" in srecord.validate(data, frozenset()).detail


def test_a_wrong_count_record_fails():
    data = image().replace(record(5, b"\x00\x02"), record(5, b"\x00\x07"))
    assert "counts 7" in srecord.validate(data, frozenset()).detail


def test_a_record_after_termination_fails():
    assert srecord.validate(image(record(1, b"\x20\x00", b"x")), frozenset()).status == "fail"


def test_address_width_must_fit_the_type():
    assert srecord.validate(record(3, b"\x00\x01"), frozenset()).status == "fail"
