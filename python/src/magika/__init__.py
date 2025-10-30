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

# ruff: noqa: D104


__version__ = "0.6.3"


import dotenv

from magika.magika import Magika
from magika.types.content_type_info import ContentTypeInfo
from magika.types.content_type_label import ContentTypeLabel
from magika.types.magika_error import MagikaError
from magika.types.magika_prediction import MagikaPrediction
from magika.types.magika_result import MagikaResult
from magika.types.overwrite_reason import OverwriteReason
from magika.types.prediction_mode import PredictionMode
from magika.types.status import Status

__all__ = [
    "ContentTypeInfo",
    "ContentTypeLabel",
    "Magika",
    "MagikaError",
    "MagikaPrediction",
    "MagikaResult",
    "OverwriteReason",
    "PredictionMode",
    "Status",
]

dotenv.load_dotenv(dotenv.find_dotenv())
