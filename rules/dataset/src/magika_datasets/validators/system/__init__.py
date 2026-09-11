# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Operating-system data structures; nothing is mounted, loaded or resolved."""

from . import applesingle, dosmbr, dsstore, intelhex, minidump, palmos, pdb, uf2, winregistry

MODULES = (
    pdb,
    minidump,
    winregistry,
    dsstore,
    applesingle,
    palmos,
    uf2,
    intelhex,
    dosmbr,
)
