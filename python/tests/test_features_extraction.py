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
import gzip
import json
import math
import string
from dataclasses import asdict, dataclass
from typing import List, Tuple

import click

from magika import Magika
from magika.types import ModelFeatures

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
@click.option("--debug", is_flag=True)
def run_tests(debug: bool) -> None:
    test_features_extraction(debug=debug)


@cli.command()
def generate_tests():
    generate_reference_features_extraction()


def test_features_extraction(debug: bool = False) -> None:
    raw_tests_cases = _get_tests_cases_from_reference()

    for raw_test_case in raw_tests_cases:
        content = base64.b64decode(raw_test_case["content"])
        test_case = FeaturesExtractionTestCase(
            metadata=FeaturesExtractionTestCaseMetadata(**raw_test_case["metadata"]),
            content=content,
            features=ModelFeatures(**raw_test_case["features"]),
        )

        features = Magika._extract_features_from_bytes(
            test_case.content,
            beg_size=test_case.metadata.beg_size,
            mid_size=test_case.metadata.mid_size,
            end_size=test_case.metadata.end_size,
            padding_token=test_case.metadata.padding_token,
            block_size=test_case.metadata.block_size,
            use_inputs_at_offsets=test_case.metadata.use_inputs_at_offsets,
        )

        with_error = False
        if features.beg != test_case.features.beg:
            with_error = True
            if debug:
                print("beg does not match")
        if features.mid != test_case.features.mid:
            with_error = True
            if debug:
                print("mid does not match")
        if features.end != test_case.features.end:
            with_error = True
            if debug:
                print("end does not match")
        try:
            assert features == test_case.features
        except AssertionError:
            with_error = True
            if debug:
                print("other fields do not match")

        if with_error:
            raise Exception


def generate_reference_features_extraction():
    print("Genearting reference features extraction tests cases...")
    tests_cases = _generate_reference_features_extraction_tests_cases()
    print(f"Generated {len(tests_cases)} tests cases")
    _dump_reference_features_extraction(tests_cases)


def _dump_reference_features_extraction(
    tests_cases: List[FeaturesExtractionTestCase],
) -> None:
    raw_tests_cases = []
    for test_case in tests_cases:
        raw_test_case = asdict(test_case)
        raw_test_case["content"] = base64.b64encode(test_case.content).decode("ascii")
        raw_tests_cases.append(raw_test_case)

    reference_features_extraction_tests_path = (
        test_utils.get_reference_features_extraction_tests_path()
    )
    reference_features_extraction_tests_path.parent.mkdir(parents=True, exist_ok=True)
    reference_features_extraction_tests_path.write_bytes(
        gzip.compress(json.dumps(raw_tests_cases).encode("ascii"))
    )
    print(f"Wrote tests cases to {reference_features_extraction_tests_path}")


def _generate_reference_features_extraction_tests_cases() -> (
    List[FeaturesExtractionTestCase]
):
    test_suite: List[Tuple[FeaturesExtractionTestCaseMetadata, bytes]] = (
        _generate_reference_features_extraction_tests_cases_inputs()
    )

    tests_cases = []
    for test_info, test_content in test_suite:
        features = Magika._extract_features_from_bytes(
            test_content,
            test_info.beg_size,
            test_info.mid_size,
            test_info.end_size,
            test_info.padding_token,
            test_info.block_size,
            use_inputs_at_offsets=test_info.use_inputs_at_offsets,
        )

        test_case = FeaturesExtractionTestCase(
            metadata=test_info,
            content=test_content,
            features=features,
        )

        tests_cases.append(test_case)

    return tests_cases


def _generate_reference_features_extraction_tests_cases_inputs() -> (
    List[Tuple[FeaturesExtractionTestCaseMetadata, bytes]]
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

    features_size = beg_size

    ws_num_options = [
        0,
        1,
        10,
        features_size - 1,
        features_size,
        features_size + 1,
        block_size - 1,
        block_size,
        block_size + 1,
        2 * block_size - 1,
        2 * block_size,
        2 * block_size + 1,
        2 * block_size + features_size - 1,
        2 * block_size + features_size,
        2 * block_size + features_size + 1,
    ]

    content_size_options = list(ws_num_options)
    content_size_options.extend([10_000, 100_000])

    tests_cases_inputs = []
    for core_content_size in content_size_options:
        for left_ws_num in ws_num_options:
            for right_ws_num in ws_num_options:
                test_metadata = FeaturesExtractionTestCaseMetadata(
                    beg_size=beg_size,
                    mid_size=mid_size,
                    end_size=end_size,
                    block_size=block_size,
                    padding_token=padding_token,
                    use_inputs_at_offsets=use_inputs_at_offsets,
                    core_content_size=core_content_size,
                    left_ws_num=left_ws_num,
                    right_ws_num=right_ws_num,
                )

                content = _generate_content_from_metadata(test_metadata)
                tests_cases_inputs.append((test_metadata, content))

    return tests_cases_inputs


def _generate_content_from_metadata(
    test_info: FeaturesExtractionTestCaseMetadata,
) -> bytes:
    """Generate content with a given "core size", with n left and right
    whitespaces, and the core content. with_ws_near_beg and with_ws_near_end
    specify if we need to put spaces near the beg/end, e.g., "A AAA". This is
    useful to test that we don't strip whitespaces that we are not supposed to
    strip."""

    content = _generate_pattern(test_info.core_content_size)

    if test_info.core_content_size >= 5:
        # inject characters that other implementations may mistakenly strip
        content[0] = ord("\x00")
        content[1] = ord(" ")
        content[-2] = ord(" ")
        content[-1] = ord("\x00")

    return (
        _generate_whitespaces(test_info.left_ws_num)
        + bytes(content)
        + _generate_whitespaces(test_info.right_ws_num)
    )


def _generate_whitespaces(size: int) -> bytes:
    whitespaces = string.whitespace
    ws_len = len(whitespaces)
    return bytes([ord(whitespaces[idx % ws_len]) for idx in range(size)])


def _generate_pattern(size: int) -> bytearray:
    """Generate a pattern we can use to test features extraction"""

    chars = string.printable[: 10 + 26 * 2]

    base_pattern = bytearray(chars.encode("ascii"))
    base_pattern_len = len(base_pattern)
    pattern = (base_pattern * int(math.ceil(size / base_pattern_len)))[:size]
    return pattern


def _get_tests_cases_from_reference() -> List:
    ref_features_extraction_tests_path = (
        test_utils.get_reference_features_extraction_tests_path()
    )

    tests_cases = json.loads(
        gzip.decompress(ref_features_extraction_tests_path.read_bytes())
    )
    return tests_cases  # type: ignore[no-any-return]


@dataclass
class FeaturesExtractionTestCase:
    metadata: FeaturesExtractionTestCaseMetadata
    content: bytes
    features: ModelFeatures


@dataclass
class FeaturesExtractionTestCaseMetadata:
    beg_size: int
    mid_size: int
    end_size: int
    block_size: int
    padding_token: int
    use_inputs_at_offsets: bool
    core_content_size: int
    left_ws_num: int
    right_ws_num: int

    __test__ = False


if __name__ == "__main__":
    cli()
