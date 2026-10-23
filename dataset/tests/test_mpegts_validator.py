from helpers import status

from magika_datasets.validators.media import mpegts


def packet(pid=0x100, adaptation=0, size=188):
    head = b"\x47" + bytes([(pid >> 8) & 0x1F, pid & 0xFF, 0x10 | (0x20 if adaptation else 0)])
    body = (bytes([adaptation]) + b"\0" * min(adaptation, 183)) if adaptation else b""
    payload = b"\xff" * (188 - len(head) - len(body))
    data = head + body + payload
    return (b"\0\0\0\0" + data) if size == 192 else data


def test_transport_stream_lattice():
    data = packet(0) + packet() * 5
    observation = mpegts.validate(data, frozenset())
    assert (observation.status, observation.format_id, observation.tags) == ("pass", "mpegts", ())
    assert status(mpegts, data[:-1]) == "fail"
    assert status(mpegts, data[: 188 * 3] + b"\x00" + data[188 * 3 + 1 :]) == "fail"  # broken sync
    assert status(mpegts, packet(0, adaptation=200) + packet() * 2) == "fail"
    assert status(mpegts, packet() * 3) == "fail"  # no PAT
    m2ts = packet(0, size=192) + packet(size=192) * 3
    observation = mpegts.validate(m2ts, frozenset())
    assert observation.status == "pass" and "m2ts" in observation.tags
    assert status(mpegts, b"\x47" + b"\0" * 100) == "not_applicable"
