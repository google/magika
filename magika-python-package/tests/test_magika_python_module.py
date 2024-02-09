# Copyright 2024 Google LLC
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

import random
import tempfile
from pathlib import Path

import pytest

from magika.magika import Magika
from magika.prediction_mode import PredictionMode
from tests import utils
from tests.utils import get_random_ascii_bytes


@pytest.mark.smoketest
def test_magika_modele_basic_tests():
    model_dir = utils.get_default_model_dir()
    test_path = utils.get_one_basic_test_file_path()

    m = Magika(model_dir=model_dir)

    _ = m.get_content_type(test_path)
    _ = m.get_content_types([test_path])


def test_magika_module_with_different_prediction_modes():
    model_dir = utils.get_default_model_dir()
    m = Magika(model_dir=model_dir, prediction_mode=PredictionMode.BEST_GUESS)
    assert m.get_output_ct_label("python", 0.01) == "python"
    assert m.get_output_ct_label("python", 0.40) == "python"
    assert m.get_output_ct_label("python", 0.60) == "python"
    assert m.get_output_ct_label("python", 0.99) == "python"

    # test that the default is HIGH_CONFIDENCE
    m = Magika(model_dir=model_dir)
    assert m.get_output_ct_label("python", 0.01) == "txt"
    assert (
        m.get_output_ct_label("python", Magika.MEDIUM_CONFIDENCE_THRESHOLD - 0.01)
        == "txt"
    )
    assert (
        m.get_output_ct_label("python", Magika.MEDIUM_CONFIDENCE_THRESHOLD + 0.01)
        == "txt"
    )
    assert m.get_output_ct_label("python", 0.99) == "python"

    m = Magika(model_dir=model_dir, prediction_mode=PredictionMode.MEDIUM_CONFIDENCE)
    assert m.get_output_ct_label("python", 0.01) == "txt"
    assert (
        m.get_output_ct_label("python", Magika.MEDIUM_CONFIDENCE_THRESHOLD - 0.01)
        == "txt"
    )
    assert m.get_output_ct_label("python", 0.60) == "python"
    assert m.get_output_ct_label("python", 0.99) == "python"

    m = Magika(model_dir=model_dir, prediction_mode=PredictionMode.HIGH_CONFIDENCE)
    assert m.get_output_ct_label("python", 0.01) == "txt"
    assert m.get_output_ct_label("python", 0.60) == "txt"
    assert m.get_output_ct_label("python", 0.99) == "python"


def test_extract_features_with_ascii():
    random.seed(42)

    _test_extract_features_with_content(get_random_ascii_bytes(0))
    _test_extract_features_with_content(get_random_ascii_bytes(1))
    _test_extract_features_with_content(get_random_ascii_bytes(2))
    _test_extract_features_with_content(get_random_ascii_bytes(8))
    _test_extract_features_with_content(get_random_ascii_bytes(10))
    _test_extract_features_with_content(get_random_ascii_bytes(256))
    _test_extract_features_with_content(get_random_ascii_bytes(511))
    _test_extract_features_with_content(get_random_ascii_bytes(512))
    _test_extract_features_with_content(get_random_ascii_bytes(513))
    _test_extract_features_with_content(get_random_ascii_bytes(1024))
    _test_extract_features_with_content(get_random_ascii_bytes(20_000))
    _test_extract_features_with_content(get_random_ascii_bytes(1_000_000))


def test_extract_features_with_spaces():
    random.seed(42)

    whitespace_sizes = [
        1,
        2,
        4,
        10,
        400,
        511,
        512,
        513,
        1023,
        1024,
        1025,
        2000,
    ]

    for whitespace_size in whitespace_sizes:
        _test_extract_features_with_content(
            (b" " * whitespace_size) + get_random_ascii_bytes(128)
        )
        _test_extract_features_with_content(
            get_random_ascii_bytes(128) + (b" " * whitespace_size)
        )
        _test_extract_features_with_content(
            (b" " * whitespace_size)
            + get_random_ascii_bytes(128)
            + (b" " * whitespace_size)
        )

    _test_extract_features_with_content(b" " * 1000)


def _test_extract_features_with_content(content: bytes):
    print(
        f"Testing with content of len {len(content)}: {content[:min(128, len(content))]!r}...{content[-min(128, len(content)):]!r}"
    )
    with tempfile.TemporaryDirectory() as td:
        td_path = Path(td)
        tf_path = td_path / "file.dat"

        with open(tf_path, "wb") as f:
            f.write(content)

        content_stripped = content.strip()

        model_dir = utils.get_default_model_dir()
        m = Magika(model_dir=model_dir)

        # beg_size, mid_size, end_size
        test_parts_sizes = [
            (512, 0, 512),
            (512, 0, 0),
            (0, 0, 512),
            (512, 0, 256),
            (256, 0, 512),
            (8, 0, 0),
            (8, 0, 8),
            (0, 0, 8),
            (1, 0, 0),
            (1, 0, 1),
            (0, 0, 1),
            (8, 0, 1),
            (8, 0, 2),
            (1, 0, 8),
            (2, 0, 8),
        ]

        for beg_size, mid_size, end_size in test_parts_sizes:
            print(
                f"Testing with {beg_size=}, {mid_size=}, {end_size=}, {len(content)=}"
            )
            features = m.extract_features(
                tf_path, beg_size=beg_size, mid_size=mid_size, end_size=end_size
            )

            beg_ints_out = features["beg"]
            mid_ints_out = features["mid"]
            end_ints_out = features["end"]

            assert len(beg_ints_out) == beg_size
            assert len(mid_ints_out) == mid_size
            assert len(end_ints_out) == end_size

            if beg_size > 0:
                if len(content_stripped) >= beg_size:
                    assert bytes(beg_ints_out) == content_stripped[:beg_size]
                else:
                    # there is some padding at the end
                    assert (
                        bytes(beg_ints_out[0 : len(content_stripped)])
                        == content_stripped
                    )
                    assert beg_ints_out[len(content_stripped) : beg_size] == [
                        Magika.PADDING_TOKEN
                    ] * (beg_size - len(content_stripped))

            # mid is not supported
            assert mid_size == 0

            if end_size > 0:
                if len(content_stripped) >= end_size:
                    assert bytes(end_ints_out) == content_stripped[-end_size:]
                else:
                    # there is some padding at the beginning
                    assert end_ints_out[0 : end_size - len(content_stripped)] == [
                        Magika.PADDING_TOKEN
                    ] * (end_size - len(content_stripped))
                    assert (
                        bytes(end_ints_out[end_size - len(content_stripped) : end_size])
                        == content_stripped
                    )
