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

`assets/content_types.yaml` is the source of truth for the human-authored fields.
This script validates the YAML, derives `rule_coverage` and `in_ml_model` from
the YARA rulesets and the released model config, and writes
`assets/content_types_kb.min.json`, the artifact consumed by all the bindings.

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
from collections.abc import Mapping
from pathlib import Path
from typing import Any

import yaml

REPO_ROOT_DIR = Path(__file__).resolve().parent.parent
ASSETS_DIR = REPO_ROOT_DIR / "assets"
CONTENT_TYPES_YAML_PATH = ASSETS_DIR / "content_types.yaml"
CONTENT_TYPES_KB_PATH = ASSETS_DIR / "content_types_kb.min.json"
RULESETS_DIR = REPO_ROOT_DIR / "rust" / "rules" / "rulesets"
DEFAULT_MODEL_DIR = REPO_ROOT_DIR / "rust" / "gen" / "model"

# The human-authored fields of each content type, in the order in which they
# must appear in the YAML source and at the start of each JSON KB entry.
FIELDS = ("mime_type", "group", "description", "extensions", "is_text")
CONTENT_TYPE_NAME_RE = re.compile(r"^[a-z0-9_]+$")

RULE_BUCKETS = ("full", "partial")
BUILTIN_CONTENT_TYPES = frozenset(
    {"directory", "empty", "symlink", "txt", "undefined", "unknown"}
)

_YARA_COMMENT_RE = re.compile(r'"(?:\\.|[^"\\\n])*"|/\*[\s\S]*?\*/|//[^\n]*')
_YARA_STRING_RE = re.compile(r'"(?:\\.|[^"\\\n])*"')
_YARA_RULE_KEYWORD_RE = re.compile(r"\brule\b")
_YARA_RULE_HEADER_RE = re.compile(
    r"(?m)^[ \t]*((?:(?:private|global)[ \t]+)*)rule[ \t]+([A-Za-z_][A-Za-z0-9_]*)\s*\{"
)
_YARA_META_BLOCK_RE = re.compile(
    r"\A\s*meta\s*:(.*?)(?=\b(?:strings|condition)\s*:)", re.DOTALL
)
_YARA_META_ENTRY_RE = re.compile(
    r'\s*([A-Za-z_][A-Za-z0-9_]*)\s*=\s*("(?:\\.|[^"\\\n])*"|[^\s"]+)'
)

YAML_NULL_TAG = "tag:yaml.org,2002:null"
YAML_BOOL_TAG = "tag:yaml.org,2002:bool"
YAML_STR_TAG = "tag:yaml.org,2002:str"


class KbError(Exception):
    """The YAML source, rulesets, or model config cannot be loaded or is invalid."""


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
    """Parses `text` using `_StrictLoader`."""
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


def _strip_yara_comments(text: str) -> str:
    return _YARA_COMMENT_RE.sub(
        lambda m: (
            m.group(0)
            if m.group(0).startswith('"')
            else re.sub(r"[^\n]", " ", m.group(0))
        ),
        text,
    )


def _parse_yar_source(
    text: str, bucket: str, source_name: str, known_labels: set[str]
) -> tuple[set[str], list[str]]:
    """Parses a single `.yar` file in `bucket` (`"full"` or `"partial"`)."""
    cleaned = _strip_yara_comments(text)
    no_strings = _YARA_STRING_RE.sub(
        lambda m: '"' + (" " * (len(m.group(0)) - 2)) + '"', cleaned
    )
    rule_matches = list(_YARA_RULE_HEADER_RE.finditer(no_strings))
    if len(_YARA_RULE_KEYWORD_RE.findall(no_strings)) != len(rule_matches):
        return set(), [f"{source_name}: malformed rule declaration"]
    if not rule_matches:
        return set(), [f"{source_name}: no rules found"]

    covered_labels: set[str] = set()
    errors: list[str] = []
    seen_rule_ids: set[str] = set()

    for i, match in enumerate(rule_matches):
        modifiers = match.group(1).split()
        rule_id = match.group(2)
        body_start = match.end()
        body_end = (
            rule_matches[i + 1].start() if i + 1 < len(rule_matches) else len(cleaned)
        )
        body = cleaned[body_start:body_end]

        if rule_id in seen_rule_ids:
            errors.append(f"{source_name}: rule {rule_id}: duplicate rule ID")
        else:
            seen_rule_ids.add(rule_id)

        if "global" in modifiers:
            errors.append(
                f"{source_name}: rule {rule_id}: global rules are not supported"
            )
            continue
        if "private" in modifiers:
            continue

        meta_match = _YARA_META_BLOCK_RE.match(body)
        if meta_match is None:
            errors.append(f"{source_name}: rule {rule_id}: missing 'meta:' section")
            continue

        meta_text = meta_match.group(1)
        meta: dict[str, str] = {}
        pos = 0
        syntax_error = False
        while pos < len(meta_text):
            if meta_text[pos:].isspace():
                break
            entry_match = _YARA_META_ENTRY_RE.match(meta_text, pos)
            if entry_match is None:
                snippet = meta_text[pos:].strip().splitlines()[0][:40]
                errors.append(
                    f"{source_name}: rule {rule_id}: invalid meta syntax near {snippet!r}"
                )
                syntax_error = True
                break
            key, raw_value = entry_match.group(1), entry_match.group(2)
            pos = entry_match.end()
            if key in meta:
                errors.append(
                    f"{source_name}: rule {rule_id}: duplicate meta key {key!r}"
                )
            else:
                meta[key] = raw_value

        if syntax_error:
            continue

        rule_errors: list[str] = []
        label: str | None = None
        if "label" not in meta:
            rule_errors.append("missing 'label' in meta")
        else:
            raw_label = meta["label"]
            if not (
                raw_label.startswith('"')
                and raw_label.endswith('"')
                and len(raw_label) > 2
            ):
                rule_errors.append(f"label must be a non-empty string, got {raw_label}")
            else:
                label = raw_label[1:-1]
                if label not in known_labels:
                    rule_errors.append(f"unknown content type label {label!r}")

        if "class" not in meta:
            rule_errors.append("missing 'class' in meta")
        else:
            raw_class = meta["class"]
            if raw_class != f'"{bucket}"':
                rule_errors.append(
                    f'class must be "{bucket}" in {bucket}/ bucket, got {raw_class}'
                )

        for key in ("enforced", "enabled"):
            if key in meta and meta[key] not in ("true", "false"):
                rule_errors.append(f"{key} must be true or false, got {meta[key]}")

        enforced = meta.get("enforced")
        enabled = meta.get("enabled")
        if enforced is not None and enabled is not None and enforced != enabled:
            rule_errors.append("'enforced' and 'enabled' disagree")

        active = enforced if enforced is not None else enabled
        if active != "true":
            rule_errors.append(
                f"rule in {bucket}/ bucket must be active (enforced = true or enabled = true)"
            )

        if rule_errors:
            errors.extend(
                f"{source_name}: rule {rule_id}: {err}" for err in rule_errors
            )
        elif label is not None:
            covered_labels.add(label)

    return covered_labels, errors


def compute_rule_coverage(
    rulesets: Mapping[str, Mapping[str, str]], known_labels: set[str]
) -> tuple[dict[str, str], list[str]]:
    """Computes `rule_coverage` for every label in `known_labels`."""
    errors: list[str] = []
    covered_by_bucket: dict[str, set[str]] = {"full": set(), "partial": set()}

    for bucket in RULE_BUCKETS:
        bucket_files = rulesets.get(bucket, {})
        for source_name, text in sorted(bucket_files.items()):
            labels, file_errors = _parse_yar_source(
                text, bucket, source_name, known_labels
            )
            covered_by_bucket[bucket].update(labels)
            errors.extend(file_errors)

    full_labels = covered_by_bucket["full"]
    partial_labels = covered_by_bucket["partial"]
    coverage: dict[str, str] = {}
    for label in known_labels:
        if label in full_labels:
            coverage[label] = "full"
        elif label in partial_labels:
            coverage[label] = "partial"
        elif label in BUILTIN_CONTENT_TYPES:
            coverage[label] = "builtin"
        else:
            coverage[label] = "none"

    return coverage, errors


def compute_ml_coverage(
    model_config: Any, known_labels: set[str], source_name: str = "<model_config>"
) -> tuple[set[str], list[str]]:
    """Computes the set of labels with `in_ml_model = True`."""
    if not isinstance(model_config, dict):
        return set(), [
            f"{source_name}: model config must be a JSON object, got {_type_name(model_config)}"
        ]

    errors: list[str] = []
    target_labels_space = model_config.get("target_labels_space")
    overwrite_map = model_config.get("overwrite_map")

    target_labels: set[str] = set()
    if not isinstance(target_labels_space, list) or not target_labels_space:
        errors.append(
            f"{source_name}: target_labels_space must be a non-empty list of strings"
        )
    else:
        for i, label in enumerate(target_labels_space):
            if not isinstance(label, str):
                errors.append(
                    f"{source_name}: target_labels_space[{i}] must be a string, got {_type_name(label)}"
                )
            elif label in target_labels:
                errors.append(
                    f"{source_name}: duplicate label {label!r} in target_labels_space"
                )
            else:
                target_labels.add(label)
                if label not in known_labels:
                    errors.append(
                        f"{source_name}: target_labels_space label {label!r} is not in the YAML KB"
                    )

    overwrite_keys: set[str] = set()
    if not isinstance(overwrite_map, dict):
        errors.append(f"{source_name}: overwrite_map must be a JSON object")
    else:
        for key, value in overwrite_map.items():
            if not isinstance(key, str) or key not in known_labels:
                errors.append(
                    f"{source_name}: overwrite_map key {key!r} is not in the YAML KB"
                )
            else:
                overwrite_keys.add(key)
            if not isinstance(value, str) or value not in known_labels:
                errors.append(
                    f"{source_name}: overwrite_map value {value!r} (for {key!r}) is not in the YAML KB"
                )

    return target_labels - overwrite_keys, errors


def _display_path(path: Path) -> str:
    try:
        return str(path.relative_to(REPO_ROOT_DIR))
    except ValueError:
        return str(path)


def _read_rulesets(rulesets_dir: Path) -> dict[str, dict[str, str]]:
    rulesets: dict[str, dict[str, str]] = {}
    for bucket in RULE_BUCKETS:
        bucket_dir = rulesets_dir / bucket
        if not bucket_dir.is_dir():
            raise KbError(f"{_display_path(bucket_dir)}: ruleset directory not found")
        yar_paths = sorted(bucket_dir.glob("*.yar"))
        if not yar_paths:
            raise KbError(f"{_display_path(bucket_dir)}: no .yar files found")
        rulesets[bucket] = {
            _display_path(path): path.read_text(encoding="utf-8") for path in yar_paths
        }
    return rulesets


def _read_model_config(model_dir: Path) -> tuple[Any, str]:
    if not model_dir.exists():
        raise KbError(f"{_display_path(model_dir)}: model directory not found")
    resolved_dir = model_dir.resolve()
    config_path = resolved_dir / "config.min.json"
    if not config_path.is_file():
        raise KbError(f"{_display_path(config_path)}: model config not found")
    display = _display_path(config_path)
    try:
        return json.loads(config_path.read_text(encoding="utf-8")), display
    except json.JSONDecodeError as e:
        raise KbError(f"{display}: invalid JSON: {e}") from e


def generate_kb_json(
    yaml_text: str,
    rulesets: Mapping[str, Mapping[str, str]],
    model_config: Any,
    source_name: str = "<string>",
    model_source_name: str = "<model_config>",
) -> str:
    """Returns the content of the JSON KB generated from the YAML, rules, and model."""
    data = load_yaml(yaml_text, source_name)
    yaml_errors = validate(data)
    if yaml_errors:
        raise KbError(
            f"{source_name}: found {len(yaml_errors)} error(s):\n"
            + "\n".join(f"- {e}" for e in yaml_errors)
        )

    known_labels = set(data.keys())
    rule_coverage, rule_errors = compute_rule_coverage(rulesets, known_labels)
    in_ml_model_labels, ml_errors = compute_ml_coverage(
        model_config, known_labels, model_source_name
    )
    derived_errors = rule_errors + ml_errors
    if derived_errors:
        raise KbError(
            f"found {len(derived_errors)} error(s) in rulesets/model config:\n"
            + "\n".join(f"- {e}" for e in derived_errors)
        )

    # The validation guarantees that the content types are sorted and that the
    # fields of each entry are in the expected order.
    kb = {
        name: {
            **{field: entry[field] for field in FIELDS},
            "rule_coverage": rule_coverage[name],
            "in_ml_model": name in in_ml_model_labels,
        }
        for name, entry in data.items()
    }
    return json.dumps(kb, separators=(",", ":"), ensure_ascii=True)


def main() -> None:
    """CLI entry point."""
    parser = argparse.ArgumentParser(
        description=f"Generate {_display_path(CONTENT_TYPES_KB_PATH)} "
        f"from {_display_path(CONTENT_TYPES_YAML_PATH)}, YARA rulesets, and model config."
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
    parser.add_argument(
        "--model-dir",
        type=Path,
        default=DEFAULT_MODEL_DIR,
        help="Model directory containing config.min.json (default: rust/gen/model).",
    )
    args = parser.parse_args()

    if args.self_test:
        sys.exit(run_self_test())

    yaml_display_path = _display_path(CONTENT_TYPES_YAML_PATH)
    kb_display_path = _display_path(CONTENT_TYPES_KB_PATH)
    try:
        rulesets = _read_rulesets(RULESETS_DIR)
        model_config, model_display_path = _read_model_config(args.model_dir)
        kb_json = generate_kb_json(
            CONTENT_TYPES_YAML_PATH.read_text(encoding="utf-8"),
            rulesets,
            model_config,
            source_name=yaml_display_path,
            model_source_name=model_display_path,
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

_SELF_TEST_VALID_RULESETS: dict[str, dict[str, str]] = {"full": {}, "partial": {}}
_SELF_TEST_VALID_MODEL_CONFIG: dict[str, Any] = {
    "target_labels_space": ["txt"],
    "overwrite_map": {},
}

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

# (description, rulesets, model_config, substring expected in the error message).
_SELF_TEST_INVALID_DERIVED_CASES: tuple[
    tuple[str, dict[str, dict[str, str]], Any, str], ...
] = (
    (
        "wrong rule class for bucket",
        {
            "full": {
                "formats.yar": (
                    'rule r1 { meta: label = "null" class = "partial" '
                    "enforced = true condition: true }"
                )
            },
            "partial": {},
        },
        _SELF_TEST_VALID_MODEL_CONFIG,
        'class must be "full" in full/ bucket, got "partial"',
    ),
    (
        "inactive rule in full bucket",
        {
            "full": {
                "formats.yar": (
                    'rule r1 { meta: label = "null" class = "full" '
                    "enforced = false condition: true }"
                )
            },
            "partial": {},
        },
        _SELF_TEST_VALID_MODEL_CONFIG,
        "rule in full/ bucket must be active",
    ),
    (
        "missing enforcement metadata",
        {
            "full": {
                "formats.yar": (
                    'rule r1 { meta: label = "null" class = "full" condition: true }'
                )
            },
            "partial": {},
        },
        _SELF_TEST_VALID_MODEL_CONFIG,
        "rule in full/ bucket must be active",
    ),
    (
        "conflicting enabled and enforced",
        {
            "full": {
                "formats.yar": (
                    'rule r1 { meta: label = "null" class = "full" '
                    "enforced = true enabled = false condition: true }"
                )
            },
            "partial": {},
        },
        _SELF_TEST_VALID_MODEL_CONFIG,
        "'enforced' and 'enabled' disagree",
    ),
    (
        "missing label in rule",
        {
            "full": {
                "formats.yar": (
                    'rule r1 { meta: class = "full" enforced = true condition: true }'
                )
            },
            "partial": {},
        },
        _SELF_TEST_VALID_MODEL_CONFIG,
        "missing 'label' in meta",
    ),
    (
        "unknown label in rule",
        {
            "full": {
                "formats.yar": (
                    'rule r1 { meta: label = "nope" class = "full" '
                    "enforced = true condition: true }"
                )
            },
            "partial": {},
        },
        _SELF_TEST_VALID_MODEL_CONFIG,
        "unknown content type label 'nope'",
    ),
    (
        "duplicate rule ID",
        {
            "full": {
                "formats.yar": (
                    'rule r1 { meta: label = "null" class = "full" enforced = true condition: true }\n'
                    'rule r1 { meta: label = "txt" class = "full" enforced = true condition: true }'
                )
            },
            "partial": {},
        },
        _SELF_TEST_VALID_MODEL_CONFIG,
        "duplicate rule ID",
    ),
    (
        "global rule is rejected",
        {
            "full": {
                "formats.yar": (
                    'global rule r1 { meta: label = "null" class = "full" '
                    "enforced = true condition: true }"
                )
            },
            "partial": {},
        },
        _SELF_TEST_VALID_MODEL_CONFIG,
        "global rules are not supported",
    ),
    (
        "unknown label in target_labels_space",
        _SELF_TEST_VALID_RULESETS,
        {"target_labels_space": ["txt", "nope"], "overwrite_map": {}},
        "target_labels_space label 'nope' is not in the YAML KB",
    ),
    (
        "unknown key in overwrite_map",
        _SELF_TEST_VALID_RULESETS,
        {"target_labels_space": ["txt"], "overwrite_map": {"nope": "txt"}},
        "overwrite_map key 'nope' is not in the YAML KB",
    ),
    (
        "unknown value in overwrite_map",
        _SELF_TEST_VALID_RULESETS,
        {"target_labels_space": ["txt"], "overwrite_map": {"null": "nope"}},
        "overwrite_map value 'nope' (for 'null') is not in the YAML KB",
    ),
)


def run_self_test() -> int:
    """Runs the validation self-tests and returns the process exit code."""
    failures: list[str] = []

    expected_valid_json = (
        '{"null":{"mime_type":null,"group":null,"description":null,'
        '"extensions":["null"],"is_text":false,"rule_coverage":"none","in_ml_model":false},'
        '"txt":{"mime_type":"text/plain","group":"text",'
        '"description":"Generic text document","extensions":["txt","text"],"is_text":true,'
        '"rule_coverage":"builtin","in_ml_model":true}}'
    )
    try:
        valid_json = generate_kb_json(
            _SELF_TEST_VALID_YAML,
            _SELF_TEST_VALID_RULESETS,
            _SELF_TEST_VALID_MODEL_CONFIG,
        )
    except KbError as e:
        failures.append(f"valid source: unexpected error: {e}")
    else:
        if valid_json != expected_valid_json:
            failures.append(f"valid source: unexpected output: {valid_json}")

    # Test rule coverage precedence (full > partial > builtin > none), private
    # rules, `enabled = true` alias, and `overwrite_map` exclusion.
    precedence_rulesets = {
        "full": {
            "formats.yar": (
                "/* block comment with rule fake { */\n"
                "private rule helper { condition: true }\n"
                "rule r_txt {\n"
                "\tmeta:\n"
                '\t\tlabel = "txt"\n'
                '        class = "full"\n'
                "        enabled = true\n"
                "    strings:\n"
                '        $s = /["]/\n'
                "    condition:\n"
                "        $s\n"
                "}\n"
            )
        },
        "partial": {
            "formats.yar": (
                'rule r_txt_partial { meta: label = "txt" class = "partial" enforced = true condition: true }\n'
                'rule r_null_partial { meta: label = "null" class = "partial" enforced = true condition: true }\n'
            )
        },
    }
    precedence_model_config = {
        "target_labels_space": ["null", "txt"],
        "overwrite_map": {"null": "txt"},
    }
    try:
        precedence_kb = json.loads(
            generate_kb_json(
                _SELF_TEST_VALID_YAML, precedence_rulesets, precedence_model_config
            )
        )
    except KbError as e:
        failures.append(f"precedence source: unexpected error: {e}")
    else:
        if (
            precedence_kb["txt"]["rule_coverage"] != "full"
            or not precedence_kb["txt"]["in_ml_model"]
            or precedence_kb["null"]["rule_coverage"] != "partial"
            or precedence_kb["null"]["in_ml_model"]
        ):
            failures.append(f"precedence source: unexpected output: {precedence_kb}")

    for description, yaml_text, expected_error in _SELF_TEST_INVALID_CASES:
        try:
            generate_kb_json(
                yaml_text,
                _SELF_TEST_VALID_RULESETS,
                _SELF_TEST_VALID_MODEL_CONFIG,
            )
        except KbError as e:
            if expected_error not in str(e):
                failures.append(
                    f"{description}: expected error containing {expected_error!r}, got: {e}"
                )
        else:
            failures.append(f"{description}: expected an error, got none")

    for (
        description,
        rulesets,
        model_config,
        expected_error,
    ) in _SELF_TEST_INVALID_DERIVED_CASES:
        try:
            generate_kb_json(_SELF_TEST_VALID_YAML, rulesets, model_config)
        except KbError as e:
            if expected_error not in str(e):
                failures.append(
                    f"{description}: expected error containing {expected_error!r}, got: {e}"
                )
        else:
            failures.append(f"{description}: expected an error, got none")

    total = 2 + len(_SELF_TEST_INVALID_CASES) + len(_SELF_TEST_INVALID_DERIVED_CASES)
    for failure in failures:
        print(f"FAIL: {failure}", file=sys.stderr)
    print(f"{total - len(failures)}/{total} self-tests passed.")
    return 1 if failures else 0


if __name__ == "__main__":
    main()
