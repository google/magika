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
import string
from pathlib import Path
from typing import List


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


def get_mitra_tests_files_dir() -> Path:
    tests_files_dir = get_tests_data_dir() / "mitra"
    assert tests_files_dir.is_dir()
    return tests_files_dir


def get_previously_missdetected_files_dir() -> Path:
    tests_files_dir = get_tests_data_dir() / "previous_missdetections"
    assert tests_files_dir.is_dir()
    return tests_files_dir


def get_basic_test_files_paths() -> List[Path]:
    tests_files_dir = get_basic_tests_files_dir()
    test_files_paths = sorted(filter(lambda p: p.is_file(), tests_files_dir.rglob("*")))
    return test_files_paths


def get_mitra_test_files_paths() -> List[Path]:
    tests_files_dir = get_mitra_tests_files_dir()
    test_files_paths = sorted(filter(lambda p: p.is_file(), tests_files_dir.rglob("*")))
    return test_files_paths


def get_previously_missdetected_files_paths() -> List[Path]:
    tests_files_dir = get_previously_missdetected_files_dir()
    test_files_paths = sorted(filter(lambda p: p.is_file(), tests_files_dir.rglob("*")))
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


def get_models_dir() -> Path:
    return Path(__file__).parent.parent / "src" / "magika" / "models"


def get_default_model_dir() -> Path:
    from magika.magika import Magika

    return get_models_dir() / Magika._get_default_model_name()
