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

import base64
import hashlib
import re
import shutil
import subprocess
import sys
import tempfile
from dataclasses import dataclass
from pathlib import Path
from typing import Optional

import click
import requests


@click.command()
@click.option("--allow-git-dirty-state", is_flag=True)
def main(allow_git_dirty_state: bool) -> None:
    repo_root_dir = Path(__file__).parent.parent.parent
    python_root_dir = repo_root_dir / "python"
    dist_output_dir = python_root_dir / "dist"
    rust_root_dir = repo_root_dir / "rust"
    # Compute paths to files we'll need to restore at the end of the build
    rust_cli_readme_path = rust_root_dir / "cli" / "README.md"
    rust_main_rs_path = rust_root_dir / "cli" / "src" / "main.rs"
    rust_cli_cargo_toml_path = rust_root_dir / "cli" / "Cargo.toml"
    rust_cli_cargo_lock_path = rust_root_dir / "cli" / "Cargo.lock"

    # Check python README links
    python_readme_path = python_root_dir / "README.md"
    check_markdown_has_only_absolute_links(python_readme_path)

    r = subprocess.run(
        ["git", "status", "--porcelain"], capture_output=True, cwd=repo_root_dir
    )
    if not allow_git_dirty_state and (len(r.stdout) > 0 or len(r.stderr) > 0):
        print("ERROR: the git repository is in a dirty state.")
        sys.exit(1)

    # delete readme as it causes issues with sdist; we'll restore it later
    rust_cli_readme_path.unlink()

    try:
        # get the rust version from Cargo.toml and patch main.rs
        rust_version = get_rust_version(rust_root_dir)
        patch_main_rs_with_version(rust_main_rs_path, rust_version)

        # get the python version from magika.__version__ and patch Cargo.toml
        python_version = get_python_version(python_root_dir)
        patch_cargo_toml_with_version(rust_cli_cargo_toml_path, python_version)

        # update Cargo.lock
        r = subprocess.run(["cargo", "check"], cwd=rust_root_dir / "cli", check=True)

        # build the package
        r = subprocess.run(
            ["uv", "build", "--out-dir", str(dist_output_dir)],
            cwd=python_root_dir,
            check=True,
        )

        wheel_path = get_wheel_path(dist_output_dir)
        platform_tag = parse_wheel_path(wheel_path).platform_tag

        if platform_tag.startswith("linux_"):
            # need to check dependencies and patch for manylinux
            magika_bin_path = rust_root_dir / "target" / "release" / "magika"
            assert magika_bin_path.is_file()
            check_bin_dynamic_libraries_are_allowed(magika_bin_path)
            patch_linux_wheel_for_manylinux(wheel_path)

        print(f"Python packages built at {dist_output_dir}")
    finally:
        # restore what we modified, no matter what happened
        r = subprocess.run(
            [
                "git",
                "restore",
                "--",
                str(rust_cli_readme_path),
                str(rust_main_rs_path),
                str(rust_cli_cargo_toml_path),
                str(rust_cli_cargo_lock_path),
            ],
            cwd=repo_root_dir,
            capture_output=True,
            check=True,
        )


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


def check_bin_dynamic_libraries_are_allowed(magika_bin_path) -> None:
    """Raises an exception if the magika rust binary depends on a non-allowed dynamic library.

    See https://peps.python.org/pep-0513/ for more details."""

    print(f"Checking dynamic dependencies of {magika_bin_path}")

    dynamic_dependencies = set()
    r = subprocess.run(
        ["ldd", str(magika_bin_path)], check=True, capture_output=True, text=True
    )
    lines = r.stdout.split("\n")[1:]
    for line in lines:
        line = line.strip()
        if line == "":
            continue
        if line.find(" => ") == -1:
            continue
        lib_name = Path(line.split(" => ")[0]).name
        dynamic_dependencies.add(lib_name)

    expected_dependencies = set(
        [
            "libstdc++.so.6",
            "libgcc_s.so.1",
            "libm.so.6",
            "libc.so.6",
        ]
    )

    if dynamic_dependencies != expected_dependencies:
        print(
            f"ERROR: magika rust bin has unexpected dynamic dependencies: {dynamic_dependencies=} {expected_dependencies=}"
        )
        sys.exit(1)


def get_wheel_path(dist_output_dir: Path) -> Path:
    paths = list(dist_output_dir.rglob("*.whl"))
    assert len(paths) == 1
    return paths[0]


def parse_wheel_path(wheel_path: Path) -> WheelInfo:
    # Format: {distribution}-{version}(-{build tag})?-{python tag}-{abi tag}-{platform tag}.whl
    # https://peps.python.org/pep-0427/
    parts = wheel_path.stem.split("-")
    if len(parts) == 6:
        # path has the build_tag
        distribution, version, build_tag, python_tag, abi_tag, platform_tag = parts
    else:
        distribution, version, python_tag, abi_tag, platform_tag = parts
        build_tag = None
    return WheelInfo(
        distribution=distribution,
        version=version,
        build_tag=build_tag,
        python_tag=python_tag,
        abi_tag=abi_tag,
        platform_tag=platform_tag,
    )


def patch_linux_wheel_for_manylinux(wheel_path: Path) -> None:
    """Patch the linux wheel to be compatible with manylinux. This is sort-of a
    hack, but it should work for now.

    Steps:
    - unzip the wheel in a temporary dir
    - patch the magika-<version>.dist-info/WHEEL file
        Tag: py3-none-linux_x86_64 => py3-none-manylinux1_x86_64
    - re-generate RECORD file with the updated hashes and sizes
    - zip it again
    - update the wheel file name
    """

    print(f"Patching {wheel_path} to be compatible with manylinux")
    wheel_info = parse_wheel_path(wheel_path)

    with tempfile.TemporaryDirectory() as td_name:
        unpacked_wheel_dir = Path(td_name)

        # Use unzip vs zipfile python library because the latter does not
        # preserve permissions. See
        # https://github.com/python/cpython/issues/59999
        subprocess.run(
            ["unzip", "-o", "-d", str(unpacked_wheel_dir), str(wheel_path)],
            check=True,
            capture_output=True,
        )
        print(f"Unpacked wheel in {unpacked_wheel_dir}")

        patch_wheel_metadata(unpacked_wheel_dir, wheel_info)
        regenerate_wheel_record(unpacked_wheel_dir, wheel_info)

        wheel_info.platform_tag = wheel_info.platform_tag.replace("linux", "manylinux1")
        new_wheel_path = wheel_path.parent / wheel_info.to_filename()

        shutil.make_archive(str(new_wheel_path), "zip", unpacked_wheel_dir)
        # make_archive appends .zip to the file name, let's remove it
        shutil.move(f"{new_wheel_path}.zip", new_wheel_path)
        print(f"Created new wheel: {new_wheel_path}")

        # remove the old wheel
        wheel_path.unlink()


def patch_wheel_metadata(unpacked_wheel_dir: Path, wheel_info: WheelInfo) -> None:
    # Patch "Tag: XX-YY-linux_ZZ": linux => manylinux1
    print("Patch WHEEL file with proper tag")

    wheel_metadata_path = (
        unpacked_wheel_dir / f"magika-{wheel_info.version}.dist-info" / "WHEEL"
    )
    new_lines = []
    for line in wheel_metadata_path.read_text().split("\n"):
        if line.startswith("Tag: "):
            new_line = line.replace("linux", "manylinux1")
        else:
            new_line = line
        new_lines.append(new_line)
    wheel_metadata_path.write_text("\n".join(new_lines))


def regenerate_wheel_record(unpacked_wheel_dir: Path, wheel_info: WheelInfo) -> None:
    print("Regenerate RECORD with updated WHEEL file")

    record_path = (
        unpacked_wheel_dir / f"magika-{wheel_info.version}.dist-info" / "RECORD"
    )
    entries = []
    for path in unpacked_wheel_dir.rglob("*"):
        if path.is_file():
            entries.append(generate_wheel_record_entry(unpacked_wheel_dir, path))
    record_entry = ",".join([str(record_path.relative_to(unpacked_wheel_dir)), "", ""])
    entries.append(record_entry)

    new_record_content = "\n".join(entries) + "\n"
    record_path.write_text(new_record_content)


def generate_wheel_record_entry(unpacked_wheel_dir: Path, path: Path) -> str:
    content = path.read_bytes()
    parts = [
        str(path.relative_to(unpacked_wheel_dir)),
        base64.b64encode(hashlib.sha256(content).digest(), altchars=b"-_")
        .rstrip(b"=")
        .decode("ascii"),
        str(len(content)),
    ]
    return ",".join(parts)


@dataclass(kw_only=True)
class WheelInfo:
    distribution: str
    version: str
    build_tag: Optional[str]
    python_tag: str
    abi_tag: str
    platform_tag: str

    def to_filename(self) -> str:
        # Format: {distribution}-{version}(-{build tag})?-{python tag}-{abi tag}-{platform tag}.whl
        parts = [
            self.distribution,
            self.version,
        ]
        if self.build_tag is not None:
            parts.append(self.build_tag)
        parts.extend([self.python_tag, self.abi_tag, self.platform_tag])
        return "-".join(parts) + ".whl"


def check_markdown_has_only_absolute_links(markdown_path: Path) -> None:
    """Check if a Markdown file contains only valid absolute links. Exits with code 1 if any issues are found."""

    if not markdown_path.is_file():
        print(f"ERROR: The path {markdown_path} is not a valid file.")
        sys.exit(1)

    markdown_content = markdown_path.read_text()

    # Find all markdown links
    link_regex = r"\[.*?\]\((.*?)\)"
    links = re.findall(link_regex, markdown_content)

    invalid_links = []
    for link in links:
        if link.startswith("https://"):
            # Check if the link is valid
            try:
                response = requests.head(link, allow_redirects=True, timeout=5)
                if response.status_code != 200:
                    invalid_links.append(
                        f"Invalid link: {link} (status code: {response.status_code})"
                    )
            except requests.RequestException as e:
                invalid_links.append(f"Error accessing link: {link} ({e})")
        else:
            invalid_links.append(f"Relative link found: {link}")

    if invalid_links:
        print(f"Issues found in {markdown_path}:")
        for error in invalid_links:
            print(error)
        sys.exit(1)

    print("Markdown link check complete.")


if __name__ == "__main__":
    # main()
    check_markdown_has_only_absolute_links(Path(__file__).parent.parent.parent / "python" / "README.md")
