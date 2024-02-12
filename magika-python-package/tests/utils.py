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

import base64
import json
import random
import string
from pathlib import Path
from typing import Dict, List, Tuple

from magika.content_types import ContentTypesManager


def get_tests_data_dir() -> Path:
    repo_root_dir = Path(__file__).parent.parent
    tests_data_dir = repo_root_dir / "tests_data"
    if tests_data_dir.is_dir():
        return tests_data_dir
    tests_data_dir = repo_root_dir.parent / "tests_data"
    assert tests_data_dir.is_dir()
    return tests_data_dir


def get_basic_tests_files_dir() -> Path:
    tests_files_dir = get_tests_data_dir() / "basic"
    assert tests_files_dir.is_dir()
    return tests_files_dir


def get_basic_test_files_paths() -> List[Path]:
    tests_files_dir = get_basic_tests_files_dir()
    test_files_paths = sorted(tests_files_dir.iterdir())
    return test_files_paths


def get_one_basic_test_file_path() -> Path:
    return get_basic_test_files_paths()[0]


def get_random_ascii_bytes(size: int) -> bytes:
    return bytes(
        [
            random.choice(bytes(string.ascii_letters.encode("ascii")))
            for _ in range(size)
        ]
    )


def get_lines_from_stream(stream: str) -> List[str]:
    candidates = stream.split("\n")
    lines = []
    for line in candidates:
        line = line.strip()
        if line == "":
            continue
        lines.append(line)
    return lines


def write_random_file_with_size(sample_path: Path, sample_size: int) -> None:
    print(f"Writing random file at {str(sample_path)} with size {sample_size}")
    assert not sample_path.is_file()
    block_size = 1024 * 1024 * 1024  # 1GB
    with open(sample_path, "wb") as f:
        if sample_size > block_size:
            for _ in range(sample_size // block_size):
                f.write(b"A" * block_size)
            if sample_size % block_size != 0:
                f.write(b"A" * (sample_size % block_size))
        else:
            f.write(b"A" * sample_size)
    print("Random file created")


def get_default_model_dir() -> Path:
    from magika.magika import Magika

    model_dir = (
        Path(__file__).parent.parent / "magika" / "models" / Magika.DEFAULT_MODEL_NAME
    )
    return model_dir


def check_magika_cli_output_matches_expected_by_ext(
    samples_paths: List[Path], stdout: str, stderr: str, **kwargs
):
    assert len(samples_paths) > 0
    json_output = kwargs.get("json_output", False)
    jsonl_output = kwargs.get("jsonl_output", False)
    mime_output = kwargs.get("mime_output", False)
    label_output = kwargs.get("label_output", False)
    compatibility_mode = kwargs.get("compatibility_mode", False)
    cpp_output = kwargs.get("cpp_output", False)
    ctm = ContentTypesManager()
    predicted_cts = get_magika_cli_output_from_stdout_stderr(stdout, stderr, **kwargs)
    assert len(predicted_cts) > 0
    assert len(samples_paths) == len(predicted_cts)
    remaining_samples_paths = samples_paths[:]
    for file_path, output in predicted_cts:
        remaining_samples_paths.remove(file_path)
        file_ext = file_path.suffix.lstrip(".")
        true_cts = ctm.get_cts_by_ext(file_ext)
        if len(true_cts) == 0:
            # We could not find the content type from the extension.  In this
            # case, we assume this is a test file path with the
            # <dataset>/<content type>/<hash> pattern
            true_ct_name = file_path.parent.name
            true_cts = [ctm.get_or_raise(true_ct_name)]
        assert len(true_cts) > 0

        true_cts_names = [ct.name for ct in true_cts]

        if json_output or jsonl_output:
            # check that each JSON entry satisfies the requirements
            assert isinstance(output, dict)
            dict_output: Dict = output  # type:ignore
            assert dict_output["output"]["ct_label"] in true_cts_names
        elif cpp_output:
            assert isinstance(output, str)
            str_output: str = output
            assert str_output.lower() in true_cts_names
        else:
            str_output: str = output  # type:ignore
            expected_outputs = []
            if mime_output:
                expected_outputs = [ctm.get_mime_type(ct.name) for ct in true_cts]
            elif label_output:
                expected_outputs = true_cts_names
            elif compatibility_mode:
                expected_outputs = [ctm.get_magic(ct.name) for ct in true_cts]
            else:
                expected_outputs = [
                    f"{ctm.get_description(ct.name)} ({ctm.get_group(ct.name)})"
                    for ct in true_cts
                ]
            assert str_output in expected_outputs

    # Check that all input samples have been scanned
    assert len(remaining_samples_paths) == 0


def get_magika_cli_output_from_stdout_stderr(
    stdout: str, stderr: str, **kwargs
) -> List[Tuple[Path, Dict | str]]:
    json_output = kwargs.get("json_output", False)
    jsonl_output = kwargs.get("jsonl_output", False)
    output_probability = kwargs.get("output_probability", False)
    generate_report = kwargs.get("generate_report", False)
    cpp_output = kwargs.get("cpp_output", False)
    """
    This function returns the output of magika for each input file. In case of
    JSON or JSONL, it returns the full information dictionary for
    each of them, not just the output content type label.
    """

    predicted_cts = []
    if json_output:
        # expect json
        entries = json.loads(stdout)
        for entry in entries:
            predicted_cts.append((Path(entry["path"]), entry))
    elif jsonl_output:
        # expect jsonl
        lines = get_lines_from_stream(stdout)
        for line in lines:
            entry = json.loads(line)
            predicted_cts.append((Path(entry["path"]), entry))
    elif cpp_output:
        # output from magika-cpp client
        lines = get_lines_from_stream(stdout)
        for line in lines:
            file_path_str, output = line.split(": ", 1)
            ct_output, score_str = output.split(" ")
            score_num = float(score_str)
            assert 0 <= score_num <= 1
            predicted_cts.append((Path(file_path_str), ct_output))
    else:
        # plain output
        lines = get_lines_from_stream(stdout)
        for line in lines:
            if output_probability:
                file_path_str, output = line.split(": ", 1)
                ct_output, score_str = output.rsplit(" ", 1)
                assert score_str.endswith("%")
                score_num_str = score_str[:-1]
                assert 0 <= int(score_num_str) <= 100
            else:
                file_path_str, ct_output = line.split(": ", 1)
            predicted_cts.append((Path(file_path_str), ct_output))

        # check that we output the expected warnings
        if generate_report:
            stderr_lines = get_lines_from_stream(stderr)
            assert len(stderr_lines) >= 1
            if generate_report:
                assert stderr_lines[0].startswith("#" * 10)
                assert stderr_lines[1].find("REPORT") >= 0
                assert stderr_lines[2].startswith("#" * 10)
                assert stderr_lines[-4].startswith("#" * 10)
                assert stderr_lines[-3].startswith("Please")
                assert stderr_lines[-2].startswith("Please")
                assert (
                    stderr_lines[-1].startswith("IMPORTANT")
                    and stderr_lines[-1].find("NOT") >= 0
                    and stderr_lines[-1].find("PII") >= 0
                )
                report_info = json.loads(stderr_lines[3])
                assert set(report_info.keys()) == {
                    "version",
                    "model_dir_name",
                    "python_version",
                    "entries",
                }
                decoded_entries = json.loads(base64.b64decode(report_info["entries"]))
                assert isinstance(decoded_entries, list)
                assert decoded_entries[0]["output"]["path"] == "<REMOVED>"
                assert isinstance(
                    decoded_entries[0]["output"]["output"]["ct_label"], str
                )

    return predicted_cts
