# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Media container validators."""

from . import asf, au, ebml, flac, flv, isobmff, midi, mp3, mpegts, ogg, riff, swf

MODULES = (riff, isobmff, ebml, flv, asf, mpegts, ogg, swf, flac, mp3, midi, au)
