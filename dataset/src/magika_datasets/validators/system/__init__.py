# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Operating-system data structures; nothing is mounted, loaded or resolved."""

from . import (
    applesingle,
    dosmbr,
    dsstore,
    hfs,
    intelhex,
    minidump,
    palmos,
    pdb,
    srecord,
    uefi_fv,
    uf2,
    vhd,
    winregistry,
)

MODULES = (
    pdb,
    minidump,
    winregistry,
    dsstore,
    applesingle,
    palmos,
    uf2,
    intelhex,
    srecord,
    dosmbr,
    vhd,
    hfs,
    uefi_fv,
)
