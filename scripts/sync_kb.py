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
"""Synchronize the content types KB, language bindings, and active model README.

`assets/content_types.yaml` is the human-editable source of truth for content
type metadata. This script validates the YAML, derives the 4 boolean coverage
fields (`in_heuristics`, `in_resolvers`, `in_ml_model`, `vetoed_by_resolvers`)
from built-in heuristics, the Rust resolvers (`rust/lib/src/resolvers/`), and
the active ML model config (`assets/active_model.txt`), maintains the monotonic
`assets/exposed_content_types.txt` allowlist, and deterministically generates
`assets/content_types_kb.min.json`, Rust/C FFI/Python/JS bindings, and the
active model's `README.md` in a single pure-Python pass.

Usage (from anywhere in the repository):

    uv run scripts/sync_kb.py               # Regenerate all KB artifacts.
    uv run scripts/sync_kb.py --check       # Fail if any KB artifact is stale.
    uv run scripts/sync_kb.py --self-test   # Run the validation self-tests.
"""

from __future__ import annotations

import io
import json
import re
import sys
from collections.abc import Mapping, Sequence
from pathlib import Path
from typing import Any

import click
import yaml

try:
    from yaml import CSafeLoader as _BaseSafeLoader
except ImportError:  # pragma: no cover - fallback when libyaml is unavailable
    from yaml import SafeLoader as _BaseSafeLoader

REPO_ROOT_DIR = Path(__file__).resolve().parent.parent
ASSETS_DIR = REPO_ROOT_DIR / "assets"
CONTENT_TYPES_YAML_PATH = ASSETS_DIR / "content_types.yaml"
CONTENT_TYPES_KB_PATH = ASSETS_DIR / "content_types_kb.min.json"
EXPOSED_CONTENT_TYPES_PATH = ASSETS_DIR / "exposed_content_types.txt"
ACTIVE_MODEL_PATH = ASSETS_DIR / "active_model.txt"
MODELS_DIR = ASSETS_DIR / "models"
RESOLVERS_RS_PATH = REPO_ROOT_DIR / "rust" / "lib" / "src" / "resolvers.rs"
RESOLVERS_DIR = REPO_ROOT_DIR / "rust" / "lib" / "src" / "resolvers"

RUST_LIB_CONTENT_RS_PATH = REPO_ROOT_DIR / "rust" / "lib" / "src" / "content.rs"
RUST_LIB_MODEL_RS_PATH = REPO_ROOT_DIR / "rust" / "lib" / "src" / "model.rs"
RUST_FFI_CONTENT_RS_PATH = REPO_ROOT_DIR / "rust" / "ffi" / "src" / "content.rs"
PYTHON_CONTENT_TYPE_LABEL_PATH = (
    REPO_ROOT_DIR / "python" / "src" / "magika" / "types" / "content_type_label.py"
)
JS_CONTENT_TYPE_LABEL_PATH = REPO_ROOT_DIR / "js" / "src" / "content-type-label.ts"
JS_CONTENT_TYPES_INFOS_PATH = REPO_ROOT_DIR / "js" / "src" / "content-types-infos.ts"

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
# Non-regular file types represented on Rust's `FileType` enum rather than `ContentType`.
SPECIAL_FILE_TYPE_LABELS = frozenset({"directory", "symlink", "unsupported"})

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

_RUST_GENERATED_HEADER = """\
// Copyright 2024 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

// DO NOT EDIT, see link below for more information:
// https://github.com/google/magika/blob/main/scripts/sync_kb.py
"""

_PYTHON_GENERATED_PREFIX = '''\
# Copyright 2024 Google LLC
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

"""Enumeration of exposed Magika content type labels."""

from magika.types.strenum import StrEnum

# NOTE: DO NOT EDIT --- This file is automatically generated by scripts/sync_kb.py.


class ContentTypeLabel(StrEnum):
    """A string-based enumeration of exposed Magika content type labels.

    This enum provides a standardized set of identifiers for content types
    recognized by Magika.
    """
'''

_TS_GENERATED_HEADER = """\
// Copyright 2024 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

// NOTE: DO NOT EDIT --- This file is automatically generated by scripts/sync_kb.py.
"""


class KbError(Exception):
    """The YAML source, resolver sources, exposed list, or model config is invalid."""


@click.command()
@click.option(
    "--check",
    is_flag=True,
    help="Do not write anything; exit with an error if any generated KB artifact is stale.",
)
@click.option(
    "--self-test",
    is_flag=True,
    help="Run the validation and codegen self-tests and exit.",
)
@click.option(
    "--model-dir",
    type=click.Path(path_type=Path),
    default=None,
    help="Model directory containing config.min.json "
    "(default: assets/models/<active_model> from assets/active_model.txt).",
)
def main(check: bool, self_test: bool, model_dir: Path | None) -> None:
    """Synchronize the content types KB, language bindings, and generated docs."""
    if check and self_test:
        raise click.UsageError("--check and --self-test are mutually exclusive.")

    if self_test:
        sys.exit(run_self_test())

    yaml_display_path = _display_path(CONTENT_TYPES_YAML_PATH)
    exposed_display_path = _display_path(EXPOSED_CONTENT_TYPES_PATH)
    try:
        if not CONTENT_TYPES_YAML_PATH.is_file():
            raise KbError(f"{yaml_display_path}: YAML KB file not found")
        if not EXPOSED_CONTENT_TYPES_PATH.is_file():
            raise KbError(
                f"{exposed_display_path}: exposed content types file not found"
            )
        resolver_sources = _read_resolver_sources(RESOLVERS_RS_PATH, RESOLVERS_DIR)
        if model_dir is not None:
            resolved_model_dir = model_dir
            model_name = model_dir.resolve().name
        else:
            model_name, resolved_model_dir = _resolve_default_model_dir(
                ACTIVE_MODEL_PATH, MODELS_DIR
            )
        model_config, model_display_path = _read_model_config(resolved_model_dir)
        artifacts = build_repo_artifacts(
            yaml_text=CONTENT_TYPES_YAML_PATH.read_text(encoding="utf-8"),
            resolver_sources=resolver_sources,
            model_name=model_name,
            model_config=model_config,
            existing_exposed_text=EXPOSED_CONTENT_TYPES_PATH.read_text(
                encoding="utf-8"
            ),
            resolved_model_dir=resolved_model_dir,
            yaml_source_name=yaml_display_path,
            model_source_name=model_display_path,
            exposed_source_name=exposed_display_path,
        )
    except KbError as e:
        click.echo(f"ERROR: {e}", err=True)
        sys.exit(1)

    stale_paths: list[str] = []
    for path, expected_bytes in artifacts.items():
        if not path.is_file() or path.read_bytes() != expected_bytes:
            stale_paths.append(_display_path(path))

    if check:
        if stale_paths:
            formatted = "\n".join(f"- {p}" for p in stale_paths)
            click.echo(
                f"ERROR: {len(stale_paths)} generated KB artifact(s) out of sync:\n"
                f"{formatted}\n"
                "Run: just sync-kb",
                err=True,
            )
            sys.exit(1)
        click.echo(
            f"All {len(artifacts)} KB artifacts are in sync with {yaml_display_path}."
        )
        return

    if not stale_paths:
        click.echo(f"All {len(artifacts)} KB artifacts are already up to date.")
        return

    for path, expected_bytes in artifacts.items():
        if not path.is_file() or path.read_bytes() != expected_bytes:
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(expected_bytes)
            click.echo(f"Updated {_display_path(path)}.")


def _display_path(path: Path) -> str:
    try:
        return str(path.resolve().relative_to(REPO_ROOT_DIR))
    except ValueError:
        try:
            return str(path.relative_to(REPO_ROOT_DIR))
        except ValueError:
            return str(path)


def _is_test_resolver_path(parts: Sequence[str]) -> bool:
    """Returns True if `parts` points to a test-only module (`**/tests/**` or `**/tests.rs`)."""
    return bool(parts) and ("tests" in parts[:-1] or parts[-1] == "tests.rs")


def _read_resolver_sources(
    resolvers_rs_path: Path, resolvers_dir: Path
) -> dict[str, str]:
    if not resolvers_rs_path.is_file():
        raise KbError(
            f"{_display_path(resolvers_rs_path)}: resolver root file not found"
        )
    if not resolvers_dir.is_dir():
        raise KbError(f"{_display_path(resolvers_dir)}: resolvers directory not found")
    rs_paths = [
        path
        for path in sorted(resolvers_dir.rglob("*.rs"))
        if not _is_test_resolver_path(path.relative_to(resolvers_dir).parts)
    ]
    if not rs_paths:
        raise KbError(f"{_display_path(resolvers_dir)}: no .rs files found")

    sources = {
        _display_path(resolvers_rs_path): resolvers_rs_path.read_text(encoding="utf-8")
    }
    for path in rs_paths:
        sources[_display_path(path)] = path.read_text(encoding="utf-8")
    return sources


def _resolve_default_model_dir(
    active_model_path: Path, models_dir: Path
) -> tuple[str, Path]:
    if not active_model_path.is_file():
        raise KbError(
            f"{_display_path(active_model_path)}: active model file not found"
        )
    model_name = active_model_path.read_text(encoding="utf-8").strip()
    if not model_name or not ACTIVE_MODEL_NAME_RE.match(model_name):
        raise KbError(
            f"{_display_path(active_model_path)}: expected a single valid model name, got {model_name!r}"
        )
    return model_name, models_dir / model_name


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


def build_repo_artifacts(
    *,
    yaml_text: str,
    resolver_sources: Mapping[str, str],
    model_name: str,
    model_config: Any,
    existing_exposed_text: str,
    resolved_model_dir: Path,
    yaml_source_name: str,
    model_source_name: str,
    exposed_source_name: str,
) -> dict[Path, bytes]:
    """Builds all generated KB, language-binding, and documentation artifacts in memory."""
    kb = build_kb(
        yaml_text,
        resolver_sources,
        model_config,
        source_name=yaml_source_name,
        model_source_name=model_source_name,
        require_heuristics=True,
    )
    validate_model_config_for_codegen(model_config, model_source_name)
    exposed_labels, exposed_text = resolve_exposed_content_types(
        existing_exposed_text, kb, model_config, exposed_source_name
    )
    kb_json = json.dumps(kb, separators=(",", ":"), ensure_ascii=True)
    model_readme_path = resolved_model_dir.resolve() / "README.md"

    return {
        EXPOSED_CONTENT_TYPES_PATH: exposed_text.encode("utf-8"),
        CONTENT_TYPES_KB_PATH: kb_json.encode("ascii"),
        RUST_LIB_CONTENT_RS_PATH: generate_rust_lib_content(
            kb, exposed_labels, model_name, model_config
        ).encode("utf-8"),
        RUST_LIB_MODEL_RS_PATH: generate_rust_lib_model(model_config).encode("utf-8"),
        RUST_FFI_CONTENT_RS_PATH: generate_rust_ffi_content(kb, exposed_labels).encode(
            "utf-8"
        ),
        PYTHON_CONTENT_TYPE_LABEL_PATH: generate_python_content_type_label(
            exposed_labels
        ).encode("utf-8"),
        JS_CONTENT_TYPE_LABEL_PATH: generate_js_content_type_label(
            exposed_labels
        ).encode("utf-8"),
        JS_CONTENT_TYPES_INFOS_PATH: generate_js_content_types_infos(
            kb, exposed_labels
        ).encode("utf-8"),
        model_readme_path: generate_model_readme(kb, model_config).encode("utf-8"),
    }


def generate_kb_json(
    yaml_text: str,
    resolver_sources: Mapping[str, str],
    model_config: Any,
    source_name: str = "<string>",
    model_source_name: str = "<model_config>",
    *,
    require_heuristics: bool = False,
) -> str:
    """Returns the minified JSON KB generated from the YAML, resolvers, and model."""
    kb = build_kb(
        yaml_text,
        resolver_sources,
        model_config,
        source_name=source_name,
        model_source_name=model_source_name,
        require_heuristics=require_heuristics,
    )
    return json.dumps(kb, separators=(",", ":"), ensure_ascii=True)


def build_kb(
    yaml_text: str,
    resolver_sources: Mapping[str, str],
    model_config: Any,
    source_name: str = "<string>",
    model_source_name: str = "<model_config>",
    *,
    require_heuristics: bool = False,
) -> dict[str, dict[str, Any]]:
    """Validates inputs and returns the ordered 9-field KB dictionary."""
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

    # Validation guarantees sorted content type keys and canonical field order.
    return {
        name: {
            **{field: entry[field] for field in FIELDS},
            "in_heuristics": name in HEURISTIC_CONTENT_TYPES,
            "in_resolvers": name in in_resolvers_labels,
            "in_ml_model": name in in_ml_model_labels,
            "vetoed_by_resolvers": name in vetoed_labels,
        }
        for name, entry in data.items()
    }


class _StrictLoader(_BaseSafeLoader):
    """A `SafeLoader` that removes YAML's implicit typing pitfalls.

    - Strings must be quoted (the field names, e.g. `mime_type`, are exempt).
      Apart from those, the only unquoted scalars allowed are `null`, `true`,
      and `false`. So YAML 1.1 surprises like `null`, `yes`, `off`, `1.0`, or
      `2024-01-01` silently becoming non-strings can't happen.
    - Duplicate and non-string mapping keys are errors (PyYAML silently keeps
      the last duplicate by default).
    """


class _NamedStringIO(io.StringIO):
    """In-memory text stream carrying a filename for PyYAML/libyaml marks."""

    def __init__(self, text: str, name: str) -> None:
        super().__init__(text)
        self.name = name


def _construct_strict_str(loader: _StrictLoader, node: yaml.ScalarNode) -> str:
    value = loader.construct_scalar(node)
    # `style` is None (SafeLoader) or "" (CSafeLoader) for plain unquoted scalars.
    if not node.style and value not in FIELDS:
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
    loader = _StrictLoader(_NamedStringIO(text, source_name))
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
        elif "\x00" in description or "\n" in description or "\r" in description:
            errors.append(
                f"description: {description!r} must not contain NUL or newline characters"
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


def label_to_screaming_snake(label: str) -> str:
    """Converts a snake_case KB label into its SCREAMING_SNAKE_CASE identifier."""
    upper = label.upper()
    return f"_{upper}" if not label[0].isalpha() else upper


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
        if _is_test_resolver_path(Path(source_name).parts):
            continue
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


def validate_model_config_for_codegen(
    model_config: Any, source_name: str = "<model_config>"
) -> None:
    """Validates all model config fields required to generate Rust `model.rs` & `content.rs`."""
    if not isinstance(model_config, dict):
        raise KbError(
            f"{source_name}: model config must be a JSON object, got {_type_name(model_config)}"
        )

    errors: list[str] = []
    for int_field in (
        "beg_size",
        "mid_size",
        "end_size",
        "min_file_size_for_dl",
        "padding_token",
        "block_size",
        "version_major",
    ):
        value = model_config.get(int_field)
        if not _is_int(value):
            errors.append(
                f"{source_name}: {int_field} must be an integer, got {_type_name(value)}"
            )

    if _is_int(model_config.get("mid_size")) and model_config["mid_size"] != 0:
        errors.append(
            f"{source_name}: unsupported mid_size {model_config['mid_size']} (expected 0)"
        )

    use_offsets = model_config.get("use_inputs_at_offsets")
    if not isinstance(use_offsets, bool):
        errors.append(
            f"{source_name}: use_inputs_at_offsets must be a boolean, got {_type_name(use_offsets)}"
        )
    elif use_offsets:
        errors.append(f"{source_name}: unsupported use_inputs_at_offsets=true")

    medium_threshold = model_config.get("medium_confidence_threshold")
    if not _is_number(medium_threshold):
        errors.append(
            f"{source_name}: medium_confidence_threshold must be a number, got {_type_name(medium_threshold)}"
        )

    target_labels_space = model_config.get("target_labels_space")
    target_set = (
        {x for x in target_labels_space if isinstance(x, str)}
        if isinstance(target_labels_space, list)
        else set()
    )
    for label in sorted(target_set & SPECIAL_FILE_TYPE_LABELS):
        errors.append(
            f"{source_name}: special non-regular file type {label!r} cannot appear in target_labels_space"
        )

    thresholds = model_config.get("thresholds")
    if not isinstance(thresholds, dict):
        errors.append(
            f"{source_name}: thresholds must be a JSON object, got {_type_name(thresholds)}"
        )
    else:
        for label, threshold in thresholds.items():
            if label not in target_set:
                errors.append(
                    f"{source_name}: thresholds key {label!r} is not in target_labels_space"
                )
            if not _is_number(threshold):
                errors.append(
                    f"{source_name}: thresholds[{label!r}] must be a number, got {_type_name(threshold)}"
                )

    overwrite_map = model_config.get("overwrite_map")
    if isinstance(overwrite_map, dict):
        for src, dst in overwrite_map.items():
            if src not in target_set:
                errors.append(
                    f"{source_name}: overwrite_map key {src!r} is not in target_labels_space"
                )
            if dst in SPECIAL_FILE_TYPE_LABELS:
                errors.append(
                    f"{source_name}: special non-regular file type {dst!r} cannot be an overwrite_map target"
                )

    if errors:
        raise KbError(
            f"found {len(errors)} error(s) in {source_name}:\n"
            + "\n".join(f"- {e}" for e in errors)
        )


def resolve_exposed_content_types(
    existing_text: str,
    kb: Mapping[str, Mapping[str, Any]],
    model_config: Mapping[str, Any],
    source_name: str = "<exposed_content_types>",
) -> tuple[list[str], str]:
    """Validates `exposed_content_types.txt` and monotonically appends newly active labels."""
    raw_lines = existing_text.splitlines()
    if not raw_lines:
        raise KbError(f"{source_name}: file must not be empty")

    errors: list[str] = []
    seen: set[str] = set()
    previous: str | None = None
    for line_no, raw_line in enumerate(raw_lines, start=1):
        label = raw_line.strip()
        if not label or label != raw_line:
            errors.append(
                f"{source_name}:{line_no}: invalid empty or whitespace-padded line {raw_line!r}"
            )
            continue
        if label in seen:
            errors.append(f"{source_name}:{line_no}: duplicate content type {label!r}")
            continue
        if previous is not None and label < previous:
            errors.append(
                f"{source_name}:{line_no}: exposed content types must be sorted alphabetically, "
                f"but {label!r} comes after {previous!r}"
            )
        previous = label
        seen.add(label)
        if label not in kb:
            errors.append(
                f"{source_name}:{line_no}: unknown content type {label!r} (not in the YAML KB)"
            )

    if errors:
        raise KbError(
            f"{source_name}: found {len(errors)} error(s):\n"
            + "\n".join(f"- {e}" for e in errors)
        )

    required_labels = {
        label
        for label, info in kb.items()
        if info["in_heuristics"] or info["in_resolvers"] or info["in_ml_model"]
    }
    required_labels.update(model_config["target_labels_space"])
    required_labels.update(model_config["overwrite_map"].values())
    if "undefined" in kb:
        required_labels.add("undefined")

    exposed_labels = sorted(seen | required_labels)
    exposed_text = "".join(f"{label}\n" for label in exposed_labels)
    return exposed_labels, exposed_text


def generate_rust_lib_content(
    kb: Mapping[str, Mapping[str, Any]],
    exposed_labels: Sequence[str],
    model_name: str,
    model_config: Mapping[str, Any],
) -> str:
    """Generates `rust/lib/src/content.rs` (rustfmt-clean)."""
    lines = [
        _RUST_GENERATED_HEADER.rstrip("\n"),
        "",
        "use crate::file::TypeInfo;",
        "",
        "/// Model name (only comparable with equality).",
        f"pub const MODEL_NAME: &str = {_rust_quote(model_name)};",
        "",
        "/// Model major version.",
        f"pub const MODEL_MAJOR_VERSION: u32 = {model_config['version_major']};",
        "",
    ]

    variants: list[tuple[str, str]] = []
    for label in exposed_labels:
        info = kb[label]
        mime_type, group, description = _resolve_entry_metadata(label, info)
        extensions = ", ".join(_rust_quote(ext) for ext in info["extensions"])
        if label not in SPECIAL_FILE_TYPE_LABELS:
            variants.append((label, description))

        lines.extend(
            [
                f"pub(crate) static {label_to_screaming_snake(label)}: TypeInfo = TypeInfo {{",
                f"    label: {_rust_quote(label)},",
                f"    mime_type: {_rust_quote(mime_type)},",
                f"    group: {_rust_quote(group)},",
                f"    description: {_rust_quote(description)},",
                f"    extensions: &[{extensions}],",
                f"    is_text: {_rust_bool(info['is_text'])},",
                f"    in_heuristics: {_rust_bool(info['in_heuristics'])},",
                f"    in_resolvers: {_rust_bool(info['in_resolvers'])},",
                f"    in_ml_model: {_rust_bool(info['in_ml_model'])},",
                f"    vetoed_by_resolvers: {_rust_bool(info['vetoed_by_resolvers'])},",
                "};",
                "",
            ]
        )

    lines.extend(
        [
            "impl TypeInfo {",
            "    /// All file type information ordered by label.",
            f"    pub const ALL: [&'static Self; {len(exposed_labels)}] = [",
            *(
                f"        &{label_to_screaming_snake(label)},"
                for label in exposed_labels
            ),
            "    ];",
            "}",
            "",
            "/// Content types for regular files.",
            "#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]",
            "#[non_exhaustive]",
            "pub enum ContentType {",
        ]
    )
    for label, doc in variants:
        lines.append(f"    /// {doc}")
        lines.append(f"    {label_to_rust_variant(label)},")
    lines.extend(
        [
            "}",
            "",
            "impl ContentType {",
            "    /// Returns a content type from its label.",
            "    pub fn from_label(label: &str) -> Option<Self> {",
            "        Some(match label {",
            *(
                f"            {_rust_quote(label)} => Self::{label_to_rust_variant(label)},"
                for label, _ in variants
            ),
            "            _ => return None,",
            "        })",
            "    }",
            "",
            "    /// Returns the content type information.",
            "    pub fn info(self) -> &'static TypeInfo {",
            "        match self {",
            *(
                f"            ContentType::{label_to_rust_variant(label)} => &{label_to_screaming_snake(label)},"
                for label, _ in variants
            ),
            "        }",
            "    }",
            "}",
            "",
        ]
    )
    return "\n".join(lines)


def generate_rust_lib_model(model_config: Mapping[str, Any]) -> str:
    """Generates `rust/lib/src/model.rs` (rustfmt-clean)."""
    target_labels_space: list[str] = list(model_config["target_labels_space"])
    num_labels = len(target_labels_space)
    label_index = {label: idx for idx, label in enumerate(target_labels_space)}

    default_threshold = float(model_config["medium_confidence_threshold"])
    thresholds_array = [default_threshold] * num_labels
    for label, threshold in model_config["thresholds"].items():
        thresholds_array[label_index[label]] = float(threshold)

    overwrite_array = list(target_labels_space)
    for src, dst in model_config["overwrite_map"].items():
        overwrite_array[label_index[src]] = dst

    thresholds_str = ", ".join(_rust_float(val) for val in thresholds_array)
    lines = [
        _RUST_GENERATED_HEADER.rstrip("\n"),
        "",
        "use crate::ContentType;",
        "use crate::config::ModelConfig;",
        "",
        "pub(crate) const CONFIG: ModelConfig = ModelConfig {",
        f"    beg_size: {model_config['beg_size']},",
        f"    end_size: {model_config['end_size']},",
        f"    medium_confidence_threshold: {_rust_float(default_threshold)},",
        f"    min_file_size_for_dl: {model_config['min_file_size_for_dl']},",
        f"    padding_token: {model_config['padding_token']},",
        f"    block_size: {model_config['block_size']},",
        "};",
        "",
        "#[rustfmt::skip]",
        f"pub(crate) const THRESHOLDS: [f32; {num_labels}] = [{thresholds_str}];",
        "/// Content types output by the model (after overwrite).",
        f"pub const MODEL_LABELS: [ContentType; {num_labels}] = [",
        *(
            f"    ContentType::{label_to_rust_variant(label)},"
            for label in overwrite_array
        ),
        "];",
        "",
        "/// Content types output by the model (before overwrite).",
        f"pub const MODEL_TRAINING_LABELS: [ContentType; {num_labels}] = [",
        *(
            f"    ContentType::{label_to_rust_variant(label)},"
            for label in target_labels_space
        ),
        "];",
        "",
    ]
    return "\n".join(lines)


def generate_rust_ffi_content(
    kb: Mapping[str, Mapping[str, Any]], exposed_labels: Sequence[str]
) -> str:
    """Generates `rust/ffi/src/content.rs` (rustfmt-clean)."""
    lines = [
        _RUST_GENERATED_HEADER.rstrip("\n"),
        "",
        "use std::ptr;",
        "",
        "use crate::MagikaTypeInfo;",
        "",
    ]

    variants: list[str] = []
    for label in exposed_labels:
        info = kb[label]
        mime_type, group, description = _resolve_entry_metadata(label, info)
        if label not in SPECIAL_FILE_TYPE_LABELS:
            variants.append(label)
        ext_ptrs = "".join(
            f"c{_rust_quote(ext)}.as_ptr(), " for ext in info["extensions"]
        )
        extensions_expr = f"[{ext_ptrs}ptr::null()].as_ptr()"

        lines.extend(
            [
                f"#[rustfmt::skip] pub(crate) static {label_to_screaming_snake(label)}: MagikaTypeInfo = MagikaTypeInfo {{",
                f"    label: c{_rust_quote(label)}.as_ptr(),",
                f"    mime_type: c{_rust_quote(mime_type)}.as_ptr(),",
                f"    group: c{_rust_quote(group)}.as_ptr(),",
                f"    description: c{_rust_quote(description)}.as_ptr(),",
                f"    extensions: {extensions_expr},",
                f"    is_text: {_rust_bool(info['is_text'])},",
                "};",
                "",
            ]
        )

    lines.extend(
        [
            "pub(crate) fn content_type_info(content_type: magika::ContentType) -> &'static MagikaTypeInfo {",
            "    match content_type {",
            *(
                f"        magika::ContentType::{label_to_rust_variant(label)} => &{label_to_screaming_snake(label)},"
                for label in variants
            ),
            "        _ => unreachable!(),",
            "    }",
            "}",
            "",
        ]
    )
    return "\n".join(lines)


def generate_python_content_type_label(exposed_labels: Sequence[str]) -> str:
    """Generates `python/src/magika/types/content_type_label.py`."""
    enum_lines = [
        f'    {label_to_screaming_snake(label)} = "{label}"' for label in exposed_labels
    ]
    return (
        _PYTHON_GENERATED_PREFIX
        + "\n"
        + "\n".join(enum_lines)
        + "\n\n    def __repr__(self) -> str:\n        return str(self)\n"
    )


def generate_js_content_type_label(exposed_labels: Sequence[str]) -> str:
    """Generates `js/src/content-type-label.ts`."""
    enum_lines = [
        f'  {label_to_screaming_snake(label)} = "{label}",' for label in exposed_labels
    ]
    return (
        _TS_GENERATED_HEADER
        + "\n"
        + "// Exposed Magika content type labels across heuristics, format resolvers, and\n"
        + "// the ML model (note: the JS binding runs basic heuristics and the ML model).\n"
        + "export enum ContentTypeLabel {\n"
        + "\n".join(enum_lines)
        + "\n}\n"
    )


def generate_js_content_types_infos(
    kb: Mapping[str, Mapping[str, Any]], exposed_labels: Sequence[str]
) -> str:
    """Generates `js/src/content-types-infos.ts`."""
    entry_blocks: list[str] = []
    for label in exposed_labels:
        enum_member = label_to_screaming_snake(label)
        is_text = _rust_bool(kb[label]["is_text"])
        entry_blocks.append(
            f"    [ContentTypeLabel.{enum_member}]: {{\n"
            f"      label: ContentTypeLabel.{enum_member},\n"
            f"      is_text: {is_text},\n"
            f"    }},"
        )

    return (
        _TS_GENERATED_HEADER
        + "\n"
        + 'import { ContentTypeInfo } from "./content-type-info.js";\n'
        + 'import { ContentTypeLabel } from "./content-type-label.js";\n\n'
        + "export type ContentTypesInfos = Record<ContentTypeLabel, ContentTypeInfo>;\n\n"
        + "export const ContentTypesInfos = {\n"
        + "  get: (): ContentTypesInfos => ({\n"
        + "\n".join(entry_blocks)
        + "\n  }),\n};\n"
    )


def generate_model_readme(
    kb: Mapping[str, Mapping[str, Any]],
    model_config: Mapping[str, Any],
) -> str:
    """Generates `assets/models/<active_model>/README.md` for the active ML model."""
    output_labels = [
        label
        for label, info in kb.items()
        if info["in_heuristics"] or info["in_resolvers"] or info["in_ml_model"]
    ]
    raw_model_labels = set(model_config["target_labels_space"])
    if "undefined" in kb:
        raw_model_labels.add("undefined")
    model_output_labels = sorted(raw_model_labels)

    lines = [
        "<!-- DO NOT EDIT: generated by scripts/sync_kb.py -->",
        "",
        "# Model documentation",
        "",
        "## Table of Contents",
        "",
        "1. [List of possible outputs](#list-of-possible-outputs)",
        "1. [List of possible model's outputs](#list-of-possible-models-outputs)",
        "",
        "## List of possible outputs",
        "",
        "This is the full list of all possible tool's outputs (which are different than the possible raw output of the model, see table below). E.g., this is the list of all possible values for Magika python module's `MagikaResult.prediction.output.label`.",
        "",
        "| Index   |      Content Type Label      | Description |",
        "|----------|:-------------:|------|",
    ]
    for idx, label in enumerate(output_labels, start=1):
        _, _, description = _resolve_entry_metadata(label, kb[label])
        lines.append(f"| {idx} | {label} | {_md_escape(description)} |")

    lines.extend(
        [
            "",
            "",
            "## List of possible model's outputs",
            "",
            'This is the full list of all possible model\'s output. E.g., this is the list of all possible values for Magika python module\'s `MagikaResult.prediction.dl.label`. Note that, in general, the list of "model outputs" is different than the "tool outputs" as in some cases the model is not even used, or the model\'s output is overwritten due to a low-confidence score or other reasons. This list is useful mostly for debugging purposes; the vast majority of client should just consult the table above.',
            "",
            "| Index   |      Content Type Label      | Description |",
            "|----------|:-------------:|------|",
        ]
    )
    for idx, label in enumerate(model_output_labels, start=1):
        _, _, description = _resolve_entry_metadata(label, kb[label])
        lines.append(f"| {idx} | {label} | {_md_escape(description)} |")

    lines.append("")
    return "\n".join(lines)


def _resolve_entry_metadata(
    label: str, info: Mapping[str, Any]
) -> tuple[str, str, str]:
    """Resolves nullable KB metadata fields using the canonical `TypeInfo` fallbacks."""
    mime_type = info["mime_type"]
    if mime_type is None:
        mime_type = "text/plain" if info["is_text"] else "application/octet-stream"
    group = info["group"] if info["group"] is not None else "unknown"
    description = info["description"] if info["description"] is not None else label
    return mime_type, group, description


def _is_int(value: Any) -> bool:
    return isinstance(value, int) and not isinstance(value, bool)


def _is_number(value: Any) -> bool:
    return isinstance(value, (int, float)) and not isinstance(value, bool)


def _rust_bool(value: bool) -> str:
    return "true" if value else "false"


def _rust_float(value: float) -> str:
    return repr(float(value))


def _rust_quote(text: str) -> str:
    """Formats `text` as a double-quoted Rust string literal matching `std::fmt::Debug`."""
    escaped: list[str] = ['"']
    for ch in text:
        code = ord(ch)
        if ch == "\\":
            escaped.append("\\\\")
        elif ch == '"':
            escaped.append('\\"')
        elif ch == "\n":
            escaped.append("\\n")
        elif ch == "\r":
            escaped.append("\\r")
        elif ch == "\t":
            escaped.append("\\t")
        elif ch == "\0":
            escaped.append("\\0")
        elif code < 0x20 or code == 0x7F:
            escaped.append(f"\\x{code:02x}")
        else:
            escaped.append(ch)
    escaped.append('"')
    return "".join(escaped)


def _md_escape(text: str) -> str:
    return text.replace("\\", "\\\\").replace("|", "\\|")


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
    "beg_size": 1024,
    "mid_size": 0,
    "end_size": 1024,
    "use_inputs_at_offsets": False,
    "medium_confidence_threshold": 0.5,
    "min_file_size_for_dl": 8,
    "padding_token": 256,
    "block_size": 4096,
    "target_labels_space": ["txt"],
    "thresholds": {},
    "overwrite_map": {},
    "version_major": 3,
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

# (description, existing_exposed_text, substring expected in error).
_SELF_TEST_INVALID_EXPOSED_CASES = (
    ("empty exposed file", "", "file must not be empty"),
    (
        "unsorted exposed file",
        "txt\nnull\n",
        "exposed content types must be sorted alphabetically, but 'null' comes after 'txt'",
    ),
    (
        "duplicate exposed entry",
        "txt\ntxt\n",
        "duplicate content type 'txt'",
    ),
    (
        "unknown exposed entry",
        "nope\ntxt\n",
        "unknown content type 'nope' (not in the YAML KB)",
    ),
    (
        "whitespace-padded exposed entry",
        " txt\n",
        "invalid empty or whitespace-padded line ' txt'",
    ),
)

# (description, model_config override, substring expected in error).
_SELF_TEST_INVALID_MODEL_CODEGEN_CASES: tuple[tuple[str, dict[str, Any], str], ...] = (
    (
        "non-zero mid_size",
        {**_SELF_TEST_VALID_MODEL_CONFIG, "mid_size": 512},
        "unsupported mid_size 512",
    ),
    (
        "use_inputs_at_offsets enabled",
        {**_SELF_TEST_VALID_MODEL_CONFIG, "use_inputs_at_offsets": True},
        "unsupported use_inputs_at_offsets=true",
    ),
    (
        "thresholds key missing from target_labels_space",
        {**_SELF_TEST_VALID_MODEL_CONFIG, "thresholds": {"null": 0.9}},
        "thresholds key 'null' is not in target_labels_space",
    ),
    (
        "overwrite_map key missing from target_labels_space",
        {**_SELF_TEST_VALID_MODEL_CONFIG, "overwrite_map": {"null": "txt"}},
        "overwrite_map key 'null' is not in target_labels_space",
    ),
)


def run_self_test() -> int:
    """Runs the validation and codegen self-tests and returns the exit code."""
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
    # extensions, VETOS, `overwrite_map`, monotonic `exposed_content_types.txt`,
    # bindings codegen, and Markdown docs generation.
    complex_yaml = """\
"3gp":
  mime_type: "video/3gpp"
  group: "video"
  description: "3GPP multimedia file"
  extensions: ["3gp", "3GPP"]
  is_text: false

"directory":
  mime_type: "inode/directory"
  group: "inode"
  description: "A directory"
  extensions: []
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

"randomtxt":
  mime_type: null
  group: null
  description: "Random text"
  extensions: []
  is_text: true

"txt":
  mime_type: "text/plain"
  group: "text"
  description: "Generic text document"
  extensions: ["txt"]
  is_text: true

"unexposed_type":
  mime_type: "application/octet-stream"
  group: "binary"
  description: "Cataloged but unexposed"
  extensions: ["unx"]
  is_text: false
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
        "rust/lib/src/resolvers/tests/dataset.rs": (
            "fn test_only_helper() -> ContentType {\n    ContentType::Null\n}\n"
        ),
    }
    complex_model_config = {
        **_SELF_TEST_VALID_MODEL_CONFIG,
        "target_labels_space": ["3gp", "randomtxt", "txt"],
        "thresholds": {"txt": 0.75},
        "overwrite_map": {"randomtxt": "txt"},
    }
    try:
        complex_kb = build_kb(complex_yaml, complex_resolvers, complex_model_config)
        validate_model_config_for_codegen(complex_model_config)
        # "null" is already exposed (inactive), "llvm_bitcode" is newly active -> union keeps "null",
        # adds active types + "randomtxt", and excludes "unexposed_type".
        exposed_labels, exposed_text = resolve_exposed_content_types(
            "3gp\nnull\n", complex_kb, complex_model_config
        )
        rust_content = generate_rust_lib_content(
            complex_kb, exposed_labels, "standard_v3_3", complex_model_config
        )
        rust_model = generate_rust_lib_model(complex_model_config)
        ffi_content = generate_rust_ffi_content(complex_kb, exposed_labels)
        py_labels = generate_python_content_type_label(exposed_labels)
        js_labels = generate_js_content_type_label(exposed_labels)
        js_infos = generate_js_content_types_infos(complex_kb, exposed_labels)
        model_readme = generate_model_readme(complex_kb, complex_model_config)
    except KbError as e:
        failures.append(f"complex resolver & codegen source: unexpected error: {e}")
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
            exposed_labels
            == ["3gp", "directory", "llvm_bitcode", "null", "randomtxt", "txt"],
            exposed_text == "3gp\ndirectory\nllvm_bitcode\nnull\nrandomtxt\ntxt\n",
            "pub(crate) static DIRECTORY: TypeInfo" in rust_content,
            "Directory," not in rust_content,
            "    _3gp," in rust_content,
            "    LlvmBitcode," in rust_content,
            'mime_type: "application/octet-stream",' in rust_content,
            'mime_type: "text/plain",' in rust_content,
            "THRESHOLDS: [f32; 3] = [0.5, 0.5, 0.75];" in rust_model,
            "ContentType::Randomtxt," in rust_model,
            "magika::ContentType::Directory" not in ffi_content,
            "pub(crate) static DIRECTORY: MagikaTypeInfo" in ffi_content,
            '    _3GP = "3gp"' in py_labels,
            "UNEXPOSED_TYPE" not in py_labels,
            '  _3GP = "3gp",' in js_labels,
            "UNEXPOSED_TYPE" not in js_labels,
            "[ContentTypeLabel.TXT]: {\n      label: ContentTypeLabel.TXT,\n      is_text: true,"
            in js_infos,
            "## List of possible outputs" in model_readme,
            "## List of possible model's outputs" in model_readme,
            "| 3 | llvm_bitcode | LLVM bitcode |" in model_readme,
            "| 2 | randomtxt | Random text |" in model_readme,
            "overwrite_map" not in model_readme,
            "unexposed_type" not in model_readme,
        )
        if not all(checks):
            failures.append(
                f"complex resolver & codegen source: check failure: {checks}"
            )

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

    valid_kb = build_kb(
        _SELF_TEST_VALID_YAML,
        _SELF_TEST_VALID_RESOLVERS,
        _SELF_TEST_VALID_MODEL_CONFIG,
    )
    for (
        description,
        exposed_text,
        expected_error,
    ) in _SELF_TEST_INVALID_EXPOSED_CASES:
        try:
            resolve_exposed_content_types(
                exposed_text, valid_kb, _SELF_TEST_VALID_MODEL_CONFIG
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
        bad_model_config,
        expected_error,
    ) in _SELF_TEST_INVALID_MODEL_CODEGEN_CASES:
        try:
            validate_model_config_for_codegen(bad_model_config)
        except KbError as e:
            if expected_error not in str(e):
                failures.append(
                    f"{description}: expected error containing {expected_error!r}, got: {e}"
                )
        else:
            failures.append(f"{description}: expected an error, got none")

    total = (
        3
        + len(_SELF_TEST_INVALID_CASES)
        + len(_SELF_TEST_INVALID_DERIVED_CASES)
        + len(_SELF_TEST_INVALID_EXPOSED_CASES)
        + len(_SELF_TEST_INVALID_MODEL_CODEGEN_CASES)
    )
    for failure in failures:
        click.echo(f"FAIL: {failure}", err=True)
    click.echo(f"{total - len(failures)}/{total} self-tests passed.")
    return 1 if failures else 0


if __name__ == "__main__":
    main()
