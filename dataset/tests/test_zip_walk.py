import io
import zipfile

import pytest

from magika_datasets.validators._shared import zip as zipwalk


def build(members, compression=zipfile.ZIP_DEFLATED):
    stream = io.BytesIO()
    with zipfile.ZipFile(stream, "w", compression) as archive:
        for name, payload in members.items():
            archive.writestr(name, payload)
    return stream.getvalue()


def test_walk_reads_members_and_verifies_crc():
    data = build({"a.txt": b"hello", "d/b.bin": b"\x00" * 10})
    package = zipwalk.walk(data)
    assert package.names == ["a.txt", "d/b.bin"]
    assert package.contents["a.txt"] == b"hello"
    stored = build({"a.txt": b"hello"}, zipfile.ZIP_STORED)
    corrupt = stored.replace(b"hello", b"jello")
    with pytest.raises(zipwalk.Malformed):
        zipwalk.walk(corrupt)


def test_walk_rejects_trailing_bytes_unsafe_names_and_budget():
    data = build({"a.txt": b"hello"})
    with pytest.raises(zipwalk.Malformed):
        zipwalk.walk(data + b"trailing")
    with pytest.raises(zipwalk.Malformed):
        zipwalk.walk(data[:-1])
    with pytest.raises(zipwalk.Malformed):
        zipwalk.walk(build({"../escape": b"x"}))
    with pytest.raises(zipwalk.Budget):
        zipwalk.walk(build({"big": b" " * (zipwalk.MEMBER + 1)}))
    with pytest.raises(zipwalk.Malformed):
        zipwalk.walk(b"PK\x03\x04garbage")
    with pytest.raises(zipwalk.Malformed):
        zipwalk.walk(b"not zip")


def test_walk_flags_encryption_as_unsupported():
    data = bytearray(build({"a.txt": b"hello"}, zipfile.ZIP_STORED))
    local_flags = data.index(b"PK\x03\x04") + 6
    data[local_flags] |= 1
    central_flags = data.index(b"PK\x01\x02") + 8
    data[central_flags] |= 1
    with pytest.raises(zipwalk.Unsupported):
        zipwalk.walk(bytes(data))


def test_member_xml_attributes_failure_to_format():
    package = zipwalk.walk(build({"m.xml": b"<broken"}))
    with pytest.raises(zipwalk.Failure) as info:
        zipwalk.member_xml(package, "m.xml", "epub")
    assert info.value.format_id == "epub"
