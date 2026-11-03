# Copyright 2025 Google LLC
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     https://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.
# /// script
# dependencies = ["click"]
#
# [tool.uv]
# required-version = ">=0.13.4"
# ///

from __future__ import annotations

import re
import subprocess
import sys
from pathlib import Path

import click

REPO_ROOT_DIR = Path(__file__).resolve().parent.parent
assert (REPO_ROOT_DIR / ".git").exists()

EXTENSIONS = (".py", ".sh", ".ts", ".rs", ".go")
RELEVANT_DIRS = ("scripts/", "python/", "js/", "go/", "rust/")
EXCLUDE_DIRS = ("js/simple_examples",)
COPYRIGHT_PATTERN = re.compile(r"Copyright", re.IGNORECASE)


@click.command()
def main() -> None:
    """
    Check for missing copyright headers in the target files.

    The command performs the following steps:
    - Retrieves the list of git-tracked files meeting the criteria.
    - Checks each file for a valid copyright header.
    - Prints any files that are missing the header.
    - Exits with status code 1 if any file is missing the header; otherwise exits successfully.
    """

    error_files = []
    for file_path in get_relevant_files_paths():
        if not has_copyright(file_path):
            error_files.append(file_path)

    if error_files:
        click.secho("Missing copyright in:", fg="red", bold=True)
        for file in error_files:
            click.echo(f"- {file}")
        sys.exit(1)
    else:
        click.secho("All files have valid copyright.", fg="green")


def get_relevant_files_paths() -> list[Path]:
    """Finds relevant, non-ignored files in the repository using Git.

    Filters the output of `git ls-files` based on three criteria defined
    in global constants:
    - File must have an extension in `EXTENSIONS`.
    - File path must start with a prefix in `RELEVANT_DIRS`.
    - File path must NOT start with a prefix in `EXCLUDE_DIRS`.
    """

    paths = []
    try:
        result = subprocess.run(
            ["git", "ls-files", "--cached", "--others", "--exclude-standard"],
            capture_output=True,
            text=True,
            check=True,
            cwd=str(REPO_ROOT_DIR),
        )
        for rel_path_str in sorted(set(result.stdout.strip().splitlines())):
            path = REPO_ROOT_DIR / rel_path_str
            if (
                path.is_file()
                and path.stat().st_size > 0
                and path.suffix in EXTENSIONS
                and rel_path_str.startswith(RELEVANT_DIRS)
                and not rel_path_str.startswith(EXCLUDE_DIRS)
            ):
                paths.append(path)
    except subprocess.CalledProcessError as e:
        click.secho(f"Git command failed: {e}", fg="red", bold=True)
        sys.exit(2)

    return paths


def has_copyright(path: Path) -> bool:
    """Checks if a file contains a copyright notice within the first N lines.

    Returns True if found, False otherwise.
    """

    with path.open("r", encoding="utf-8") as f:
        for _ in range(5):
            line = f.readline()
            if COPYRIGHT_PATTERN.search(line):
                return True
    return False


if __name__ == "__main__":
    main()
