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

from tests import utils


def test_magika_cli_with_mitra_test_files() -> None:
    test_files_paths = utils.get_mitra_test_files_paths()

    stdout, stderr = run_magika_python_cli(test_files_paths)
    utils.check_magika_cli_output_matches_expected_by_ext(
        test_files_paths, stdout, stderr
    )


def deprecated_test_magika_cli_with_really_many_files() -> None:
    test_file_path = utils.get_one_basic_test_file_path()

    for n in [10000]:
        test_files_paths = [test_file_path] * n
        stdout, stderr = run_magika_python_cli(test_files_paths)
        utils.check_magika_cli_output_matches_expected_by_ext(
            test_files_paths, stdout, stderr
        )


def deprecated_test_magika_cli_with_big_file() -> None:
    def signal_handler(signum: int, frame: Any) -> None:
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
            _ = run_magika_python_cli([sample_path])
            signal.alarm(0)
            print("Done running Magika")
