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
from magika.types import ContentTypeLabel
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

    results = m.identify_paths(tests_paths)

    for test_path, result in zip(tests_paths, results):
        assert result.ok
        file_ext = test_path.suffix.lstrip(".")
        true_cts_labels = get_content_types_from_ext(m, file_ext)
        assert len(true_cts_labels) > 0
        assert result.value.output.label in true_cts_labels


def test_magika_module_with_basic_tests_by_path() -> None:
    model_dir = utils.get_default_model_dir()
    tests_paths = utils.get_basic_test_files_paths()

    m = Magika(model_dir=model_dir)

    for test_path in tests_paths:
        result = m.identify_path(test_path)
        assert result.ok
        file_ext = test_path.suffix.lstrip(".")
        true_cts_labels = get_content_types_from_ext(m, file_ext)
        assert len(true_cts_labels) > 0
        assert result.value.output.label in true_cts_labels


def test_magika_module_with_basic_tests_by_bytes() -> None:
    model_dir = utils.get_default_model_dir()
    tests_paths = utils.get_basic_test_files_paths()

    m = Magika(model_dir=model_dir)

    for test_path in tests_paths:
        content = test_path.read_bytes()
        result = m.identify_bytes(content)
        assert result.ok
        file_ext = test_path.suffix.lstrip(".")
        true_cts_labels = get_content_types_from_ext(m, file_ext)
        assert len(true_cts_labels) > 0
        assert result.value.output.label in true_cts_labels


def test_magika_module_with_empty_content() -> None:
    m = Magika()

    empty_content = b""

    res = m.identify_bytes(empty_content)
    assert res.ok
    assert res.value.dl.label == ContentTypeLabel.UNDEFINED
    assert res.value.output.label == ContentTypeLabel.EMPTY
    assert res.value.score == 1.0

    with tempfile.TemporaryDirectory() as td:
        tf_path = Path(td) / "empty.dat"
        tf_path.write_bytes(empty_content)
        res = m.identify_path(tf_path)
        assert res.ok
        assert res.value.dl.label == ContentTypeLabel.UNDEFINED
        assert res.value.output.label == ContentTypeLabel.EMPTY
        assert res.value.score == 1.0


def test_magika_module_with_short_content() -> None:
    m = Magika()

    text_content = b"asd"
    binary_content = b"\x80\x80\x80"

    for content, expected_ct_label in zip(
        [text_content, binary_content],
        [ContentTypeLabel.TXT, ContentTypeLabel.UNKNOWN],
    ):
        with tempfile.TemporaryDirectory() as td:
            # prediction via path
            tf_path = Path(td) / "file.txt"
            tf_path.write_bytes(content)
            res = m.identify_path(tf_path)
            assert res.ok
            assert res.value.dl.label == ContentTypeLabel.UNDEFINED
            assert res.value.output.label == expected_ct_label
            assert res.value.score == 1.0

            # prediction via content
            res = m.identify_bytes(content)
            assert res.ok
            assert res.value.dl.label == ContentTypeLabel.UNDEFINED
            assert res.value.output.label == expected_ct_label
            assert res.value.score == 1.0


def test_magika_module_with_different_prediction_modes() -> None:
    model_dir = utils.get_default_model_dir()
    m = Magika(model_dir=model_dir, prediction_mode=PredictionMode.BEST_GUESS)
    assert (
        m._get_output_ct_label_from_dl_result(ContentTypeLabel.PYTHON, 0.01)
        == ContentTypeLabel.PYTHON
    )
    assert (
        m._get_output_ct_label_from_dl_result(ContentTypeLabel.PYTHON, 0.40)
        == ContentTypeLabel.PYTHON
    )
    assert (
        m._get_output_ct_label_from_dl_result(ContentTypeLabel.PYTHON, 0.60)
        == ContentTypeLabel.PYTHON
    )
    assert (
        m._get_output_ct_label_from_dl_result(ContentTypeLabel.PYTHON, 0.99)
        == ContentTypeLabel.PYTHON
    )

    m = Magika(model_dir=model_dir, prediction_mode=PredictionMode.MEDIUM_CONFIDENCE)
    assert (
        m._get_output_ct_label_from_dl_result(ContentTypeLabel.PYTHON, 0.01)
        == ContentTypeLabel.TXT
    )
    assert (
        m._get_output_ct_label_from_dl_result(
            ContentTypeLabel.PYTHON, m._model_config.medium_confidence_threshold - 0.01
        )
        == ContentTypeLabel.TXT
    )
    assert (
        m._get_output_ct_label_from_dl_result(ContentTypeLabel.PYTHON, 0.60)
        == ContentTypeLabel.PYTHON
    )
    assert (
        m._get_output_ct_label_from_dl_result(ContentTypeLabel.PYTHON, 0.99)
        == ContentTypeLabel.PYTHON
    )

    m = Magika(model_dir=model_dir, prediction_mode=PredictionMode.HIGH_CONFIDENCE)
    high_confidence_threshold = m._model_config.thresholds.get(
        ContentTypeLabel.PYTHON, m._model_config.medium_confidence_threshold
    )
    assert (
        m._get_output_ct_label_from_dl_result(ContentTypeLabel.PYTHON, 0.01)
        == ContentTypeLabel.TXT
    )
    assert (
        m._get_output_ct_label_from_dl_result(
            ContentTypeLabel.PYTHON, high_confidence_threshold - 0.01
        )
        == ContentTypeLabel.TXT
    )
    assert (
        m._get_output_ct_label_from_dl_result(
            ContentTypeLabel.PYTHON, high_confidence_threshold + 0.01
        )
        == ContentTypeLabel.PYTHON
    )
    assert (
        m._get_output_ct_label_from_dl_result(ContentTypeLabel.PYTHON, 0.99)
        == ContentTypeLabel.PYTHON
    )

    # test that the default is HIGH_CONFIDENCE
    m = Magika(model_dir=model_dir)
    high_confidence_threshold = m._model_config.thresholds.get(
        ContentTypeLabel.PYTHON, m._model_config.medium_confidence_threshold
    )
    assert (
        m._get_output_ct_label_from_dl_result(ContentTypeLabel.PYTHON, 0.01)
        == ContentTypeLabel.TXT
    )
    assert (
        m._get_output_ct_label_from_dl_result(
            ContentTypeLabel.PYTHON, high_confidence_threshold - 0.01
        )
        == ContentTypeLabel.TXT
    )
    assert (
        m._get_output_ct_label_from_dl_result(
            ContentTypeLabel.PYTHON, high_confidence_threshold + 0.01
        )
        == ContentTypeLabel.PYTHON
    )
    assert (
        m._get_output_ct_label_from_dl_result(ContentTypeLabel.PYTHON, 0.99)
        == ContentTypeLabel.PYTHON
    )


def get_content_types_from_ext(magika: Magika, ext: str) -> list[ContentTypeLabel]:
    labels = []
    for ct_label, ct_info in magika._cts_infos.items():
        ct_info = magika._get_ct_info(ct_label)
        if ext in ct_info.extensions:
            labels.append(ct_label)
    return labels
