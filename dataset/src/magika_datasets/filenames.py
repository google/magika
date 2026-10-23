# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0

"""Conventional source filenames; corroborating detector evidence is required."""

import re
from pathlib import PurePosixPath

NAMES = {
    "dockerfile": {"Dockerfile"},
    "makefile": {"Makefile", "makefile", "GNUmakefile"},
    "gemfile": {"Gemfile"},
    "gitattributes": {".gitattributes"},
    "gitmodules": {".gitmodules"},
    "htaccess": {".htaccess"},
    "ignorefile": {".gitignore", ".dockerignore", ".npmignore"},
}


def extension_candidates(path):
    """Return final and compound suffixes, ignoring directories and dotfile names.

    ``db.MV.DB`` yields ``mv.db`` and ``db``. These are discovery hints;
    overlapping suffixes do not establish which format owns the file.
    """
    parts = PurePosixPath(path).suffixes
    return {"".join(parts[index:])[1:].lower() for index in range(len(parts))}


def matches_name(format_id, name):
    """Match a basename as a discovery hint, never as sufficient label evidence."""
    if name in NAMES.get(format_id, set()):
        return True
    return (
        format_id == "dockerfile"
        and re.fullmatch(
            r"Dockerfile\.[A-Za-z0-9][A-Za-z0-9_.-]*|[A-Za-z0-9][A-Za-z0-9_.-]*\.[Dd]ockerfile",
            name,
        )
        is not None
    )
