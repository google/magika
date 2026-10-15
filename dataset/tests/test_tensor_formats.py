import io
import json
import struct
import zipfile
import zlib

from helpers import status

from magika_datasets.validators.archive import zip as zipfamily
from magika_datasets.validators.data import avro, gguf, numpy, safetensors


def zigzag(value):
    value = (value << 1) ^ (value >> 63)
    out = b""
    while True:
        byte = value & 0x7F
        value >>= 7
        out += bytes([byte | (0x80 if value else 0)])
        if not value:
            return out


def avro_file(codec=b"null", blocks=2, sync=b"S" * 16, bad_sync=False):
    schema = json.dumps(
        {"type": "record", "name": "R", "fields": [{"name": "x", "type": "int"}]}
    ).encode()
    header = b"Obj\x01" + zigzag(2)
    header += zigzag(len(b"avro.codec")) + b"avro.codec" + zigzag(len(codec)) + codec
    header += zigzag(len(b"avro.schema")) + b"avro.schema" + zigzag(len(schema)) + schema
    header += zigzag(0) + sync
    body = b""
    for index in range(blocks):
        payload = zigzag(index) * 3
        if codec == b"deflate":
            compressor = zlib.compressobj(wbits=-15)
            payload = compressor.compress(payload) + compressor.flush()
        body += (
            zigzag(3) + zigzag(len(payload)) + payload + (b"X" * 16 if bad_sync and index else sync)
        )
    return header + body


def test_avro_container():
    observation = avro.validate(avro_file(), frozenset())
    assert (observation.status, observation.format_id, observation.tags) == (
        "pass",
        "avro",
        ("codec_null",),
    )
    assert avro.validate(avro_file(codec=b"deflate"), frozenset()).status == "pass"
    assert status(avro, avro_file(bad_sync=True)) == "fail"
    assert status(avro, avro_file()[:-3]) == "fail"
    assert status(avro, avro_file() + b"\0") == "fail"
    assert status(avro, b"Obj\x02" + b"\0" * 40) == "not_applicable"
    bomb = avro_file(codec=b"deflate")
    compressor = zlib.compressobj(wbits=-15)
    huge = compressor.compress(b"\0" * (65 * 1024 * 1024)) + compressor.flush()
    header_end = bomb.index(b"S" * 16) + 16
    assert (
        status(avro, bomb[:header_end] + zigzag(1) + zigzag(len(huge)) + huge + b"S" * 16)
        == "inconclusive"
    )


def npy_file(descr="<f8", shape=(2, 3), fortran=False, payload=None, version=1):
    literal = (
        descr if descr.startswith("[") else f"'{descr}'"
    )  # structured dtypes are list literals
    header = f"{{'descr': {literal}, 'fortran_order': {fortran}, 'shape': {shape}, }}"
    prefix = b"\x93NUMPY" + bytes([version, 0])
    length_size = 2 if version == 1 else 4
    padding = 64 - (len(prefix) + length_size + len(header) + 1) % 64
    header = header + " " * padding + "\n"
    data = (
        prefix
        + (struct.pack("<H", len(header)) if version == 1 else struct.pack("<I", len(header)))
        + header.encode("latin-1")
    )
    size = 6
    if payload is None:
        payload = b"\x01" * 8 * size
    return data + payload


def test_npy_headers_and_payload_sizes():
    observation = numpy.validate(npy_file(), frozenset())
    assert (observation.status, observation.format_id) == ("pass", "npy")
    assert status(numpy, npy_file(version=2)) == "pass"
    assert status(numpy, npy_file()[:-1]) == "fail"
    assert status(numpy, npy_file() + b"\0") == "fail"
    assert status(numpy, npy_file(descr="<U3", payload=b"\0" * 72)) == "pass"
    assert (
        status(numpy, npy_file(descr="[('a', '<i4'), ('b', '|u1', (2,))]", payload=b"\0" * 36))
        == "pass"
    )
    assert status(numpy, npy_file(descr="|O")) == "inconclusive"
    assert status(numpy, b"\x93NUMPY\x01\x00" + b"\0\0") == "fail"
    assert status(numpy, b"\x93NUMPZ") == "not_applicable"


def test_npz_archives_are_dispatched_by_the_zip_family():
    stream = io.BytesIO()
    with zipfile.ZipFile(stream, "w") as archive:
        archive.writestr("a.npy", npy_file())
        archive.writestr("b.npy", npy_file(shape=(1,), payload=b"\0" * 8))
    observation = zipfamily.validate(stream.getvalue(), frozenset())
    assert (observation.status, observation.format_id) == ("pass", "npz")
    stream = io.BytesIO()
    with zipfile.ZipFile(stream, "w") as archive:
        archive.writestr("a.npy", npy_file()[:-2])
    assert zipfamily.validate(stream.getvalue(), frozenset()).status == "fail"


def safetensors_file(bad=None):
    header = {
        "__metadata__": {"format": "pt"},
        "w": {"dtype": "F32", "shape": [2, 2], "data_offsets": [0, 16]},
        "b": {"dtype": "F16", "shape": [3], "data_offsets": [16, 22 if bad != "gap" else 24]},
    }
    if bad == "dtype":
        header["b"]["dtype"] = "X99"
    encoded = json.dumps(header).encode()
    encoded += b" " * (-len(encoded) % 8)
    return struct.pack("<Q", len(encoded)) + encoded + b"\0" * 22


def test_safetensors_offsets_and_dtypes():
    observation = safetensors.validate(safetensors_file(), frozenset())
    assert (observation.status, observation.format_id) == ("pass", "safetensors")
    assert status(safetensors, safetensors_file(bad="gap")) == "fail"
    assert status(safetensors, safetensors_file(bad="dtype")) == "fail"
    assert status(safetensors, safetensors_file()[:-1]) == "fail"
    assert status(safetensors, safetensors_file() + b"\0") == "fail"
    assert status(safetensors, struct.pack("<Q", 5) + b"[1,2]" + b"\0" * 4) == "not_applicable"
    assert status(safetensors, b"\0" * 64) == "not_applicable"


def gguf_string(text):
    return struct.pack("<Q", len(text)) + text


def gguf_file(bad=None):
    kv = gguf_string(b"general.architecture") + struct.pack("<I", 8) + gguf_string(b"llama")
    kv += gguf_string(b"general.alignment") + struct.pack("<I", 4) + struct.pack("<I", 32)
    kv += (
        gguf_string(b"tokens")
        + struct.pack("<I", 9)
        + struct.pack("<IQ", 8, 2)
        + gguf_string(b"a")
        + gguf_string(b"b")
    )
    tensors = (
        gguf_string(b"w")
        + struct.pack("<I", 2)
        + struct.pack("<QQ", 4, 2)
        + struct.pack("<I", 0)
        + struct.pack("<Q", 0)
    )  # F32 4x2 = 32 bytes
    tensors += (
        gguf_string(b"q")
        + struct.pack("<I", 1)
        + struct.pack("<Q", 32)
        + struct.pack("<I", 8)
        + struct.pack("<Q", 32 if bad != "offset" else 4096)
    )  # Q8_0 one block = 34 bytes
    header = b"GGUF" + struct.pack("<IQQ", 3, 2, 3) + kv + tensors
    header += b"\0" * (-len(header) % 32)
    data = b"\x01" * 32 + b"\x02" * 34
    data += b"\0" * (-len(data) % 32)
    return header + data


def test_gguf_metadata_and_tensor_ranges():
    observation = gguf.validate(gguf_file(), frozenset())
    assert (observation.status, observation.format_id) == ("pass", "gguf")
    assert "arch_llama" in observation.tags
    assert status(gguf, gguf_file(bad="offset")) == "fail"
    assert status(gguf, gguf_file()[:-40]) == "fail"
    assert status(gguf, b"GGUF" + struct.pack("<I", 9) + b"\0" * 40) == "fail"
    assert status(gguf, b"GGUX" + b"\0" * 40) == "not_applicable"


def test_keras_archives_are_dispatched_by_the_zip_family():
    stream = io.BytesIO()
    with zipfile.ZipFile(stream, "w") as archive:
        archive.writestr("config.json", json.dumps({"class_name": "Sequential"}))
        archive.writestr("metadata.json", json.dumps({"keras_version": "3.0"}))
        archive.writestr("model.weights.h5", b"\x89HDF\r\n\x1a\n" + b"\0" * 16)
    observation = zipfamily.validate(stream.getvalue(), frozenset())
    assert (observation.status, observation.format_id) == ("pass", "keras")
