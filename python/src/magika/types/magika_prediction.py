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

"""Module defining the MagikaPrediction dataclass."""

from __future__ import annotations

from dataclasses import dataclass

from magika.types.content_type_info import ContentTypeInfo
from magika.types.overwrite_reason import OverwriteReason


@dataclass(frozen=True)
class MagikaPrediction:
    """Encodes the detailed result of Magika's content type inference.

    This dataclass holds both the raw Deep Learning model's prediction and the
    final, potentially modified, output prediction.

    Attributes:
        dl: The raw prediction from the Deep Learning (DL) model.
        output: The final, consolidated content type prediction, which may
            differ from `dl` due to heuristics or post-processing.
        score: The confidence score (0.0 to 1.0) associated with the final
            prediction.
        overwrite_reason: The reason the `output` might have overridden the
            raw `dl` prediction.
    """

    dl: ContentTypeInfo
    output: ContentTypeInfo
    score: float
    overwrite_reason: OverwriteReason
