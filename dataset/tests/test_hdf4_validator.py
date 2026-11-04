import struct

from magika_datasets.validators.data import hdf4


def build(pad: bool = False, overrun: bool = False) -> bytes:
    elements = [(1962, 1, b"abc"), (1965, 1, b"defgh")]
    block_end = 4 + 6 + 12 * len(elements)
    payload = b"".join(body for _, _, body in elements)
    descriptors = b""
    position = block_end
    for tag, ref, body in elements:
        descriptors += struct.pack(">HHII", tag, ref, position, len(body) + (100 if overrun else 0))
        position += len(body)
    data = b"\x0e\x03\x13\x01" + struct.pack(">HI", len(elements), 0) + descriptors + payload
    return data + (b"\0" if pad else b"")


def test_file_passes_and_even_padding_is_tagged():
    assert hdf4.validate(build(), frozenset()).status == "pass"
    assert hdf4.validate(build(pad=True), frozenset()).tags == ("pad_byte",)


def test_overrun_and_unreferenced_bytes_fail():
    assert hdf4.validate(build(overrun=True), frozenset()).status == "fail"
    assert hdf4.validate(build() + b"\0\0", frozenset()).status == "fail"
