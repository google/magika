# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""YAML documents parsed with PyYAML's SafeLoader under a node budget; hint required."""

import yaml

from ..contract import Observation

FAMILY = "text"
FORMAT_IDS = ("yaml",)
SCOPE = "UTF-8 text composed by SafeLoader (no arbitrary tags) with a node budget across all documents, at least one mapping or sequence root; nothing constructed beyond safe scalars and containers; hint required"
REQUIRES_HINT = True
CONTEXT_REQUIRED = True
NODES = 1_000_000


class Budget(Exception):
    pass


class BoundedLoader(yaml.SafeLoader):
    def __init__(self, stream):
        super().__init__(stream)
        self.nodes = 0

    def compose_node(self, parent, index):
        self.nodes += 1
        if self.nodes > NODES:
            raise Budget("Node budget exceeded")
        return super().compose_node(parent, index)


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    try:
        text = data.decode("utf-8-sig")
    except UnicodeDecodeError:
        return Observation("fail", "Not UTF-8 text", "yaml")
    if "\0" in text:
        return Observation("fail", "NUL bytes in text", "yaml")
    loader = BoundedLoader(text)
    documents, structured = 0, False
    try:
        while loader.check_node():
            node = loader.get_node()
            documents += 1
            structured |= isinstance(node, (yaml.MappingNode, yaml.SequenceNode))
            loader.construct_document(node)  # safe constructors only; unknown tags raise
    except Budget as error:
        return Observation("inconclusive", str(error), "yaml")
    except RecursionError:
        return Observation("inconclusive", "Nesting exceeds the recursion limit", "yaml")
    except yaml.YAMLError as error:
        return Observation("fail", f"PyYAML: {str(error).splitlines()[0][:80]}", "yaml")
    finally:
        loader.dispose()
    if not documents or not structured:
        return Observation("inconclusive", "No mapping or sequence document", "yaml")
    return Observation(
        "pass", f"{documents} YAML document(s) composed safely ({loader.nodes} nodes)", "yaml"
    )
