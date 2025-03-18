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

"""
This script assumes that the `magika` python package has been already installed,
and that the `magika` client is available in the PATH. The script tests that the
`magika` client appears functional.

This script should only rely on dependencies installed with `pip install
magika`; this script is used as part of "build & test package" github action,
and the dev dependencies are not available.
"""

import subprocess
import sys
from pathlib import Path
from typing import Optional

import click


@click.command()
@click.option(
    "--client-path",
    type=click.Path(exists=False, resolve_path=False),
)
def main(client_path: Optional[Path]) -> None:
    """Tests the Rust or Python Magika client. By default, it runs "magika"
    (expected in PATH). Use --client-path to specify a different client
    executable. Note that "client_path" may not point to a valid file, but still
    be a valid target to run as it may be found in the PATH.
    """

    basic_tests_dir = (
        Path(__file__).resolve().parent.parent.parent / "tests_data" / "basic"
    )
    assert basic_tests_dir.is_dir()

    if client_path is None:
        client_path = Path("magika")
    print(f'Testing client: "{client_path}"')

    p = subprocess.run([str(client_path), "--version"], capture_output=True, text=True)
    print(f'Output of "magika --version": {p.stdout.strip()}')

    p = subprocess.run(
        [str(client_path), "-r", "--label", "--no-colors", str(basic_tests_dir)],
        capture_output=True,
        text=True,
    )

    if p.returncode != 0:
        print("ERROR: magika CLI exited with non-zero status.")
        print(f"stdout:\n{p.stdout}\n" + "-" * 40)
        print(f"stderr:\n{p.stderr}\n" + "-" * 40)
        sys.exit(1)

    if p.stderr != "":
        print(f"WARNING: p.stderr not empty: {p.stderr}")

    with_error = False
    lines = p.stdout.split("\n")
    for line in lines:
        line = line.strip()
        if line == "":
            continue
        file_path_str, file_output_str = line.split(": ", 1)
        file_path = Path(file_path_str)
        output_label = file_output_str.strip().split(" ", 1)[0]
        expected_label = file_path.parent.name
        if expected_label != output_label:
            with_error = True
            print(
                f"ERROR: Misprediction for {file_path}: expected_label={expected_label}, output_label={output_label}"
            )

    if with_error:
        print("ERROR: There was at least one misprediction")
        sys.exit(1)

    print("All examples were predicted correctly")


if __name__ == "__main__":
    main()
