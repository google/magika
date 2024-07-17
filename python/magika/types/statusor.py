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

from typing import Generic, Optional, TypeVar

from magika.types.status import Status

T = TypeVar("T")


class StatusOr(Generic[T]):
    def __init__(self, *, status: Status = Status.OK, value: Optional[T] = None):
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
    def ok(self):
        return self._status == Status.OK

    @property
    def status(self) -> Status:
        return self._status

    @property
    def value(self) -> T:
        if self.ok:
            assert self._value is not None
            return self._value
        raise ValueError("value is not set when status != OK")

    def __repr__(self) -> str:
        return str(self)

    def __str__(self) -> str:
        return f"StatusOr(status={self.status}, value={self.value})"
