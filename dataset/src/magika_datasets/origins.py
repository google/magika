# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0

"""Identity from where a sample came, when a pinned public source names it unambiguously.

This is evidence about provenance, not about bytes. A repository is authoritative about
its own source, and for source code that is the only evidence there is: no magic number
says "this is C", so every structural validator declines and the detectors that map to a
class are weak on text. Measured against the curated labels it agrees on 6,867 of 6,893
samples, and it is ranked below every other tier because those 26 misses are real.
"""

import json
import re

from . import generated
from .filenames import extension_candidates, matches_name

BLOB = re.compile(
    r"^github:https://github\.com/[^/]+/[^/]+/blob/(?P<revision>[0-9a-f]{40})/(?P<path>.+):[0-9a-f]+$"
)


def blob_paths(origins) -> list[str]:
    """Paths from origins pinned to a full commit id. A branch name is not a pin."""
    return [match.group("path") for origin in origins if (match := BLOB.match(origin))]


def extension_owners(formats: dict) -> dict[str, str]:
    """Suffixes owned by exactly one ordinary class.

    A suffix two classes share names neither, and a negative class is never named: its
    samples are deliberately malformed, so a plausible extension says nothing about them.
    """
    owners: dict[str, set[str]] = {}
    for name, record in formats.items():
        if json.loads(record.get("metadata_json") or "{}").get("class_role") == "negative":
            continue
        for suffix in record.get("extensions") or []:
            owners.setdefault(suffix.lower(), set()).add(name)
    return {suffix: next(iter(names)) for suffix, names in owners.items() if len(names) == 1}


class Provenance:
    """Prepared suffix ownership for one taxonomy.

    Built once per run: deriving it costs a pass over every class, and the ladder asks
    about it for every sample.
    """

    def __init__(self, formats: dict):
        self.formats = formats
        self.owners = extension_owners(formats)

    def identity(self, origins) -> str | None:
        return identity(origins, self.formats, self.owners)


def identity(origins, formats: dict, owners: dict | None = None) -> str | None:
    """The single class every pinned path agrees on, by suffix or conventional name.

    A generated origin names its class outright, but only when regenerating it reproduces
    the SHA-256 it records: the generator defines the class, the hash proves the bytes.
    """
    made = {generated.identity(o) for o in origins if o.startswith("generated:")}
    if made:
        kind = made.pop()
        return kind if not made and kind in formats else None
    owners = extension_owners(formats) if owners is None else owners
    claimed = set()
    for path in blob_paths(origins):
        name = path.rsplit("/", 1)[-1]
        found = {owners[suffix] for suffix in extension_candidates(path) if suffix in owners}
        found |= {kind for kind in formats if matches_name(kind, name)}
        if not found:
            return None
        claimed |= found
    return next(iter(claimed)) if len(claimed) == 1 else None
