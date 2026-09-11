import io
import tarfile

from helpers import status

from magika_datasets.validators.archive import tar


def archive(fmt=tarfile.USTAR_FORMAT, names=("a.txt", "dir/b.bin")):
    stream = io.BytesIO()
    with tarfile.open(fileobj=stream, mode="w", format=fmt) as output:
        for index, name in enumerate(names):
            info = tarfile.TarInfo(name)
            payload = bytes([index + 1]) * (512 * (index + 1))
            info.size = len(payload)
            output.addfile(info, io.BytesIO(payload))
    return stream.getvalue()


def test_tar_checksums_blocks_and_termination():
    data = archive()
    assert status(tar, data) == "pass"
    assert status(tar, archive(tarfile.GNU_FORMAT, ("x" * 150,))) == "pass"
    assert status(tar, archive(tarfile.PAX_FORMAT, ("y" * 150,))) == "pass"
    corrupt = bytearray(data)
    corrupt[0] ^= 1
    assert status(tar, bytes(corrupt)) == "fail"
    assert status(tar, data[:-1]) == "fail"
    assert status(tar, data[:900]) == "fail"
    assert status(tar, b"\x00" * 1024) == "not_applicable"
    assert status(tar, b"not a tar" * 100) == "not_applicable"


def test_tar_without_full_terminator_is_inconclusive():
    data = archive()
    body = data[: 512 + 512 + 512 + 1024]  # two headers and their whole-block payloads
    assert status(tar, body) == "inconclusive"
    assert status(tar, body + b"\x00" * 512) == "inconclusive"
    assert status(tar, body + b"\x00" * 1024) == "pass"


def test_v7_header_without_magic_needs_a_hint():
    data = bytearray(archive())
    data[257:265] = b"\x00" * 8  # strip the ustar magic and version
    total = sum(bytes(data[:148]) + b" " * 8 + bytes(data[156:512]))
    data[148:156] = f"{total:06o}\x00 ".encode()
    assert status(tar, bytes(data)) == "not_applicable"
    assert status(tar, bytes(data), frozenset({"tar"})) == "pass"
