import os
from pathlib import Path
import random
import string
from typing import List


tests_data_dir = Path(os.environ["MAGIKA_TESTS_DATA_DIR"])
assert tests_data_dir.is_dir()


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


def get_basic_tests_files_dir() -> Path:
    tests_files_dir = tests_data_dir / "basic"
    assert tests_files_dir.is_dir()
    return tests_files_dir


def get_basic_test_files_paths() -> List[Path]:
    tests_files_dir = get_basic_tests_files_dir()
    test_files_paths = sorted(tests_files_dir.iterdir())
    return test_files_paths


def get_one_basic_test_file_path() -> Path:
    return get_basic_test_files_paths()[0]


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
