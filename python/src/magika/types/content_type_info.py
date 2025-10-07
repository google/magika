# Copyright 2025 Google LLC
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


import warnings
from dataclasses import dataclass
from typing import List

from magika.logger import get_logger
from magika.types.content_type_label import ContentTypeLabel


@dataclass(frozen=True)
class ContentTypeInfo:
    label: ContentTypeLabel
    mime_type: str
    group: str
    description: str
    extensions: List[str]
    is_text: bool

    @property
    def ct_label(self) -> str:
        warnings.warn(
            "`.ct_label` is deprecated and will be removed in a future version. Use `.label` instead. Consult the documentation for more information.",
            category=DeprecationWarning,
            stacklevel=2,
        )
        return str(self.label)

    @property
    def score(self) -> float:
        error_msg = "Unsupported field error: `.score.` is not stored anymore in the `dl` or `output` objects; it is now stored in `MagikaResult`. Consult the documentation for more information."
        log = get_logger()
        log.error(error_msg)
        raise AttributeError(error_msg)

    @property
    def magic(self) -> str:
        warnings.warn(
            "`.magic` is deprecated and will be removed in a future version. Use `.description` instead. Consult the documentation for more information.",
            category=DeprecationWarning,
            stacklevel=2,
        )
        return self.description
