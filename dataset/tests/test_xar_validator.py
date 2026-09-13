import hashlib
import struct
import zlib

from magika_datasets.validators.archive import xar


def archive(
    toc_extra: str = "",
    heap_extra: bytes = b"",
    corrupt_digest: bool = False,
    payload: bytes = b"hello heap",
    archived: str | None = None,
) -> bytes:
    archived = archived or hashlib.sha1(payload).hexdigest()
    toc = (
        '<?xml version="1.0"?><xar><toc><checksum style="sha1"><offset>0</offset><size>20</size></checksum>'
        f'<file id="1"><name>a</name><data><offset>20</offset><length>{len(payload)}</length><size>{len(payload)}</size>'
        f'<archived-checksum style="sha1">{archived}</archived-checksum></data></file>'
        f"{toc_extra}</toc></xar>"
    ).encode()
    compressed = zlib.compress(toc)
    header = b"xar!" + struct.pack(">HHQQI", 28, 1, len(compressed), len(toc), 1)
    digest = hashlib.sha1(compressed).digest()
    if corrupt_digest:
        digest = bytes(20)
    return header + compressed + digest + payload + heap_extra


def test_archive_passes_with_digest_verified():
    result = xar.validate(archive(), frozenset())
    assert result.status == "pass" and result.tags == ("toc_sha1",)


def test_digest_mismatch_fails():
    assert xar.validate(archive(corrupt_digest=True), frozenset()).status == "fail"


def test_unreferenced_heap_bytes_are_tagged():
    result = xar.validate(archive(heap_extra=b"zz"), frozenset())
    assert result.status == "pass" and "unreferenced_heap_bytes" in result.tags


def test_archived_checksum_mismatch_fails():
    assert xar.validate(archive(archived="0" * 40), frozenset()).status == "fail"


def test_range_outside_heap_fails():
    extra = "<file><data><offset>20</offset><length>500</length></data></file>"
    assert xar.validate(archive(toc_extra=extra), frozenset()).status == "fail"


def test_truncated_toc_fails():
    assert xar.validate(archive()[:40], frozenset()).status == "fail"
