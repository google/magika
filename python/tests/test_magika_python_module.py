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

from magika import Magika, PredictionMode
from magika.content_types import ContentType, ContentTypesManager
from tests import utils
from tests.utils import get_random_ascii_bytes


@pytest.mark.smoketest
def test_magika_module_one_basic_test() -> None:
    model_dir = utils.get_default_model_dir()
    test_path = utils.get_one_basic_test_file_path()

    m = Magika(model_dir=model_dir)

    _ = m.identify_path(test_path)
    _ = m.identify_paths([test_path])


@pytest.mark.smoketest
def test_magika_module_with_default_model() -> None:
    test_path = utils.get_one_basic_test_file_path()

    m = Magika()

    _ = m.identify_path(test_path)
    _ = m.identify_paths([test_path])


def test_magika_module_with_basic_tests_by_paths() -> None:
    model_dir = utils.get_default_model_dir()
    tests_paths = utils.get_basic_test_files_paths()

    m = Magika(model_dir=model_dir)
    ctm = ContentTypesManager()

    results = m.identify_paths(tests_paths)

    for test_path, result in zip(tests_paths, results):
        file_ext = test_path.suffix.lstrip(".")
        true_cts = ctm.get_cts_by_ext(file_ext)
        assert len(true_cts) > 0
        true_cts_labels = [ct.name for ct in true_cts]
        assert result.path == str(test_path)
        assert result.output.ct_label in true_cts_labels


def test_magika_module_with_basic_tests_by_path() -> None:
    model_dir = utils.get_default_model_dir()
    tests_paths = utils.get_basic_test_files_paths()

    m = Magika(model_dir=model_dir)
    ctm = ContentTypesManager()

    for test_path in tests_paths:
        result = m.identify_path(test_path)
        file_ext = test_path.suffix.lstrip(".")
        true_cts = ctm.get_cts_by_ext(file_ext)
        assert len(true_cts) > 0
        true_cts_labels = [ct.name for ct in true_cts]
        assert result.path == str(test_path)
        assert result.output.ct_label in true_cts_labels


def test_magika_module_with_basic_tests_by_bytes() -> None:
    model_dir = utils.get_default_model_dir()
    tests_paths = utils.get_basic_test_files_paths()

    m = Magika(model_dir=model_dir)
    ctm = ContentTypesManager()

    for test_path in tests_paths:
        content = test_path.read_bytes()
        result = m.identify_bytes(content)
        file_ext = test_path.suffix.lstrip(".")
        true_cts = ctm.get_cts_by_ext(file_ext)
        assert len(true_cts) > 0
        true_cts_labels = [ct.name for ct in true_cts]
        assert result.path == "-"
        assert result.output.ct_label in true_cts_labels


def test_magika_module_with_empty_content() -> None:
    m = Magika()

    empty_content = b""

    res = m.identify_bytes(empty_content)
    assert res.path == "-"
    assert res.dl.ct_label is None
    assert res.output.ct_label == ContentType.EMPTY
    assert res.output.score == 1.0

    with tempfile.TemporaryDirectory() as td:
        tf_path = Path(td) / "empty.dat"
        tf_path.write_bytes(empty_content)
        res = m.identify_path(tf_path)
        assert res.path == str(tf_path)
        assert res.dl.ct_label is None
        assert res.output.score == 1.0


def test_magika_module_with_short_content() -> None:
    m = Magika()

    text_content = b"asd"
    binary_content = b"\x80\x80\x80"

    res = m.identify_bytes(text_content)
    assert res.path == "-"
    assert res.dl.ct_label is None
    assert res.output.ct_label == ContentType.GENERIC_TEXT
    assert res.output.score == 1.0

    res = m.identify_bytes(binary_content)
    assert res.path == "-"
    assert res.dl.ct_label is None
    assert res.output.ct_label == ContentType.UNKNOWN
    assert res.output.score == 1.0

    for content, expected_ct_label in zip(
        [text_content, binary_content], [ContentType.GENERIC_TEXT, ContentType.UNKNOWN]
    ):
        with tempfile.TemporaryDirectory() as td:
            tf_path = Path(td) / "file.txt"
            tf_path.write_bytes(content)
            res = m.identify_path(tf_path)
            assert res.path == str(tf_path)
            assert res.dl.ct_label is None
            assert res.output.ct_label == expected_ct_label
            assert res.output.score == 1.0


def test_magika_module_with_different_prediction_modes() -> None:
    model_dir = utils.get_default_model_dir()
    m = Magika(model_dir=model_dir, prediction_mode=PredictionMode.BEST_GUESS)
    assert m._get_output_ct_label_from_dl_result("python", 0.01) == "python"
    assert m._get_output_ct_label_from_dl_result("python", 0.40) == "python"
    assert m._get_output_ct_label_from_dl_result("python", 0.60) == "python"
    assert m._get_output_ct_label_from_dl_result("python", 0.99) == "python"

    # test that the default is HIGH_CONFIDENCE
    m = Magika(model_dir=model_dir)
    assert m._get_output_ct_label_from_dl_result("python", 0.01) == "txt"
    assert (
        m._get_output_ct_label_from_dl_result(
            "python", m._medium_confidence_threshold - 0.01
        )
        == "txt"
    )
    assert (
        m._get_output_ct_label_from_dl_result(
            "python", m._medium_confidence_threshold + 0.01
        )
        == "txt"
    )
    assert m._get_output_ct_label_from_dl_result("python", 0.99) == "python"

    m = Magika(model_dir=model_dir, prediction_mode=PredictionMode.MEDIUM_CONFIDENCE)
    assert m._get_output_ct_label_from_dl_result("python", 0.01) == "txt"
    assert (
        m._get_output_ct_label_from_dl_result(
            "python", m._medium_confidence_threshold - 0.01
        )
        == "txt"
    )
    assert m._get_output_ct_label_from_dl_result("python", 0.60) == "python"
    assert m._get_output_ct_label_from_dl_result("python", 0.99) == "python"

    m = Magika(model_dir=model_dir, prediction_mode=PredictionMode.HIGH_CONFIDENCE)
    assert m._get_output_ct_label_from_dl_result("python", 0.01) == "txt"
    assert m._get_output_ct_label_from_dl_result("python", 0.60) == "txt"
    assert m._get_output_ct_label_from_dl_result("python", 0.99) == "python"


def test_extract_features_with_ascii() -> None:
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


def test_extract_features_with_spaces() -> None:
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


def _test_extract_features_with_content(content: bytes) -> None:
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
            features = m._extract_features_from_path(
                tf_path, beg_size=beg_size, mid_size=mid_size, end_size=end_size
            )

            beg_ints_out = features.beg
            mid_ints_out = features.mid
            end_ints_out = features.end

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
                        m._padding_token
                    ] * (beg_size - len(content_stripped))

            # mid is not supported
            assert mid_size == 0

            if end_size > 0:
                if len(content_stripped) >= end_size:
                    assert bytes(end_ints_out) == content_stripped[-end_size:]
                else:
                    # there is some padding at the beginning
                    assert end_ints_out[0 : end_size - len(content_stripped)] == [
                        m._padding_token
                    ] * (end_size - len(content_stripped))
                    assert (
                        bytes(end_ints_out[end_size - len(content_stripped) : end_size])
                        == content_stripped
                    )
