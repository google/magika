import io
import struct
import zlib

from helpers import status
from PIL import Image

from magika_datasets.validators.image import jng


def chunk(kind, body=b""):
    return struct.pack(">I", len(body)) + kind + body + struct.pack(">I", zlib.crc32(kind + body))


def image(jpeg_split=1, header=None):
    output = io.BytesIO()
    Image.new("RGB", (4, 4), "green").save(output, format="JPEG")
    jpeg = output.getvalue()
    header = header or struct.pack(">IIBBBBBBBB", 4, 4, 10, 8, 8, 0, 0, 0, 0, 0)
    parts = [
        jpeg[i : i + len(jpeg) // jpeg_split + 1]
        for i in range(0, len(jpeg), len(jpeg) // jpeg_split + 1)
    ]
    return (
        jng.SIGNATURE
        + chunk(b"JHDR", header)
        + b"".join(chunk(b"JDAT", p) for p in parts)
        + chunk(b"IEND")
    )


def test_jng_structure_and_jpeg_payload():
    assert status(jng, image()) == "pass"
    assert status(jng, image(jpeg_split=3)) == "pass"
    assert status(jng, image()[:-6]) == "fail"
    assert status(jng, image() + b"x") == "fail"
    broken = bytearray(image())
    broken[40] ^= 1
    assert status(jng, bytes(broken)) == "fail"
    no_data = (
        jng.SIGNATURE
        + chunk(b"JHDR", struct.pack(">IIBBBBBBBB", 4, 4, 10, 8, 8, 0, 0, 0, 0, 0))
        + chunk(b"IEND")
    )
    assert status(jng, no_data) == "fail"
    assert (
        status(jng, image(header=struct.pack(">IIBBBBBBBB", 0, 4, 10, 8, 8, 0, 0, 0, 0, 0)))
        == "fail"
    )
    assert status(jng, b"\x89PNG\r\n\x1a\n" + b"rest") == "not_applicable"
