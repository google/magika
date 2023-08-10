import os
from pathlib import Path
import subprocess
import json
from typing import List, Dict, Optional, Any, Tuple
import tempfile
import signal

import pytest

from magika.content_types import ContentTypesManager, ContentType

from tests import utils


@pytest.mark.smoketest
def test_magika_cli_with_one_test_file():
    test_file_path = utils.get_basic_test_files_paths()[0]

    stdout, stderr = run_magika_cli([test_file_path])
    check_magika_cli_output_matches_expected([test_file_path], stdout, stderr)

    stdout, stderr = run_magika_cli([test_file_path], extra_cli_options=["--json"])
    check_magika_cli_output_matches_expected(
        [test_file_path], stdout, stderr, json_output_flag=True
    )

    stdout, stderr = run_magika_cli([test_file_path], extra_cli_options=["--jsonl"])
    check_magika_cli_output_matches_expected(
        [test_file_path], stdout, stderr, jsonl_output_flag=True
    )

    stdout, stderr = run_magika_cli([test_file_path], extra_cli_options=["-p"])
    check_magika_cli_output_matches_expected(
        [test_file_path], stdout, stderr, with_probability_flag=True
    )

    stdout, stderr = run_magika_cli([test_file_path], extra_cli_options=["-i"])
    check_magika_cli_output_matches_expected(
        [test_file_path], stdout, stderr, mime_output_flag=True
    )


def test_magika_cli_with_basic_test_files():
    test_files_paths = utils.get_basic_test_files_paths()

    for n in [1, 2, 5, 10, len(test_files_paths)]:
        stdout, stderr = run_magika_cli(test_files_paths[:n])
        check_magika_cli_output_matches_expected(test_files_paths[:n], stdout, stderr)


def test_magika_cli_with_basic_test_files_and_json_output():
    test_files_paths = utils.get_basic_test_files_paths()

    for n in [1, 2, 5, 10, len(test_files_paths)]:
        stdout, stderr = run_magika_cli(test_files_paths[:n], json_output_flag=True)
        check_magika_cli_output_matches_expected(
            test_files_paths[:n], stdout, stderr, json_output_flag=True
        )

        stdout, stderr = run_magika_cli(
            test_files_paths[:n], extra_cli_options=["--json"]
        )
        check_magika_cli_output_matches_expected(
            test_files_paths[:n], stdout, stderr, json_output_flag=True
        )


def test_magika_cli_with_basic_test_files_and_jsonl_output():
    test_files_paths = utils.get_basic_test_files_paths()

    for n in [1, 2, 5, 10, len(test_files_paths)]:
        stdout, stderr = run_magika_cli(test_files_paths[:n], jsonl_output_flag=True)
        check_magika_cli_output_matches_expected(
            test_files_paths[:n], stdout, stderr, jsonl_output_flag=True
        )

        stdout, stderr = run_magika_cli(
            test_files_paths[:n], extra_cli_options=["--jsonl"]
        )
        check_magika_cli_output_matches_expected(
            test_files_paths[:n], stdout, stderr, jsonl_output_flag=True
        )


def test_magika_cli_with_basic_test_files_and_probability():
    test_files_paths = utils.get_basic_test_files_paths()

    for n in [1, 2, 5, 10, len(test_files_paths)]:
        stdout, stderr = run_magika_cli(
            test_files_paths[:n], with_probability_flag=True
        )
        check_magika_cli_output_matches_expected(
            test_files_paths[:n], stdout, stderr, with_probability_flag=True
        )

        stdout, stderr = run_magika_cli(test_files_paths[:n], extra_cli_options=["-p"])
        check_magika_cli_output_matches_expected(
            test_files_paths[:n], stdout, stderr, with_probability_flag=True
        )

        stdout, stderr = run_magika_cli(
            test_files_paths[:n], extra_cli_options=["--with-probability"]
        )
        check_magika_cli_output_matches_expected(
            test_files_paths[:n], stdout, stderr, with_probability_flag=True
        )


def test_magika_cli_with_basic_test_files_and_mime_output():
    test_files_paths = utils.get_basic_test_files_paths()

    for n in [1, 2, 5, 10, len(test_files_paths)]:
        stdout, stderr = run_magika_cli(test_files_paths[:n], mime_output_flag=True)
        check_magika_cli_output_matches_expected(
            test_files_paths[:n], stdout, stderr, mime_output_flag=True
        )

        stdout, stderr = run_magika_cli(test_files_paths[:n], extra_cli_options=["-i"])
        check_magika_cli_output_matches_expected(
            test_files_paths[:n], stdout, stderr, mime_output_flag=True
        )

        stdout, stderr = run_magika_cli(
            test_files_paths[:n], extra_cli_options=["--mime"]
        )
        check_magika_cli_output_matches_expected(
            test_files_paths[:n], stdout, stderr, mime_output_flag=True
        )


def test_magika_cli_with_basic_test_files_and_compatibility_mode():
    test_files_paths = utils.get_basic_test_files_paths()

    for n in [1, 2, 5, 10, len(test_files_paths)]:
        stdout, stderr = run_magika_cli(
            test_files_paths[:n], compatibility_mode_flag=True
        )
        check_magika_cli_output_matches_expected(
            test_files_paths[:n], stdout, stderr, compatibility_mode_flag=True
        )

        stdout, stderr = run_magika_cli(test_files_paths[:n], extra_cli_options=["-c"])
        check_magika_cli_output_matches_expected(
            test_files_paths[:n], stdout, stderr, compatibility_mode_flag=True
        )

        stdout, stderr = run_magika_cli(
            test_files_paths[:n], extra_cli_options=["--compatibility-mode"]
        )
        check_magika_cli_output_matches_expected(
            test_files_paths[:n], stdout, stderr, compatibility_mode_flag=True
        )


def check_magika_cli_output_matches_expected(
    samples_paths: List[Path], stdout: str, stderr: str, **kwargs: Dict[str, Any]
):
    assert len(samples_paths) > 0
    mime_output_flag = kwargs.get("mime_output_flag", False)
    compatibility_mode_flag = kwargs.get("compatibility_mode_flag", False)
    ctm = ContentTypesManager()
    predicted_cts = get_magika_cli_output_from_stdout_stderr(
        samples_paths, stdout, stderr, **kwargs
    )
    assert len(predicted_cts) > 0
    assert len(samples_paths) == len(predicted_cts)
    for file_path, output in predicted_cts:
        file_ext = file_path.suffix.lstrip(".")
        if mime_output_flag:
            expected_output = ctm.get_mime_type(
                ctm.get_ct_by_ext_or_raise(file_ext).name
            )
        elif compatibility_mode_flag:
            expected_output = ctm.get_magic(ctm.get_ct_by_ext_or_raise(file_ext).name)
        else:
            ct_group = ctm.get_group(ctm.get_ct_by_ext_or_raise(file_ext).name)
            ct_label = ctm.get_ct_by_ext_or_raise(file_ext).name
            expected_output = f"{ct_group}::{ct_label}"
        assert output == expected_output


def test_magika_cli_with_multiple_copies_of_the_same_file():
    max_repetitions_num = 10
    test_file_path = utils.get_one_basic_test_file_path()
    test_files_paths = [test_file_path] * max_repetitions_num

    for n in [2, max_repetitions_num]:
        stdout, stderr = run_magika_cli(test_files_paths[:n])
        check_magika_cli_output_matches_expected(test_files_paths[:n], stdout, stderr)

        stdout, stderr = run_magika_cli(test_files_paths[:n], json_output_flag=True)
        check_magika_cli_output_matches_expected(
            test_files_paths[:n], stdout, stderr, json_output_flag=True
        )

        stdout, stderr = run_magika_cli(test_files_paths[:n], jsonl_output_flag=True)
        check_magika_cli_output_matches_expected(
            test_files_paths[:n], stdout, stderr, jsonl_output_flag=True
        )


def test_magika_cli_with_many_files():
    test_file_path = utils.get_one_basic_test_file_path()

    for n in [100, 1000]:
        test_files_paths = [test_file_path] * n
        stdout, stderr = run_magika_cli(test_files_paths)
        check_magika_cli_output_matches_expected(test_files_paths, stdout, stderr)


@pytest.mark.slow
def test_magika_cli_with_really_many_files():
    test_file_path = utils.get_one_basic_test_file_path()

    for n in [10000]:
        test_files_paths = [test_file_path] * n
        stdout, stderr = run_magika_cli(test_files_paths)
        check_magika_cli_output_matches_expected(test_files_paths, stdout, stderr)


@pytest.mark.slow
def test_magika_cli_with_big_file():
    def signal_handler(signum, frame):
        raise Exception("Timeout")

    signal.signal(signal.SIGALRM, signal_handler)

    # It should take much less than this, but pytest weird scheduling sometimes
    # creates unexpected slow downs.
    timeout = 2

    for sample_size in [1000, 10000, 1_000_000, 1_000_000_000, 10_000_000_000]:
        with tempfile.TemporaryDirectory() as td:
            sample_path = Path(td) / "sample.dat"
            utils.write_random_file_with_size(sample_path, sample_size)
            print(f"Starting running Magika with a timeout of {timeout}")
            signal.alarm(timeout)
            _ = run_magika_cli([sample_path])
            signal.alarm(0)
            print("Done running Magika")


def test_magika_cli_with_bad_input():
    test_file_path = utils.get_one_basic_test_file_path()

    with pytest.raises(subprocess.CalledProcessError):
        run_magika_cli([])

    with pytest.raises(subprocess.CalledProcessError):
        run_magika_cli([Path("/this/does/not/exist")])

    with pytest.raises(subprocess.CalledProcessError):
        run_magika_cli([test_file_path], json_output_flag=True, jsonl_output_flag=True)

    with pytest.raises(subprocess.CalledProcessError):
        run_magika_cli([test_file_path], extra_cli_options=["--non-existing-option"])


def test_magika_cli_with_colors():
    test_file_path = utils.get_one_basic_test_file_path()

    # check that it does not crash when using colors and that we are actually
    # using colors
    stdout, stderr = run_magika_cli([test_file_path], with_colors_flag=True)
    assert stdout.find("\033") >= 0 or stderr.find("\033") >= 0
    stdout, stderr = run_magika_cli(
        [test_file_path], with_colors_flag=True, mime_output_flag=True
    )
    assert stdout.find("\033") >= 0 or stderr.find("\033") >= 0
    stdout, stderr = run_magika_cli(
        [test_file_path], with_colors_flag=True, verbose_flag=True, debug_flag=True
    )
    assert stdout.find("\033") >= 0 or stderr.find("\033") >= 0


def test_magika_cli_with_no_colors():
    test_file_path = utils.get_one_basic_test_file_path()

    # check that we are not using colors when --no-colors is passed
    stdout, stderr = run_magika_cli([test_file_path], with_colors_flag=False)
    assert stdout.find("\033") == -1 and stderr.find("\033") == -1
    stdout, stderr = run_magika_cli(
        [test_file_path], with_colors_flag=False, mime_output_flag=True
    )
    assert stdout.find("\033") == -1 and stderr.find("\033") == -1
    stdout, stderr = run_magika_cli(
        [test_file_path], with_colors_flag=False, verbose_flag=True, debug_flag=True
    )
    assert stdout.find("\033") == -1 and stderr.find("\033") == -1


def test_magika_cli_print_version():
    stdout, stderr = run_magika_cli([], extra_cli_options=["--version"])

    lines = utils.get_lines_from_stream(stdout)
    assert len(lines) == 2
    assert lines[0].startswith("Magika version")
    assert lines[1].startswith("Model hash")

    assert stderr == ""


def test_magika_cli_list_content_types():
    test_file_path = utils.get_one_basic_test_file_path()

    stdout, stderr = run_magika_cli([], list_flag=True)

    lines = utils.get_lines_from_stream(stdout)
    header = lines[0]
    assert header.find("Content Type Label") >= 0
    assert header.find("MIME Type") >= 0
    assert header.find("Description") >= 0
    assert stderr == ""

    with pytest.raises(subprocess.CalledProcessError):
        run_magika_cli([test_file_path], list_flag=True)


def get_magika_cli_output_from_stdout_stderr(
    samples_paths: List[Path], stdout: str, stderr: str, **kwargs: Dict[str, Any]
) -> List:
    # This expects that magika is run with --no-colors

    json_output_flag = kwargs.get("json_output_flag", False)
    jsonl_output_flag = kwargs.get("jsonl_output_flag", False)
    compatibility_mode_flag = kwargs.get("compatibility_mode_flag", False)
    with_probability_flag = kwargs.get("with_probability_flag", False)

    predicted_cts = []
    if json_output_flag:
        # expect json
        entries = json.loads(stdout)
        for entry in entries:
            ct_group = entry["output"]["group"]
            ct_label = entry["output"]["ct_label"]
            predicted_cts.append((Path(entry["path"]), f"{ct_group}::{ct_label}"))
    elif jsonl_output_flag:
        # expect jsonl
        lines = utils.get_lines_from_stream(stdout)
        for line in lines:
            entry = json.loads(line)
            ct_group = entry["output"]["group"]
            ct_label = entry["output"]["ct_label"]
            predicted_cts.append((Path(entry["path"]), f"{ct_group}::{ct_label}"))
    else:
        # plain output
        lines = utils.get_lines_from_stream(stdout)
        for line in lines:
            if compatibility_mode_flag:
                if line.startswith("WARNING"):
                    # warning about lack of current support for compatibility mode
                    continue
            if with_probability_flag:
                file_path_str, output = line.split(": ", 1)
                ct, score_str = output.split(" ")
                assert score_str.endswith("%")
                score_num_str = score_str[:-1]
                assert 0 <= int(score_num_str) <= 100
            else:
                file_path_str, ct = line.split(": ", 1)
            predicted_cts.append((Path(file_path_str), ct))
    return predicted_cts


def run_magika_cli(
    samples_paths: List[Path],
    model_dir: Optional[Path] = None,
    threshold: Optional[float] = None,
    json_output_flag: bool = False,
    jsonl_output_flag: bool = False,
    mime_output_flag: bool = False,
    with_probability_flag: bool = False,
    compatibility_mode_flag: bool = False,
    recursive_flag: bool = False,
    with_colors_flag: bool = False,
    verbose_flag: bool = False,
    debug_flag: bool = False,
    list_flag: bool = False,
    extra_cli_options: Optional[List[str]] = None,
) -> Tuple[str, str]:
    cmd = [
        "magika",
    ]
    cmd.extend(map(str, samples_paths))
    if model_dir is not None:
        cmd.append(str(model_dir))
    if threshold is not None:
        cmd.append(str(threshold))
    if json_output_flag is True:
        cmd.append("--json")
    if jsonl_output_flag is True:
        cmd.append("--jsonl")
    if mime_output_flag is True:
        cmd.append("--mime")
    if with_probability_flag is True:
        cmd.append("--with-probability")
    if compatibility_mode_flag is True:
        cmd.append("--compatibility-mode")
    if recursive_flag is True:
        cmd.append("--recursive")
    if with_colors_flag:
        cmd.append("--colors")
    else:
        cmd.append("--no-colors")
    if verbose_flag is True:
        cmd.append("--verbose")
    if debug_flag is True:
        cmd.append("--debug")
    if list_flag is True:
        cmd.append("--list")
    if extra_cli_options is not None:
        cmd.extend(extra_cli_options)

    p = subprocess.run(cmd, capture_output=True, text=True, check=True)
    return p.stdout, p.stderr
