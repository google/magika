# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Office package validation without running macros or loading external resources.

OPC dispatch in `opc.py` is reached through `archive/zip.py`; no module registers here.
"""

from . import cfb, onenote

MODULES = (
    cfb,
    onenote,
)
