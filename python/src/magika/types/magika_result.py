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

from magika.logger import get_logger
from magika.types.content_type_info import ContentTypeInfo


@dataclass(frozen=True)
class MagikaResult:
    dl: ContentTypeInfo
    output: ContentTypeInfo
    score: float

    # Access to .path is not supported anymore.  As we
    # cannot implement this in a backward compatibile way, we raise an
    # informative error message instead of leading to inscrutable crashes.
    @property
    def path(self) -> str:
        error_msg = (
            "Unsupported field error: `.path` is not stored anymore in `MagikaResult`. "
            "Consult the documentation for more information."
        )
        log = get_logger()
        log.error(error_msg)
        raise AttributeError(error_msg)
