# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Text model definitions whose structure declares its own size: LightGBM and ncnn."""

import re

from ..contract import Observation

FAMILY = "text"
FORMAT_IDS = ("lightgbm", "ncnn")
SCOPE = "LightGBM text model: tree header, version, num_class, a tree_sizes list whose length matches the sequential Tree=N blocks, and the end-of-trees marker. ncnn param: 7767517 magic, declared layer and blob counts, exactly that many layer lines each carrying the input and output names it declares; weights files are not read"
VERSION = re.compile(rb"version=v\d+")


def lightgbm(data: bytes) -> Observation:
    lines = data.decode("utf-8", "strict").splitlines()
    header = {}
    for line in lines:
        if not line.strip():
            break
        key, _, value = line.partition("=")
        header[key] = value
    if not header.get("num_class", "").isdigit():
        return Observation("fail", "Header has no integer num_class", "lightgbm")
    try:
        sizes = [int(x) for x in header.get("tree_sizes", "").split()]
    except ValueError:
        return Observation("fail", "tree_sizes is not a list of integers", "lightgbm")
    trees = [int(line[5:]) for line in lines if re.fullmatch(r"Tree=\d+", line)]
    if trees != list(range(len(trees))):
        return Observation("fail", "Tree blocks are not numbered 0..n-1", "lightgbm")
    if sizes and len(sizes) != len(trees):
        return Observation("fail", f"{len(sizes)} tree_sizes for {len(trees)} trees", "lightgbm")
    if "end of trees" not in lines:
        return Observation("fail", "No end-of-trees marker", "lightgbm")
    return Observation("pass", f"LightGBM model with {len(trees)} trees", "lightgbm")


def ncnn(data: bytes) -> Observation:
    lines = [line for line in data.decode("utf-8", "strict").splitlines() if line.strip()]
    try:
        layers, blobs = (int(x) for x in lines[1].split())
    except (IndexError, ValueError):
        return Observation("fail", "Second line is not a layer and blob count", "ncnn")
    body = lines[2:]
    if len(body) != layers:
        return Observation("fail", f"{len(body)} layer lines for {layers} declared", "ncnn")
    produced = set()
    for number, line in enumerate(body):
        fields = line.split()
        try:
            inputs, outputs = int(fields[2]), int(fields[3])
        except (IndexError, ValueError):
            return Observation("fail", f"Layer {number} has no input and output counts", "ncnn")
        names = fields[4 : 4 + inputs + outputs]
        if len(names) != inputs + outputs or any("=" in name for name in names):
            return Observation("fail", f"Layer {number} lists fewer blobs than it declares", "ncnn")
        produced.update(names[inputs:])
    if len(produced) > blobs:
        return Observation("fail", f"{len(produced)} blobs produced, {blobs} declared", "ncnn")
    return Observation("pass", f"ncnn param with {layers} layers and {blobs} blobs", "ncnn")


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    try:
        if data.startswith((b"tree\n", b"tree\r\n")) and VERSION.match(data.split(b"\n", 2)[1]):
            return lightgbm(data)
        if data.split(b"\n", 1)[0].strip() == b"7767517":
            return ncnn(data)
    except UnicodeError:
        return Observation(
            "fail", "Model text is not UTF-8", "lightgbm" if data.startswith(b"tree") else "ncnn"
        )
    return None
