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

from typing import List


class PredictionMode:
    BEST_GUESS = "best-guess"
    MEDIUM_CONFIDENCE = "medium-confidence"
    HIGH_CONFIDENCE = "high-confidence"

    @staticmethod
    def get_valid_prediction_modes() -> List[str]:
        return [
            PredictionMode.BEST_GUESS,
            PredictionMode.MEDIUM_CONFIDENCE,
            PredictionMode.HIGH_CONFIDENCE,
        ]
