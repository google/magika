import struct

from helpers import status

from magika_datasets.validators.data import dbase, pcap, pcapng, redis_rdb, shapefile, torrent


def shp(records=2, trailing=b""):
    body = b""
    for number in range(1, records + 1):
        content = struct.pack("<I", 1) + struct.pack("<dd", 1.5, 2.5)  # point shape
        body += struct.pack(">II", number, len(content) // 2) + content
    length = (100 + len(body)) // 2
    header = (
        struct.pack(">I", 9994)
        + b"\0" * 20
        + struct.pack(">I", length)
        + struct.pack("<II", 1000, 1)
        + struct.pack("<8d", 0, 0, 0, 0, 0, 0, 0, 0)
    )
    return header + body + trailing


def shx(entries=3, bad=False):
    body, offset = b"", 50
    for number in range(entries):
        words = 10
        body += struct.pack(">II", offset if not (bad and number == 1) else offset + 2, words)
        offset += 4 + words
    length = (100 + len(body)) // 2
    header = (
        struct.pack(">I", 9994)
        + b"\0" * 20
        + struct.pack(">I", length)
        + struct.pack("<II", 1000, 1)
        + struct.pack("<8d", 0, 0, 0, 0, 0, 0, 0, 0)
    )
    return header + body


def test_shapefile_index_files():
    observation = shapefile.validate(shx(), frozenset())
    assert (observation.status, observation.tags) == ("pass", ("index",))
    assert status(shapefile, shx(bad=True)) == "fail"


def test_shapefile_records():
    observation = shapefile.validate(shp(), frozenset())
    assert (observation.status, observation.format_id) == ("pass", "shapefile")
    assert status(shapefile, shp()[:-4]) == "fail"
    assert status(shapefile, shp(trailing=b"\0\0")) == "fail"
    assert status(shapefile, struct.pack(">I", 9995) + b"\0" * 96) == "not_applicable"


def dbf(records=2, trailing=b"", eof=True):
    fields = struct.pack("<11sB", b"NAME", ord("C")) + struct.pack("<IB", 0, 10) + b"\0" * 15
    header_len = 32 + len(fields) + 1
    record_len = 1 + 10
    header = struct.pack("<BBBBIHH", 0x03, 24, 1, 1, records, header_len, record_len) + b"\0" * 20
    body = b"".join(b" " + b"x" * 10 for _ in range(records))
    return header + fields + b"\r" + body + (b"\x1a" if eof else b"") + trailing


def test_dbase_records():
    observation = dbase.validate(dbf(), frozenset())
    assert (observation.status, observation.format_id) == ("pass", "dbase")
    assert status(dbase, dbf(eof=False)) == "pass"
    assert status(dbase, dbf()[:-5]) == "fail"
    assert status(dbase, dbf(trailing=b"\0\0")) == "fail"
    assert status(dbase, b"\x99" + dbf()[1:]) == "not_applicable"
    assert dbase.PREFIX_ONLY


def pcap_file(magic=0xA1B2C3D4, packets=2, bad=False):
    order = "<"
    header = struct.pack(order + "IHHiIII", magic, 2, 4, 0, 0, 65535, 1)
    body = b""
    for index in range(packets):
        payload = bytes([index]) * 20
        body += (
            struct.pack(
                order + "IIII", 0, 0, len(payload) if not bad else len(payload) + 1, len(payload)
            )
            + payload
        )
    return header + body


def test_pcap_packets():
    observation = pcap.validate(pcap_file(), frozenset())
    assert (observation.status, observation.format_id) == ("pass", "pcap")
    assert "nanoseconds" in pcap.validate(pcap_file(magic=0xA1B23C4D), frozenset()).tags
    assert status(pcap, pcap_file(bad=True)) == "fail"
    assert status(pcap, pcap_file()[:-3]) == "fail"
    assert status(pcap, pcap_file() + b"\0") == "fail"
    assert status(pcap, b"\xd4\xc3\xb2\xa1" + b"\0" * 4) == "fail"
    assert status(pcap, b"other") == "not_applicable"


def block(kind, body, bad=False):
    length = 12 + len(body) + (-len(body) % 4)
    return (
        struct.pack("<II", kind, length)
        + body
        + b"\0" * (-len(body) % 4)
        + struct.pack("<I", length if not bad else length + 4)
    )


def pcapng_file(bad=False, trailing=b""):
    shb = block(0x0A0D0D0A, struct.pack("<IHHq", 0x1A2B3C4D, 1, 0, -1))
    idb = block(1, struct.pack("<HHI", 1, 0, 65535))
    epb = block(6, struct.pack("<IIIII", 0, 0, 0, 5, 5) + b"hello", bad=bad)
    return shb + idb + epb + trailing


def test_pcapng_blocks():
    observation = pcapng.validate(pcapng_file(), frozenset())
    assert (observation.status, observation.format_id) == ("pass", "pcapng")
    assert status(pcapng, pcapng_file(bad=True)) == "fail"
    assert status(pcapng, pcapng_file()[:-4]) == "fail"
    assert status(pcapng, pcapng_file(trailing=b"\0\0\0\0")) == "fail"
    assert status(pcapng, b"\x0a\x0d\x0d\x0a" + b"\0" * 20) == "fail"
    assert status(pcapng, b"\x0a\x0d\x0d\x0b") == "not_applicable"


def torrent_file(pieces=2, trailing=b""):
    info = (
        b"d6:lengthi1024e4:name4:file12:piece lengthi512e6:pieces"
        + str(20 * pieces).encode()
        + b":"
        + b"\x11" * 20 * pieces
        + b"e"
    )
    return b"d8:announce21:http://tracker/announ4:info" + info + b"e" + trailing


def test_torrent_bencode():
    observation = torrent.validate(torrent_file(), frozenset())
    assert (observation.status, observation.format_id) == ("pass", "torrent")
    assert status(torrent, torrent_file()[:-3]) == "fail"
    assert status(torrent, torrent_file(trailing=b"e")) == "fail"
    assert status(torrent, b"d4:infod6:piecesi5eee") == "fail"
    assert status(torrent, b"li1ee") == "not_applicable"
    assert status(torrent, b"dictionary text") == "not_applicable"


def rdb(version=b"0011", trailing=b"", checksum=True):
    body = b"REDIS" + version
    body += b"\xfa" + b"\x09redis-ver" + b"\x057.0.0"  # AUX
    body += b"\xfe\x00"  # SELECTDB 0
    body += b"\xfb\x02\x00"  # RESIZEDB
    body += b"\x00" + b"\x03key" + b"\x05value"  # string
    body += (
        b"\xfc" + struct.pack("<Q", 1700000000000) + b"\x00" + b"\x01k" + b"\x01v"
    )  # expire ms + string
    body += (
        b"\xf9\x05" + b"\x0e" + b"\x01l" + b"\x01" + b"\x03abc"
    )  # freq, quicklist with one ziplist blob
    body += (
        b"\x12" + b"\x01q" + b"\x01" + b"\x02" + b"\x02lp"
    )  # quicklist v2: one (container, blob)
    body += b"\xff"
    if checksum:
        body += b"\0" * 8
    return body + trailing


def test_redis_rdb_opcodes():
    observation = redis_rdb.validate(rdb(), frozenset())
    assert (observation.status, observation.format_id) == ("pass", "redis_rdb")
    assert status(redis_rdb, rdb(trailing=b"\0")) == "fail"
    assert status(redis_rdb, rdb()[:-9]) == "fail"
    assert status(redis_rdb, rdb(version=b"0004", checksum=False)) == "pass"
    assert status(redis_rdb, b"REDIS00xx" + b"\0" * 10) == "fail"
    assert status(redis_rdb, b"REDIT") == "not_applicable"
    stream = rdb()[:-9] + b"\x0f\x01s" + b"\x00" + b"\xff" + b"\0" * 8
    assert status(redis_rdb, stream) == "inconclusive"
