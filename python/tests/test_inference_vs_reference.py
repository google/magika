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

from __future__ import annotations

import json
from dataclasses import asdict, dataclass
from typing import List, Optional

import click
import dacite
import pytest
from tqdm import tqdm

from magika import ContentTypeLabel, Magika
from magika.types.status import Status

try:
    from tests import utils as test_utils
except ImportError:
    # Hack to support both `uv run pytest tests/` and `uv run ./tests/test_...
    # <command>`
    import sys
    from pathlib import Path

    sys.path.append(str(Path(__file__).parent.parent))
    from tests import utils as test_utils


@click.group()
def cli():
    pass


@cli.command()
@click.option("--debug/--no-debug", is_flag=True, default=True)
def run_tests(debug: bool) -> None:
    test_inference_vs_reference(debug=debug)


@cli.command()
def generate_tests():
    _generate_reference_for_inference()


def test_inference_vs_reference(debug: bool = False) -> None:
    repo_root_dir = test_utils.get_repo_root_dir()

    m = Magika()
    model_name = m.get_model_name()

    examples_by_path = _get_examples_by_path(model_name)
    if debug:
        print(f"Loaded {len(examples_by_path)} examples by path")

    for example in tqdm(examples_by_path, disable=not debug):
        abs_path = repo_root_dir / example.path
        result = m.identify_path(abs_path)
        assert result.path == abs_path
        assert result.status == example.status
        if result.ok:
            assert result.prediction.dl.label == example.prediction.dl
            assert result.prediction.output.label == example.prediction.output
            assert result.prediction.score == pytest.approx(example.prediction.score)


def _get_examples_by_path(
    model_name: str,
) -> List[ExampleByPath]:
    reference_for_inference_examples_by_path = (
        test_utils.get_reference_for_inference_examples_by_path_path(model_name)
    )
    return [
        dacite.from_dict(
            ExampleByPath,
            entry,
            config=dacite.Config(cast=[Status, ContentTypeLabel]),
        )
        for entry in json.loads(reference_for_inference_examples_by_path.read_text())
    ]


def _generate_reference_for_inference():
    model_name = Magika._get_default_model_name()
    examples_by_path = _generate_examples_by_path(model_name)
    _dump_examples_by_path(model_name, examples_by_path)


def _generate_examples_by_path(
    model_name: str,
) -> List[ExampleByPath]:
    print(f'Generating examples by path for model "{model_name}"...')

    repo_root_dir = test_utils.get_repo_root_dir()

    m = Magika()
    assert m.get_model_name() == model_name

    tests_paths = test_utils.get_basic_test_files_paths()
    examples_by_path = []
    for test_path in tqdm(tests_paths):
        result = m.identify_path(test_path)
        if result.ok:
            example = ExampleByPath(
                path=str(test_path.resolve().relative_to(repo_root_dir)),
                status=result.status,
                prediction=Prediction(
                    dl=result.prediction.dl.label,
                    output=result.prediction.output.label,
                    score=result.prediction.score,
                ),
            )
        else:
            example = ExampleByPath(
                path=str(test_path),
                status=result.status,
                prediction=None,
            )
        examples_by_path.append(example)

    return examples_by_path


def _dump_examples_by_path(
    model_name: str,
    examples_by_path: List[ExampleByPath],
) -> None:
    examples_by_path_path = (
        test_utils.get_reference_for_inference_examples_by_path_path(model_name)
    )
    examples_by_path_path.parent.mkdir(parents=True, exist_ok=True)
    examples_by_path_path.write_text(
        json.dumps(
            [asdict(example) for example in examples_by_path],
            separators=(",", ":"),
        )
    )
    print(f"Wrote {len(examples_by_path)} examples by path to {examples_by_path_path}")


@dataclass
class ExampleByPath:
    path: str
    status: Status
    prediction: Optional[Prediction]


@dataclass
class Prediction:
    dl: ContentTypeLabel
    output: ContentTypeLabel
    score: float


if __name__ == "__main__":
    cli()
