#!/usr/bin/env python
# Copyright 2023-2024 Google LLC
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

from __future__ import annotations

import re
import subprocess
from pathlib import Path


# TODO(https://github.com/PyO3/maturin/issues/2163): Remove this file when fixed.
def main() -> None:
    repo_root_dir = Path(__file__).parent.parent.parent
    python_root_dir = repo_root_dir / "python"
    rust_root_dir = repo_root_dir / "rust"
    # Compute paths to files we'll need to restore at the end of the build
    rust_main_rs_path = rust_root_dir / "cli" / "src" / "main.rs"
    rust_cli_cargo_toml_path = rust_root_dir / "cli" / "Cargo.toml"

    # get the rust version from Cargo.toml and patch main.rs
    rust_version = get_rust_version(rust_root_dir)
    patch_main_rs_with_version(rust_main_rs_path, rust_version)

    # get the python version from magika.__version__ and patch Cargo.toml
    python_version = get_python_version(python_root_dir)
    patch_cargo_toml_with_version(rust_cli_cargo_toml_path, python_version)

    # update Cargo.lock
    subprocess.run(["cargo", "check"], cwd=rust_root_dir / "cli", check=True)


def get_rust_version(rust_root_dir: Path) -> str:
    cargo_path = rust_root_dir / "cli" / "Cargo.toml"
    version = extract_with_regex(cargo_path, 'version = "([A-Za-z0-9.-]+)".*')
    print(f"Extracted rust version: {version}")
    return version


def get_python_version(python_root_dir: Path) -> str:
    init_path = python_root_dir / "src" / "magika" / "__init__.py"
    version = extract_with_regex(init_path, '__version__ = "([A-Za-z0-9.-]+)"')
    print(f"Extracted python version: {version}")
    return version


def patch_main_rs_with_version(rust_main_rs_path: Path, version: str) -> None:
    print(f'Patching {rust_main_rs_path} with rust version "{version}"')
    patch_line_matching_prefix(
        rust_main_rs_path,
        "        let binary = clap::crate_version!();",
        f'        let binary = "{version}";',
    )


def patch_cargo_toml_with_version(cargo_toml_path: Path, version: str) -> None:
    print(f'Patching {cargo_toml_path} with python version "{version}"')
    patch_line_matching_prefix(cargo_toml_path, "version = ", f'version = "{version}"')


def extract_with_regex(file_path: Path, regex: str) -> str:
    """Extract a string via regex. This raises an exception if no or more than
    one matches are found."""

    lines = file_path.read_text().split("\n")
    output = None
    for line in lines:
        m = re.fullmatch(regex, line)
        if m:
            if output is not None:
                raise Exception(
                    f'ERROR: Found more than one match for "{regex}" in {file_path}'
                )
            output = m.group(1)
    if output is None:
        raise Exception(f'No hits for "{regex}" in {file_path}')
    return output


def patch_line_matching_prefix(file_path: Path, prefix: str, new_line: str) -> None:
    """Patch a line starting with a given prefix with a new line. This raises an
    exception if no such line or more than one lines with a given prefix are
    found."""

    lines = file_path.read_text().split("\n")
    already_found = False
    for line_idx in range(len(lines)):
        line = lines[line_idx]
        if line.startswith(prefix):
            if already_found:
                raise Exception(
                    f'ERROR: Found more than one line with prefix "{prefix}" in "{file_path}"'
                )
            already_found = True
            lines[line_idx] = new_line

    if not already_found:
        raise Exception(
            f'ERROR: Did not find any line with prefix "{prefix}" in "{file_path}"'
        )
    file_path.write_text("\n".join(lines))


if __name__ == "__main__":
    main()
