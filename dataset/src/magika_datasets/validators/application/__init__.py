# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Application data structures; nothing is resolved, opened or executed."""

from . import catalog, chm, dmg, hlp, lnk, pgp, wad

MODULES = (
    lnk,
    wad,
    hlp,
    chm,
    dmg,
    catalog,
    pgp,
)
