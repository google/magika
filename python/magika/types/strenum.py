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
    ``auto()`` behavior uses the member name as its value.

    Example usage::

        class Example(StrEnum):
            UPPER_CASE = auto()
            lower_case = auto()
            MixedCase = auto()

        assert Example.UPPER_CASE == "UPPER_CASE"
        assert Example.lower_case == "lower_case"
        assert Example.MixedCase == "MixedCase"
    """

    def __new__(cls, value: Union[str, StrEnum], *args, **kwargs):
        if not isinstance(value, (str, enum.auto)):
            raise TypeError(
                f"Values of StrEnums must be strings: {value!r} is a {type(value)}"
            )
        return super().__new__(cls, value, *args, **kwargs)

    def __str__(self):
        return str(self.value)

    def _generate_next_value_(name, *_):
        return name


class LowerCaseStrEnum(StrEnum):
    def _generate_next_value_(name, *_):
        return name.lower().replace("_", "-")
