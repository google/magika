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


from dataclasses import dataclass
from typing import Dict, List

from magika.types.content_type_label import ContentTypeLabel


@dataclass(frozen=True)
class ModelFeatures:
    beg: List[int]
    mid: List[int]
    end: List[int]
    # for ISO
    offset_0x8000_0x8007: List[int]
    offset_0x8800_0x8807: List[int]
    offset_0x9000_0x9007: List[int]
    # for UDF
    offset_0x9800_0x9807: List[int]


@dataclass(frozen=True)
class ModelOutput:
    label: ContentTypeLabel
    score: float


@dataclass(frozen=True)
class ModelConfig:
    beg_size: int
    mid_size: int
    end_size: int
    use_inputs_at_offsets: bool
    medium_confidence_threshold: float
    min_file_size_for_dl: int
    padding_token: int
    block_size: int
    target_labels_space: List[ContentTypeLabel]
    thresholds: Dict[ContentTypeLabel, float]
    overwrite_map: Dict[ContentTypeLabel, ContentTypeLabel]
