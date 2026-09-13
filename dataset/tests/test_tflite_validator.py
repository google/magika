import struct

from magika_datasets.validators.data import tflite


class Builder:
    """Front-to-back flatbuffer writer: parents are written first and their uoffsets patched later."""

    def __init__(self):
        self.buf = bytearray(8)

    def align(self):
        while len(self.buf) % 4:
            self.buf.append(0)

    def table(self, fields: list) -> tuple[int, list[int]]:
        """fields: 4-byte values, "ref" placeholders or None (absent). Returns (table, field positions)."""
        self.align()
        vtable = struct.pack("<HH", 4 + 2 * len(fields), 4 + 4 * len(fields))
        vtable += b"".join(
            struct.pack("<H", 0 if value is None else 4 + 4 * index)
            for index, value in enumerate(fields)
        )
        vtable_start = len(self.buf)
        self.buf += vtable
        self.align()
        table = len(self.buf)
        self.buf += struct.pack("<i", table - vtable_start)
        positions = []
        for value in fields:
            positions.append(len(self.buf))
            self.buf += value if isinstance(value, bytes) else bytes(4)
        return table, positions

    def patch(self, position: int, target: int):
        struct.pack_into("<I", self.buf, position, target - position)

    def vector(self, count: int, element: bytes = bytes(4)) -> tuple[int, list[int]]:
        self.align()
        start = len(self.buf)
        self.buf += struct.pack("<I", count)
        positions = []
        for _ in range(count):
            positions.append(len(self.buf))
            self.buf += element
        return start, positions

    def string(self, text: bytes) -> int:
        self.align()
        start = len(self.buf)
        self.buf += struct.pack("<I", len(text)) + text + b"\0"
        return start


def build() -> bytes:
    b = Builder()
    model, m = b.table([struct.pack("<I", 3), "ref", "ref", None, "ref"])
    struct.pack_into("<I", b.buf, 0, model)
    b.buf[4:8] = b"TFL3"
    codes, code_slots = b.vector(1)
    b.patch(m[1], codes)
    code, _ = b.table([struct.pack("<I", 0)])
    b.patch(code_slots[0], code)
    subgraphs, subgraph_slots = b.vector(1)
    b.patch(m[2], subgraphs)
    subgraph, s = b.table(["ref", None, None, "ref"])
    b.patch(subgraph_slots[0], subgraph)
    tensors, tensor_slots = b.vector(1)
    b.patch(s[0], tensors)
    tensor, t = b.table(["ref", None, None, "ref"])
    b.patch(tensor_slots[0], tensor)
    shape, _ = b.vector(2, struct.pack("<i", 1))
    b.patch(t[0], shape)
    b.patch(t[3], b.string(b"input"))
    operators, operator_slots = b.vector(1)
    b.patch(s[3], operators)
    operator, o = b.table([struct.pack("<I", 0), "ref", "ref", None])
    b.patch(operator_slots[0], operator)
    inputs, _ = b.vector(1, struct.pack("<i", 0))
    b.patch(o[1], inputs)
    b.patch(o[2], inputs)
    buffers, buffer_slots = b.vector(1)
    b.patch(m[4], buffers)
    buffer, f = b.table(["ref"])
    b.patch(buffer_slots[0], buffer)
    payload, _ = b.vector(4, b"\x01")
    b.patch(f[0], payload)
    return bytes(b.buf)


def test_model_passes():
    result = tflite.validate(build(), frozenset())
    assert result.status == "pass", result
    assert "1 tensors" in result.detail and "1 operators" in result.detail


def test_truncated_model_fails():
    assert tflite.validate(build()[:-6], frozenset()).status == "fail"
