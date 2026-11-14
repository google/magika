import io
import struct

import pytest
from helpers import status
from PIL import Image

from magika_datasets.validators.image import icns, ico, pbm, psd, qoi, tga


def rendered(fmt, mode="RGB", size=(8, 8), **options):
    output = io.BytesIO()
    Image.new(mode, size, "red").save(output, format=fmt, **options)
    return output.getvalue()


@pytest.mark.parametrize(
    "module,data",
    [
        (ico, rendered("ICO", "RGBA", (32, 32), sizes=[(16, 16), (32, 32)])),
        (pbm, rendered("PPM")),
        (pbm, rendered("PPM", "1")),
        (icns, rendered("ICNS", "RGBA", (16, 16))),
        (qoi, rendered("QOI", "RGBA")),
    ],
)
def test_pillow_images_pass_and_truncations_fail(module, data):
    assert status(module, data) == "pass", module.__name__
    assert status(module, data[: len(data) // 2]) != "pass", module.__name__
    assert status(module, b"other bytes" * 4) == "not_applicable", module.__name__


def test_cursor_and_icon_directory_bounds():
    data = bytearray(rendered("ICO", "RGBA", (16, 16), sizes=[(16, 16)], bitmap_format="bmp"))
    observation = ico.validate(bytes(data), frozenset())
    assert observation.status == "pass" and observation.tags == ()
    data[2] = 2  # CUR
    observation = ico.validate(bytes(data), frozenset())
    assert observation.status == "pass" and observation.tags == ("cursor",)
    struct.pack_into("<I", data, 6 + 12, 0xFFFFFF)  # first entry offset outside the file
    assert status(ico, bytes(data)) == "fail"


def test_tga_needs_a_hint_and_checks_header():
    data = rendered("TGA", "RGB", (4, 4))
    assert tga.REQUIRES_HINT
    assert status(tga, data, frozenset({"tga"})) == "pass"
    assert status(tga, data[:30], frozenset({"tga"})) == "fail"
    assert status(tga, b"\0\0\7" + data[3:], frozenset({"tga"})) == "fail"
    footer = data + b"\0" * 8 + b"TRUEVISION-XFILE.\0"
    assert "tga_footer" in tga.validate(footer, frozenset({"tga"})).tags


def test_psd_versions():
    header = b"8BPS\0\1" + b"\0" * 6 + struct.pack(">HIIHH", 1, 4, 4, 8, 1)
    body = struct.pack(">I", 0) * 3 + struct.pack(">H", 0) + b"\x7f" * 16
    assert status(psd, header + body) == "pass"
    assert status(psd, header + body[:-8]) == "fail"
    assert status(psd, header.replace(b"\0\1", b"\0\2", 1) + body) == "inconclusive"


def test_icns_length_field_and_table():
    data = rendered("ICNS", "RGBA", (16, 16))
    assert status(icns, data + b"x") == "fail"
    broken = bytearray(data)
    struct.pack_into(">I", broken, 12, 1)  # first icon length below header size
    assert status(icns, bytes(broken)) == "fail"


def test_qoi_end_marker():
    data = rendered("QOI", "RGBA")
    assert status(qoi, data[:-1] + b"\0") == "fail"


def test_psd_layers_are_not_decoded_only_the_composite():
    from magika_datasets.validators._shared import pillow

    header = b"8BPS\0\1" + b"\0" * 6 + struct.pack(">HIIHH", 1, 4, 4, 8, 1)
    # A layer section that claims one layer with an unsupported compression code.
    layer = (
        struct.pack(">IIhIIII", 0, 0, 1, 0, 0, 4, 4)
        + struct.pack(">H", 1)
        + struct.pack(">hI", 0, 2 + 16)
        + b"8BIMnorm"
        + b"\xff\0\0\0\0"
        + b"\0\0\0"
        + struct.pack(">I", 0)
    )
    layer_section = struct.pack(">I", len(layer)) + layer
    body = (
        struct.pack(">I", 0) * 2
        + struct.pack(">I", len(layer_section))
        + layer_section
        + struct.pack(">H", 0)
        + b"\x7f" * 16
    )
    assert pillow.inspect(header + body, "PSD")[0] == "pass"
    assert status(psd, header + body) == "pass"
