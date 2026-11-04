# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Contract every validator module follows; results are evidence, never labels."""

from dataclasses import dataclass
from types import ModuleType

STATUSES = ("pass", "fail", "inconclusive")


@dataclass(frozen=True)
class Observation:
    """One structural observation. `None` from validate() means not applicable."""

    status: str
    detail: str
    format_id: str
    tags: tuple[str, ...] = ()
    generic: bool = False
    """True when the format is a container that more specific formats also satisfy.

    A generic pass is evidence of integrity but does not relabel a sample hinted as
    a more specific format the validator could not prove.
    """

    def __post_init__(self):
        if self.status not in STATUSES:
            raise ValueError(f"Unknown observation status {self.status!r}")


def describe(module: ModuleType) -> dict:
    """Validated module metadata; a misdeclared module is a programming error."""
    for name in ("FAMILY", "FORMAT_IDS", "SCOPE"):
        if not hasattr(module, name):
            raise TypeError(f"{module.__name__} does not declare {name}")
    if not isinstance(module.FORMAT_IDS, tuple) or not module.FORMAT_IDS:
        raise TypeError(f"{module.__name__}.FORMAT_IDS must be a non-empty tuple")
    return {
        "name": f"{module.FAMILY}/{module.__name__.rsplit('.', 1)[-1]}",
        "family": module.FAMILY,
        "format_ids": module.FORMAT_IDS,
        "scope": module.SCOPE,
        "context_required": bool(getattr(module, "CONTEXT_REQUIRED", False)),
        "requires_hint": bool(getattr(module, "REQUIRES_HINT", False)),
        "prefix_only": bool(getattr(module, "PREFIX_ONLY", False)),
        "shared": tuple(getattr(module, "SHARED_FORMAT_IDS", ())),
    }
