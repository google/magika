# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""3D geometry and scene validators; nothing is rendered."""

from . import blend, dwg, fbx, gltf, lightwave, maya, ply, stl, threeds

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
)
