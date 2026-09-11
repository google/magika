import struct

from helpers import status

from magika_datasets.validators.system import dsstore, minidump, pdb, winregistry


def msf7(blocks=4, block_size=512, directory_bytes=100, corrupt=None):
    data = bytearray(blocks * block_size)
    magic = b"Microsoft C/C++ MSF 7.00\r\n\x1aDS\0\0\0"
    data[: len(magic)] = magic
    struct.pack_into("<IIIIII", data, 32, block_size, 1, blocks, directory_bytes, 0, 3)
    struct.pack_into("<I", data, 3 * block_size, 2)  # block map lists directory block 2
    if corrupt == "map":
        struct.pack_into("<I", data, 3 * block_size, 99)
    if corrupt == "size":
        struct.pack_into("<I", data, 40, blocks + 1)
    return bytes(data)


def test_pdb_msf_superblock_and_directory_map():
    observation = pdb.validate(msf7(), frozenset())
    assert (observation.status, observation.format_id) == ("pass", "pdb")
    assert status(pdb, msf7(corrupt="map")) == "fail"
    assert status(pdb, msf7(corrupt="size")) == "fail"
    assert status(pdb, msf7()[:-1]) == "fail"
    old = (
        b"Microsoft C/C++ program database 2.00\r\n\x1aJG\0\0"
        + struct.pack("<IHHII", 1024, 1, 2, 64, 0)
        + b"\0" * (2048 - 60)
    )
    observation = pdb.validate(old, frozenset())
    assert observation.status == "pass" and "msf_2" in observation.tags
    assert status(pdb, b"Microsoft C/C++ MSF 9.00" + b"\0" * 100) == "not_applicable"


def dump(streams=2, bad=False):
    header = b"MDMP" + struct.pack("<IIIIIQ", 0xA793, streams, 32, 0, 0, 0)
    directory = b""
    body_offset = 32 + 12 * streams
    body = b""
    for index in range(streams):
        payload = bytes([index]) * 16
        rva = 0xFFFFFF if bad and index == 1 else body_offset + len(body)
        directory += struct.pack("<III", 3 + index, len(payload), rva)
        body += payload
    return header + directory + body


def test_minidump_streams():
    observation = minidump.validate(dump(), frozenset())
    assert (observation.status, observation.format_id) == ("pass", "minidump")
    assert status(minidump, dump(bad=True)) == "fail"
    assert status(minidump, dump()[:-4]) == "fail"
    assert status(minidump, b"MDMP" + struct.pack("<I", 0x1234) + b"\0" * 40) == "fail"
    assert status(minidump, b"MDMQ" + b"\0" * 40) == "not_applicable"


def hive(bins=2, corrupt=None, dirty=False):
    base = bytearray(4096)
    base[:4] = b"regf"
    struct.pack_into(
        "<IIQIIIIIII", base, 4, 7, 7 if not dirty else 8, 0, 1, 5, 0, 1, 0x20, bins * 4096, 1
    )
    checksum = 0
    for index in range(0, 508, 4):
        checksum ^= struct.unpack_from("<I", base, index)[0]
    struct.pack_into("<I", base, 508, checksum ^ (1 if corrupt == "checksum" else 0))
    body = b""
    for index in range(bins):
        block = bytearray(4096)
        block[:4] = b"hbin"
        struct.pack_into("<II", block, 4, index * 4096, 4096)
        if corrupt == "bin" and index == 1:
            block[:4] = b"xbin"
        body += bytes(block)
    return bytes(base) + body


def test_registry_hive_checksum_and_bins():
    observation = winregistry.validate(hive(), frozenset())
    assert (observation.status, observation.format_id, observation.tags) == ("pass", "hve", ())
    assert "dirty" in winregistry.validate(hive(dirty=True), frozenset()).tags
    assert status(winregistry, hive(corrupt="checksum")) == "fail"
    assert status(winregistry, hive(corrupt="bin")) == "fail"
    assert status(winregistry, hive()[:-100]) == "fail"
    assert status(winregistry, b"regg" + b"\0" * 5000) == "not_applicable"


def store(bad=False):
    root_offset = 2048
    root = struct.pack(">II", 2, 0)  # block count, unknown
    addresses = [(4096 << 0) | 5, ((8192 if not bad else 0xFFFFF0) << 0) | 5]  # offset | log2(size)
    root += struct.pack(">II", *addresses)
    root += struct.pack(">I", 0) * 32  # free lists
    root += struct.pack(">I", 1) + b"\x04DSDB" + struct.pack(">I", 1)  # TOC
    header = (
        struct.pack(">I", 1)
        + b"Bud1"
        + struct.pack(">III", root_offset, len(root), root_offset)
        + b"\0" * 16
    )
    data = bytearray(4 + 8192 + 32)
    data[: len(header)] = header
    data[4 + root_offset : 4 + root_offset + len(root)] = root
    return bytes(data)


def test_ds_store_root_block_and_addresses():
    observation = dsstore.validate(store(), frozenset())
    assert (observation.status, observation.format_id) == ("pass", "dsstore")
    assert status(dsstore, store(bad=True)) == "fail"
    assert status(dsstore, store()[:2000]) == "fail"
    assert status(dsstore, b"\0\0\0\x01Bud2" + b"\0" * 100) == "not_applicable"
