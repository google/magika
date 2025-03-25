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
from dataclasses import dataclass
from typing import List

from magika import Magika
from magika.seekable import Buffer
from magika.types import ModelFeatures
from tests.utils import get_features_extraction_tests_path


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


def test_features_extraction_v2(debug: bool = False) -> None:
    tests_cases = _get_tests_cases_from_reference()

    for test_case in tests_cases:
        test_info = TestInfo(**test_case["test_info"])
        test_content = base64.b64decode(test_case["content"])
        expected_features = ModelFeatures(**test_case["features_v2"])

        s = Buffer(test_content)
        features = Magika._extract_features_from_seekable(
            s,
            beg_size=test_info.beg_size,
            mid_size=test_info.mid_size,
            end_size=test_info.end_size,
            padding_token=test_info.padding_token,
            block_size=test_info.block_size,
            use_inputs_at_offsets=True,
        )

        with_error = False
        if features.beg != expected_features.beg:
            with_error = True
            if debug:
                print("beg does not match")
        if features.mid != expected_features.mid:
            with_error = True
            if debug:
                print("mid does not match")
        if features.end != expected_features.end:
            with_error = True
            if debug:
                print("end does not match")
        try:
            assert expected_features == features
        except AssertionError:
            with_error = True
            if debug:
                print("other fields do not match")

        if with_error:
            raise Exception


def _get_tests_cases_from_reference() -> List:
    ref_features_extraction_tests_path = get_features_extraction_tests_path()

    tests_cases = json.loads(
        gzip.decompress(ref_features_extraction_tests_path.read_bytes())
    )
    return tests_cases  # type: ignore[no-any-return]


if __name__ == "__main__":
    test_features_extraction_v2(debug=True)
