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
from pathlib import Path

import click


@click.command()
def main() -> None:
    tests_data_dir = (Path(__file__).parent.parent.parent / "tests_data").resolve()
    basic_tests_dir = tests_data_dir / "basic"

    subprocess.run(["magika", "-r", str(basic_tests_dir)], check=True)

    # TODO: add the actual tests


if __name__ == "__main__":
    main()
