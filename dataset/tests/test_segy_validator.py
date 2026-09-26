import struct

from magika_datasets.validators.data import segy


def headers(samples=4, code=5, fixed=1, prefix=">"):
    text = bytes([segy.EBCDIC_C, 0x40, 0xF1]).ljust(segy.TEXT, b"\x40")
    binary = bytearray(segy.BINARY)
    struct.pack_into(prefix + "H", binary, 16, 1000)  # sample interval
    struct.pack_into(prefix + "H", binary, 20, samples)  # samples per trace
    struct.pack_into(prefix + "H", binary, 24, code)
    struct.pack_into(prefix + "HHh", binary, 300, 0x0100, fixed, 0)
    return text + bytes(binary)


def trace(samples, width=4, own=0, prefix=">"):
    header = bytearray(segy.TRACE)
    struct.pack_into(prefix + "H", header, 114, own)
    return bytes(header) + b"\0" * (samples * width)


def test_fixed_length_traces_tile_the_file():
    result = segy.validate(headers() + trace(4) * 3, frozenset())
    assert result.status == "pass" and result.detail.startswith("3 traces of format code 5")


def test_variable_traces_use_their_own_sample_count():
    data = headers(samples=0, fixed=0) + trace(2, own=2) + trace(7, own=7)
    assert segy.validate(data, frozenset()).status == "pass"


def test_little_endian_headers_are_read():
    data = headers(prefix="<") + trace(4, prefix="<") * 2
    assert segy.validate(data, frozenset()).status == "pass"


def test_a_partial_trace_fails():
    assert segy.validate(headers() + trace(4) * 2 + b"\0" * 9, frozenset()).status == "fail"


def test_an_unknown_format_code_fails():
    assert "format code" in segy.validate(headers(code=4) + trace(4), frozenset()).detail
