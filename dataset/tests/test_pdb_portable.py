import struct

from helpers import status

from magika_datasets.validators.system import pdb


def portable_pdb(streams=("#Pdb", "#~", "#Strings"), bad=False):
    version = b"PDB v1.0\0\0\0\0"
    root = (
        b"BSJB"
        + struct.pack("<HHII", 1, 1, 0, len(version))
        + version
        + struct.pack("<HH", 0, len(streams))
    )
    headers = b""
    for index, name in enumerate(streams):
        headers += struct.pack("<II", 0, 0) + name.encode() + b"\0"
        headers += b"\0" * (-len(headers) % 4)
    body = b"\x7f" * 32
    data = bytearray(root + headers + body)
    offset = len(root)
    cursor = len(root) + len(headers)
    for index, name in enumerate(streams):
        struct.pack_into("<II", data, offset, 0xFFFFFF if bad and index == 1 else cursor, 8)
        cursor += 8
        offset += 8 + len(name) + 1
        offset += -offset % 4
    return bytes(data)


def test_portable_pdb_metadata_root():
    observation = pdb.validate(portable_pdb(), frozenset())
    assert (observation.status, observation.tags) == ("pass", ("portable_pdb",))
    assert status(pdb, portable_pdb(bad=True)) == "fail"
    assert status(pdb, portable_pdb(streams=("#~", "#Strings"))) == "fail"
    assert status(pdb, portable_pdb()[:-40]) == "fail"
