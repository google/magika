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
This script should only rely on dependencies installed with `pip install
magika`; this script is used as part of "build & test package" github action,
and the dev dependencies are not available.
"""

import sys
from pathlib import Path

import click

from magika import ContentTypeLabel, Magika, PredictionMode


@click.command()
def main() -> None:
    m = Magika(prediction_mode=PredictionMode.HIGH_CONFIDENCE)

    print(f"Magika instance details: {m}")

    res = m.identify_bytes(b"text")
    assert res.dl.label == ContentTypeLabel.UNDEFINED
    assert res.output.label == ContentTypeLabel.TXT
    assert res.score == 1.0

    res = m.identify_bytes(b"\xff\xff\xff")
    assert res.dl.label == ContentTypeLabel.UNDEFINED
    assert res.output.label == ContentTypeLabel.UNKNOWN
    assert res.score == 1.0

    basic_tests_dir = (
        Path(__file__).parent.parent.parent / "tests_data" / "basic"
    ).resolve()

    files_paths = sorted(filter(lambda p: p.is_file(), basic_tests_dir.rglob("*")))

    with_error = False
    for file_path in files_paths:
        res = m.identify_path(file_path)
        output_label = res.output.label
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
