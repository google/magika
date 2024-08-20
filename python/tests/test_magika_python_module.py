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

import signal
import tempfile
from pathlib import Path
from typing import Any

import pytest

from magika import Magika, PredictionMode
from magika.types import ContentTypeLabel, Status
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
        expected_ct_label = get_expected_content_type_label_from_test_file_path(
            test_path
        )
        assert result.value.output.label == expected_ct_label


def test_magika_module_with_basic_tests_by_path() -> None:
    model_dir = utils.get_default_model_dir()
    tests_paths = utils.get_basic_test_files_paths()

    m = Magika(model_dir=model_dir)

    for test_path in tests_paths:
        result = m.identify_path(test_path)
        assert result.ok
        expected_ct_label = get_expected_content_type_label_from_test_file_path(
            test_path
        )
        assert result.value.output.label == expected_ct_label


def test_magika_module_with_basic_tests_by_bytes() -> None:
    model_dir = utils.get_default_model_dir()
    tests_paths = utils.get_basic_test_files_paths()

    m = Magika(model_dir=model_dir)

    for test_path in tests_paths:
        content = test_path.read_bytes()
        result = m.identify_bytes(content)
        assert result.ok
        expected_ct_label = get_expected_content_type_label_from_test_file_path(
            test_path
        )
        assert result.value.output.label == expected_ct_label


def test_magika_module_with_mitra_tests_by_paths() -> None:
    model_dir = utils.get_default_model_dir()
    tests_paths = utils.get_mitra_test_files_paths()

    m = Magika(model_dir=model_dir)

    results = m.identify_paths(tests_paths)

    for test_path, result in zip(tests_paths, results):
        print(f"Test: {test_path}")
        assert result.ok
        expected_ct_label = get_expected_content_type_label_from_test_file_path(
            test_path
        )
        assert result.value.output.label == expected_ct_label


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


def test_magika_module_with_python_and_non_python_content() -> None:
    python_content = (
        b"import flask\nimport requests\n\ndef foo(a):\n    print(f'Test {a}')\n"
    )
    non_python_content = b"xmport asd\nxmport requests"

    m = Magika()

    res = m.identify_bytes(python_content)
    assert res.ok
    assert res.value.output.label == ContentTypeLabel.PYTHON

    res = m.identify_bytes(non_python_content)
    assert res.ok
    assert res.value.output.label == ContentTypeLabel.TXT


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


def test_magika_module_with_directory() -> None:
    m = Magika()

    with tempfile.TemporaryDirectory() as td:
        td_path = Path(td)
        res = m.identify_path(td_path)
        assert res.ok
        assert res.value.dl.label == ContentTypeLabel.UNDEFINED
        assert res.value.output.label == ContentTypeLabel.DIRECTORY
        assert res.value.score == 1.0


def test_magika_module_multiple_copies_of_the_same_file() -> None:
    with tempfile.TemporaryDirectory() as td:
        test_path = Path(td) / "test.txt"
        test_path.write_text("test")

        test_paths = [test_path] * 3

        m = Magika()
        results = m.identify_paths(test_paths)
        assert len(results) == len(test_paths)
        for result in results:
            assert result.ok
            assert result.value.output.label == ContentTypeLabel.TXT


def test_magika_cli_with_many_files() -> None:
    test_file_path = utils.get_one_basic_test_file_path()

    m = Magika()

    for n in [10, 100]:
        test_files_paths = [test_file_path] * n
        results = m.identify_paths(test_files_paths)
        for result in results:
            assert result.ok
            # TODO: check that the result is actually correct


def test_magika_module_with_symlink() -> None:
    with tempfile.TemporaryDirectory() as td:
        test_path = Path(td) / "test.txt"
        test_path.write_text("test")

        symlink_path = Path(td) / "symlink-test.txt"
        symlink_path.symlink_to(test_path)

        m = Magika()
        res = m.identify_path(test_path)
        assert res.ok
        assert res.value.output.label == ContentTypeLabel.TXT
        res = m.identify_path(symlink_path)
        assert res.ok
        assert res.value.output.label == ContentTypeLabel.TXT

        m = Magika(no_dereference=True)
        res = m.identify_path(test_path)
        assert res.ok
        assert res.value.output.label == ContentTypeLabel.TXT
        res = m.identify_path(symlink_path)
        assert res.ok
        assert res.value.output.label == ContentTypeLabel.SYMLINK


def test_magika_module_with_non_existing_file() -> None:
    m = Magika()

    with tempfile.TemporaryDirectory() as td:
        non_existing_path = Path(td) / "non_existing.txt"

        res = m.identify_path(non_existing_path)
        assert not res.ok
        assert res.status == Status.FILE_NOT_FOUND_ERROR


def test_magika_module_with_permission_error() -> None:
    m = Magika()

    with tempfile.TemporaryDirectory() as td:
        unreadable_test_path = Path(td) / "test.txt"
        unreadable_test_path.write_text("text")

        unreadable_test_path.chmod(0o000)

        res = m.identify_path(unreadable_test_path)
        assert not res.ok
        assert res.status == Status.PERMISSION_ERROR


@pytest.mark.skip
def test_magika_module_with_really_many_files() -> None:
    test_file_path = utils.get_one_basic_test_file_path()

    m = Magika()

    for n in [10000]:
        test_files_paths = [test_file_path] * n

        results = m.identify_paths(test_files_paths)
        for result in results:
            assert result.ok
            # TODO: add more checks


@pytest.mark.slow
def test_magika_module_with_big_file() -> None:
    def signal_handler(signum: int, frame: Any) -> None:
        raise Exception("Timeout")

    signal.signal(signal.SIGALRM, signal_handler)

    # It should take much less than this, but pytest weird scheduling sometimes
    # creates unexpected slow downs.
    timeout = 2

    m = Magika()

    for sample_size in [1000, 10000, 1_000_000, 1_000_000_000, 10_000_000_000]:
        with tempfile.TemporaryDirectory() as td:
            sample_path = Path(td) / "sample.dat"
            utils.write_random_file_with_size(sample_path, sample_size)
            print(f"Starting running Magika with a timeout of {timeout}")
            signal.alarm(timeout)
            res = m.identify_path(sample_path)
            assert res.ok
            signal.alarm(0)
            print("Done running Magika")


def get_expected_content_type_label_from_test_file_path(
    test_path: Path,
) -> ContentTypeLabel:
    return ContentTypeLabel(test_path.parent.name)
