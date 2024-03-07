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

import base64
import gzip
import json
import math
import random
import string
import tempfile
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import List, Tuple

from magika import Magika
from magika.seekable import Buffer
from tests.utils import get_tests_data_dir

random.seed(42)


@dataclass
class TestInfo:
    beg_size: int
    mid_size: int
    end_size: int
    block_size: int
    padding_token: int
    core_content_size: int
    left_ws_num: int
    right_ws_num: int

    __test__ = False


def test_features_extraction(debug: bool = False) -> None:
    """This iterates over the content in the test suite and checks whether the
    trivial implementaion matches the python module one, which is the reference
    code."""

    beg_size = 512
    mid_size = 512
    end_size = 512
    block_size = 4096
    padding_token = 256

    features_size = beg_size
    assert mid_size == features_size
    assert end_size == features_size

    test_suite = get_features_extraction_test_suite(
        beg_size=beg_size,
        mid_size=mid_size,
        end_size=end_size,
        block_size=block_size,
        padding_token=padding_token,
    )

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


def test_features_extraction_v2(debug: bool = False) -> None:
    features_size = 256
    padding_token = 256
    block_size = 4096

    s = Buffer(b"test")
    f = Magika._extract_features_from_seekable_v2(
        s, features_size, features_size, features_size, padding_token, block_size
    )
    assert f.offset_0x8000_0x8007 == [padding_token] * 8
    assert f.offset_0x8800_0x8807 == [padding_token] * 8
    assert f.offset_0x9000_0x9007 == [padding_token] * 8
    assert f.offset_0x9800_0x9807 == [padding_token] * 8

    s = Buffer(b"x" * 0x8007)
    f = Magika._extract_features_from_seekable_v2(s, 512, 512, 512, 256, 4096)
    assert f.offset_0x8000_0x8007 == [padding_token] * 8
    assert f.offset_0x8800_0x8807 == [padding_token] * 8
    assert f.offset_0x9000_0x9007 == [padding_token] * 8
    assert f.offset_0x9800_0x9807 == [padding_token] * 8

    s = Buffer(b"x" * 0x8008)
    f = Magika._extract_features_from_seekable_v2(s, 512, 512, 512, 256, 4096)
    assert f.offset_0x8000_0x8007 == [ord("x")] * 8
    assert f.offset_0x8800_0x8807 == [padding_token] * 8
    assert f.offset_0x9000_0x9007 == [padding_token] * 8
    assert f.offset_0x9800_0x9807 == [padding_token] * 8

    s = Buffer(b"x" * 0x10000)
    f = Magika._extract_features_from_seekable_v2(s, 512, 512, 512, 256, 4096)
    assert f.offset_0x8000_0x8007 == [ord("x")] * 8
    assert f.offset_0x8800_0x8807 == [ord("x")] * 8
    assert f.offset_0x9000_0x9007 == [ord("x")] * 8
    assert f.offset_0x9800_0x9807 == [ord("x")] * 8


def get_features_extraction_test_suite(
    beg_size: int, mid_size: int, end_size: int, block_size: int, padding_token: int
) -> List[Tuple[TestInfo, bytes]]:
    # for now we only support tests with beg_size == mid_size == end_size
    features_size = beg_size
    assert mid_size == features_size
    assert end_size == features_size

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
                test_info = TestInfo(
                    beg_size=beg_size,
                    mid_size=mid_size,
                    end_size=end_size,
                    block_size=block_size,
                    padding_token=padding_token,
                    core_content_size=core_content_size,
                    left_ws_num=left_ws_num,
                    right_ws_num=right_ws_num,
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


def generate_features_extraction_reference():
    beg_size = 512
    mid_size = 512
    end_size = 512
    block_size = 1024
    padding_token = 256

    test_suite = get_features_extraction_test_suite(
        beg_size=beg_size,
        mid_size=mid_size,
        end_size=end_size,
        block_size=block_size,
        padding_token=padding_token,
    )

    ref_features_extraction_tests = []

    for test_info, test_content in test_suite:
        s = Buffer(test_content)
        features_v1 = Magika._extract_features_from_seekable(
            s, beg_size, mid_size, end_size, padding_token, block_size
        )
        features_v2 = Magika._extract_features_from_seekable_v2(
            s, beg_size, mid_size, end_size, padding_token, block_size
        )

        test_case = {
            "test_info": asdict(test_info),
            "content": base64.b64encode(test_content).decode("ascii"),
            "features_v1": asdict(features_v1),
            "features_v2": asdict(features_v2),
        }
        ref_features_extraction_tests.append(test_case)

    ref_features_extraction_tests_path = (
        get_tests_data_dir() / "features_extraction" / "reference.json.gz"
    )
    ref_features_extraction_tests_path.parent.mkdir(parents=True, exist_ok=True)
    ref_features_extraction_tests_path.write_bytes(
        gzip.compress(json.dumps(ref_features_extraction_tests).encode("ascii"))
    )


if __name__ == "__main__":
    test_features_extraction()
    test_features_extraction_v2()
