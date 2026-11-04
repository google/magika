# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""3D geometry and scene validators; nothing is rendered."""

from . import (
    alembic,
    blend,
    dwg,
    fbx,
    flatgeobuf,
    gltf,
    lightwave,
    maya,
    ply,
    pmtiles,
    rhino,
    stl,
    threeds,
    wkt,
)

MODULES = (
    blend,
    threeds,
    ply,
    stl,
    fbx,
    gltf,
    dwg,
    lightwave,
    maya,
    rhino,
    flatgeobuf,
    alembic,
    pmtiles,
    wkt,
)
