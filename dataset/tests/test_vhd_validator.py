import struct

from magika_datasets.validators.system import vhd


def seal(record: bytes, at: int) -> bytes:
    total = sum(record[:at]) + sum(record[at + 4 :])
    return record[:at] + struct.pack(">I", ~total & 0xFFFFFFFF) + record[at + 4 :]


def footer(kind: int, current: int, offset: int = 0xFFFFFFFFFFFFFFFF) -> bytes:
    body = vhd.COOKIE + struct.pack(">II", 2, 0x00010000) + struct.pack(">Q", offset)
    body += struct.pack(">I", 0) + b"test" + struct.pack(">I", 0x00010000) + b"Wi2k"
    body += struct.pack(">QQ", current, current) + struct.pack(">I", 0) + struct.pack(">I", kind)
    body = body.ljust(512, b"\0")
    return seal(body, 64)


def fixed(size: int = 4 * 512) -> bytes:
    return b"\x5a" * size + footer(vhd.FIXED, size)


def dynamic(entries=(0xFFFFFFFF, 4), block: int = 4096) -> bytes:
    head = footer(vhd.DYNAMIC, len(entries) * block, offset=512)
    table = 1536
    sparse = vhd.SPARSE + struct.pack(
        ">QQIIII", 0xFFFFFFFFFFFFFFFF, table, 0x00010000, len(entries), block, 0
    )
    sparse = seal(sparse.ljust(1024, b"\0"), 36)
    bat = b"".join(struct.pack(">I", e) for e in entries).ljust(512, b"\xff")
    body = head + sparse + bat + b"\0" * 512 + b"\x11" * block  # sector 4: bitmap, then block
    return body + head


def test_a_fixed_disk_passes():
    result = vhd.validate(fixed(), frozenset())
    assert result.status == "pass" and result.detail.startswith("Fixed disk of 2048 bytes")


def test_a_fixed_disk_with_a_wrong_size_fails():
    assert "size differs" in vhd.validate(b"\0" + fixed(), frozenset()).detail


def test_a_footer_checksum_mismatch_fails():
    data = bytearray(fixed())
    data[-100] ^= 1
    assert "checksum" in vhd.validate(bytes(data), frozenset()).detail


def test_a_dynamic_disk_walks_its_allocation_table():
    result = vhd.validate(dynamic(), frozenset())
    assert result.status == "pass" and "1 of 2 blocks allocated" in result.detail


def test_a_block_past_the_end_fails():
    assert vhd.validate(dynamic(entries=(90,)), frozenset()).status == "fail"


def test_a_truncated_dynamic_disk_fails():
    assert "No conectix footer" in vhd.validate(dynamic()[:-512], frozenset()).detail
