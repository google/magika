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

# ruff: noqa: D100, D101, D102, D103, D107

"""Backport of StrEnum.

StrEnum was introduced in python 3.11, but we do not rely on it because we aim
at supporting python since version 3.8.

The following code has been taken (and adapted) from:
https://github.com/irgeek/StrEnum/blob/master/strenum/__init__.py#L21
"""

from __future__ import annotations

import enum
from typing import Union


class StrEnum(str, enum.Enum):
    """StrEnum is a Python ``enum.Enum`` that inherits from ``str``.

    The default ``auto()`` behavior uses the lower-case version of the name.
    This is meant to reflect the behavior of `enum.StrEnum`, available from
    Python 3.11.
    """

    def __new__(cls, value: Union[str, StrEnum], *args, **kwargs):  # type: ignore[no-untyped-def]
        if not isinstance(value, (str, enum.auto)):
            raise TypeError(
                f"Values of StrEnums must be strings: {value!r} is a {type(value)}"
            )
        return super().__new__(cls, value, *args, **kwargs)

    def __str__(self) -> str:
        return str(self.value)

    def _generate_next_value_(name, *_):  # type: ignore[no-untyped-def,override]
        return name


class LowerCaseStrEnum(StrEnum):
    def _generate_next_value_(name, *_):  # type: ignore[no-untyped-def,override]
        return name.lower()
