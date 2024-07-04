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
from pathlib import Path
from typing import Optional

from magika.types.content_type_info import ContentTypeInfo
from magika.types.strenum import StrEnum


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

    @staticmethod
    def from_cts_info_and_score(
        path: Path, dl: ContentTypeInfo, output: ContentTypeInfo, score: float
    ) -> MagikaResult:
        return MagikaResult(path=path, dl=dl, output=output, score=score, error=None)

    @staticmethod
    def from_error(path: Path, error: MagikaResultError) -> MagikaResult:
        return MagikaResult(path=path, dl=None, output=None, score=None, error=error)

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
