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

from __future__ import annotations

import dataclasses
import io
import signal
import tempfile
from pathlib import Path
from typing import Any, List, Optional

import pytest

from magika import Magika, MagikaError, PredictionMode
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


def test_magika_module_identify_stream_does_not_alter_position() -> None:
    m = Magika()

    contents = [
        b"",
        b"short",
        b"A" * 100,
        b"A" * 1000,
        b"A" * 10000,
    ]
    for content in contents:
        stream = io.BytesIO(content)
        # seek to a specific non-special position
        pos = min(2, len(content))
        stream.seek(pos)
        res = m.identify_stream(stream)
        assert res.ok
        assert stream.tell() == pos


def test_magika_module_with_whitespaces() -> None:
    m = Magika()

    ws_nums = sorted(
        {
            1,
            m._model_config.min_file_size_for_dl - 1,
            m._model_config.min_file_size_for_dl,
            m._model_config.min_file_size_for_dl + 1,
            m._model_config.beg_size - 1,
            m._model_config.beg_size,
            m._model_config.beg_size + 1,
            m._model_config.end_size - 1,
            m._model_config.end_size,
            m._model_config.end_size + 1,
            m._model_config.beg_size + m._model_config.end_size - 1,
            m._model_config.beg_size + m._model_config.end_size,
            m._model_config.beg_size + m._model_config.end_size + 1,
            m._model_config.beg_size + m._model_config.end_size + 1,
            m._model_config.block_size - 1,
            m._model_config.block_size,
            m._model_config.block_size + 1,
            2 * m._model_config.block_size - 1,
            2 * m._model_config.block_size,
            2 * m._model_config.block_size + 1,
            4 * m._model_config.block_size - 1,
            4 * m._model_config.block_size,
            4 * m._model_config.block_size + 1,
        }
    )

    for ws_num in ws_nums:
        print(f"Calling indentify_bytes with {ws_num} whitespaces")
        content = b" " * ws_num
        res = m.identify_bytes(content)
        assert (
            res.ok
            and res.dl.label == ContentTypeLabel.UNDEFINED
            and res.output.label == ContentTypeLabel.TXT
        )
        res = m.identify_stream(io.BytesIO(content))
        assert (
            res.ok
            and res.dl.label == ContentTypeLabel.UNDEFINED
            and res.output.label == ContentTypeLabel.TXT
        )
        with tempfile.TemporaryDirectory() as td:
            tf_path = Path(td) / "test.bin"
            tf_path.write_bytes(content)
            res = m.identify_path(tf_path)
            assert (
                res.ok
                and res.dl.label == ContentTypeLabel.UNDEFINED
                and res.output.label == ContentTypeLabel.TXT
            )


def test_magika_module_with_different_prediction_modes() -> None:
    model_dir = utils.get_default_model_dir()
    m = Magika(model_dir=model_dir, prediction_mode=PredictionMode.BEST_GUESS)
    assert m._get_output_label_from_dl_label_and_score(
        ContentTypeLabel.PYTHON, 0.01
    ) == (
        ContentTypeLabel.PYTHON,
        OverwriteReason.NONE,
    )
    assert m._get_output_label_from_dl_label_and_score(
        ContentTypeLabel.PYTHON, 0.40
    ) == (
        ContentTypeLabel.PYTHON,
        OverwriteReason.NONE,
    )
    assert m._get_output_label_from_dl_label_and_score(
        ContentTypeLabel.PYTHON, 0.60
    ) == (
        ContentTypeLabel.PYTHON,
        OverwriteReason.NONE,
    )
    assert m._get_output_label_from_dl_label_and_score(
        ContentTypeLabel.PYTHON, 0.99
    ) == (
        ContentTypeLabel.PYTHON,
        OverwriteReason.NONE,
    )

    m = Magika(model_dir=model_dir, prediction_mode=PredictionMode.MEDIUM_CONFIDENCE)
    assert m._get_output_label_from_dl_label_and_score(
        ContentTypeLabel.PYTHON, 0.01
    ) == (
        ContentTypeLabel.TXT,
        OverwriteReason.LOW_CONFIDENCE,
    )
    assert m._get_output_label_from_dl_label_and_score(
        ContentTypeLabel.PYTHON, m._model_config.medium_confidence_threshold - 0.01
    ) == (ContentTypeLabel.TXT, OverwriteReason.LOW_CONFIDENCE)
    assert m._get_output_label_from_dl_label_and_score(
        ContentTypeLabel.PYTHON, 0.60
    ) == (
        ContentTypeLabel.PYTHON,
        OverwriteReason.NONE,
    )
    assert m._get_output_label_from_dl_label_and_score(
        ContentTypeLabel.PYTHON, 0.99
    ) == (
        ContentTypeLabel.PYTHON,
        OverwriteReason.NONE,
    )

    m = Magika(model_dir=model_dir, prediction_mode=PredictionMode.HIGH_CONFIDENCE)
    high_confidence_threshold = m._model_config.thresholds.get(
        ContentTypeLabel.PYTHON, m._model_config.medium_confidence_threshold
    )
    assert m._get_output_label_from_dl_label_and_score(
        ContentTypeLabel.PYTHON, 0.01
    ) == (
        ContentTypeLabel.TXT,
        OverwriteReason.LOW_CONFIDENCE,
    )
    assert m._get_output_label_from_dl_label_and_score(
        ContentTypeLabel.PYTHON, high_confidence_threshold - 0.01
    ) == (ContentTypeLabel.TXT, OverwriteReason.LOW_CONFIDENCE)
    assert m._get_output_label_from_dl_label_and_score(
        ContentTypeLabel.PYTHON, high_confidence_threshold + 0.01
    ) == (ContentTypeLabel.PYTHON, OverwriteReason.NONE)
    assert m._get_output_label_from_dl_label_and_score(
        ContentTypeLabel.PYTHON, 0.99
    ) == (
        ContentTypeLabel.PYTHON,
        OverwriteReason.NONE,
    )

    # test that the default is HIGH_CONFIDENCE
    m = Magika(model_dir=model_dir)
    high_confidence_threshold = m._model_config.thresholds.get(
        ContentTypeLabel.PYTHON, m._model_config.medium_confidence_threshold
    )
    assert m._get_output_label_from_dl_label_and_score(
        ContentTypeLabel.PYTHON, 0.01
    ) == (
        ContentTypeLabel.TXT,
        OverwriteReason.LOW_CONFIDENCE,
    )
    assert m._get_output_label_from_dl_label_and_score(
        ContentTypeLabel.PYTHON, high_confidence_threshold - 0.01
    ) == (ContentTypeLabel.TXT, OverwriteReason.LOW_CONFIDENCE)
    assert m._get_output_label_from_dl_label_and_score(
        ContentTypeLabel.PYTHON, high_confidence_threshold + 0.01
    ) == (ContentTypeLabel.PYTHON, OverwriteReason.NONE)
    assert m._get_output_label_from_dl_label_and_score(
        ContentTypeLabel.PYTHON, 0.99
    ) == (
        ContentTypeLabel.PYTHON,
        OverwriteReason.NONE,
    )


def test_magika_module_overwrite_reason() -> None:
    m_high = Magika(prediction_mode=PredictionMode.HIGH_CONFIDENCE)
    m_medium = Magika(prediction_mode=PredictionMode.MEDIUM_CONFIDENCE)
    m_best = Magika(prediction_mode=PredictionMode.BEST_GUESS)

    python_high_confidence_threshold = m_high._model_config.thresholds.get(
        ContentTypeLabel.PYTHON, m_high._model_config.medium_confidence_threshold
    )
    medium_confidence_threshold = m_medium._model_config.medium_confidence_threshold

    assert m_high._get_output_label_from_dl_label_and_score(
        ContentTypeLabel.PYTHON, python_high_confidence_threshold + 0.01
    ) == (ContentTypeLabel.PYTHON, OverwriteReason.NONE)
    assert m_high._get_output_label_from_dl_label_and_score(
        ContentTypeLabel.PYTHON, python_high_confidence_threshold - 0.01
    ) == (ContentTypeLabel.TXT, OverwriteReason.LOW_CONFIDENCE)

    assert m_medium._get_output_label_from_dl_label_and_score(
        ContentTypeLabel.PYTHON, medium_confidence_threshold + 0.01
    ) == (ContentTypeLabel.PYTHON, OverwriteReason.NONE)
    assert m_medium._get_output_label_from_dl_label_and_score(
        ContentTypeLabel.PYTHON, medium_confidence_threshold - 0.01
    ) == (ContentTypeLabel.TXT, OverwriteReason.LOW_CONFIDENCE)

    assert m_best._get_output_label_from_dl_label_and_score(
        ContentTypeLabel.PYTHON, medium_confidence_threshold + 0.01
    ) == (ContentTypeLabel.PYTHON, OverwriteReason.NONE)
    assert m_best._get_output_label_from_dl_label_and_score(
        ContentTypeLabel.PYTHON, medium_confidence_threshold - 0.01
    ) == (ContentTypeLabel.PYTHON, OverwriteReason.NONE)

    for overwrite_map_ct_key in sorted(m_high._model_config.overwrite_map.keys()):
        overwrite_map_ct_value = m_high._model_config.overwrite_map[
            overwrite_map_ct_key
        ]
        is_overwrite_map_ct_target_text = m_high._cts_infos[
            overwrite_map_ct_value
        ].is_text
        overwrite_map_ct_high_confidence_threshold = (
            m_high._model_config.thresholds.get(
                overwrite_map_ct_key, m_high._model_config.medium_confidence_threshold
            )
        )
        assert m_high._get_output_label_from_dl_label_and_score(
            overwrite_map_ct_key, overwrite_map_ct_high_confidence_threshold + 0.01
        ) == (overwrite_map_ct_value, OverwriteReason.OVERWRITE_MAP)
        assert m_high._get_output_label_from_dl_label_and_score(
            overwrite_map_ct_key, overwrite_map_ct_high_confidence_threshold - 0.01
        ) == (
            ContentTypeLabel.TXT
            if is_overwrite_map_ct_target_text
            else ContentTypeLabel.UNKNOWN,
            OverwriteReason.LOW_CONFIDENCE,
        )

    for generic_ct in [ContentTypeLabel.TXT, ContentTypeLabel.UNKNOWN]:
        generic_type_high_confidence_threshold = m_high._model_config.thresholds.get(
            generic_ct,
            m_high._model_config.medium_confidence_threshold,
        )
        assert m_high._get_output_label_from_dl_label_and_score(
            generic_ct,
            generic_type_high_confidence_threshold - 0.01,
        ) == (generic_ct, OverwriteReason.NONE)
        assert m_medium._get_output_label_from_dl_label_and_score(
            generic_ct, medium_confidence_threshold - 0.01
        ) == (generic_ct, OverwriteReason.NONE)


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


# Symlink dereference behavior in rust/lib Session is deferred for future alignment.
# def test_magika_module_with_symlink() -> None:
#     with tempfile.TemporaryDirectory() as td:
#         test_path = Path(td) / "test.txt"
#         test_path.write_text("test")
#
#         symlink_path = Path(td) / "symlink-test.txt"
#         symlink_path.symlink_to(test_path)
#
#         m = Magika()
#         res = m.identify_path(test_path)
#         assert res.path == test_path
#         assert res.ok
#         assert res.prediction.output.label == ContentTypeLabel.TXT
#         res = m.identify_path(symlink_path)
#         assert res.path == symlink_path
#         assert res.ok
#         assert res.prediction.output.label == ContentTypeLabel.TXT
#
#         m = Magika(no_dereference=True)
#         res = m.identify_path(test_path)
#         assert res.path == test_path
#         assert res.ok
#         assert res.prediction.output.label == ContentTypeLabel.TXT
#         res = m.identify_path(symlink_path)
#         assert res.path == symlink_path
#         assert res.ok
#         assert res.prediction.output.label == ContentTypeLabel.SYMLINK


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

    # Check that an empty, non-accessible file is marked as "permission error".
    # Note that on some file-systems, one can read the file size even without
    # read permission, and it would thus be possible to return "empty" (this is
    # what we were actually doing in the past). However, returning
    # "permission_error" makes the expected behavior consistent across file
    # systems and it simplifies the implementation.
    with tempfile.TemporaryDirectory() as td:
        unreadable_test_path = Path(td) / "test.txt"
        unreadable_test_path.write_text("")

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


def test_magika_imports():
    imported_modules = utils.get_imported_objects_after_wildcard()

    # Check that Magika and other public classes are correctly imported
    from magika import (
        ContentTypeInfo,
        ContentTypeLabel,
        Magika,
        MagikaError,
        MagikaPrediction,
        MagikaResult,
        OverwriteReason,
        PredictionMode,
        Status,
    )

    assert imported_modules.get("ContentTypeInfo") == ContentTypeInfo
    assert imported_modules.get("ContentTypeLabel") == ContentTypeLabel
    assert imported_modules.get("Magika") == Magika
    assert imported_modules.get("MagikaError") == MagikaError
    assert imported_modules.get("MagikaPrediction") == MagikaPrediction
    assert imported_modules.get("MagikaResult") == MagikaResult
    assert imported_modules.get("OverwriteReason") == OverwriteReason
    assert imported_modules.get("PredictionMode") == PredictionMode
    assert imported_modules.get("Status") == Status

    # Check that internal classes are not imported
    assert imported_modules.get("ModelFeatures") is None
    assert imported_modules.get("ModelOutput") is None


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


def test_magika_result_asdict() -> None:
    m = Magika()

    # Successful scan
    res = m.identify_bytes(b"import os\n")
    assert res.ok
    d = res.asdict()
    assert isinstance(d, dict)
    assert d["path"] == "-"
    assert d["status"] == Status.OK
    assert "prediction" in d
    pred_dict = d["prediction"]
    assert "dl" in pred_dict
    assert "output" in pred_dict
    assert "score" in pred_dict
    assert "overwrite_reason" in pred_dict
    assert isinstance(pred_dict["score"], float)
    assert pred_dict["overwrite_reason"] in {
        OverwriteReason.NONE,
        OverwriteReason.LOW_CONFIDENCE,
        OverwriteReason.OVERWRITE_MAP,
    }

    # Verify nested ContentTypeInfo dict fields
    for key in ("dl", "output"):
        ct_dict = pred_dict[key]
        assert "label" in ct_dict
        assert "mime_type" in ct_dict
        assert "group" in ct_dict
        assert "description" in ct_dict
        assert "extensions" in ct_dict
        assert "is_text" in ct_dict
        assert isinstance(ct_dict["extensions"], list)
        assert isinstance(ct_dict["is_text"], bool)

    # Failed scan
    res_err = m.identify_path("/non_existing_path_12345.txt")
    assert not res_err.ok
    d_err = res_err.asdict()
    assert d_err["path"] == "/non_existing_path_12345.txt"
    assert d_err["status"] == Status.FILE_NOT_FOUND_ERROR
    assert "prediction" not in d_err


def test_magika_and_result_str_and_repr() -> None:
    m = Magika()
    expected_m_str = (
        f'Magika(module_version="{m.get_module_version()}", '
        f'model_name="{m.get_model_name()}")'
    )
    assert str(m) == expected_m_str
    assert repr(m) == expected_m_str

    # Successful result
    res_ok = m.identify_bytes(b"hello world\n")
    expected_ok_str = (
        f"MagikaResult(path={res_ok.path}, status={res_ok.status}, "
        f"prediction={res_ok.prediction})"
    )
    assert str(res_ok) == expected_ok_str
    assert repr(res_ok) == expected_ok_str

    # Failed result
    res_err = m.identify_path("/non_existing.txt")
    expected_err_str = f"MagikaResult(path={res_err.path}, status={res_err.status})"
    assert str(res_err) == expected_err_str
    assert repr(res_err) == expected_err_str


def test_magika_result_direct_instantiation() -> None:
    # Direct instantiation with status != OK
    res_err = MagikaResult(
        path=Path("foo.txt"),
        status=Status.FILE_NOT_FOUND_ERROR,
        prediction=None,
    )
    assert not res_err.ok
    assert res_err.status == Status.FILE_NOT_FOUND_ERROR
    assert res_err.path == Path("foo.txt")
    with pytest.raises(ValueError, match="prediction is not set when status != OK"):
        _ = res_err.prediction
    with pytest.raises(ValueError, match="prediction is not set when status != OK"):
        _ = res_err.dl
    with pytest.raises(ValueError, match="prediction is not set when status != OK"):
        _ = res_err.output
    with pytest.raises(ValueError, match="prediction is not set when status != OK"):
        _ = res_err.score

    # Direct instantiation with status OK and prediction
    ct_info = ContentTypeInfo(
        label=ContentTypeLabel.TXT,
        mime_type="text/plain",
        group="text",
        description="Text document",
        extensions=["txt"],
        is_text=True,
    )
    dummy_pred = MagikaPrediction(
        dl=ct_info,
        output=ct_info,
        score=1.0,
        overwrite_reason=OverwriteReason.NONE,
    )
    res_ok = MagikaResult(
        path=Path("foo.txt"),
        status=Status.OK,
        prediction=dummy_pred,
    )
    assert res_ok.ok
    assert res_ok.status == Status.OK
    assert res_ok.prediction == dummy_pred
    assert res_ok.dl == ct_info
    assert res_ok.output == ct_info
    assert res_ok.score == 1.0


def test_content_type_info_field_types() -> None:
    m = Magika()
    res = m.identify_bytes(b"def foo(): pass\n")
    assert res.ok

    for ct in (res.prediction.dl, res.prediction.output):
        assert isinstance(ct.label, ContentTypeLabel)
        assert isinstance(ct.mime_type, str)
        assert isinstance(ct.group, str)
        assert isinstance(ct.description, str)
        assert isinstance(ct.extensions, list)
        for ext in ct.extensions:
            assert isinstance(ext, str)
        assert isinstance(ct.is_text, bool)


def test_content_type_label_str_behavior() -> None:
    # ContentTypeLabel inherits from str
    assert isinstance(ContentTypeLabel.PYTHON, str)
    assert ContentTypeLabel.PYTHON.value == "python"
    assert str(ContentTypeLabel.PYTHON) == "python"
    assert repr(ContentTypeLabel.PYTHON) == "python"
    assert ContentTypeLabel.PYTHON.startswith("py")
    assert ContentTypeLabel("python") == ContentTypeLabel.PYTHON

    py_str: Any = "python"
    assert ContentTypeLabel.PYTHON == py_str

    # Special labels check
    assert ContentTypeLabel.UNDEFINED.value == "undefined"
    assert ContentTypeLabel.EMPTY.value == "empty"
    assert ContentTypeLabel.DIRECTORY.value == "directory"
    assert ContentTypeLabel.SYMLINK.value == "symlink"
    assert ContentTypeLabel.TXT.value == "txt"
    assert ContentTypeLabel.UNKNOWN.value == "unknown"


def test_prediction_mode_enums_and_valid_modes() -> None:
    modes = PredictionMode.get_valid_prediction_modes()
    assert isinstance(modes, list)
    assert len(modes) == 3
    assert set(modes) == {"best_guess", "medium_confidence", "high_confidence"}

    assert isinstance(PredictionMode.HIGH_CONFIDENCE, str)
    assert PredictionMode.HIGH_CONFIDENCE.value == "high_confidence"
    assert PredictionMode.MEDIUM_CONFIDENCE.value == "medium_confidence"
    assert PredictionMode.BEST_GUESS.value == "best_guess"

    pm_str: Any = "high_confidence"
    assert PredictionMode.HIGH_CONFIDENCE == pm_str


def test_status_and_overwrite_reason_enums() -> None:
    assert isinstance(Status.OK, str)
    assert Status.OK.value == "ok"
    assert Status.FILE_NOT_FOUND_ERROR.value == "file_not_found_error"
    assert Status.PERMISSION_ERROR.value == "permission_error"
    assert Status.UNKNOWN.value == "unknown"

    status_str: Any = "ok"
    assert Status.OK == status_str

    assert isinstance(OverwriteReason.NONE, str)
    assert OverwriteReason.NONE.value == "none"
    assert OverwriteReason.LOW_CONFIDENCE.value == "low_confidence"
    assert OverwriteReason.OVERWRITE_MAP.value == "overwrite_map"

    reason_str: Any = "none"
    assert OverwriteReason.NONE == reason_str


def test_magika_prediction_immutability() -> None:
    m = Magika()
    res = m.identify_bytes(b"hello")
    assert res.ok

    with pytest.raises((dataclasses.FrozenInstanceError, AttributeError)):
        res.prediction.score = 0.5  # type: ignore[misc]

    with pytest.raises((dataclasses.FrozenInstanceError, AttributeError)):
        res.prediction.output = res.prediction.dl  # type: ignore[misc]


def test_magika_constructor_options_and_errors() -> None:
    # Test verbose, debug, use_colors flags construct successfully
    _ = Magika(verbose=True)
    _ = Magika(debug=True)
    _ = Magika(use_colors=True)
    _ = Magika(verbose=True, debug=True, use_colors=True)

    # Test invalid model_dir raises MagikaError
    with pytest.raises(MagikaError, match="model dir not found"):
        Magika(model_dir=Path("/non_existent_directory_magika_test_12345"))


def test_identify_paths_empty_list() -> None:
    m = Magika()
    results = m.identify_paths([])
    assert isinstance(results, list)
    assert len(results) == 0


def test_identify_paths_mixed_statuses() -> None:
    m = Magika()

    with tempfile.TemporaryDirectory() as td:
        valid_file = Path(td) / "valid.txt"
        valid_file.write_text("valid content")

        non_existing = Path(td) / "does_not_exist.txt"

        unreadable_file = Path(td) / "unreadable.txt"
        unreadable_file.write_text("secret")
        unreadable_file.chmod(0o000)

        empty_file = Path(td) / "empty.txt"
        empty_file.write_text("")

        subdir = Path(td) / "subdir"
        subdir.mkdir()

        batch = [valid_file, non_existing, unreadable_file, empty_file, subdir]
        results = m.identify_paths(batch)

        assert len(results) == len(batch)

        # valid_file
        assert results[0].path == valid_file
        assert results[0].ok
        assert results[0].status == Status.OK
        assert results[0].output.label == ContentTypeLabel.TXT

        # non_existing
        assert results[1].path == non_existing
        assert not results[1].ok
        assert results[1].status == Status.FILE_NOT_FOUND_ERROR

        # unreadable_file
        assert results[2].path == unreadable_file
        assert not results[2].ok
        assert results[2].status == Status.PERMISSION_ERROR

        # empty_file
        assert results[3].path == empty_file
        assert results[3].ok
        assert results[3].output.label == ContentTypeLabel.EMPTY

        # subdir
        assert results[4].path == subdir
        assert results[4].ok
        assert results[4].output.label == ContentTypeLabel.DIRECTORY


def test_broken_symlink() -> None:
    with tempfile.TemporaryDirectory() as td:
        target = Path(td) / "non_existing_target.txt"
        symlink = Path(td) / "broken_link.txt"
        symlink.symlink_to(target)

        # Default no_dereference=False follows symlink -> target not found
        m_follow = Magika(no_dereference=False)
        res_follow = m_follow.identify_path(symlink)
        assert res_follow.path == symlink
        assert not res_follow.ok
        assert res_follow.status == Status.FILE_NOT_FOUND_ERROR

        # no_dereference=True identifies symlink directly -> SYMLINK, ok=True
        m_no_follow = Magika(no_dereference=True)
        res_no_follow = m_no_follow.identify_path(symlink)
        assert res_no_follow.path == symlink
        assert res_no_follow.ok
        assert res_no_follow.output.label == ContentTypeLabel.SYMLINK
        assert res_no_follow.dl.label == ContentTypeLabel.UNDEFINED
        assert res_no_follow.score == 1.0


def test_prediction_mode_via_identify_apis() -> None:
    test_path = utils.get_one_basic_test_file_path()

    for mode in (
        PredictionMode.BEST_GUESS,
        PredictionMode.MEDIUM_CONFIDENCE,
        PredictionMode.HIGH_CONFIDENCE,
    ):
        m = Magika(prediction_mode=mode)
        res_path = m.identify_path(test_path)
        assert res_path.ok
        assert isinstance(res_path.output.label, ContentTypeLabel)

        res_bytes = m.identify_bytes(b"import json\n")
        assert res_bytes.ok
        assert isinstance(res_bytes.output.label, ContentTypeLabel)


def test_special_device_file() -> None:
    null_dev = Path("/dev/null")
    if null_dev.exists() and not null_dev.is_file() and not null_dev.is_dir():
        m = Magika()
        res = m.identify_path(null_dev)
        assert res.ok
        assert res.output.label == ContentTypeLabel.UNKNOWN
        assert res.dl.label == ContentTypeLabel.UNDEFINED
        assert res.score == 1.0
