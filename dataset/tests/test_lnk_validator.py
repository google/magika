import struct

from helpers import status

from magika_datasets.validators.application import lnk


def shortcut(*, unicode=True, arguments=True, extra=True, idlist=True, terminator=True, clsid=None):
    flags = 0
    body = b""
    if idlist:
        flags |= 1
        items = struct.pack("<H", 4) + b"ab" + struct.pack("<H", 3) + b"c" + struct.pack("<H", 0)
        body += struct.pack("<H", len(items)) + items
    flags |= 2  # LinkInfo
    info = struct.pack("<IIIIIII", 0x1C + 7, 0x1C, 1, 0x1C, 0x1C + 4, 0, 0x1C + 6) + b"\0\0\0\0C:\0"
    body += info
    strings = [(1 << 3, "..\\target.exe")]
    if arguments:
        strings.append((1 << 5, "--run"))
    for bit, text in strings:
        flags |= bit
        encoded = text.encode("utf-16-le") if unicode else text.encode("latin-1")
        body += struct.pack("<H", len(text)) + encoded
    if unicode:
        flags |= 1 << 7
    if extra:
        env = b"C:\\tool.exe".ljust(260, b"\0") + ("C:\\tool.exe".encode("utf-16-le")).ljust(
            520, b"\0"
        )
        body += struct.pack("<II", 8 + len(env), 0xA0000001) + env
    if terminator:
        body += struct.pack("<I", 0)
    header = (
        struct.pack("<I", 0x4C)
        + (clsid or lnk.CLSID)
        + struct.pack("<II", flags, 0x20)
        + b"\0" * 24
    )
    header += struct.pack("<IiIH", 0, 0, 1, 0) + b"\0" * 10
    assert len(header) == 0x4C
    return header + body


def test_shortcut_structures_and_flags():
    observation = lnk.validate(shortcut(), frozenset())
    assert (observation.status, observation.format_id) == ("pass", "lnk")
    assert set(observation.tags) == {"has_arguments", "has_environment", "unicode"}
    assert (
        lnk.validate(shortcut(unicode=False, arguments=False, extra=False), frozenset()).tags == ()
    )
    assert status(lnk, shortcut(idlist=False)) == "pass"
    assert status(lnk, shortcut()[:-6]) == "fail"
    assert status(lnk, shortcut(terminator=False)) == "fail"
    assert status(lnk, shortcut() + b"\0\0\0\0") == "fail"
    assert status(lnk, shortcut(clsid=b"\0" * 16)) == "not_applicable"
    assert status(lnk, b"L\0\0\0" + b"x" * 100) == "not_applicable"
