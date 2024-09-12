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

import enum
from typing import List

from magika.types.strenum import LowerCaseStrEnum


class PredictionMode(LowerCaseStrEnum):
    BEST_GUESS = enum.auto()
    MEDIUM_CONFIDENCE = enum.auto()
    HIGH_CONFIDENCE = enum.auto()

    @staticmethod
    def get_valid_prediction_modes() -> List[str]:
        return [pm for pm in PredictionMode]
