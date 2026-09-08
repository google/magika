# Copyright 2026 Google LLC
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

from typing import List, Optional

class MagikaResult:
    path: Optional[str]
    status: str
    ok: bool
    label: str
    mime_type: str
    group: str
    description: str
    extensions: List[str]
    is_text: bool
    score: float
    dl_label: str
    overwrite_reason: str

class Magika:
    def __init__(self) -> None: ...
    def identify_bytes(self, data: bytes) -> MagikaResult: ...
    def identify_path(self, path: str) -> MagikaResult: ...
    def identify_paths(self, paths: List[str]) -> List[MagikaResult]: ...
    def get_model_name(self) -> str: ...
