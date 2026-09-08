from magika_datasets.validators.data import onnx


def varint(value: int) -> bytes:
    out = b""
    while True:
        byte = value & 0x7F
        value >>= 7
        if value:
            out += bytes([byte | 0x80])
        else:
            return out + bytes([byte])


def field(number: int, wire: int, payload: bytes | int) -> bytes:
    key = varint(number << 3 | wire)
    if wire == 0:
        return key + varint(payload)
    return key + varint(len(payload)) + payload


def build(graph_field_wire: int = 2) -> bytes:
    node = field(1, 2, b"x") + field(2, 2, b"y") + field(4, 2, b"Relu")
    tensor = (
        field(1, 2, varint(2) + varint(3))
        + field(2, 0, 1)
        + field(8, 2, b"w")
        + field(9, 2, bytes(24))
    )
    graph = field(1, 2, node) + field(2, 2, b"g") + field(5, 2, tensor)
    return (
        field(1, 0, 8)
        + field(2, 2, b"test")
        + field(7, graph_field_wire, graph if graph_field_wire == 2 else 5)
        + field(8, 2, field(2, 0, 17))
    )


def test_model_passes():
    assert onnx.validate(build(), frozenset()).status == "pass"


def test_wrong_wire_type_and_truncation_fail():
    assert onnx.validate(build(graph_field_wire=0), frozenset()).status == "fail"
    assert onnx.validate(build()[:-3], frozenset()).status == "fail"
