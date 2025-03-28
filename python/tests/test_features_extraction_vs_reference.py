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

import base64
import io
import json
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import List, Tuple

import click
import dacite
from tqdm import tqdm

from magika import Magika
from magika.types import ModelFeatures, Seekable

try:
    from tests import utils as test_utils
except ImportError:
    # Hack to support both `uv run pytest tests/` and `uv run ./tests/test_...
    # <command>`
    import sys

    sys.path.append(str(Path(__file__).parent.parent))
    from tests import utils as test_utils


@click.group()
def cli():
    pass


@cli.command()
@click.option("--debug/--no-debug", is_flag=True, default=True)
def run_tests(debug: bool) -> None:
    test_features_extraction_vs_reference(debug=debug)


@cli.command()
@click.option("--test-mode", is_flag=True)
def generate_tests(test_mode: bool) -> None:
    _generate_reference_features_extraction(test_mode=test_mode)


def test_features_extraction_vs_reference(debug: bool = False) -> None:
    examples = _get_examples_from_reference()
    if debug:
        print(f"Loaded {len(examples)} tests cases")

    for example in tqdm(examples, disable=not debug):
        example_content = base64.b64decode(example.content_base64)

        features = Magika._extract_features_from_seekable(
            Seekable(io.BytesIO(example_content)),
            beg_size=example.args.beg_size,
            mid_size=example.args.mid_size,
            end_size=example.args.end_size,
            padding_token=example.args.padding_token,
            block_size=example.args.block_size,
            use_inputs_at_offsets=example.args.use_inputs_at_offsets,
        )
        _check_features_vs_reference_example_features(
            features, example.features, debug=debug
        )


def test_reference_generation() -> None:
    _generate_reference_features_extraction(test_mode=True)


def _generate_reference_features_extraction(test_mode: bool) -> None:
    print("Genearting reference features extraction tests cases...")
    tests_cases = _generate_reference_features_extraction_tests_cases()
    print(f"Generated {len(tests_cases)} tests cases")
    _dump_reference_features_extraction_examples(tests_cases, test_mode=test_mode)


def _dump_reference_features_extraction_examples(
    examples: List[FeaturesExtractionExample],
    test_mode: bool,
) -> None:
    reference_features_extraction_examples_path = (
        test_utils.get_reference_features_extraction_examples_path()
    )

    if test_mode:
        print('WARNING: running in "test_mode", not writing examples to file')
    else:
        reference_features_extraction_examples_path.parent.mkdir(
            parents=True, exist_ok=True
        )
        reference_features_extraction_examples_path.write_bytes(
            test_utils.gzip_compress(
                json.dumps([asdict(example) for example in examples]).encode("ascii")
            )
        )
        print(f"Wrote tests cases to {reference_features_extraction_examples_path}")


def _generate_reference_features_extraction_tests_cases() -> (
    List[FeaturesExtractionExample]
):
    tests_cases_inputs: List[
        Tuple[FeaturesExtractionExampleArgs, FeaturesExtractionExampleMetadata, bytes]
    ] = _generate_reference_features_extraction_tests_cases_inputs()

    tests_cases = []
    for test_args, test_metadata, test_content in tests_cases_inputs:
        features = Magika._extract_features_from_seekable(
            Seekable(io.BytesIO(test_content)),
            test_args.beg_size,
            test_args.mid_size,
            test_args.end_size,
            test_args.padding_token,
            test_args.block_size,
            test_args.use_inputs_at_offsets,
        )

        example = FeaturesExtractionExample(
            args=test_args,
            metadata=test_metadata,
            content_base64=base64.b64encode(test_content).decode("ascii"),
            features=features,
        )

        tests_cases.append(example)

    return tests_cases


def _generate_reference_features_extraction_tests_cases_inputs() -> (
    List[Tuple[FeaturesExtractionExampleArgs, FeaturesExtractionExampleMetadata, bytes]]
):
    beg_size = 128
    mid_size = 0
    end_size = 64
    block_size = 512
    padding_token = 256
    use_inputs_at_offsets = False

    assert mid_size == 0
    assert use_inputs_at_offsets is False
    assert beg_size < block_size
    assert mid_size < block_size
    assert end_size < block_size

    ws_num_options = [
        0,
        1,
        10,
        beg_size - 1,
        beg_size,
        beg_size + 1,
        end_size - 1,
        end_size,
        end_size + 1,
        beg_size + end_size - 1,
        beg_size + end_size,
        beg_size + end_size + 1,
        block_size - 1,
        block_size,
        block_size + 1,
        2 * block_size - 1,
        2 * block_size,
        2 * block_size + 1,
        4 * block_size - 1,
        4 * block_size,
        4 * block_size + 1,
    ]

    content_size_options = list(ws_num_options)

    tests_cases_inputs = []
    for core_content_size in content_size_options:
        for left_ws_num in ws_num_options:
            for right_ws_num in ws_num_options:
                test_args = FeaturesExtractionExampleArgs(
                    beg_size=beg_size,
                    mid_size=mid_size,
                    end_size=end_size,
                    block_size=block_size,
                    padding_token=padding_token,
                    use_inputs_at_offsets=use_inputs_at_offsets,
                )
                test_metadata = FeaturesExtractionExampleMetadata(
                    core_content_size=core_content_size,
                    left_ws_num=left_ws_num,
                    right_ws_num=right_ws_num,
                )

                content = _generate_content_from_metadata(test_metadata)
                tests_cases_inputs.append((test_args, test_metadata, content))

    return tests_cases_inputs


def _generate_content_from_metadata(
    test_info: FeaturesExtractionExampleMetadata,
) -> bytes:
    """Generate content with a given "core size", with n left and right
    whitespaces, and the core content. with_ws_near_beg and with_ws_near_end
    specify if we need to put spaces near the beg/end, e.g., "A AAA". This is
    useful to test that we don't strip whitespaces that we are not supposed to
    strip."""

    content = bytearray(
        test_utils.generate_pattern(test_info.core_content_size, only_printable=True)
    )

    if test_info.core_content_size >= 5:
        # inject characters that other implementations may mistakenly strip
        content[0] = ord("\x00")
        content[1] = ord(" ")
        content[-2] = ord(" ")
        content[-1] = ord("\x00")

    return (
        test_utils.generate_whitespaces(test_info.left_ws_num)
        + bytes(content)
        + test_utils.generate_whitespaces(test_info.right_ws_num)
    )


def _get_examples_from_reference() -> List[FeaturesExtractionExample]:
    ref_features_extraction_examples_path = (
        test_utils.get_reference_features_extraction_examples_path()
    )

    return [
        dacite.from_dict(FeaturesExtractionExample, example)
        for example in json.loads(
            test_utils.gzip_decompress(
                ref_features_extraction_examples_path.read_bytes()
            )
        )
    ]


def _check_features_vs_reference_example_features(
    features: ModelFeatures, example_features: ModelFeatures, debug: bool = False
) -> None:
    with_error = False
    if features.beg != example_features.beg:
        with_error = True
        if debug:
            print("beg does not match")
    if features.mid != example_features.mid:
        with_error = True
        if debug:
            print("mid does not match")
    if features.end != example_features.end:
        with_error = True
        if debug:
            print("end does not match")
    try:
        assert features == example_features
    except AssertionError:
        with_error = True
        if debug:
            print("other fields do not match")

    if with_error:
        raise Exception


@dataclass
class FeaturesExtractionExample:
    """Data model for features_extraction_examples.json.gz."""

    args: FeaturesExtractionExampleArgs
    metadata: FeaturesExtractionExampleMetadata
    content_base64: str
    features: ModelFeatures


@dataclass
class FeaturesExtractionExampleArgs:
    beg_size: int
    mid_size: int
    end_size: int
    block_size: int
    padding_token: int
    use_inputs_at_offsets: bool


@dataclass
class FeaturesExtractionExampleMetadata:
    core_content_size: int
    left_ws_num: int
    right_ws_num: int


if __name__ == "__main__":
    cli()
