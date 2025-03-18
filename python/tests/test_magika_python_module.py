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

import io
import signal
import tempfile
from pathlib import Path
from typing import Any, List, Optional

import pytest

from magika import Magika, PredictionMode
from magika.types import (
    ContentTypeInfo,
    ContentTypeLabel,
    MagikaPrediction,
    MagikaResult,
    Status,
)
from magika.types.overwrite_reason import OverwriteReason
from tests import utils


@pytest.mark.smoketest
def test_magika_module_check_version() -> None:
    import magika as magika_module

    assert isinstance(magika_module.__version__, str)

    m = Magika()
    assert m.get_module_version() == magika_module.__version__

    # Check that, when we don't specify `model_dir`, Magika uses the default
    # model.
    assert m.get_model_name() == m._get_default_model_name()


@pytest.mark.smoketest
def test_magika_module_with_one_test_file() -> None:
    test_path = utils.get_one_basic_test_file_path()

    m = Magika()

    _ = m.identify_path(test_path)
    _ = m.identify_paths([test_path])
    _ = m.identify_bytes(test_path.read_bytes())
    with open(test_path, "rb") as f:
        _ = m.identify_stream(f)


@pytest.mark.smoketest
def test_magika_module_with_explicit_model_dir() -> None:
    model_dir = utils.get_default_model_dir()
    test_path = utils.get_one_basic_test_file_path()

    m = Magika(model_dir=model_dir)

    _ = m.identify_path(test_path)
    _ = m.identify_paths([test_path])
    _ = m.identify_bytes(test_path.read_bytes())
    with open(test_path, "rb") as f:
        _ = m.identify_stream(f)


def test_magika_module_with_basic_tests_by_paths() -> None:
    tests_paths = utils.get_basic_test_files_paths()

    m = Magika()
    results = m.identify_paths(tests_paths)
    check_results_vs_expected_results(tests_paths, results)


def test_magika_module_with_basic_tests_by_path() -> None:
    tests_paths = utils.get_basic_test_files_paths()

    m = Magika()

    for test_path in tests_paths:
        result = m.identify_path(test_path)
        check_result_vs_expected_result(test_path, result)


def test_magika_module_with_basic_tests_by_bytes() -> None:
    tests_paths = utils.get_basic_test_files_paths()

    m = Magika()

    for test_path in tests_paths:
        content = test_path.read_bytes()
        result = m.identify_bytes(content)
        check_result_vs_expected_result(
            test_path, result, expected_result_path=Path("-")
        )


def test_magika_module_with_basic_tests_by_stream() -> None:
    tests_paths = utils.get_basic_test_files_paths()

    m = Magika()

    for test_path in tests_paths:
        with open(test_path, "rb") as f:
            result = m.identify_stream(f)
        check_result_vs_expected_result(
            test_path, result, expected_result_path=Path("-")
        )


def test_magika_module_with_all_models() -> None:
    tests_paths = utils.get_basic_test_files_paths()

    models_dir = utils.get_models_dir()
    for model_dir in models_dir.iterdir():
        m = Magika(model_dir=model_dir)
        for test_path in tests_paths:
            result = m.identify_path(test_path)
            check_result_vs_expected_result(test_path, result)


def test_magika_module_with_previously_missdetected_samples() -> None:
    model_dir = utils.get_default_model_dir()
    tests_paths = utils.get_previously_missdetected_files_paths()

    m = Magika(model_dir=model_dir)
    results = m.identify_paths(tests_paths)
    check_results_vs_expected_results(tests_paths, results)


def test_magika_module_with_empty_content() -> None:
    m = Magika()

    empty_content = b""

    res = m.identify_bytes(empty_content)
    assert res.ok
    assert res.path == Path("-")
    assert res.prediction.dl.label == ContentTypeLabel.UNDEFINED
    assert res.prediction.output.label == ContentTypeLabel.EMPTY
    assert res.prediction.score == 1.0

    with tempfile.TemporaryDirectory() as td:
        tf_path = Path(td) / "empty.dat"
        tf_path.write_bytes(empty_content)
        res = m.identify_path(tf_path)
        assert res.path == tf_path
        assert res.ok
        assert res.prediction.dl.label == ContentTypeLabel.UNDEFINED
        assert res.prediction.output.label == ContentTypeLabel.EMPTY
        assert res.prediction.score == 1.0

    res = m.identify_stream(io.BytesIO(b""))
    assert res.path == Path("-")
    assert res.ok
    assert res.prediction.dl.label == ContentTypeLabel.UNDEFINED
    assert res.prediction.output.label == ContentTypeLabel.EMPTY
    assert res.prediction.score == 1.0


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
            assert res.path == tf_path
            assert res.ok
            assert res.prediction.dl.label == ContentTypeLabel.UNDEFINED
            assert res.prediction.output.label == expected_ct_label
            assert res.prediction.score == 1.0

            # prediction via bytes
            res = m.identify_bytes(content)
            assert res.path == Path("-")
            assert res.ok
            assert res.prediction.dl.label == ContentTypeLabel.UNDEFINED
            assert res.prediction.output.label == expected_ct_label
            assert res.prediction.score == 1.0

            # prediction via stream
            res = m.identify_stream(io.BytesIO(content))
            assert res.path == Path("-")
            assert res.ok
            assert res.prediction.dl.label == ContentTypeLabel.UNDEFINED
            assert res.prediction.output.label == expected_ct_label
            assert res.prediction.score == 1.0


def test_magika_module_with_python_and_non_python_content() -> None:
    python_content = (
        b"import flask\nimport requests\n\ndef foo(a):\n    print(f'Test {a}')\n"
    )
    non_python_content = b"clearly not python"

    m = Magika()

    res = m.identify_bytes(python_content)
    assert res.ok
    assert res.prediction.output.label == ContentTypeLabel.PYTHON

    res = m.identify_bytes(non_python_content)
    assert res.ok
    assert res.prediction.output.label == ContentTypeLabel.TXT


def test_magika_module_with_different_prediction_modes() -> None:
    model_dir = utils.get_default_model_dir()
    m = Magika(model_dir=model_dir, prediction_mode=PredictionMode.BEST_GUESS)
    assert m._get_output_ct_label_from_dl_result(ContentTypeLabel.PYTHON, 0.01) == (
        ContentTypeLabel.PYTHON,
        OverwriteReason.NONE,
    )
    assert m._get_output_ct_label_from_dl_result(ContentTypeLabel.PYTHON, 0.40) == (
        ContentTypeLabel.PYTHON,
        OverwriteReason.NONE,
    )
    assert m._get_output_ct_label_from_dl_result(ContentTypeLabel.PYTHON, 0.60) == (
        ContentTypeLabel.PYTHON,
        OverwriteReason.NONE,
    )
    assert m._get_output_ct_label_from_dl_result(ContentTypeLabel.PYTHON, 0.99) == (
        ContentTypeLabel.PYTHON,
        OverwriteReason.NONE,
    )

    m = Magika(model_dir=model_dir, prediction_mode=PredictionMode.MEDIUM_CONFIDENCE)
    assert m._get_output_ct_label_from_dl_result(ContentTypeLabel.PYTHON, 0.01) == (
        ContentTypeLabel.TXT,
        OverwriteReason.LOW_CONFIDENCE,
    )
    assert m._get_output_ct_label_from_dl_result(
        ContentTypeLabel.PYTHON, m._model_config.medium_confidence_threshold - 0.01
    ) == (ContentTypeLabel.TXT, OverwriteReason.LOW_CONFIDENCE)
    assert m._get_output_ct_label_from_dl_result(ContentTypeLabel.PYTHON, 0.60) == (
        ContentTypeLabel.PYTHON,
        OverwriteReason.NONE,
    )
    assert m._get_output_ct_label_from_dl_result(ContentTypeLabel.PYTHON, 0.99) == (
        ContentTypeLabel.PYTHON,
        OverwriteReason.NONE,
    )

    m = Magika(model_dir=model_dir, prediction_mode=PredictionMode.HIGH_CONFIDENCE)
    high_confidence_threshold = m._model_config.thresholds.get(
        ContentTypeLabel.PYTHON, m._model_config.medium_confidence_threshold
    )
    assert m._get_output_ct_label_from_dl_result(ContentTypeLabel.PYTHON, 0.01) == (
        ContentTypeLabel.TXT,
        OverwriteReason.LOW_CONFIDENCE,
    )
    assert m._get_output_ct_label_from_dl_result(
        ContentTypeLabel.PYTHON, high_confidence_threshold - 0.01
    ) == (ContentTypeLabel.TXT, OverwriteReason.LOW_CONFIDENCE)
    assert m._get_output_ct_label_from_dl_result(
        ContentTypeLabel.PYTHON, high_confidence_threshold + 0.01
    ) == (ContentTypeLabel.PYTHON, OverwriteReason.NONE)
    assert m._get_output_ct_label_from_dl_result(ContentTypeLabel.PYTHON, 0.99) == (
        ContentTypeLabel.PYTHON,
        OverwriteReason.NONE,
    )

    # test that the default is HIGH_CONFIDENCE
    m = Magika(model_dir=model_dir)
    high_confidence_threshold = m._model_config.thresholds.get(
        ContentTypeLabel.PYTHON, m._model_config.medium_confidence_threshold
    )
    assert m._get_output_ct_label_from_dl_result(ContentTypeLabel.PYTHON, 0.01) == (
        ContentTypeLabel.TXT,
        OverwriteReason.LOW_CONFIDENCE,
    )
    assert m._get_output_ct_label_from_dl_result(
        ContentTypeLabel.PYTHON, high_confidence_threshold - 0.01
    ) == (ContentTypeLabel.TXT, OverwriteReason.LOW_CONFIDENCE)
    assert m._get_output_ct_label_from_dl_result(
        ContentTypeLabel.PYTHON, high_confidence_threshold + 0.01
    ) == (ContentTypeLabel.PYTHON, OverwriteReason.NONE)
    assert m._get_output_ct_label_from_dl_result(ContentTypeLabel.PYTHON, 0.99) == (
        ContentTypeLabel.PYTHON,
        OverwriteReason.NONE,
    )


def test_magika_module_with_directory() -> None:
    m = Magika()

    with tempfile.TemporaryDirectory() as td:
        td_path = Path(td)
        res = m.identify_path(td_path)
        assert res.path == td_path
        assert res.ok
        assert res.prediction.dl.label == ContentTypeLabel.UNDEFINED
        assert res.prediction.output.label == ContentTypeLabel.DIRECTORY
        assert res.prediction.score == 1.0


def test_magika_module_multiple_copies_of_the_same_file() -> None:
    with tempfile.TemporaryDirectory() as td:
        test_path = Path(td) / "test.txt"
        test_path.write_text("test")

        test_paths = [test_path] * 3

        m = Magika()
        results = m.identify_paths(test_paths)
        assert len(results) == len(test_paths)
        for result in results:
            assert result.path == test_path
            assert result.ok
            assert result.prediction.output.label == ContentTypeLabel.TXT


def test_magika_module_with_symlink() -> None:
    with tempfile.TemporaryDirectory() as td:
        test_path = Path(td) / "test.txt"
        test_path.write_text("test")

        symlink_path = Path(td) / "symlink-test.txt"
        symlink_path.symlink_to(test_path)

        m = Magika()
        res = m.identify_path(test_path)
        assert res.path == test_path
        assert res.ok
        assert res.prediction.output.label == ContentTypeLabel.TXT
        res = m.identify_path(symlink_path)
        assert res.path == symlink_path
        assert res.ok
        assert res.prediction.output.label == ContentTypeLabel.TXT

        m = Magika(no_dereference=True)
        res = m.identify_path(test_path)
        assert res.path == test_path
        assert res.ok
        assert res.prediction.output.label == ContentTypeLabel.TXT
        res = m.identify_path(symlink_path)
        assert res.path == symlink_path
        assert res.ok
        assert res.prediction.output.label == ContentTypeLabel.SYMLINK


def test_magika_module_with_non_existing_file() -> None:
    m = Magika()

    with tempfile.TemporaryDirectory() as td:
        non_existing_path = Path(td) / "non_existing.txt"

        res = m.identify_path(non_existing_path)
        assert res.path == non_existing_path
        assert not res.ok
        assert res.status == Status.FILE_NOT_FOUND_ERROR


def test_magika_module_with_permission_error() -> None:
    m = Magika()

    with tempfile.TemporaryDirectory() as td:
        unreadable_test_path = Path(td) / "test.txt"
        unreadable_test_path.write_text("text")

        unreadable_test_path.chmod(0o000)

        res = m.identify_path(unreadable_test_path)
        assert res.path == unreadable_test_path
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


def test_api_call_with_bad_types() -> None:
    m = Magika()

    _ = m.identify_path(Path("/non_existing.txt"))
    _ = m.identify_path("/non_existing.txt")
    with pytest.raises(TypeError):
        _ = m.identify_path(b"/non_existing.txt")  # type: ignore[arg-type]

    _ = m.identify_paths([Path("/non_existing.txt")])
    _ = m.identify_paths(["/non_existing.txt"])
    _ = m.identify_paths([Path("/non_existing.txt"), Path("/not_existing2.txt")])
    _ = m.identify_paths([Path("/non_existing.txt"), "/not_existing2.txt"])
    _ = m.identify_paths(["/non_existing.txt", "/not_existing2.txt"])
    with pytest.raises(TypeError):
        _ = m.identify_paths(Path("/non_existing.txt"))  # type: ignore[arg-type]
    with pytest.raises(TypeError):
        _ = m.identify_paths([b"/non_existing.txt"])  # type: ignore[list-item]
    with pytest.raises(TypeError):
        _ = m.identify_paths([Path("/non_existing.txt"), b"/not_existing2.txt"])  # type: ignore[list-item]

    _ = m.identify_bytes(b"bytes content")
    with pytest.raises(TypeError):
        _ = m.identify_bytes("str content")  # type: ignore[arg-type]

    _ = m.identify_stream(io.BytesIO(b"bytes stream content"))
    with pytest.raises(TypeError):
        _ = m.identify_stream(io.StringIO("str stream content"))  # type: ignore[arg-type]
    with pytest.raises(TypeError):
        _ = m.identify_stream(b"bytes content")  # type: ignore[arg-type]
    with pytest.raises(TypeError):
        _ = m.identify_stream("str content")  # type: ignore[arg-type]


def test_access_magika_result_and_prediction():
    m = Magika()

    res = m.identify_bytes(b"text")
    assert isinstance(res, MagikaResult)
    assert isinstance(res.path, Path)
    assert isinstance(res.ok, bool)
    assert isinstance(res.status, Status)
    assert isinstance(res.prediction, MagikaPrediction)
    assert isinstance(res.prediction.dl, ContentTypeInfo)
    assert isinstance(res.prediction.output, ContentTypeInfo)
    assert isinstance(res.prediction.score, float)
    # test access to forwarded properties
    assert isinstance(res.dl, ContentTypeInfo)
    assert isinstance(res.output, ContentTypeInfo)
    assert isinstance(res.score, float)
    # test access to non-existing properties
    with pytest.raises(AttributeError):
        _ = res.foo  # type: ignore[attr-defined]
    with pytest.raises(AttributeError):
        _ = res.prediction.foo  # type: ignore[attr-defined]

    res = m.identify_path(Path("/non_existing.txt"))
    assert isinstance(res, MagikaResult)
    assert isinstance(res.path, Path)
    assert isinstance(res.ok, bool)
    assert isinstance(res.status, Status)
    with pytest.raises(ValueError):
        _ = res.prediction
    with pytest.raises(ValueError):
        _ = res.prediction.dl
    with pytest.raises(ValueError):
        _ = res.prediction.output
    with pytest.raises(ValueError):
        _ = res.prediction.score
    with pytest.raises(ValueError):
        _ = res.dl
    with pytest.raises(ValueError):
        _ = res.output
    with pytest.raises(ValueError):
        _ = res.score
    with pytest.raises(AttributeError):
        _ = res.foo  # type: ignore[attr-defined]
    with pytest.raises(ValueError):
        _ = res.prediction.foo  # type: ignore[attr-defined]


def test_access_backward_compatibility_layer() -> None:
    m = Magika()

    res = m.identify_bytes(b"text")
    assert isinstance(res, MagikaResult)
    assert isinstance(res.path, Path)
    assert isinstance(res.ok, bool)
    assert isinstance(res.status, Status)
    assert isinstance(res.prediction, MagikaPrediction)
    assert isinstance(res.prediction.dl, ContentTypeInfo)
    assert isinstance(res.prediction.output, ContentTypeInfo)
    assert isinstance(res.prediction.score, float)

    with pytest.warns(DeprecationWarning):
        assert res.dl.ct_label == res.prediction.dl.label
    with pytest.warns(DeprecationWarning):
        assert res.output.ct_label == res.prediction.output.label

    with pytest.raises(AttributeError):
        _ = res.dl.score
    with pytest.raises(AttributeError):
        _ = res.output.score

    with pytest.warns(DeprecationWarning):
        assert res.dl.magic == res.prediction.dl.description
    with pytest.warns(DeprecationWarning):
        assert res.output.magic == res.prediction.output.description


def test_get_model_and_output_content_types() -> None:
    m = Magika()
    output_content_types = m.get_output_content_types()
    output_content_types_set = set(output_content_types)
    model_content_types = m.get_model_content_types()
    model_content_types_set = set(model_content_types)

    assert isinstance(output_content_types, List)
    assert len(output_content_types) > 0
    assert isinstance(model_content_types, List)
    assert len(model_content_types) > 0

    for ct in output_content_types:
        assert isinstance(ct, ContentTypeLabel)

    # Check for no duplicates
    assert len(output_content_types) == len(output_content_types_set)

    # Check basic properties about special ContentTypeLabel entries
    special_output_content_types = {
        ContentTypeLabel.DIRECTORY,
        ContentTypeLabel.EMPTY,
        ContentTypeLabel.SYMLINK,
        ContentTypeLabel.TXT,
        ContentTypeLabel.UNKNOWN,
    }
    special_model_content_types = {ContentTypeLabel.UNDEFINED}
    assert special_output_content_types.issubset(output_content_types_set)
    assert not special_model_content_types.issubset(output_content_types_set)
    assert special_model_content_types.issubset(model_content_types_set)
    assert not special_output_content_types.issubset(model_content_types_set)

    # Spot check for popular content types
    assert {
        ContentTypeLabel.ELF,
        ContentTypeLabel.PDF,
    }.issubset(output_content_types_set)
    assert {
        ContentTypeLabel.ELF,
        ContentTypeLabel.PDF,
    }.issubset(model_content_types_set)


def get_expected_content_type_label_from_test_file_path(
    test_path: Path,
) -> ContentTypeLabel:
    return ContentTypeLabel(test_path.parent.name)


def check_result_vs_expected_result(
    file_path: Path, result: MagikaResult, expected_result_path: Optional[Path] = None
) -> None:
    if expected_result_path is None:
        expected_result_path = file_path
    assert result.path == expected_result_path
    assert result.ok
    expected_ct_label = get_expected_content_type_label_from_test_file_path(file_path)
    assert result.prediction.output.label == expected_ct_label


def check_results_vs_expected_results(
    files_paths: List[Path], results: List[MagikaResult]
) -> None:
    for file_path, result in zip(files_paths, results):
        check_result_vs_expected_result(file_path, result)
