#!/usr/bin/env python3
# Copyright 2026 Google LLC
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.
# /// script
# dependencies = ["pyyaml"]
# ///
"""Generate the content types KB from its human-editable YAML source.

`assets/content_types.yaml` is the source of truth. This script validates it and
writes `assets/content_types_kb.min.json`, the artifact consumed by all the
bindings.

Usage (from anywhere in the repository):

    uv run scripts/sync_kb.py               # Regenerate the JSON KB.
    uv run scripts/sync_kb.py --check       # Fail if the JSON KB is stale.
    uv run scripts/sync_kb.py --self-test   # Run the validation self-tests.
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path
from typing import Any

import yaml

REPO_ROOT_DIR = Path(__file__).resolve().parent.parent
ASSETS_DIR = REPO_ROOT_DIR / "assets"
CONTENT_TYPES_YAML_PATH = ASSETS_DIR / "content_types.yaml"
CONTENT_TYPES_KB_PATH = ASSETS_DIR / "content_types_kb.min.json"

# The fields of each content type, in the order in which they must appear in the
# YAML source and in which they are written to the JSON KB.
FIELDS = ("mime_type", "group", "description", "extensions", "is_text")
CONTENT_TYPE_NAME_RE = re.compile(r"^[a-z0-9_]+$")

YAML_NULL_TAG = "tag:yaml.org,2002:null"
YAML_BOOL_TAG = "tag:yaml.org,2002:bool"
YAML_STR_TAG = "tag:yaml.org,2002:str"


class KbError(Exception):
    """The YAML source cannot be loaded or is invalid."""


class _StrictLoader(yaml.SafeLoader):
    """A `SafeLoader` that removes YAML's implicit typing pitfalls.

    - Strings must be quoted (the field names, e.g. `mime_type`, are exempt).
      Apart from those, the only unquoted scalars allowed are `null`, `true`,
      and `false`. So YAML 1.1 surprises like `null`, `yes`, `off`, `1.0`, or
      `2024-01-01` silently becoming non-strings can't happen.
    - Duplicate and non-string mapping keys are errors (PyYAML silently keeps
      the last duplicate by default).
    """


def _construct_strict_str(loader: _StrictLoader, node: yaml.ScalarNode) -> str:
    value = loader.construct_scalar(node)
    # `style` is None for plain (unquoted) scalars.
    if node.style is None and value not in FIELDS:
        raise yaml.constructor.ConstructorError(
            None,
            None,
            f'found unquoted value {value!r}: quote strings (e.g. "{value}"); '
            "only null, true, and false may be unquoted",
            node.start_mark,
        )
    return value


def _construct_strict_mapping(
    loader: _StrictLoader, node: yaml.MappingNode
) -> dict[str, Any]:
    mapping: dict[str, Any] = {}
    for key_node, value_node in node.value:
        key = loader.construct_object(key_node, deep=True)
        if not isinstance(key, str):
            raise yaml.constructor.ConstructorError(
                "while constructing a mapping",
                node.start_mark,
                f"found non-string key {key!r} (quote it)",
                key_node.start_mark,
            )
        if key in mapping:
            raise yaml.constructor.ConstructorError(
                "while constructing a mapping",
                node.start_mark,
                f"found duplicate key {key!r}",
                key_node.start_mark,
            )
        mapping[key] = loader.construct_object(value_node, deep=True)
    return mapping


_StrictLoader.add_constructor(YAML_STR_TAG, _construct_strict_str)
_StrictLoader.add_constructor(
    yaml.resolver.BaseResolver.DEFAULT_MAPPING_TAG, _construct_strict_mapping
)
# Only `null`, `true`, and `false` resolve to non-strings when unquoted; every
# other plain scalar resolves to a string (and is then rejected, see above).
_StrictLoader.yaml_implicit_resolvers = {}
_StrictLoader.add_implicit_resolver(YAML_NULL_TAG, re.compile(r"^null$"), ["n"])
_StrictLoader.add_implicit_resolver(
    YAML_BOOL_TAG, re.compile(r"^(?:true|false)$"), ["t", "f"]
)


def load_yaml(text: str, source_name: str = "<string>") -> Any:
    loader = _StrictLoader(text)
    # Shown in the error locations (it defaults to "<unicode string>").
    loader.name = source_name
    try:
        return loader.get_single_data()
    except yaml.YAMLError as e:
        raise KbError(f"{source_name}: invalid YAML: {e}") from e
    finally:
        loader.dispose()


def validate(data: Any) -> list[str]:
    """Validates the loaded YAML source and returns all the errors found."""

    if not isinstance(data, dict):
        return [f"top level must be a mapping, got {_type_name(data)}"]

    errors: list[str] = []
    previous_name: str | None = None
    for name, entry in data.items():
        if not CONTENT_TYPE_NAME_RE.match(name):
            errors.append(
                f"{name!r}: content type name must match {CONTENT_TYPE_NAME_RE.pattern}"
            )
        if previous_name is not None and name < previous_name:
            errors.append(
                f"{name!r}: content types must be sorted by name, but it comes after {previous_name!r}"
            )
        previous_name = name
        errors.extend(f"{name}: {error}" for error in _validate_entry(entry))
    return errors


def _validate_entry(entry: Any) -> list[str]:
    if not isinstance(entry, dict):
        return [f"entry must be a mapping, got {_type_name(entry)}"]

    keys = tuple(entry.keys())
    if keys != FIELDS:
        missing = [f for f in FIELDS if f not in entry]
        unknown = [k for k in keys if k not in FIELDS]
        problems = []
        if missing:
            problems.append(f"missing {missing}")
        if unknown:
            problems.append(f"unknown {unknown}")
        if not problems:
            problems.append(f"got {list(keys)}")
        return [
            f"fields must be exactly {list(FIELDS)}, in this order ({'; '.join(problems)})"
        ]

    errors: list[str] = []
    for field in ("mime_type", "group", "description"):
        value = entry[field]
        if value is not None and not (isinstance(value, str) and value):
            errors.append(
                f"{field}: must be a non-empty string or null, got {value!r} ({_type_name(value)})"
            )

    extensions = entry["extensions"]
    if not isinstance(extensions, list):
        errors.append(
            f"extensions: must be a list of strings, got {extensions!r} ({_type_name(extensions)})"
        )
    else:
        seen: set[str] = set()
        for i, ext in enumerate(extensions):
            if not isinstance(ext, str):
                errors.append(
                    f"extensions[{i}]: must be a string, got {ext!r} ({_type_name(ext)}); "
                    'strings must be quoted (e.g. "null")'
                )
            elif not ext or ext.startswith(".") or ext != ext.strip():
                errors.append(
                    f"extensions[{i}]: {ext!r} must be non-empty, without a leading '.' or surrounding whitespace"
                )
            elif ext in seen:
                errors.append(f"extensions[{i}]: duplicate extension {ext!r}")
            else:
                seen.add(ext)

    is_text = entry["is_text"]
    if not isinstance(is_text, bool):
        errors.append(
            f"is_text: must be true or false, got {is_text!r} ({_type_name(is_text)})"
        )

    return errors


def _type_name(value: Any) -> str:
    return "null" if value is None else type(value).__name__


def generate_kb_json(yaml_text: str, source_name: str = "<string>") -> str:
    """Returns the content of the JSON KB generated from the YAML source."""

    data = load_yaml(yaml_text, source_name)
    errors = validate(data)
    if errors:
        raise KbError(
            f"{source_name}: found {len(errors)} error(s):\n"
            + "\n".join(f"- {e}" for e in errors)
        )
    # The validation guarantees that the content types are sorted and that the
    # fields of each entry are in the expected order.
    kb = {
        name: {field: entry[field] for field in FIELDS} for name, entry in data.items()
    }
    return json.dumps(kb, separators=(",", ":"), ensure_ascii=True)


def main() -> None:
    parser = argparse.ArgumentParser(
        description=f"Generate {CONTENT_TYPES_KB_PATH.relative_to(REPO_ROOT_DIR)} "
        f"from {CONTENT_TYPES_YAML_PATH.relative_to(REPO_ROOT_DIR)}."
    )
    mode = parser.add_mutually_exclusive_group()
    mode.add_argument(
        "--check",
        action="store_true",
        help="Do not write anything; exit with an error if the JSON KB is stale.",
    )
    mode.add_argument(
        "--self-test",
        action="store_true",
        help="Run the validation self-tests and exit.",
    )
    args = parser.parse_args()

    if args.self_test:
        sys.exit(run_self_test())

    yaml_display_path = CONTENT_TYPES_YAML_PATH.relative_to(REPO_ROOT_DIR)
    kb_display_path = CONTENT_TYPES_KB_PATH.relative_to(REPO_ROOT_DIR)
    try:
        kb_json = generate_kb_json(
            CONTENT_TYPES_YAML_PATH.read_text(encoding="utf-8"), str(yaml_display_path)
        )
    except KbError as e:
        print(f"ERROR: {e}", file=sys.stderr)
        sys.exit(1)
    kb_bytes = kb_json.encode("ascii")

    if args.check:
        if (
            not CONTENT_TYPES_KB_PATH.is_file()
            or CONTENT_TYPES_KB_PATH.read_bytes() != kb_bytes
        ):
            print(
                f"ERROR: {kb_display_path} is not in sync with {yaml_display_path}.\n"
                "Run: uv run scripts/sync_kb.py",
                file=sys.stderr,
            )
            sys.exit(1)
        print(f"{kb_display_path} is in sync with {yaml_display_path}.")
        return

    if (
        CONTENT_TYPES_KB_PATH.is_file()
        and CONTENT_TYPES_KB_PATH.read_bytes() == kb_bytes
    ):
        print(f"{kb_display_path} is already up to date.")
        return
    CONTENT_TYPES_KB_PATH.write_bytes(kb_bytes)
    print(f"Updated {kb_display_path}.")


# A valid YAML source with two entries. The self-test cases are derived from it.
_SELF_TEST_VALID_YAML = """\
# A comment.
"null":
  mime_type: null
  group: null
  description: null
  extensions: ["null"]
  is_text: false

"txt":
  mime_type: "text/plain"
  group: "text"
  description: "Generic text document"
  extensions: ["txt", 'text']
  is_text: true
"""

# (description, YAML source, substring expected in the error message).
_SELF_TEST_INVALID_CASES = (
    (
        "duplicate content type",
        _SELF_TEST_VALID_YAML + '"txt":\n  is_text: true\n',
        "found duplicate key 'txt'",
    ),
    (
        "duplicate field",
        _SELF_TEST_VALID_YAML.replace(
            "  is_text: true\n", "  is_text: true\n  is_text: false\n"
        ),
        "found duplicate key 'is_text'",
    ),
    (
        "unquoted null content type",
        _SELF_TEST_VALID_YAML.replace('"null":', "null:"),
        "found non-string key None",
    ),
    (
        "unquoted content type",
        _SELF_TEST_VALID_YAML.replace('"txt":', "txt:"),
        "found unquoted value 'txt'",
    ),
    (
        "unquoted string value",
        _SELF_TEST_VALID_YAML.replace('group: "text"', "group: text"),
        "found unquoted value 'text'",
    ),
    (
        "unquoted null extension",
        _SELF_TEST_VALID_YAML.replace('["null"]', "[null]"),
        "null: extensions[0]: must be a string",
    ),
    (
        "unquoted number extension",
        _SELF_TEST_VALID_YAML.replace('"txt", ', "123, "),
        "found unquoted value '123'",
    ),
    (
        "extension with leading dot",
        _SELF_TEST_VALID_YAML.replace('"txt", ', '".txt", '),
        "txt: extensions[0]: '.txt' must be non-empty, without a leading '.'",
    ),
    (
        "duplicate extension",
        _SELF_TEST_VALID_YAML.replace("'text'", '"txt"'),
        "txt: extensions[1]: duplicate extension 'txt'",
    ),
    (
        "YAML 1.1 boolean is_text",
        _SELF_TEST_VALID_YAML.replace("is_text: true", "is_text: yes"),
        "found unquoted value 'yes'",
    ),
    (
        "capitalized boolean is_text",
        _SELF_TEST_VALID_YAML.replace("is_text: true", "is_text: True"),
        "found unquoted value 'True'",
    ),
    (
        "quoted boolean is_text",
        _SELF_TEST_VALID_YAML.replace("is_text: true", 'is_text: "true"'),
        "txt: is_text: must be true or false, got 'true' (str)",
    ),
    (
        "empty value",
        _SELF_TEST_VALID_YAML.replace('group: "text"', "group:"),
        "found unquoted value ''",
    ),
    (
        "empty description",
        _SELF_TEST_VALID_YAML.replace(
            'description: "Generic text document"', 'description: ""'
        ),
        "txt: description: must be a non-empty string or null",
    ),
    (
        "unsorted content types",
        _SELF_TEST_VALID_YAML + '"abc":\n' + _SELF_TEST_VALID_YAML.split('"txt":\n')[1],
        "'abc': content types must be sorted by name, but it comes after 'txt'",
    ),
    (
        "invalid content type name",
        _SELF_TEST_VALID_YAML.replace('"txt":', '"Txt":'),
        "'Txt': content type name must match",
    ),
    (
        "unknown field",
        _SELF_TEST_VALID_YAML.replace(
            "  is_text: true\n", '  is_text: true\n  "in_ml_model": true\n'
        ),
        "unknown ['in_ml_model']",
    ),
    (
        "missing field",
        _SELF_TEST_VALID_YAML.replace('  group: "text"\n', ""),
        "missing ['group']",
    ),
    (
        "fields out of order",
        _SELF_TEST_VALID_YAML.replace(
            '  mime_type: "text/plain"\n  group: "text"\n',
            '  group: "text"\n  mime_type: "text/plain"\n',
        ),
        "in this order (got ['group', 'mime_type',",
    ),
    (
        "top level is not a mapping",
        '- "txt"\n',
        "top level must be a mapping, got list",
    ),
)


def run_self_test() -> int:
    """Runs the validation self-tests and returns the process exit code."""

    failures: list[str] = []

    expected_valid_json = (
        '{"null":{"mime_type":null,"group":null,"description":null,'
        '"extensions":["null"],"is_text":false},'
        '"txt":{"mime_type":"text/plain","group":"text",'
        '"description":"Generic text document","extensions":["txt","text"],"is_text":true}}'
    )
    try:
        valid_json = generate_kb_json(_SELF_TEST_VALID_YAML)
    except KbError as e:
        failures.append(f"valid source: unexpected error: {e}")
    else:
        if valid_json != expected_valid_json:
            failures.append(f"valid source: unexpected output: {valid_json}")

    for description, yaml_text, expected_error in _SELF_TEST_INVALID_CASES:
        try:
            generate_kb_json(yaml_text)
        except KbError as e:
            if expected_error not in str(e):
                failures.append(
                    f"{description}: expected error containing {expected_error!r}, got: {e}"
                )
        else:
            failures.append(f"{description}: expected an error, got none")

    total = 1 + len(_SELF_TEST_INVALID_CASES)
    for failure in failures:
        print(f"FAIL: {failure}", file=sys.stderr)
    print(f"{total - len(failures)}/{total} self-tests passed.")
    return 1 if failures else 0


if __name__ == "__main__":
    main()
