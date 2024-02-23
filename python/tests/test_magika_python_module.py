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

import tempfile
from pathlib import Path

import pytest

from magika import Magika, PredictionMode
from magika.content_types import ContentType, ContentTypesManager
from tests import utils


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
