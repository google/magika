# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Font container validators; glyph programs are never executed."""

from . import sfnt, woff

MODULES = (sfnt, woff)
