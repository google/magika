import struct

from magika_datasets.validators.data import tfrecord


def record(payload: bytes) -> bytes:
    length = struct.pack("<Q", len(payload))
    return (
        length
        + struct.pack("<I", tfrecord.masked(length))
        + payload
        + struct.pack("<I", tfrecord.masked(payload))
    )


def test_records_tile_the_file():
    result = tfrecord.validate(record(b"example one") + record(b""), frozenset())
    assert result.status == "pass" and result.detail.startswith("2 records (11 payload")


def test_a_corrupt_payload_fails():
    data = bytearray(record(b"example") + record(b"two"))
    data[14] ^= 1
    assert "data CRC-32C" in tfrecord.validate(bytes(data), frozenset()).detail


def test_truncation_fails():
    assert tfrecord.validate((record(b"abc") * 2)[:-2], frozenset()).status == "fail"


def test_an_unchecked_first_length_is_not_claimed():
    assert tfrecord.validate(b"\x05" + b"\0" * 30, frozenset()) is None
