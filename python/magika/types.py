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

import dataclasses
from dataclasses import dataclass
from pathlib import Path
from typing import List, Optional

from magika.content_types import ContentType
from magika.strenum import StrEnum


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
    ct_label: ContentType
    score: float


@dataclass(kw_only=True)
class ContentTypeInfo:
    name: ContentType
    mime_type: str
    group: str
    description: str
    is_text: bool


@dataclass(kw_only=True)
class ModelConfig:
    beg_size: int
    mid_size: int
    end_size: int
    use_inputs_at_offsets: bool
    medium_confidence_threshold: float
    min_file_size_for_dl: int
    padding_token: int
    block_size: int
    target_labels_space: list[ContentType]
    thresholds: dict[ContentType, float]
    overwrite_map: dict[ContentType, ContentType]


class MagikaResult:
    def __init__(
        self,
        path: Path,
        dl: Optional[ContentTypeInfo] = None,
        output: Optional[ContentTypeInfo] = None,
        score: Optional[float] = None,
        error: Optional[MagikaResultError] = None,
    ):
        """Constructor for MagikaResult. Instances are supposed to be created
        via factor methods, not via this one.
        """

        self._success: bool = error is None

        self._path: Path = path
        self._dl: Optional[ContentTypeInfo] = dl
        self._output: Optional[ContentTypeInfo] = output
        self._score: Optional[float] = score
        self._error: Optional[MagikaResultError] = error

        # consistency checks
        if self._success:
            assert self._dl is not None
            assert self._output is not None
            assert self._score is not None
            assert self._error is None
        else:
            assert self._dl is None
            assert self._output is None
            assert self._score is None
            assert self._error is not None

    @property
    def success(self) -> bool:
        return self._success

    @property
    def path(self) -> Path:
        return self._path

    @path.setter
    def path(self, new_path: Path) -> None:
        self._path = new_path

    @property
    def dl(self) -> ContentTypeInfo:
        assert self._success
        assert self._dl is not None
        return self._dl

    @property
    def output(self) -> ContentTypeInfo:
        assert self._success
        assert self._output is not None
        return self._output

    @property
    def score(self) -> float:
        assert self._success
        assert self._score is not None
        return self._score

    @property
    def error(self) -> MagikaResultError:
        assert not self._success
        assert self._error is not None
        return self._error

    def asdict(self) -> dict:
        if self.success:
            return {
                "path": self.path,
                "dl": dataclasses.asdict(self.dl),
                "output": dataclasses.asdict(self.output),
                "score": self.score,
                "success": self.success,
            }
        else:
            return {
                "success": self.success,
                "error": self.error,
            }


class MagikaResultError(StrEnum):
    # Used when a file path does not exist
    FILE_DOES_NOT_EXIST = "file_does_not_exist"

    # Used when a file path exists, but there are permission issues, e.g., can't
    # read file
    PERMISSION_ERROR = "permission_error"


@dataclass
class FeedbackReport:
    hash: str
    features: ModelFeaturesV2
    result: MagikaResult
