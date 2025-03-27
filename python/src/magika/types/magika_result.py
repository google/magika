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

import dataclasses
from pathlib import Path
from typing import Any, Dict, Optional

from magika.types.content_type_info import ContentTypeInfo
from magika.types.magika_prediction import MagikaPrediction
from magika.types.status import Status


class MagikaResult:
    def __init__(
        self,
        *,
        path: Path,
        status: Status = Status.OK,
        prediction: Optional[MagikaPrediction] = None,
    ):
        self._path = path
        self._status = status
        self._prediction = prediction

    def __post_init__(self) -> None:
        assert self._path is not None
        if self._status == Status.OK:
            if self._prediction is None:
                raise ValueError("prediction must be set when status == OK")
        else:
            if self._prediction is not None:
                raise ValueError("prediction cannot be set when status != OK")

    @property
    def path(self) -> Path:
        return self._path

    @property
    def ok(self) -> bool:
        return self._status == Status.OK

    @property
    def status(self) -> Status:
        return self._status

    @property
    def prediction(self) -> MagikaPrediction:
        if self.ok:
            assert self._prediction is not None
            return self._prediction
        raise ValueError("prediction is not set when status != OK")

    # In the vast majority of cases, Magika API would return a status == OK (and
    # there is no code path that leads `identify_bytes()` to return an error).
    # To optimize for such frequent scenario, we add the following properties to
    # forward the underlying value. Clients that want to make use of the full
    # power of the absl-like StatusOr pattern can still do so, but we do not
    # force all clients, regardless of their complexity or criticality, to use
    # the more verbose `mr.prediction.output`.
    @property
    def dl(self) -> ContentTypeInfo:
        return self.prediction.dl

    @property
    def output(self) -> ContentTypeInfo:
        return self.prediction.output

    @property
    def score(self) -> float:
        return self.prediction.score

    def asdict(self) -> Dict:
        out: Dict[str, Any] = {
            "path": str(self.path),
            "status": self.status,
        }
        if self.ok:
            out["prediction"] = dataclasses.asdict(self.prediction)
        return out

    def __repr__(self) -> str:
        return str(self)

    def __str__(self) -> str:
        if self.ok:
            return f"MagikaResult(path={self.path}, status={self.status}, prediction={self.prediction})"
        else:
            return f"MagikaResult(path={self.path}, status={self.status})"
