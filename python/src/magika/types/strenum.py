"""
We use a StrEnum backport instead of relying on the newly introduced StrEnum
as we want to support at least python 3.8; StrEnum was introduced in python
3.11.

The following code has been taken (and adapted) from:
https://github.com/irgeek/StrEnum/blob/master/strenum/__init__.py#L21
"""

from __future__ import annotations

import enum
from typing import Union


class StrEnum(str, enum.Enum):
    """
    StrEnum is a Python ``enum.Enum`` that inherits from ``str``. The default
    ``auto()`` behavior uses the lower-case version of the name. This is meant
    to reflect the behavior of `enum.StrEnum`, available from Python 3.11.
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
