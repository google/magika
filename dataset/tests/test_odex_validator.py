import hashlib
import struct
import zlib

from magika_datasets.validators.executable import odex


def dex_file(stale_signature: bool = False) -> bytes:
    body = bytearray(0x70 + 16)
    body[0:8] = b"dex\n035\0"
    struct.pack_into("<III", body, 32, len(body), 0x70, 0x12345678)
    struct.pack_into("<I", body, 52, 0x70)  # map list with zero items at 0x70
    body[12:32] = hashlib.sha1(bytes(body[32:])).digest()
    if stale_signature:
        body[12] ^= 0xFF
    struct.pack_into("<I", body, 8, zlib.adler32(bytes(body[12:])))
    return bytes(body)


def container(dex: bytes, corrupt: bool = False) -> bytes:
    deps, opt = b"deps" * 3, b"opt!" * 2
    dex_offset = 40
    deps_offset = dex_offset + len(dex)
    opt_offset = deps_offset + len(deps)
    checksum = zlib.adler32(deps + opt) ^ (1 if corrupt else 0)
    header = b"dey\n036\0" + struct.pack(
        "<8I", dex_offset, len(dex), deps_offset, len(deps), opt_offset, len(opt), 0, checksum
    )
    return header + dex + deps + opt


def test_odex_with_stale_dex_signature_passes():
    result = odex.validate(container(dex_file(stale_signature=True)), frozenset())
    assert result.status == "pass"


def test_bad_tail_checksum_and_bad_dex_fail():
    assert odex.validate(container(dex_file(), corrupt=True), frozenset()).status == "fail"
    broken = bytearray(dex_file())
    broken[40] ^= 1  # header field changes both digests
    assert odex.validate(container(bytes(broken)), frozenset()).status == "fail"
