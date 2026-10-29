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
# dependencies = ["click", "pyyaml"]
#
# [tool.uv]
# required-version = ">=0.13.4"
# ///
"""Generate the content types KB from its human-editable YAML source.

`assets/content_types.yaml` is the source of truth for the human-authored fields.
This script validates the YAML, derives the 4 boolean coverage fields
(`in_heuristics`, `in_resolvers`, `in_ml_model`, `vetoed_by_resolvers`) from
built-in heuristics, the Rust resolvers (`rust/lib/src/resolvers/`), and the
active ML model config (`assets/active_model.txt`), and writes
`assets/content_types_kb.min.json`, the artifact consumed by all the bindings.

Usage (from anywhere in the repository):

    uv run scripts/sync_kb.py               # Regenerate the JSON KB.
    uv run scripts/sync_kb.py --check       # Fail if the JSON KB is stale.
    uv run scripts/sync_kb.py --self-test   # Run the validation self-tests.
"""

from __future__ import annotations

import json
import re
import sys
from collections.abc import Mapping
from pathlib import Path
from typing import Any

import click
import yaml

REPO_ROOT_DIR = Path(__file__).resolve().parent.parent
ASSETS_DIR = REPO_ROOT_DIR / "assets"
CONTENT_TYPES_YAML_PATH = ASSETS_DIR / "content_types.yaml"
CONTENT_TYPES_KB_PATH = ASSETS_DIR / "content_types_kb.min.json"
ACTIVE_MODEL_PATH = ASSETS_DIR / "active_model.txt"
MODELS_DIR = ASSETS_DIR / "models"
RESOLVERS_RS_PATH = REPO_ROOT_DIR / "rust" / "lib" / "src" / "resolvers.rs"
RESOLVERS_DIR = REPO_ROOT_DIR / "rust" / "lib" / "src" / "resolvers"

# The human-authored fields of each content type, in the order in which they
# must appear in the YAML source and at the start of each JSON KB entry.
FIELDS = ("mime_type", "group", "description", "extensions", "is_text")
CONTENT_TYPE_NAME_RE = re.compile(r"^[a-z0-9_]+$")
MIME_TYPE_RE = re.compile(
    r"^[a-z0-9][a-z0-9!#$&\-\^_.+]*/[a-zA-Z0-9][a-zA-Z0-9!#$&\-\^_.+]*$"
)
EXTENSION_RE = re.compile(r"^[A-Za-z0-9][A-Za-z0-9_+.-]*$")
ACTIVE_MODEL_NAME_RE = re.compile(r"^[A-Za-z0-9_.-]+$")

VALID_GROUPS = frozenset(
    {
        "application",
        "archive",
        "audio",
        "binary",
        "code",
        "database",
        "document",
        "executable",
        "font",
        "geometry",
        "gis",
        "image",
        "inode",
        "model",
        "scientific",
        "text",
        "undefined",
        "unknown",
        "video",
    }
)

HEURISTIC_CONTENT_TYPES = frozenset(
    {"directory", "empty", "symlink", "txt", "unknown", "unsupported"}
)

_CFG_TEST_RE = re.compile(r"#\s*\[\s*cfg\s*\(\s*test\s*\)\s*\]")
_IDENT_RE = re.compile(r"[A-Za-z_][A-Za-z0-9_]*")
_CONTENT_TYPE_USE_ALIAS_RE = re.compile(
    r"\bContentType\s+as\s+([A-Za-z_][A-Za-z0-9_]*)\b"
)
_CONTENT_TYPE_TYPE_ALIAS_RE = re.compile(
    r"\btype\s+([A-Za-z_][A-Za-z0-9_]*)\s*=\s*(?:[A-Za-z0-9_]+\s*::\s*)*ContentType\b"
)

YAML_NULL_TAG = "tag:yaml.org,2002:null"
YAML_BOOL_TAG = "tag:yaml.org,2002:bool"
YAML_STR_TAG = "tag:yaml.org,2002:str"


class KbError(Exception):
    """The YAML source, resolver sources, or model config cannot be loaded or is invalid."""


@click.command()
@click.option(
    "--check",
    is_flag=True,
    help="Do not write anything; exit with an error if the JSON KB is stale.",
)
@click.option(
    "--self-test",
    is_flag=True,
    help="Run the validation self-tests and exit.",
)
@click.option(
    "--model-dir",
    type=click.Path(path_type=Path),
    default=None,
    help="Model directory containing config.min.json "
    "(default: assets/models/<active_model> from assets/active_model.txt).",
)
def main(check: bool, self_test: bool, model_dir: Path | None) -> None:
    """Generate assets/content_types_kb.min.json from YAML, Rust resolvers, and model config."""
    if check and self_test:
        raise click.UsageError("--check and --self-test are mutually exclusive.")

    if self_test:
        sys.exit(run_self_test())

    yaml_display_path = _display_path(CONTENT_TYPES_YAML_PATH)
    kb_display_path = _display_path(CONTENT_TYPES_KB_PATH)
    try:
        resolver_sources = _read_resolver_sources(RESOLVERS_RS_PATH, RESOLVERS_DIR)
        resolved_model_dir = (
            model_dir
            if model_dir is not None
            else _resolve_default_model_dir(ACTIVE_MODEL_PATH, MODELS_DIR)
        )
        model_config, model_display_path = _read_model_config(resolved_model_dir)
        kb_json = generate_kb_json(
            CONTENT_TYPES_YAML_PATH.read_text(encoding="utf-8"),
            resolver_sources,
            model_config,
            source_name=yaml_display_path,
            model_source_name=model_display_path,
            require_heuristics=True,
        )
    except KbError as e:
        click.echo(f"ERROR: {e}", err=True)
        sys.exit(1)
    kb_bytes = kb_json.encode("ascii")

    if check:
        if (
            not CONTENT_TYPES_KB_PATH.is_file()
            or CONTENT_TYPES_KB_PATH.read_bytes() != kb_bytes
        ):
            click.echo(
                f"ERROR: {kb_display_path} is not in sync with {yaml_display_path}.\n"
                "Run: uv run scripts/sync_kb.py",
                err=True,
            )
            sys.exit(1)
        click.echo(f"{kb_display_path} is in sync with {yaml_display_path}.")
        return

    if (
        CONTENT_TYPES_KB_PATH.is_file()
        and CONTENT_TYPES_KB_PATH.read_bytes() == kb_bytes
    ):
        click.echo(f"{kb_display_path} is already up to date.")
        return
    CONTENT_TYPES_KB_PATH.write_bytes(kb_bytes)
    click.echo(f"Updated {kb_display_path}.")


def _display_path(path: Path) -> str:
    try:
        return str(path.relative_to(REPO_ROOT_DIR))
    except ValueError:
        return str(path)


def _read_resolver_sources(
    resolvers_rs_path: Path, resolvers_dir: Path
) -> dict[str, str]:
    if not resolvers_rs_path.is_file():
        raise KbError(
            f"{_display_path(resolvers_rs_path)}: resolver root file not found"
        )
    if not resolvers_dir.is_dir():
        raise KbError(f"{_display_path(resolvers_dir)}: resolvers directory not found")
    rs_paths = sorted(resolvers_dir.rglob("*.rs"))
    if not rs_paths:
        raise KbError(f"{_display_path(resolvers_dir)}: no .rs files found")

    sources = {
        _display_path(resolvers_rs_path): resolvers_rs_path.read_text(encoding="utf-8")
    }
    for path in rs_paths:
        sources[_display_path(path)] = path.read_text(encoding="utf-8")
    return sources


def _resolve_default_model_dir(active_model_path: Path, models_dir: Path) -> Path:
    if not active_model_path.is_file():
        raise KbError(
            f"{_display_path(active_model_path)}: active model file not found"
        )
    model_name = active_model_path.read_text(encoding="utf-8").strip()
    if not model_name or not ACTIVE_MODEL_NAME_RE.match(model_name):
        raise KbError(
            f"{_display_path(active_model_path)}: expected a single valid model name, got {model_name!r}"
        )
    return models_dir / model_name


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
    resolver_sources: Mapping[str, str],
    model_config: Any,
    source_name: str = "<string>",
    model_source_name: str = "<model_config>",
    *,
    require_heuristics: bool = False,
) -> str:
    """Returns the content of the JSON KB generated from the YAML, resolvers, and model."""
    data = load_yaml(yaml_text, source_name)
    yaml_errors = validate(data)
    if yaml_errors:
        raise KbError(
            f"{source_name}: found {len(yaml_errors)} error(s):\n"
            + "\n".join(f"- {e}" for e in yaml_errors)
        )

    known_labels = set(data.keys())
    derived_errors: list[str] = []
    if require_heuristics:
        missing_heuristics = sorted(HEURISTIC_CONTENT_TYPES - known_labels)
        if missing_heuristics:
            derived_errors.append(
                f"{source_name}: missing built-in heuristic content type(s): {missing_heuristics}"
            )

    in_resolvers_labels, vetoed_labels, resolver_errors = compute_resolver_coverage(
        resolver_sources, known_labels
    )
    in_ml_model_labels, ml_errors = compute_ml_coverage(
        model_config, known_labels, model_source_name
    )
    derived_errors.extend(resolver_errors)
    derived_errors.extend(ml_errors)
    if derived_errors:
        raise KbError(
            f"found {len(derived_errors)} error(s) in resolvers/model config:\n"
            + "\n".join(f"- {e}" for e in derived_errors)
        )

    # The validation guarantees that the content types are sorted and that the
    # fields of each entry are in the expected order.
    kb = {
        name: {
            **{field: entry[field] for field in FIELDS},
            "in_heuristics": name in HEURISTIC_CONTENT_TYPES,
            "in_resolvers": name in in_resolvers_labels,
            "in_ml_model": name in in_ml_model_labels,
            "vetoed_by_resolvers": name in vetoed_labels,
        }
        for name, entry in data.items()
    }
    return json.dumps(kb, separators=(",", ":"), ensure_ascii=True)


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
    if not data:
        return ["top level mapping must not be empty"]

    errors: list[str] = []
    previous_name: str | None = None
    rust_variants: dict[str, str] = {}
    for name, entry in data.items():
        if (
            not CONTENT_TYPE_NAME_RE.match(name)
            or name.startswith("_")
            or name.endswith("_")
            or "__" in name
        ):
            errors.append(
                f"{name!r}: content type name must match {CONTENT_TYPE_NAME_RE.pattern} "
                "without leading, trailing, or consecutive underscores"
            )
        else:
            variant = label_to_rust_variant(name)
            if variant in rust_variants:
                errors.append(
                    f"{name!r}: Rust ContentType variant {variant!r} collides with {rust_variants[variant]!r}"
                )
            else:
                rust_variants[variant] = name

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

    mime_type = entry["mime_type"]
    if mime_type is not None:
        if not (isinstance(mime_type, str) and mime_type):
            errors.append(
                f"mime_type: must be a non-empty string or null, got {mime_type!r} ({_type_name(mime_type)})"
            )
        elif not MIME_TYPE_RE.match(mime_type):
            errors.append(
                f"mime_type: {mime_type!r} must be a valid 'type/subtype' media type"
            )

    group = entry["group"]
    if group is not None:
        if not (isinstance(group, str) and group):
            errors.append(
                f"group: must be a non-empty string or null, got {group!r} ({_type_name(group)})"
            )
        elif group not in VALID_GROUPS:
            errors.append(
                f"group: unknown group {group!r} (expected one of {sorted(VALID_GROUPS)})"
            )

    description = entry["description"]
    if description is not None:
        if not (isinstance(description, str) and description):
            errors.append(
                f"description: must be a non-empty string or null, got {description!r} ({_type_name(description)})"
            )
        elif description != description.strip():
            errors.append(
                f"description: {description!r} must not have leading or trailing whitespace"
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
            elif not EXTENSION_RE.match(ext):
                errors.append(
                    f"extensions[{i}]: {ext!r} contains invalid extension characters"
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


def label_to_rust_variant(label: str) -> str:
    """Converts a snake_case KB label into its Rust `ContentType` enum variant."""
    parts: list[str] = []
    for i, word in enumerate(label.split("_")):
        if i == 0 and not word[0].isalpha():
            parts.append("_")
        parts.append(word[0].upper() + word[1:])
    return "".join(parts)


def _type_name(value: Any) -> str:
    return "null" if value is None else type(value).__name__


def compute_resolver_coverage(
    resolver_sources: Mapping[str, str], known_labels: set[str]
) -> tuple[set[str], set[str], list[str]]:
    """Extracts `in_resolvers` and `vetoed_by_resolvers` from Rust resolver sources."""
    if not resolver_sources:
        return set(), set(), ["no Rust resolver source files provided"]

    variant_to_label = {label_to_rust_variant(label): label for label in known_labels}
    errors: list[str] = []
    in_resolvers: set[str] = set()
    vetoed_by_resolvers: set[str] = set()
    vetos_found_in: list[str] = []

    for source_name, raw_text in sorted(resolver_sources.items()):
        no_comments, lex_errors = _strip_rust_comments_and_literals(
            raw_text, source_name
        )
        if lex_errors:
            errors.extend(lex_errors)
            continue

        active_code, cfg_errors = _strip_cfg_test_items(no_comments, source_name)
        if cfg_errors:
            errors.extend(cfg_errors)
            continue

        enum_names = {"ContentType"}
        for alias_match in _CONTENT_TYPE_USE_ALIAS_RE.finditer(active_code):
            if alias_match.group(1) != "_":
                enum_names.add(alias_match.group(1))
        for alias_match in _CONTENT_TYPE_TYPE_ALIAS_RE.finditer(active_code):
            if alias_match.group(1) != "_":
                enum_names.add(alias_match.group(1))
        names_group = "|".join(re.escape(name) for name in sorted(enum_names))

        glob_re = re.compile(rf"\b(?:{names_group})\s*::\s*\*")
        for glob_match in glob_re.finditer(active_code):
            errors.append(
                f"{source_name}: glob imports of ContentType variants "
                f"({glob_match.group(0)!r}) are not allowed in resolvers; "
                "qualify variants explicitly"
            )

        code_without_vetos, file_vetos, file_veto_locs, veto_errors = (
            _extract_and_blank_vetos(
                active_code, source_name, variant_to_label, names_group
            )
        )
        errors.extend(veto_errors)
        vetos_found_in.extend(file_veto_locs)
        vetoed_by_resolvers.update(file_vetos)

        variant_re = re.compile(rf"\b({names_group})\s*::\s*([A-Za-z0-9_]+)\b")
        for match in variant_re.finditer(code_without_vetos):
            prefix, variant = match.group(1), match.group(2)
            label = variant_to_label.get(variant)
            if label is None:
                errors.append(
                    f"{source_name}: unknown {prefix}::{variant} (not in the YAML KB)"
                )
            else:
                in_resolvers.add(label)

    if not vetos_found_in and not errors:
        errors.append("const VETOS: &[ContentType] = &[...]; not found in resolvers")
    elif len(vetos_found_in) > 1:
        errors.append(
            f"multiple const VETOS definitions found in resolvers: {vetos_found_in}"
        )

    vetos_without_resolver = sorted(vetoed_by_resolvers - in_resolvers)
    for label in vetos_without_resolver:
        vetos_loc = vetos_found_in[0] if vetos_found_in else "<resolvers>"
        errors.append(
            f"{vetos_loc}: {label!r} is in VETOS (vetoed_by_resolvers) "
            "but is never emitted by resolvers (in_resolvers)"
        )

    return in_resolvers, vetoed_by_resolvers, errors


def _strip_rust_comments_and_literals(
    text: str, source_name: str
) -> tuple[str, list[str]]:
    """Replaces Rust comments and string/char literal bodies with spaces."""
    out: list[str] = []
    n = len(text)
    i = 0

    def _is_prefix_boundary(pos: int) -> bool:
        return pos == 0 or not (text[pos - 1].isalnum() or text[pos - 1] in "_'")

    while i < n:
        # Line comment `// ...`
        if text.startswith("//", i):
            out.append("  ")
            i += 2
            while i < n and text[i] != "\n":
                out.append(" ")
                i += 1
            continue

        # Nested block comment `/* ... */`
        if text.startswith("/*", i):
            depth = 1
            out.append("  ")
            i += 2
            while i < n and depth > 0:
                if text.startswith("/*", i):
                    depth += 1
                    out.append("  ")
                    i += 2
                elif text.startswith("*/", i):
                    depth -= 1
                    out.append("  ")
                    i += 2
                else:
                    out.append("\n" if text[i] == "\n" else " ")
                    i += 1
            if depth > 0:
                return "", [f"{source_name}: unterminated block comment"]
            continue

        # Raw string / byte-string / c-string literal `r#"..."#`, `br#"..."#`, `cr#"..."#`
        if _is_prefix_boundary(i):
            raw_prefix_len = (
                2
                if text[i : i + 2] in ("br", "rb", "cr", "rc")
                else (1 if text[i] == "r" else 0)
            )
            if raw_prefix_len > 0:
                j = i + raw_prefix_len
                while j < n and text[j] == "#":
                    j += 1
                num_hashes = j - (i + raw_prefix_len)
                if j < n and text[j] == '"':
                    closing = '"' + ("#" * num_hashes)
                    out.append((" " * (j - i)) + '"')
                    k = j + 1
                    while k < n and not text.startswith(closing, k):
                        out.append("\n" if text[k] == "\n" else " ")
                        k += 1
                    if k >= n:
                        return "", [f"{source_name}: unterminated raw string literal"]
                    out.append('"' + (" " * num_hashes))
                    i = k + len(closing)
                    continue

        # Regular string / byte-string / c-string literal `"..."`, `b"..."`, `c"..."`
        is_prefixed_str = (
            text[i] in ("b", "c")
            and i + 1 < n
            and text[i + 1] == '"'
            and _is_prefix_boundary(i)
        )
        if is_prefixed_str or text[i] == '"':
            if is_prefixed_str:
                out.append(" ")
                i += 1
            out.append('"')
            i += 1
            while i < n and text[i] != '"':
                if text[i] == "\\":
                    if i + 1 >= n:
                        return "", [f"{source_name}: unterminated string literal"]
                    out.append(" ")
                    out.append("\n" if text[i + 1] == "\n" else " ")
                    i += 2
                else:
                    out.append("\n" if text[i] == "\n" else " ")
                    i += 1
            if i >= n:
                return "", [f"{source_name}: unterminated string literal"]
            out.append('"')
            i += 1
            continue

        # Byte character literal `b'...'`
        if (
            text[i] == "b"
            and i + 1 < n
            and text[i + 1] == "'"
            and _is_prefix_boundary(i)
        ):
            end = _scan_char_literal_end(text, i + 1)
            if end is None:
                return "", [f"{source_name}: unterminated byte character literal"]
            out.append(" '" + (" " * (end - i - 2)) + "'")
            i = end + 1
            continue

        # Character literal `'c'` / `'\...'` (leaving lifetimes `'a` untouched)
        if text[i] == "'":
            end = _scan_char_literal_end(text, i)
            if end is not None:
                out.append("'" + (" " * (end - i - 1)) + "'")
                i = end + 1
                continue
            out.append("'")
            i += 1
            continue

        out.append(text[i])
        i += 1

    return "".join(out), []


def _scan_char_literal_end(text: str, quote_pos: int) -> int | None:
    """Returns the index of the closing `'` for a char literal at `quote_pos`, or `None` if it is a lifetime."""
    n = len(text)
    i = quote_pos + 1
    if i >= n:
        return None
    if text[i] == "\\":
        # Consume `\` and the escaped character unconditionally (handles `'\''` and `'\\'`).
        if i + 2 >= n or text[i + 1] == "\n":
            return None
        j = i + 2
        while j < n and text[j] not in ("'", "\n"):
            j += 1
        return j if j < n and text[j] == "'" else None
    if i + 1 < n and text[i] not in ("'", "\n") and text[i + 1] == "'":
        return i + 1
    return None


def _strip_cfg_test_items(text: str, source_name: str) -> tuple[str, list[str]]:
    """Replaces `#[cfg(test)]`-annotated Rust items with spaces."""
    buf = text
    pos = 0
    while True:
        match = _CFG_TEST_RE.search(buf, pos)
        if match is None:
            return buf, []

        start = match.start()
        idx = match.end()
        n = len(buf)

        # Skip any chained outer attributes `#[...]` before the annotated item.
        while idx < n:
            idx = _skip_ws(buf, idx)
            if idx < n and buf[idx] == "#":
                bracket_pos = _skip_ws(buf, idx + 1)
                if bracket_pos < n and buf[bracket_pos] == "[":
                    end_attr = _find_matching_delim(buf, bracket_pos, "[", "]")
                    if end_attr is None:
                        return "", [
                            f"{source_name}: unterminated attribute after #[cfg(test)]"
                        ]
                    idx = end_attr + 1
                    continue
            break

        keyword, body_scan_pos = _read_rust_item_keyword(buf, idx)
        end = _find_rust_item_end(buf, body_scan_pos, keyword)
        if end is None:
            return "", [f"{source_name}: unterminated #[cfg(test)] item"]

        blanked = re.sub(r"[^\n]", " ", buf[start:end])
        buf = buf[:start] + blanked + buf[end:]
        pos = end


def _skip_ws(text: str, pos: int) -> int:
    n = len(text)
    while pos < n and text[pos].isspace():
        pos += 1
    return pos


def _find_matching_delim(
    text: str, open_pos: int, open_ch: str, close_ch: str
) -> int | None:
    depth = 1
    i = open_pos + 1
    n = len(text)
    while i < n and depth > 0:
        if text[i] == open_ch:
            depth += 1
        elif text[i] == close_ch:
            depth -= 1
            if depth == 0:
                return i
        i += 1
    return None


def _read_rust_item_keyword(text: str, pos: int) -> tuple[str, int]:
    """Skips Rust visibility/item modifiers and returns `(keyword, scan_start)`."""
    n = len(text)
    idx = _skip_ws(text, pos)

    # Optional `pub` / `pub(...)`
    ident = _IDENT_RE.match(text, idx)
    if ident is not None and ident.group(0) == "pub":
        idx = _skip_ws(text, ident.end())
        if idx < n and text[idx] == "(":
            close_paren = _find_matching_delim(text, idx, "(", ")")
            if close_paren is not None:
                idx = _skip_ws(text, close_paren + 1)

    # Optional `async` / `unsafe` / `extern "..."` / `const` (when modifying `fn`)
    while idx < n:
        ident = _IDENT_RE.match(text, idx)
        if ident is None:
            return "", idx
        word = ident.group(0)
        next_idx = _skip_ws(text, ident.end())
        if word in ("async", "unsafe"):
            idx = next_idx
            continue
        if word == "extern":
            if next_idx + 1 < n and text[next_idx : next_idx + 2] == '""':
                next_idx = _skip_ws(text, next_idx + 2)
            idx = next_idx
            continue
        if word == "const":
            next_ident = _IDENT_RE.match(text, next_idx)
            if next_ident is not None and next_ident.group(0) in ("fn", "unsafe"):
                idx = next_idx
                continue
        return word, idx

    return "", idx


def _find_rust_item_end(text: str, pos: int, keyword: str) -> int | None:
    """Finds the exclusive end index of a Rust item or annotated expression."""
    n = len(text)
    paren = 0
    bracket = 0
    brace = 0
    block_item = keyword in ("mod", "fn", "struct", "enum", "union", "trait", "impl")
    semi_item = keyword in ("use", "const", "static", "type", "let")

    i = pos
    while i < n:
        ch = text[i]
        if ch == "(":
            paren += 1
        elif ch == ")":
            if (
                paren == 0
                and bracket == 0
                and brace == 0
                and not (block_item or semi_item)
            ):
                return i
            paren = max(0, paren - 1)
        elif ch == "[":
            bracket += 1
        elif ch == "]":
            if (
                paren == 0
                and bracket == 0
                and brace == 0
                and not (block_item or semi_item)
            ):
                return i
            bracket = max(0, bracket - 1)
        elif ch == "{":
            brace += 1
        elif ch == "}":
            if (
                paren == 0
                and bracket == 0
                and brace == 0
                and not (block_item or semi_item)
            ):
                return i
            brace = max(0, brace - 1)
            if block_item and paren == 0 and bracket == 0 and brace == 0:
                return i + 1
        elif ch == ";" and paren == 0 and bracket == 0 and brace == 0:
            return i + 1
        elif (
            ch == ","
            and paren == 0
            and bracket == 0
            and brace == 0
            and not (block_item or semi_item)
        ):
            return i + 1
        i += 1

    return None


def _extract_and_blank_vetos(
    active_code: str,
    source_name: str,
    variant_to_label: Mapping[str, str],
    names_group: str,
) -> tuple[str, set[str], list[str], list[str]]:
    """Extracts `const VETOS` labels and blanks the declaration from `active_code`."""
    veto_labels: set[str] = set()
    veto_locs: list[str] = []
    errors: list[str] = []
    seen_variants: set[str] = set()
    vetos_decl_re = re.compile(
        rf"\bconst\s+VETOS\s*:\s*&\s*\[\s*(?:{names_group})\s*\]\s*=\s*&\s*\[(.*?)\]\s*;",
        re.DOTALL,
    )
    vetos_item_re = re.compile(rf"^({names_group})\s*::\s*([A-Za-z0-9_]+)$")

    def _replace_vetos(match: re.Match[str]) -> str:
        veto_locs.append(source_name)
        body = match.group(1)
        for raw_item in body.split(","):
            item = raw_item.strip()
            if not item:
                continue
            item_match = vetos_item_re.match(item)
            if item_match is None:
                errors.append(
                    f"{source_name}: invalid entry {item!r} in VETOS "
                    "(expected ContentType::<Variant>)"
                )
                continue
            prefix, variant = item_match.group(1), item_match.group(2)
            if variant in seen_variants:
                errors.append(f"{source_name}: duplicate {prefix}::{variant} in VETOS")
                continue
            seen_variants.add(variant)
            label = variant_to_label.get(variant)
            if label is None:
                errors.append(
                    f"{source_name}: unknown {prefix}::{variant} in VETOS "
                    "(not in the YAML KB)"
                )
            else:
                veto_labels.add(label)
        return re.sub(r"[^\n]", " ", match.group(0))

    blanked_code = vetos_decl_re.sub(_replace_vetos, active_code)
    return blanked_code, veto_labels, veto_locs, errors


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

_SELF_TEST_VALID_RESOLVERS: dict[str, str] = {
    "rust/lib/src/resolvers.rs": "pub(crate) const VETOS: &[ContentType] = &[];\n",
}
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
        "extension with invalid slash",
        _SELF_TEST_VALID_YAML.replace('"txt", ', '"a/b", '),
        "txt: extensions[0]: 'a/b' contains invalid extension characters",
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
        "untrimmed description",
        _SELF_TEST_VALID_YAML.replace(
            'description: "Generic text document"',
            'description: " Generic text document "',
        ),
        "must not have leading or trailing whitespace",
    ),
    (
        "invalid mime_type without slash",
        _SELF_TEST_VALID_YAML.replace(
            'mime_type: "text/plain"', 'mime_type: "textplain"'
        ),
        "txt: mime_type: 'textplain' must be a valid 'type/subtype' media type",
    ),
    (
        "invalid mime_type with parameter",
        _SELF_TEST_VALID_YAML.replace(
            'mime_type: "text/plain"', 'mime_type: "text/plain; charset=utf-8"'
        ),
        "must be a valid 'type/subtype' media type",
    ),
    (
        "unknown group",
        _SELF_TEST_VALID_YAML.replace('group: "text"', 'group: "not_a_group"'),
        "txt: group: unknown group 'not_a_group'",
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
        "leading underscore in content type name",
        _SELF_TEST_VALID_YAML.replace('"null":', '"_null":'),
        "'_null': content type name must match",
    ),
    (
        "colliding Rust enum variants",
        _SELF_TEST_VALID_YAML.replace('"null":', '"foo3d":').replace(
            '"txt":', '"foo_3d":'
        ),
        "'foo_3d': Rust ContentType variant 'Foo3d' collides with 'foo3d'",
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

# (description, resolver_sources, model_config, substring expected in error).
_SELF_TEST_INVALID_DERIVED_CASES: tuple[tuple[str, dict[str, str], Any, str], ...] = (
    (
        "missing const VETOS declaration",
        {"rust/lib/src/resolvers.rs": "fn empty() {}\n"},
        _SELF_TEST_VALID_MODEL_CONFIG,
        "const VETOS: &[ContentType] = &[...]; not found in resolvers",
    ),
    (
        "multiple const VETOS declarations",
        {
            "rust/lib/src/resolvers.rs": "pub(crate) const VETOS: &[ContentType] = &[];\n",
            "rust/lib/src/resolvers/extra.rs": "pub(crate) const VETOS: &[ContentType] = &[];\n",
        },
        _SELF_TEST_VALID_MODEL_CONFIG,
        "multiple const VETOS definitions found",
    ),
    (
        "VETOS entry not emitted by resolvers",
        {
            "rust/lib/src/resolvers.rs": (
                "pub(crate) const VETOS: &[ContentType] = &[ContentType::Null];\n"
            ),
            "rust/lib/src/resolvers/magic/mod.rs": (
                "// ContentType::Null in comment does not count!\n"
                "#[cfg(test)]\n"
                "fn test_only() -> ContentType { ContentType::Null }\n"
            ),
        },
        _SELF_TEST_VALID_MODEL_CONFIG,
        "'null' is in VETOS (vetoed_by_resolvers) but is never emitted by resolvers (in_resolvers)",
    ),
    (
        "duplicate entry in VETOS",
        {
            "rust/lib/src/resolvers.rs": (
                "pub(crate) const VETOS: &[ContentType] = &[\n"
                "    ContentType::Null,\n"
                "    ContentType::Null,\n"
                "];\n"
                "fn emit() -> ContentType { ContentType::Null }\n"
            ),
        },
        _SELF_TEST_VALID_MODEL_CONFIG,
        "duplicate ContentType::Null in VETOS",
    ),
    (
        "malformed entry in VETOS",
        {
            "rust/lib/src/resolvers.rs": (
                "pub(crate) const VETOS: &[ContentType] = &[OTHER_CONST];\n"
            ),
        },
        _SELF_TEST_VALID_MODEL_CONFIG,
        "invalid entry 'OTHER_CONST' in VETOS",
    ),
    (
        "unknown ContentType variant in VETOS",
        {
            "rust/lib/src/resolvers.rs": (
                "pub(crate) const VETOS: &[ContentType] = &[ContentType::Nope];\n"
            ),
        },
        _SELF_TEST_VALID_MODEL_CONFIG,
        "unknown ContentType::Nope in VETOS",
    ),
    (
        "unknown ContentType variant in resolver",
        {
            "rust/lib/src/resolvers.rs": "pub(crate) const VETOS: &[ContentType] = &[];\n",
            "rust/lib/src/resolvers/magic/mod.rs": (
                "use crate::ContentType as Label;\n"
                "pub(crate) fn f() -> Label { Label::Nope }\n"
            ),
        },
        _SELF_TEST_VALID_MODEL_CONFIG,
        "unknown Label::Nope (not in the YAML KB)",
    ),
    (
        "glob import of ContentType variants is rejected",
        {
            "rust/lib/src/resolvers.rs": "pub(crate) const VETOS: &[ContentType] = &[];\n",
            "rust/lib/src/resolvers/magic/mod.rs": (
                "use crate::ContentType as Label;\nuse Label::*;\n"
            ),
        },
        _SELF_TEST_VALID_MODEL_CONFIG,
        "glob imports of ContentType variants ('Label::*') are not allowed in resolvers",
    ),
    (
        "unterminated block comment in resolver",
        {
            "rust/lib/src/resolvers.rs": "/* outer /* nested */ still open\n",
        },
        _SELF_TEST_VALID_MODEL_CONFIG,
        "unterminated block comment",
    ),
    (
        "unknown label in target_labels_space",
        _SELF_TEST_VALID_RESOLVERS,
        {"target_labels_space": ["txt", "nope"], "overwrite_map": {}},
        "target_labels_space label 'nope' is not in the YAML KB",
    ),
    (
        "unknown key in overwrite_map",
        _SELF_TEST_VALID_RESOLVERS,
        {"target_labels_space": ["txt"], "overwrite_map": {"nope": "txt"}},
        "overwrite_map key 'nope' is not in the YAML KB",
    ),
    (
        "unknown value in overwrite_map",
        _SELF_TEST_VALID_RESOLVERS,
        {"target_labels_space": ["txt"], "overwrite_map": {"null": "nope"}},
        "overwrite_map value 'nope' (for 'null') is not in the YAML KB",
    ),
)


def run_self_test() -> int:
    """Runs the validation self-tests and returns the process exit code."""
    failures: list[str] = []

    expected_valid_json = (
        '{"null":{"mime_type":null,"group":null,"description":null,'
        '"extensions":["null"],"is_text":false,'
        '"in_heuristics":false,"in_resolvers":false,"in_ml_model":false,"vetoed_by_resolvers":false},'
        '"txt":{"mime_type":"text/plain","group":"text",'
        '"description":"Generic text document","extensions":["txt","text"],"is_text":true,'
        '"in_heuristics":true,"in_resolvers":false,"in_ml_model":true,"vetoed_by_resolvers":false}}'
    )
    try:
        valid_json = generate_kb_json(
            _SELF_TEST_VALID_YAML,
            _SELF_TEST_VALID_RESOLVERS,
            _SELF_TEST_VALID_MODEL_CONFIG,
        )
    except KbError as e:
        failures.append(f"valid source: unexpected error: {e}")
    else:
        if valid_json != expected_valid_json:
            failures.append(f"valid source: unexpected output: {valid_json}")

    # Test Rust comment/literal/`#[cfg(test)]` stripping, `ContentType as Label`
    # alias resolution, `_3gp` / `LlvmBitcode` variant mapping, case-sensitive
    # extensions, VETOS, and `overwrite_map`.
    complex_yaml = """\
"3gp":
  mime_type: "video/3gpp"
  group: "video"
  description: "3GPP multimedia file"
  extensions: ["3gp", "3GPP"]
  is_text: false

"llvm_bitcode":
  mime_type: "application/x-llvm"
  group: "executable"
  description: "LLVM bitcode"
  extensions: ["bc"]
  is_text: false

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
  extensions: ["txt"]
  is_text: true
"""
    complex_resolvers = {
        "rust/lib/src/resolvers.rs": (
            "pub(crate) const VETOS: &[ContentType] = &[\n"
            "    ContentType::_3gp,\n"
            "    // ContentType::Txt,\n"
            "];\n"
            "#[cfg(test)]\n"
            "mod tests {\n"
            "    fn fake() -> ContentType { ContentType::Txt }\n"
            "}\n"
        ),
        "rust/lib/src/resolvers/facts/mod.rs": (
            "#[cfg(test)]\n"
            "pub(crate) use self::zip::tests::{archive, stored};\n"
            "#[cfg(test)]\n"
            "const FAKE: [ContentType; 1] = [ContentType::Null];\n"
            "pub(crate) fn scan<'a>(buf: &'a [u8]) -> Option<ContentType> {\n"
            "    let _ = (b'\\'', b'\"', '\"', '\\'', b'\\\\');\n"
            '    let _ = "ContentType::Null";\n'
            '    let _ = r#"ContentType::Null"#;\n'
            '    let _ = br#"ContentType::Txt"#;\n'
            "    /* outer /* nested ContentType::Null */ ContentType::Txt */\n"
            "    // ContentType::Null\n"
            "    if buf.is_empty() { return Some(ContentType::_3gp); }\n"
            "    None\n"
            "}\n"
        ),
        "rust/lib/src/resolvers/magic/mod.rs": (
            "use crate::ContentType as Label;\n"
            "#[cfg(test)]\n"
            "pub(crate) mod notworking;\n"
            "#[cfg(test)]\n"
            "#[allow(dead_code)]\n"
            "fn disabled() -> Label { Label::Null }\n"
            "pub(crate) fn identify() -> Option<Label> {\n"
            "    Some(Label::LlvmBitcode)\n"
            "}\n"
        ),
    }
    complex_model_config = {
        "target_labels_space": ["3gp", "null", "txt"],
        "overwrite_map": {"null": "txt"},
    }
    try:
        complex_kb = json.loads(
            generate_kb_json(complex_yaml, complex_resolvers, complex_model_config)
        )
    except KbError as e:
        failures.append(f"complex resolver source: unexpected error: {e}")
    else:
        checks = (
            complex_kb["3gp"]["in_resolvers"] is True,
            complex_kb["3gp"]["vetoed_by_resolvers"] is True,
            complex_kb["3gp"]["in_ml_model"] is True,
            complex_kb["3gp"]["in_heuristics"] is False,
            complex_kb["llvm_bitcode"]["in_resolvers"] is True,
            complex_kb["llvm_bitcode"]["vetoed_by_resolvers"] is False,
            complex_kb["llvm_bitcode"]["in_ml_model"] is False,
            complex_kb["null"]["in_resolvers"] is False,
            complex_kb["null"]["vetoed_by_resolvers"] is False,
            complex_kb["null"]["in_ml_model"] is False,
            complex_kb["txt"]["in_heuristics"] is True,
            complex_kb["txt"]["in_resolvers"] is False,
            complex_kb["txt"]["in_ml_model"] is True,
        )
        if not all(checks):
            failures.append(f"complex resolver source: unexpected output: {complex_kb}")

    # Test `require_heuristics=True` rejects missing built-in heuristic entries.
    try:
        generate_kb_json(
            _SELF_TEST_VALID_YAML,
            _SELF_TEST_VALID_RESOLVERS,
            _SELF_TEST_VALID_MODEL_CONFIG,
            require_heuristics=True,
        )
    except KbError as e:
        if "missing built-in heuristic content type(s)" not in str(e):
            failures.append(f"require_heuristics check: unexpected error: {e}")
    else:
        failures.append("require_heuristics check: expected an error, got none")

    for description, yaml_text, expected_error in _SELF_TEST_INVALID_CASES:
        try:
            generate_kb_json(
                yaml_text,
                _SELF_TEST_VALID_RESOLVERS,
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
        resolver_sources,
        model_config,
        expected_error,
    ) in _SELF_TEST_INVALID_DERIVED_CASES:
        try:
            generate_kb_json(_SELF_TEST_VALID_YAML, resolver_sources, model_config)
        except KbError as e:
            if expected_error not in str(e):
                failures.append(
                    f"{description}: expected error containing {expected_error!r}, got: {e}"
                )
        else:
            failures.append(f"{description}: expected an error, got none")

    total = 3 + len(_SELF_TEST_INVALID_CASES) + len(_SELF_TEST_INVALID_DERIVED_CASES)
    for failure in failures:
        click.echo(f"FAIL: {failure}", err=True)
    click.echo(f"{total - len(failures)}/{total} self-tests passed.")
    return 1 if failures else 0


if __name__ == "__main__":
    main()
