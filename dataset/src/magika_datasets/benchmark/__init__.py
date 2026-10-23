# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Benchmark Magika against other file-type detectors on a hydrated corpus snapshot.

`snapshot` materializes the verified files and their truth, `tools` runs and parses each
detector, `metrics` scores and renders, and `run` drives quality and Hyperfine timing.
"""
