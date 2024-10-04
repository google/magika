#!/usr/bin/env python3
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


"""
This script tests a given model against the tests cases, check whether the
predictions are correct, and raise an error otherwise.

This is useful when evaluating new models.
"""

import sys
from pathlib import Path

import click

import magika
from magika import colors


@click.command()
@click.argument("model_dir_or_name")
def main(model_dir_or_name: str) -> None:
    if Path(model_dir_or_name).is_dir():
        model_dir = Path(model_dir_or_name)
    else:
        models_dir = Path(magika.__file__).parent / "models"
        model_dir = models_dir / model_dir_or_name

    if not model_dir.is_dir():
        log_error(f"{model_dir_or_name} is not a dir nor a model name")
        sys.exit(1)

    m = magika.Magika(model_dir=model_dir)

    with_error = False

    tests_data_dir = Path(__file__).parent.parent.parent / "tests_data"
    tests_dirs_names = ["basic", "previous_missdetections"]

    for tests_dir_name in tests_dirs_names:
        tests_dir = tests_data_dir / tests_dir_name
        for test_path in tests_dir.rglob("*"):
            if not test_path.is_file():
                continue

            mr = m.identify_path(test_path)
            assert mr.ok

            prediceted_content_type = mr.prediction.output.label
            expected_content_type = test_path.parent.name
            if prediceted_content_type != expected_content_type:
                with_error = True
                log_error(
                    f'{test_path} predicted as "{prediceted_content_type}" (score: {mr.prediction.score:.4f}), expected "{expected_content_type}".'
                )

    if with_error:
        log_error("There was at least one error.")
    else:
        log_ok("All tests examples were predicted correctly.")


def log_ok(msg: str) -> None:
    print(f"{colors.GREEN}{msg}{colors.RESET}")


def log_error(msg: str) -> None:
    print(f"{colors.RED}ERROR: {msg}{colors.RESET}")


if __name__ == "__main__":
    main()
