import io

from helpers import status
from PIL import Image

from magika_datasets.validators._shared.pillow import inspect
from magika_datasets.validators.image import jp2


def test_boxed_and_raw_jpeg2000_decode():
    for raw in (False, True):
        output = io.BytesIO()
        Image.new("RGB", (8, 8), "blue").save(output, format="JPEG2000", no_jp2=raw)
        data = output.getvalue()
        assert status(jp2, data) == "pass"
        assert status(jp2, data[:30]) == "fail"
    assert status(jp2, b"not jpeg2000") == "not_applicable"


def test_missing_openjpeg_is_not_a_file_failure(monkeypatch):
    monkeypatch.setattr("PIL.features.check", lambda _: False)
    assert inspect(b"ignored", "JPEG2000")[0] == "inconclusive"
