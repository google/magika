import hashlib
import struct

from magika_datasets.validators.archive import rpm


def section(entries: list[tuple[int, int, bytes]], pad: bool) -> bytes:
    index, store = b"", b""
    for tag, kind, value in entries:
        if kind in (6, 8, 9):
            value += b"\0"
        index += struct.pack(">IIII", tag, kind, len(store), 1 if kind != 7 else len(value))
        store += value
    out = (
        b"\x8e\xad\xe8\x01"
        + bytes(4)
        + struct.pack(">II", len(entries), len(store))
        + index
        + store
    )
    return out + bytes((-len(out)) % 8) if pad else out


def package(
    payload: bytes = b"\x1f\x8bgzip-ish", compressor: bytes = b"gzip", size: int | None = None
) -> bytes:
    lead = (
        b"\xed\xab\xee\xdb\x03\x00"
        + struct.pack(">HH", 0, 1)
        + b"pkg".ljust(66, b"\0")
        + struct.pack(">HH", 1, 5)
        + bytes(16)
    )
    main = section([(1000, 6, b"pkg"), (1125, 6, compressor)], pad=False)
    body = main + payload
    signature = section(
        [
            (1000, 4, struct.pack(">I", len(body) if size is None else size)),
            (1004, 7, hashlib.md5(body).digest()),
            (273, 6, hashlib.sha256(main).hexdigest().encode()),
        ],
        pad=True,
    )
    return lead + signature + body


def test_package_digests_verify():
    result = rpm.validate(package(), frozenset())
    assert (
        result.status == "pass"
        and "3 signature checks" in result.detail
        and "payload_gzip" in result.tags
    )


def test_size_and_md5_mismatches_fail():
    assert rpm.validate(package(size=5), frozenset()).status == "fail"
    assert rpm.validate(package() + b"x", frozenset()).status == "fail"


def test_payload_compressor_mismatch_fails():
    assert rpm.validate(package(compressor=b"xz"), frozenset()).status == "fail"
