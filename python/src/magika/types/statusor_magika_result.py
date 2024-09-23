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

from typing import Optional

from magika.types.content_type_info import ContentTypeInfo
from magika.types.magika_result import MagikaResult
from magika.types.status import Status


class StatusOrMagikaResult:
    def __init__(
        self, *, status: Status = Status.OK, value: Optional[MagikaResult] = None
    ):
        self._status = status
        self._value = value

    def __post_init__(self) -> None:
        if self._status == Status.OK:
            if self._value is None:
                raise ValueError("value must be set when status == OK")
        else:
            if self._value is not None:
                raise ValueError("value cannot be set when status != OK")

    @property
    def ok(self) -> bool:
        return self._status == Status.OK

    @property
    def status(self) -> Status:
        return self._status

    @property
    def value(self) -> MagikaResult:
        if self.ok:
            assert self._value is not None
            return self._value
        raise ValueError("value is not set when status != OK")

    # In the vast majority of cases, Magika API would not return a status != OK
    # (and there is no code path that leads `identify_bytes()` to return an
    # error). To optimize for the most frequent scenario, we add the following
    # properties to forward the underlying value. Clients that want to make use
    # of the full power of the StatusOr[T] patter can still do so, but we do not
    # force all clients, regardless of their complexity or criticality to use
    # the more verbose `mr.value.output`.
    @property
    def dl(self) -> ContentTypeInfo:
        return self.value.dl

    @property
    def output(self) -> ContentTypeInfo:
        return self.value.output

    @property
    def score(self) -> float:
        return self.value.score

    def __repr__(self) -> str:
        return str(self)

    def __str__(self) -> str:
        if self.ok:
            return f"StatusOr(status={self.status}, value={self.value})"
        else:
            return f"StatusOr(status={self.status})"
