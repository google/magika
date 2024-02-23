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

import json
import math
import random
import string
import tempfile
from dataclasses import dataclass
from pathlib import Path
from typing import List, Tuple

from magika import Magika

random.seed(42)


@dataclass
class TestInfo:
    core_content_size: int
    left_ws_num: int
    right_ws_num: int
    with_ws_near_beg: bool
    with_ws_near_end: bool
    start_with_null_byte: bool
    end_with_null_byte: bool

    __test__ = False


def test_features_extraction(debug: bool = False) -> None:
    """This iterates over the content in the test suite and checks whether the
    trivial implementaion matches the python module one, which is the reference
    code."""

    beg_size = 512
    mid_size = 512
    end_size = 512
    padding_token = 256
    block_size = 4096

    features_size = beg_size
    assert mid_size == features_size
    assert end_size == features_size

    test_suite = get_features_extraction_test_suite(features_size, block_size)

    for test_info, test_content in test_suite:
        if debug:
            print(f"Test details: {test_info} =>")

        features_from_bytes = Magika._extract_features_from_bytes(
            test_content, beg_size, mid_size, end_size, padding_token, block_size
        )
        with tempfile.TemporaryDirectory() as td:
            tf_path = Path(td) / "test.dat"
            tf_path.write_bytes(test_content)
            features_from_path = Magika._extract_features_from_path(
                tf_path, beg_size, mid_size, end_size, padding_token, block_size
            )

        comparison = {}
        comparison["beg"] = features_from_bytes.beg == features_from_path.beg
        comparison["mid"] = features_from_bytes.mid == features_from_path.mid
        comparison["end"] = features_from_bytes.end == features_from_path.end
        comparison["all"] = set(comparison.values()) == set([True])

        if debug:
            print("comparison: " + json.dumps(comparison))

        if comparison["all"] is False:
            raise


def get_features_extraction_test_suite(
    features_size: int, block_size: int
) -> List[Tuple[TestInfo, bytes]]:
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

    test_suite = []
    for core_content_size in content_size_options:
        for left_ws_num in ws_num_options:
            for right_ws_num in ws_num_options:
                for with_ws_near_beg in [False, True]:
                    for with_ws_near_end in [False, True]:
                        # check if we need to skip this combination
                        if with_ws_near_beg or with_ws_near_end:
                            if core_content_size < 3:
                                continue

                        test_info = TestInfo(
                            core_content_size=core_content_size,
                            left_ws_num=left_ws_num,
                            right_ws_num=right_ws_num,
                            with_ws_near_beg=with_ws_near_beg,
                            with_ws_near_end=with_ws_near_end,
                            start_with_null_byte=False,
                            end_with_null_byte=False,
                        )

                        content = _generate_content(test_info)
                        test_suite.append((test_info, content))

    # add another tests manually
    test_info = TestInfo(
        core_content_size=1000,
        left_ws_num=20,
        right_ws_num=20,
        with_ws_near_beg=True,
        with_ws_near_end=True,
        start_with_null_byte=True,
        end_with_null_byte=True,
    )
    content = _generate_content(test_info)
    test_suite.append((test_info, content))

    return test_suite


def _generate_content(test_info: TestInfo) -> bytes:
    """Generate content with a given "core size", with n left and right
    whitespaces, and the core content. with_ws_near_beg and with_ws_near_end
    specify if we need to put spaces near the beg/end, e.g., "A AAA". This is
    useful to test that we don't strip whitespaces that we are not supposed to
    strip."""

    if test_info.with_ws_near_beg or test_info.with_ws_near_end:
        assert test_info.core_content_size >= 3

    content = _generate_pattern(test_info.core_content_size)
    if test_info.with_ws_near_beg:
        content[1] = ord(" ")
    if test_info.with_ws_near_end:
        content[-2] = ord(" ")

    # This is useful to test that \0 is not stripped
    if test_info.start_with_null_byte:
        content = bytearray(b"\x00") + content
    if test_info.end_with_null_byte:
        content = content + bytearray(b"\x00")

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


if __name__ == "__main__":
    test_features_extraction(debug=False)
