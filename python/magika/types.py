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

from dataclasses import dataclass
from typing import List, Optional


@dataclass
class ModelFeatures:
    beg: List[int]
    mid: List[int]
    end: List[int]


@dataclass
class ModelFeaturesV2:
    beg: List[int]
    mid: List[int]
    end: List[int]
    # for ISO
    offset_0x8000_0x8007: List[int]
    offset_0x8800_0x8807: List[int]
    offset_0x9000_0x9007: List[int]
    # for UDF
    offset_0x9800_0x9807: List[int]


@dataclass
class ModelOutput:
    ct_label: str
    score: float


@dataclass
class MagikaResult:
    path: str
    dl: ModelOutputFields
    output: MagikaOutputFields


@dataclass
class ModelOutputFields:
    ct_label: Optional[str]
    score: Optional[float]
    group: Optional[str]
    mime_type: Optional[str]
    magic: Optional[str]
    description: Optional[str]


@dataclass
class MagikaOutputFields:
    ct_label: str
    score: float
    group: str
    mime_type: str
    magic: str
    description: str


@dataclass
class FeedbackReport:
    hash: str
    features: ModelFeatures
    result: MagikaResult
