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
# requires-python = ">=3.11"
# dependencies = ["click", "pyyaml"]
#
# [tool.uv]
# required-version = ">=0.13.4"
# ///
"""Synchronize the content types KB, language bindings, and generated docs.

`assets/content_types.yaml` is the human-editable source of truth for content
type metadata. This script validates the YAML, derives the 4 boolean coverage
fields (`in_heuristics`, `in_resolvers`, `in_ml_model`, `vetoed_by_resolvers`)
from built-in heuristics, the Rust resolvers (`rust/lib/src/resolvers/`), and
the active ML model config (`assets/active_model.txt`), maintains the monotonic
`assets/exposed_content_types.txt` allowlist, and deterministically generates
`assets/content_types_kb.min.json`, Rust/C FFI/Python/JS bindings, the active
model's documentation (`assets/models/<active_model>/README.md` and
`website-ng/src/content/docs/models/<active_model>.md`), the decoupled `main`
supported-content-types reference, and the per-version index
(`website-ng/src/content/docs/content-types/{main,index}.md`) in a single
pure-Python pass.

Usage (from anywhere in the repository):

    uv run scripts/sync_kb.py                           # Regenerate all KB artifacts.
    uv run scripts/sync_kb.py --check                   # Fail if any KB artifact is stale.
    uv run scripts/sync_kb.py --self-test               # Run the validation self-tests.
    uv run scripts/sync_kb.py --verify-release python   # Gate publishing (release CI).

Release snapshots (`website-ng/src/content/docs/content-types/<artifact>-v*.md`)
are driven by the versions declared in each artifact's own manifest
(`rust/lib/Cargo.toml`, `rust/cli/Cargo.toml`, `python/pyproject.toml`,
`js/package.json`). Between releases these carry a pre-release suffix (e.g.
`2.0.0-dev`, `2.0.0.dev0`) and are ignored. The release PR drops the suffix;
running this script then writes the snapshot of the new stable version and pins
its sha256 in `assets/content_types_snapshots.sha256` (`--check` fails until
it does). Pinned snapshots are frozen: `--check` fails if one is modified or
deleted, or if an unpinned one appears. At publish time, `--verify-release`
checks that the released tree reproduces its snapshot byte for byte.
"""

from __future__ import annotations

import hashlib
import io
import json
import re
import sys
import tempfile
import tomllib
from collections.abc import Container, Mapping, Sequence
from dataclasses import dataclass
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
WEBSITE_DOCS_DIR = REPO_ROOT_DIR / "website-ng" / "src" / "content" / "docs"
WEBSITE_MODELS_DIR = WEBSITE_DOCS_DIR / "models"
WEBSITE_CONTENT_TYPES_DIR = WEBSITE_DOCS_DIR / "content-types"
MAIN_CONTENT_TYPES_DOC_PATH = WEBSITE_CONTENT_TYPES_DIR / "main.md"
CONTENT_TYPES_INDEX_DOC_PATH = WEBSITE_CONTENT_TYPES_DIR / "index.md"
# Live docs in `content-types/` regenerated on every sync (never frozen).
GENERATED_CONTENT_TYPES_DOC_NAMES = frozenset(
    {MAIN_CONTENT_TYPES_DOC_PATH.name, CONTENT_TYPES_INDEX_DOC_PATH.name}
)
SNAPSHOTS_MANIFEST_PATH = ASSETS_DIR / "content_types_snapshots.sha256"
# `sha256sum`-compatible manifest line: "<sha256>  <snapshot file name>".
_SNAPSHOTS_MANIFEST_LINE_RE = re.compile(r"^([0-9a-f]{64})  ([a-z0-9_.-]+\.md)$")

# The human-authored fields of each content type, in the order in which they
# must appear in the YAML source and at the start of each JSON KB entry.
FIELDS = ("mime_type", "group", "description", "extensions", "is_text")
CONTENT_TYPE_NAME_RE = re.compile(r"^[a-z0-9_]+$")
MIME_TYPE_RE = re.compile(
    r"^[a-z0-9][a-z0-9!#$&\-\^_.+]*/[a-zA-Z0-9][a-zA-Z0-9!#$&\-\^_.+]*$"
)
EXTENSION_RE = re.compile(r"^[A-Za-z0-9][A-Za-z0-9_+.-]*$")
ACTIVE_MODEL_NAME_RE = re.compile(r"^[A-Za-z0-9_.-]+$")
# A stable release version; anything else (e.g. `2.0.0-dev`, `2.0.0.dev0`,
# `1.0.4rc1`) is a pre-release and never gets a snapshot.
STABLE_VERSION_RE = re.compile(r"^[0-9]+\.[0-9]+\.[0-9]+$")
_SNAPSHOT_FILE_NAME_RE = re.compile(r"^([a-z]+)-v([0-9]+)_([0-9]+)_([0-9]+)\.md$")

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
# Byte/stream heuristics active in the pure-TypeScript TensorFlow.js runtime.
JS_HEURISTIC_CONTENT_TYPES = frozenset({"empty", "txt", "unknown"})
# Non-regular file types represented on Rust's `FileType` enum rather than `ContentType`.
SPECIAL_FILE_TYPE_LABELS = frozenset({"directory", "symlink", "unsupported"})


@dataclass(frozen=True)
class ReleaseArtifact:
    """A published artifact whose supported content types are snapshotted per release."""

    key: str  # Snapshot file prefix and `--verify-release` argument.
    title: str  # e.g. "Python `magika`".
    short_title: str  # e.g. "Python".
    version_file: Path  # Manifest declaring the artifact's version.
    js_pipeline: bool  # TF.js runtime: byte heuristics + ML model, no resolvers.


RELEASE_ARTIFACTS: tuple[ReleaseArtifact, ...] = (
    ReleaseArtifact(
        "cli",
        "`magika` CLI",
        "CLI",
        REPO_ROOT_DIR / "rust" / "cli" / "Cargo.toml",
        False,
    ),
    ReleaseArtifact(
        "rust",
        "Rust `magika` library",
        "Rust",
        REPO_ROOT_DIR / "rust" / "lib" / "Cargo.toml",
        False,
    ),
    ReleaseArtifact(
        "python",
        "Python `magika`",
        "Python",
        REPO_ROOT_DIR / "python" / "pyproject.toml",
        False,
    ),
    ReleaseArtifact(
        "js",
        "JavaScript / TypeScript `magika`",
        "JavaScript",
        REPO_ROOT_DIR / "js" / "package.json",
        True,
    ),
)
RELEASE_ARTIFACTS_BY_KEY = {artifact.key: artifact for artifact in RELEASE_ARTIFACTS}

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
    help="Do not write anything; exit with an error if any generated KB artifact is "
    "stale, a declared stable version has no release snapshot, or a frozen release "
    "snapshot was tampered with.",
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
@click.option(
    "--verify-release",
    type=click.Choice([artifact.key for artifact in RELEASE_ARTIFACTS]),
    default=None,
    help="Release gate (run by the publishing workflows): check that ARTIFACT declares "
    "a stable version whose release snapshot is pinned and reproduced byte for byte "
    "by this tree, then exit.",
)
@click.option(
    "--expected-version",
    default=None,
    help="With --verify-release: also require the declared version to equal this "
    "one (e.g. the version in the release tag).",
)
def main(
    check: bool,
    self_test: bool,
    model_dir: Path | None,
    verify_release: str | None,
    expected_version: str | None,
) -> None:
    """Synchronize the content types KB, language bindings, and generated docs."""
    if sum((check, self_test, verify_release is not None)) > 1:
        raise click.UsageError(
            "--check, --self-test, and --verify-release are mutually exclusive."
        )
    if expected_version is not None and verify_release is None:
        raise click.UsageError("--expected-version requires --verify-release.")

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
        kb = build_kb(
            CONTENT_TYPES_YAML_PATH.read_text(encoding="utf-8"),
            resolver_sources,
            model_config,
            source_name=yaml_display_path,
            model_source_name=model_display_path,
            require_heuristics=True,
        )
        declared_versions = read_declared_versions(RELEASE_ARTIFACTS)

        if verify_release is not None:
            artifact = RELEASE_ARTIFACTS_BY_KEY[verify_release]
            snapshot_path = verify_release_snapshot(
                kb,
                artifact,
                declared_versions[artifact.key],
                model_name,
                expected_version=expected_version,
                snapshots_dir=WEBSITE_CONTENT_TYPES_DIR,
                manifest_path=SNAPSHOTS_MANIFEST_PATH,
            )
            click.echo(
                f"OK: {_display_path(snapshot_path)} is pinned and reproduced by this "
                f"tree for {artifact.title} {declared_versions[artifact.key]}."
            )
            return

        manifest = _read_snapshots_manifest(SNAPSHOTS_MANIFEST_PATH)
        new_snapshots = plan_release_snapshots(
            kb, model_name, declared_versions, pinned=manifest
        )
        snapshot_texts = {
            name: (WEBSITE_CONTENT_TYPES_DIR / name).read_text(encoding="utf-8")
            for name in manifest
            if (WEBSITE_CONTENT_TYPES_DIR / name).is_file()
        }
        snapshot_texts.update(new_snapshots)
        artifacts = build_repo_artifacts(
            kb=kb,
            model_name=model_name,
            model_config=model_config,
            existing_exposed_text=EXPOSED_CONTENT_TYPES_PATH.read_text(
                encoding="utf-8"
            ),
            resolved_model_dir=resolved_model_dir,
            model_source_name=model_display_path,
            exposed_source_name=exposed_display_path,
            snapshot_texts=snapshot_texts,
        )
        num_frozen, frozen_errors = verify_frozen_snapshots(
            WEBSITE_CONTENT_TYPES_DIR, SNAPSHOTS_MANIFEST_PATH
        )
    except KbError as e:
        click.echo(f"ERROR: {e}", err=True)
        sys.exit(1)

    stale_paths: list[str] = []
    for path, expected_bytes in artifacts.items():
        if not path.is_file() or path.read_bytes() != expected_bytes:
            stale_paths.append(_display_path(path))

    if check:
        if new_snapshots:
            formatted = "\n".join(
                f"- {_display_path(WEBSITE_CONTENT_TYPES_DIR / name)}"
                for name in new_snapshots
            )
            click.echo(
                f"ERROR: {len(new_snapshots)} declared stable release version(s) have no "
                f"release snapshot yet:\n{formatted}\nRun: just sync-kb",
                err=True,
            )
        if stale_paths:
            formatted = "\n".join(f"- {p}" for p in stale_paths)
            click.echo(
                f"ERROR: {len(stale_paths)} generated KB artifact(s) out of sync:\n"
                f"{formatted}\n"
                "Run: just sync-kb",
                err=True,
            )
        _echo_frozen_snapshot_errors(frozen_errors)
        if new_snapshots or stale_paths or frozen_errors:
            sys.exit(1)
        click.echo(
            f"All {len(artifacts)} KB artifacts are in sync with {yaml_display_path}, "
            f"and all {num_frozen} frozen release snapshots are unchanged."
        )
        return

    try:
        for name, doc in new_snapshots.items():
            path = write_frozen_snapshot(
                WEBSITE_CONTENT_TYPES_DIR, SNAPSHOTS_MANIFEST_PATH, name, doc
            )
            click.echo(
                f"Froze new release snapshot {_display_path(path)} (pinned in "
                f"{_display_path(SNAPSHOTS_MANIFEST_PATH)})."
            )
    except KbError as e:
        click.echo(f"ERROR: {e}", err=True)
        sys.exit(1)

    if not stale_paths:
        click.echo(f"All {len(artifacts)} KB artifacts are already up to date.")
    for path, expected_bytes in artifacts.items():
        if not path.is_file() or path.read_bytes() != expected_bytes:
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(expected_bytes)
            click.echo(f"Updated {_display_path(path)}.")

    # Frozen snapshots are never regenerated, so violations need a manual fix.
    _echo_frozen_snapshot_errors(frozen_errors)
    if frozen_errors:
        sys.exit(1)


def _echo_frozen_snapshot_errors(errors: Sequence[str]) -> None:
    if not errors:
        return
    formatted = "\n".join(f"- {e}" for e in errors)
    click.echo(
        f"ERROR: {len(errors)} frozen release snapshot problem(s):\n{formatted}\n"
        "Release snapshots are immutable: restore them with git. New ones are created "
        "by 'just sync-kb' when an artifact declares a new stable version.",
        err=True,
    )


def read_declared_versions(
    artifacts: Sequence[ReleaseArtifact],
) -> dict[str, str]:
    """Reads the version each release artifact declares in its own manifest."""
    versions: dict[str, str] = {}
    for artifact in artifacts:
        if not artifact.version_file.is_file():
            raise KbError(f"{_display_path(artifact.version_file)}: file not found")
        versions[artifact.key] = _parse_declared_version(
            artifact.version_file.name,
            artifact.version_file.read_text(encoding="utf-8"),
            _display_path(artifact.version_file),
        )
    return versions


def _parse_declared_version(file_name: str, text: str, source_name: str) -> str:
    """Extracts the package version from `Cargo.toml`, `pyproject.toml`, or `package.json`."""
    try:
        if file_name == "package.json":
            version = json.loads(text).get("version")
        elif file_name == "Cargo.toml":
            version = tomllib.loads(text).get("package", {}).get("version")
        elif file_name == "pyproject.toml":
            version = tomllib.loads(text).get("project", {}).get("version")
        else:
            raise KbError(f"{source_name}: unsupported version file")
    except (json.JSONDecodeError, tomllib.TOMLDecodeError) as e:
        raise KbError(f"{source_name}: cannot parse: {e}") from e
    if not isinstance(version, str) or not version:
        raise KbError(f"{source_name}: no literal package version found")
    return version


def snapshot_file_name(artifact_key: str, version: str) -> str:
    """Returns the release snapshot file name, e.g. `python-v1_0_4.md`."""
    return f"{artifact_key}-v{version.replace('.', '_')}.md"


def plan_release_snapshots(
    kb: Mapping[str, Mapping[str, Any]],
    model_name: str,
    declared_versions: Mapping[str, str],
    *,
    pinned: Container[str],
) -> dict[str, str]:
    """Returns `{file name: doc}` for declared stable versions that have no pinned snapshot."""
    docs: dict[str, str] = {}
    for artifact in RELEASE_ARTIFACTS:
        version = declared_versions[artifact.key]
        if not STABLE_VERSION_RE.match(version):
            continue
        name = snapshot_file_name(artifact.key, version)
        if name not in pinned:
            docs[name] = generate_release_snapshot_doc(
                kb, artifact, version, model_name
            )
    return docs


def verify_release_snapshot(
    kb: Mapping[str, Mapping[str, Any]],
    artifact: ReleaseArtifact,
    declared_version: str,
    model_name: str,
    *,
    expected_version: str | None,
    snapshots_dir: Path,
    manifest_path: Path,
) -> Path:
    """Checks that this tree is a releasable state of `artifact`; returns its snapshot path."""
    version_file = _display_path(artifact.version_file)
    if expected_version is not None and expected_version != declared_version:
        raise KbError(
            f"{version_file} declares version {declared_version!r}, but the release "
            f"expects {expected_version!r} (does the release tag match the release PR?)"
        )
    if not STABLE_VERSION_RE.match(declared_version):
        raise KbError(
            f"{version_file} declares {declared_version!r}, which is not a stable release "
            "version: the release PR must drop the pre-release suffix"
        )

    name = snapshot_file_name(artifact.key, declared_version)
    path = snapshots_dir / name
    manifest = _read_snapshots_manifest(manifest_path)
    if name not in manifest or not path.is_file():
        raise KbError(
            f"{_display_path(path)}: release snapshot is missing or not pinned in "
            f"{_display_path(manifest_path)} (run 'just sync-kb' in the release PR)"
        )
    data = path.read_bytes()
    if hashlib.sha256(data).hexdigest() != manifest[name]:
        raise KbError(
            f"{_display_path(path)}: modified after being frozen "
            f"(sha256 differs from {_display_path(manifest_path)})"
        )
    expected_doc = generate_release_snapshot_doc(
        kb, artifact, declared_version, model_name
    )
    if data != expected_doc.encode("utf-8"):
        raise KbError(
            f"{_display_path(path)}: does not match the supported content types of this "
            "tree (the release PR probably changed after its snapshot was frozen, e.g. "
            f"by a rebase). As {artifact.title} {declared_version} is not published yet: "
            f"delete the snapshot and its line in {_display_path(manifest_path)}, run "
            "'just sync-kb', merge, and release from the new commit"
        )
    return path


def write_frozen_snapshot(
    snapshots_dir: Path, manifest_path: Path, file_name: str, doc: str
) -> Path:
    """Writes a new release snapshot and pins its sha256; refuses to overwrite one."""
    manifest = _read_snapshots_manifest(manifest_path)
    path = snapshots_dir / file_name
    if path.exists() or file_name in manifest:
        raise KbError(
            f"{_display_path(path)}: release snapshot already exists and is frozen"
        )
    data = doc.encode("utf-8")
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(data)
    manifest[file_name] = hashlib.sha256(data).hexdigest()
    manifest_path.write_text(
        "".join(f"{digest}  {name}\n" for name, digest in sorted(manifest.items())),
        encoding="utf-8",
    )
    return path


def verify_frozen_snapshots(
    snapshots_dir: Path, manifest_path: Path
) -> tuple[int, list[str]]:
    """Returns `(num_pinned_snapshots, errors)` for modified, missing, or unpinned snapshots."""
    manifest = _read_snapshots_manifest(manifest_path)
    manifest_display = _display_path(manifest_path)
    errors: list[str] = []
    on_disk = (
        sorted(
            p.name
            for p in snapshots_dir.glob("*.md")
            if p.name not in GENERATED_CONTENT_TYPES_DOC_NAMES
        )
        if snapshots_dir.is_dir()
        else []
    )
    for name in on_disk:
        if name not in manifest:
            errors.append(
                f"{_display_path(snapshots_dir / name)}: release snapshot is not pinned in "
                f"{manifest_display}"
            )
    for name, digest in manifest.items():
        path = snapshots_dir / name
        if not path.is_file():
            errors.append(
                f"{_display_path(path)}: pinned in {manifest_display} but missing"
            )
        elif hashlib.sha256(path.read_bytes()).hexdigest() != digest:
            errors.append(
                f"{_display_path(path)}: modified after being frozen "
                f"(sha256 differs from {manifest_display})"
            )
    return len(manifest), errors


def _read_snapshots_manifest(manifest_path: Path) -> dict[str, str]:
    """Parses the snapshots manifest into `{file name: sha256}` (empty if absent)."""
    if not manifest_path.is_file():
        return {}
    source_name = _display_path(manifest_path)
    entries: dict[str, str] = {}
    errors: list[str] = []
    previous: str | None = None
    for line_no, line in enumerate(
        manifest_path.read_text(encoding="utf-8").splitlines(), start=1
    ):
        match = _SNAPSHOTS_MANIFEST_LINE_RE.match(line)
        if match is None:
            errors.append(
                f"{source_name}:{line_no}: expected '<sha256>  <snapshot>.md', got {line!r}"
            )
            continue
        digest, name = match.groups()
        if name in GENERATED_CONTENT_TYPES_DOC_NAMES:
            errors.append(
                f"{source_name}:{line_no}: {name} is regenerated by 'just sync-kb' and cannot be frozen"
            )
        elif name in entries:
            errors.append(f"{source_name}:{line_no}: duplicate entry {name!r}")
        elif previous is not None and name < previous:
            errors.append(
                f"{source_name}:{line_no}: entries must be sorted, but {name!r} comes after {previous!r}"
            )
        else:
            entries[name] = digest
            previous = name
    if errors:
        raise KbError(
            f"{source_name}: found {len(errors)} error(s):\n"
            + "\n".join(f"- {e}" for e in errors)
        )
    return entries


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
    kb: Mapping[str, Mapping[str, Any]],
    model_name: str,
    model_config: Any,
    existing_exposed_text: str,
    resolved_model_dir: Path,
    model_source_name: str,
    exposed_source_name: str,
    snapshot_texts: Mapping[str, str],
) -> dict[Path, bytes]:
    """Builds all generated KB, language-binding, and documentation artifacts in memory."""
    validate_model_config_for_codegen(model_config, model_source_name)
    exposed_labels, exposed_text = resolve_exposed_content_types(
        existing_exposed_text, kb, model_config, exposed_source_name
    )
    kb_json = json.dumps(kb, separators=(",", ":"), ensure_ascii=True)
    resolved_dir = resolved_model_dir.resolve()
    model_readme_path = resolved_dir / "README.md"

    artifacts: dict[Path, bytes] = {
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
        model_readme_path: generate_model_readme(kb, model_name, model_config).encode(
            "utf-8"
        ),
        MAIN_CONTENT_TYPES_DOC_PATH: generate_main_content_types_doc(
            kb, model_name
        ).encode("utf-8"),
        CONTENT_TYPES_INDEX_DOC_PATH: generate_content_types_index_doc(
            kb, model_name, snapshot_texts
        ).encode("utf-8"),
    }
    if resolved_dir.parent == MODELS_DIR.resolve():
        website_model_path = WEBSITE_MODELS_DIR / f"{model_name}.md"
        artifacts[website_model_path] = generate_website_model_doc(
            kb, model_name, model_config
        ).encode("utf-8")
    return artifacts


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
    entry_lines: list[str] = []
    for label in exposed_labels:
        info = kb[label]
        mime_type, group, description = _resolve_entry_metadata(label, info)
        default_mime = (
            "text/plain" if info["is_text"] else "application/octet-stream"
        )
        compact_mime = "" if mime_type == default_mime else mime_type
        compact_group = "" if group == "unknown" else group
        compact_desc = "" if description == label else description
        extensions_expr = (
            "[" + ", ".join(_ts_quote(ext) for ext in info["extensions"]) + "]"
        )
        flags = (
            (1 if info["is_text"] else 0)
            | (2 if info["in_heuristics"] else 0)
            | (4 if info["in_resolvers"] else 0)
            | (8 if info["in_ml_model"] else 0)
            | (16 if info["vetoed_by_resolvers"] else 0)
        )
        entry_lines.append(
            f"  [{_ts_quote(label)}, {_ts_quote(compact_mime)}, "
            f"{_ts_quote(compact_group)}, {_ts_quote(compact_desc)}, "
            f"{extensions_expr}, {flags}],"
        )

    lines = [
        _TS_GENERATED_HEADER.rstrip("\n"),
        "",
        'import { ContentTypeInfo } from "./content-type-info.js";',
        'import { ContentTypeLabel } from "./content-type-label.js";',
        "",
        "export type ContentTypesInfos = Record<ContentTypeLabel, ContentTypeInfo>;",
        "",
        '// Compact tuple encoding: [label, mime_type ("" = default), group ("" = "unknown"),',
        '// description ("" = label), extensions, flags].',
        "// Flags bitmask: 1 = is_text, 2 = in_heuristics, 4 = in_resolvers,",
        "// 8 = in_ml_model, 16 = vetoed_by_resolvers.",
        "type CompactContentTypeInfo = readonly [",
        "  `${ContentTypeLabel}`,",
        "  string,",
        "  string,",
        "  string,",
        "  readonly string[],",
        "  number,",
        "];",
        "",
        "// prettier-ignore",
        "const COMPACT_CONTENT_TYPES_INFOS: readonly CompactContentTypeInfo[] = [",
        *entry_lines,
        "];",
        "",
        "function hydrateContentTypeInfo([",
        "  rawLabel,",
        "  mimeType,",
        "  group,",
        "  description,",
        "  extensions,",
        "  flags,",
        "]: CompactContentTypeInfo): ContentTypeInfo {",
        "  const label = rawLabel as ContentTypeLabel;",
        "  const isText = (flags & 1) !== 0;",
        '  const defaultMime = isText ? "text/plain" : "application/octet-stream";',
        "  return {",
        "    label,",
        "    mime_type: mimeType || defaultMime,",
        '    group: group || "unknown",',
        "    description: description || rawLabel,",
        "    extensions: [...extensions],",
        "    is_text: isText,",
        "    in_heuristics: (flags & 2) !== 0,",
        "    in_resolvers: (flags & 4) !== 0,",
        "    in_ml_model: (flags & 8) !== 0,",
        "    vetoed_by_resolvers: (flags & 16) !== 0,",
        "  };",
        "}",
        "",
        "let cachedInfosMap: Map<string, ContentTypeInfo> | undefined;",
        "",
        "function getCachedInfosMap(): Map<string, ContentTypeInfo> {",
        "  if (cachedInfosMap === undefined) {",
        "    cachedInfosMap = new Map(",
        "      COMPACT_CONTENT_TYPES_INFOS.map((entry) => {",
        "        const info = hydrateContentTypeInfo(entry);",
        "        return [info.label, info];",
        "      }),",
        "    );",
        "  }",
        "  return cachedInfosMap;",
        "}",
        "",
        "export const ContentTypesInfos = {",
        "  get: (): ContentTypesInfos => {",
        "    const infos = {} as ContentTypesInfos;",
        "    for (const entry of COMPACT_CONTENT_TYPES_INFOS) {",
        "      const info = hydrateContentTypeInfo(entry);",
        "      infos[info.label] = info;",
        "    }",
        "    return infos;",
        "  },",
        "  getInfo: (label: ContentTypeLabel | string): ContentTypeInfo | undefined => {",
        "    const info = getCachedInfosMap().get(label);",
        "    if (info === undefined) {",
        "      return undefined;",
        "    }",
        "    return { ...info, extensions: [...info.extensions] };",
        "  },",
        "};",
        "",
    ]
    return "\n".join(lines)


def generate_model_readme(
    kb: Mapping[str, Mapping[str, Any]],
    model_name: str,
    model_config: Mapping[str, Any],
) -> str:
    """Generates `assets/models/<active_model>/README.md` strictly for the ML model."""
    body = _render_model_doc_body(
        kb,
        model_name,
        model_config,
        index_url="https://securityresearch.google/magika/content-types",
        main_coverage_url="https://securityresearch.google/magika/content-types/main",
    )
    return "\n".join(
        [
            "<!-- DO NOT EDIT: generated by scripts/sync_kb.py -->",
            "",
            f"# `{model_name}` Model Documentation",
            "",
            body,
        ]
    )


def generate_website_model_doc(
    kb: Mapping[str, Mapping[str, Any]],
    model_name: str,
    model_config: Mapping[str, Any],
) -> str:
    """Generates `website-ng/src/content/docs/models/<active_model>.md` strictly for the ML model."""
    body = _render_model_doc_body(
        kb,
        model_name,
        model_config,
        index_url="/magika/content-types",
        main_coverage_url="/magika/content-types/main",
    )
    return "\n".join(
        [
            "---",
            f"title: {model_name} model",
            "---",
            "",
            "<!-- DO NOT EDIT: generated by scripts/sync_kb.py -->",
            "",
            body,
        ]
    )


def _render_model_doc_body(
    kb: Mapping[str, Mapping[str, Any]],
    model_name: str,
    model_config: Mapping[str, Any],
    *,
    index_url: str,
    main_coverage_url: str,
) -> str:
    """Renders the resolver-independent Markdown body for a deep learning model."""
    target_labels = sorted(model_config["target_labels_space"])
    overwrite_map: Mapping[str, str] = model_config["overwrite_map"]
    thresholds: Mapping[str, Any] = model_config["thresholds"]
    effective_ml_count = len(set(target_labels) - set(overwrite_map.keys()))

    if thresholds:
        thresholds_summary = (
            f"default `medium_confidence_threshold` = `{model_config['medium_confidence_threshold']}`, "
            f"with per-label high-confidence overrides on {len(thresholds)} label(s) (see `Notes` below)."
        )
    else:
        thresholds_summary = (
            f"default `medium_confidence_threshold` = `{model_config['medium_confidence_threshold']}` "
            "(no per-label overrides)."
        )

    if overwrite_map:
        formatted_pairs = ", ".join(
            f"`{src}` -> `{dst}`" for src, dst in sorted(overwrite_map.items())
        )
        overwrite_summary = (
            f"{formatted_pairs} (raw calibration classes that may appear in `dl.label` "
            "before being remapped when producing `output.label`)."
        )
    else:
        overwrite_summary = "none."

    lines = [
        f"> **Looking for Magika's supported output content types?** This document covers only the **`{model_name}` deep learning model** itself. Magika (the CLI and language bindings) combines built-in heuristics, native format resolvers, and the deep learning model, so the full output space (`output.label`) depends on the artifact and version. See [Supported Content Types by Version]({index_url}) (or the [`main` branch coverage reference]({main_coverage_url})).",
        "",
        "## Model Overview & Configuration",
        "",
        f"- **Trained target classes (`target_labels_space`)**: {len(target_labels)} labels ({effective_ml_count} effective model-predicted content types after `overwrite_map`).",
        f"- **Input feature window**: first `{model_config['beg_size']}` bytes (`beg_size`) and last `{model_config['end_size']}` bytes (`end_size`) read within `{model_config['block_size']}`-byte blocks (`block_size`), padded with token `{model_config['padding_token']}` (`padding_token`); minimum file size for model inference is `{model_config['min_file_size_for_dl']}` bytes (`min_file_size_for_dl`).",
        f"- **Confidence thresholds**: {thresholds_summary}",
        f"- **Canonical label remapping (`overwrite_map`)**: {overwrite_summary}",
        "- **Skipped-inference sentinel (`undefined`)**: not a neural-network output class, but reported in `dl.label` whenever the model was not evaluated (for example on directories, symlinks, empty or tiny files, or inputs resolved ahead of model inference).",
        "",
        "## Model Output Labels",
        "",
        f"The table below lists all {len(target_labels)} labels in `{model_name}`'s `target_labels_space` in alphabetical order (for the exact 0-based softmax class order, consult `target_labels_space` in `config.min.json`).",
        "",
        "| Label | Description | MIME Type | Group | Notes |",
        "| --- | --- | --- | --- | --- |",
    ]
    for label in target_labels:
        mime_type, group, description = _resolve_entry_metadata(label, kb[label])
        notes_parts: list[str] = []
        if label in overwrite_map:
            notes_parts.append(f"Remapped to `{overwrite_map[label]}`")
        if label in thresholds:
            notes_parts.append(f"Threshold: `{thresholds[label]}`")
        notes = "; ".join(notes_parts)
        lines.append(
            f"| `{label}` | {_md_escape(description)} | `{mime_type}` | `{group}` | {notes} |"
        )

    lines.append("")
    return "\n".join(lines)


def generate_main_content_types_doc(
    kb: Mapping[str, Mapping[str, Any]],
    model_name: str,
) -> str:
    """Generates `website-ng/src/content/docs/content-types/main.md` for the `main` branch."""
    rows: list[
        tuple[str, str, str, str, Sequence[str], bool, bool, bool, bool, bool]
    ] = []
    for label, info in kb.items():
        if not (info["in_heuristics"] or info["in_resolvers"] or info["in_ml_model"]):
            continue
        mime_type, group, description = _resolve_entry_metadata(label, info)
        rows.append(
            (
                label,
                mime_type,
                group,
                description,
                info["extensions"],
                info["is_text"],
                info["in_heuristics"],
                info["in_resolvers"],
                info["in_ml_model"],
                info["vetoed_by_resolvers"],
            )
        )

    native_total = len(rows)
    special_present = [r[0] for r in rows if r[0] in SPECIAL_FILE_TYPE_LABELS]
    regular_total = native_total - len(special_present)
    heuristics_labels = [r[0] for r in rows if r[6]]
    resolvers_count = sum(1 for r in rows if r[7])
    ml_count = sum(1 for r in rows if r[8])
    vetoed_count = sum(1 for r in rows if r[9])
    js_total = sum(
        1
        for label, info in kb.items()
        if label in JS_HEURISTIC_CONTENT_TYPES or info["in_ml_model"]
    )
    heuristics_fmt = ", ".join(f"`{label}`" for label in heuristics_labels)
    special_fmt = ", ".join(f"`{label}`" for label in special_present)

    lines = [
        "---",
        "title: Supported Content Types (main)",
        "---",
        "",
        "<!-- DO NOT EDIT: generated by scripts/sync_kb.py -->",
        "",
        f"This reference lists the supported output content types (`output.label`) and detection stage coverage on the unreleased **`main`** development branch (default model: [`{model_name}`](/magika/models/{model_name})). For released package versions, see the [per-version index](/magika/content-types).",
        "",
        "## Pipeline Coverage Summary",
        "",
        f"- **Native Rust pipeline (`magika` CLI, Rust `magika`, Python `magika`, C `magika-ffi`)**: **{native_total}** supported outputs (**{regular_total}** regular-file `ContentType`s plus **{len(special_present)}** filesystem path `FileType`s: {special_fmt}).",
        f"  - **Heuristics (`in_heuristics`)**: **{len(heuristics_labels)}** content types ({heuristics_fmt}) — path metadata checks, empty-file detection, and short-file / low-confidence fallbacks.",
        f"  - **Format Resolvers (`in_resolvers`)**: **{resolvers_count}** content types (**{vetoed_count}** marked `vetoed_by_resolvers`, meaning the resolver matches all valid files of that format and vetoes unverified ML predictions).",
        f"  - **ML Model (`in_ml_model`)**: **{ml_count}** content types predicted by [`{model_name}`](/magika/models/{model_name}) (after `overwrite_map`).",
        f"- **JavaScript / TypeScript TensorFlow.js pipeline (`magika` on `main`)**: **{js_total}** supported outputs — runs byte/stream heuristics (`empty`, `txt`, `unknown`) plus the [`{model_name}`](/magika/models/{model_name}) ML model (`in_ml_model`), without filesystem `stat` checks or native Rust format resolvers (`in_resolvers`).",
        "",
        "## Supported Content Types",
        "",
        *_render_content_types_table(rows),
        "",
    ]
    return "\n".join(lines)


def generate_release_snapshot_doc(
    kb: Mapping[str, Mapping[str, Any]],
    artifact: ReleaseArtifact,
    version: str,
    model_name: str,
) -> str:
    """Renders the frozen supported-content-types snapshot of `artifact` at `version`."""
    rows: list[
        tuple[str, str, str, str, Sequence[str], bool, bool, bool, bool, bool]
    ] = []
    for label, info in kb.items():
        if artifact.js_pipeline:
            in_heur = label in JS_HEURISTIC_CONTENT_TYPES
            in_res = False
            in_ml = bool(info["in_ml_model"])
            vetoed = False
        else:
            in_heur = bool(info["in_heuristics"])
            in_res = bool(info["in_resolvers"])
            in_ml = bool(info["in_ml_model"])
            vetoed = bool(info["vetoed_by_resolvers"])

        if not (in_heur or in_res or in_ml):
            continue
        mime_type, group, description = _resolve_entry_metadata(label, info)
        rows.append(
            (
                label,
                mime_type,
                group,
                description,
                info["extensions"],
                info["is_text"],
                in_heur,
                in_res,
                in_ml,
                vetoed,
            )
        )

    total = len(rows)
    heuristics_labels = [r[0] for r in rows if r[6]]
    resolvers_count = sum(1 for r in rows if r[7])
    ml_count = sum(1 for r in rows if r[8])
    vetoed_count = sum(1 for r in rows if r[9])
    heuristics_fmt = ", ".join(f"`{label}`" for label in heuristics_labels)

    if artifact.js_pipeline:
        resolvers_line = "  - **Format Resolvers (`in_resolvers`)**: **0** content types (the pure-TypeScript TensorFlow.js runtime does not run native Rust format resolvers)."
    elif resolvers_count == 0:
        resolvers_line = "  - **Format Resolvers (`in_resolvers`)**: **0** content types (this release predates native format resolvers)."
    else:
        resolvers_line = (
            f"  - **Format Resolvers (`in_resolvers`)**: **{resolvers_count}** content types "
            f"(**{vetoed_count}** marked `vetoed_by_resolvers`)."
        )

    lines = [
        "---",
        f"title: Supported Content Types ({artifact.short_title} {version})",
        "---",
        "",
        f"<!-- DO NOT EDIT: frozen release snapshot generated by scripts/sync_kb.py for {artifact.title} {version} (declared in {_display_path(artifact.version_file)}). Its sha256 is pinned in assets/content_types_snapshots.sha256 and enforced by 'just sync-kb --check'. -->",
        "",
        f"This reference is the frozen output-space (`output.label`) and detection stage-coverage snapshot for **{artifact.title} `{version}`** (default model: [`{model_name}`](/magika/models/{model_name})). For other versions and the `main` development branch, see the [per-version index](/magika/content-types).",
        "",
        "## Pipeline Coverage Summary",
        "",
        f"- **Total supported outputs**: **{total}** content types.",
        f"  - **Heuristics (`in_heuristics`)**: **{len(heuristics_labels)}** content types ({heuristics_fmt}).",
        resolvers_line,
        f"  - **ML Model (`in_ml_model`)**: **{ml_count}** content types predicted by [`{model_name}`](/magika/models/{model_name}) (after `overwrite_map`).",
        "",
        "## Supported Content Types",
        "",
        *_render_content_types_table(rows),
        "",
    ]
    return "\n".join(lines)


_SNAPSHOT_MODEL_RE = re.compile(r"default model: \[`([^`]+)`\]")
_SNAPSHOT_RESOLVERS_RE = re.compile(
    r"Format Resolvers \(`in_resolvers`\)\*\*: \*\*([0-9]+)\*\*"
)


def generate_content_types_index_doc(
    kb: Mapping[str, Mapping[str, Any]],
    model_name: str,
    snapshot_texts: Mapping[str, str],
) -> str:
    """Generates `content-types/index.md`: `main` plus every frozen release snapshot."""
    native_total = sum(
        1
        for info in kb.values()
        if info["in_heuristics"] or info["in_resolvers"] or info["in_ml_model"]
    )
    entries: list[tuple[int, tuple[int, ...], str]] = []
    artifact_order = {artifact.key: i for i, artifact in enumerate(RELEASE_ARTIFACTS)}
    for name, text in snapshot_texts.items():
        name_match = _SNAPSHOT_FILE_NAME_RE.match(name)
        if name_match is None or name_match.group(1) not in RELEASE_ARTIFACTS_BY_KEY:
            raise KbError(
                f"{name}: release snapshot names must be '<artifact>-v<MAJOR>_<MINOR>_<PATCH>.md' "
                f"with <artifact> in {sorted(RELEASE_ARTIFACTS_BY_KEY)}"
            )
        artifact = RELEASE_ARTIFACTS_BY_KEY[name_match.group(1)]
        version_parts = tuple(int(x) for x in name_match.groups()[1:])
        version = ".".join(str(x) for x in version_parts)
        model_match = _SNAPSHOT_MODEL_RE.search(text)
        resolvers_match = _SNAPSHOT_RESOLVERS_RE.search(text)
        if model_match is None or resolvers_match is None:
            raise KbError(f"{name}: cannot find the default model or resolvers summary")
        if artifact.js_pipeline:
            stages = "Byte Heuristics + ML Model (TF.js)"
        elif int(resolvers_match.group(1)) > 0:
            stages = "Heuristics + Format Resolvers + ML Model"
        else:
            stages = "Heuristics + ML Model"
        snapshot_model = model_match.group(1)
        count = sum(1 for line in text.splitlines() if line.startswith("| `"))
        row = (
            f"| {artifact.title} `{version}` | {stages} | "
            f"[`{snapshot_model}`](/magika/models/{snapshot_model}) | "
            f"[{count} content types](/magika/content-types/{name.removesuffix('.md')}) |"
        )
        entries.append(
            (artifact_order[artifact.key], tuple(-x for x in version_parts), row)
        )

    lines = [
        "---",
        "title: Supported Content Types by Version",
        "---",
        "",
        "<!-- DO NOT EDIT: generated by scripts/sync_kb.py -->",
        "",
        "Each reference page below lists every content type that a branch or released package can emit in `output.label`, together with its metadata (`mime_type`, `group`, `description`, `extensions`, `is_text`) and orthogonal stage-coverage flags (`in_heuristics`, `in_resolvers`, `in_ml_model`, `vetoed_by_resolvers`). Release snapshots are frozen when the release is cut; see [Models & Supported Content Types](/magika/core-concepts/models-and-content-types) for background.",
        "",
        "| Version / Artifact | Pipeline Stages | Default Model | Supported Content Types |",
        "| --- | --- | --- | --- |",
        f"| `main` (unreleased development branch) | Heuristics + Format Resolvers + ML Model | [`{model_name}`](/magika/models/{model_name}) | [{native_total} content types](/magika/content-types/main) |",
        *(row for _, _, row in sorted(entries)),
        "",
    ]
    return "\n".join(lines)


def _render_content_types_table(
    rows: Sequence[
        tuple[str, str, str, str, Sequence[str], bool, bool, bool, bool, bool]
    ],
) -> list[str]:
    """Renders the unified 10-column supported-content-types Markdown table."""
    table_lines = [
        "| Label | Description | MIME Type | Group | Extensions | Text | Heuristics | Resolvers | ML Model | Vetoed |",
        "| --- | --- | --- | --- | --- | :---: | :---: | :---: | :---: | :---: |",
    ]
    for (
        label,
        mime_type,
        group,
        description,
        extensions,
        is_text,
        in_heuristics,
        in_resolvers,
        in_ml_model,
        vetoed_by_resolvers,
    ) in rows:
        ext_cell = ", ".join(f"`{ext}`" for ext in extensions) if extensions else "—"
        text_cell = "✓" if is_text else ""
        heur_cell = "✓" if in_heuristics else ""
        res_cell = "✓" if in_resolvers else ""
        ml_cell = "✓" if in_ml_model else ""
        veto_cell = "✓" if vetoed_by_resolvers else ""
        table_lines.append(
            f"| `{label}` | {_md_escape(description)} | `{mime_type}` | `{group}` | {ext_cell} | {text_cell} | {heur_cell} | {res_cell} | {ml_cell} | {veto_cell} |"
        )
    return table_lines


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


def _ts_quote(text: str) -> str:
    """Formats `text` as a double-quoted TypeScript string literal."""
    return (
        json.dumps(text, ensure_ascii=False)
        .replace("\u2028", "\\u2028")
        .replace("\u2029", "\\u2029")
    )


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
        model_readme = generate_model_readme(
            complex_kb, "standard_v3_3", complex_model_config
        )
        website_model_doc = generate_website_model_doc(
            complex_kb, "standard_v3_3", complex_model_config
        )
        kb_without_resolvers = build_kb(
            complex_yaml, _SELF_TEST_VALID_RESOLVERS, complex_model_config
        )
        model_readme_without_resolvers = generate_model_readme(
            kb_without_resolvers, "standard_v3_3", complex_model_config
        )
        website_model_without_resolvers = generate_website_model_doc(
            kb_without_resolvers, "standard_v3_3", complex_model_config
        )
        main_content_types_doc = generate_main_content_types_doc(
            complex_kb, "standard_v3_3"
        )
        py_snap_doc = generate_release_snapshot_doc(
            complex_kb, RELEASE_ARTIFACTS_BY_KEY["python"], "1.1.0", "standard_v3_3"
        )
        js_snap_doc = generate_release_snapshot_doc(
            complex_kb, RELEASE_ARTIFACTS_BY_KEY["js"], "1.0.0", "standard_v3_3"
        )
        planned = plan_release_snapshots(
            complex_kb,
            "standard_v3_3",
            {"cli": "1.1.0", "rust": "2.0.0-dev", "python": "1.0.4", "js": "1.1.0-dev"},
            pinned={"cli-v1_1_0.md"},
        )
        index_doc = generate_content_types_index_doc(
            complex_kb,
            "standard_v3_3",
            {
                "js-v1_0_0.md": js_snap_doc,
                "python-v1_0_4.md": planned.get("python-v1_0_4.md", ""),
                "python-v1_1_0.md": py_snap_doc,
            },
        )
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
            '  ["3gp", "video/3gpp", "video", "3GPP multimedia file", ["3gp", "3GPP"], 28],'
            in js_infos,
            '  ["null", "", "", "", ["null"], 0],' in js_infos,
            '  ["randomtxt", "", "", "Random text", [], 1],' in js_infos,
            '  ["txt", "", "text", "Generic text document", ["txt"], 11],'
            in js_infos,
            '"unexposed_type"' not in js_infos,
            "getInfo: (label: ContentTypeLabel | string): ContentTypeInfo | undefined =>"
            in js_infos,
            "# `standard_v3_3` Model Documentation" in model_readme,
            "## Model Overview & Configuration" in model_readme,
            "## Model Output Labels" in model_readme,
            "List of possible outputs" not in model_readme,
            "llvm_bitcode" not in model_readme,
            "`directory`" not in model_readme,
            "| `randomtxt` | Random text | `text/plain` | `unknown` | Remapped to `txt` |"
            in model_readme,
            "| `txt` | Generic text document | `text/plain` | `text` | Threshold: `0.75` |"
            in model_readme,
            "unexposed_type" not in model_readme,
            model_readme == model_readme_without_resolvers,
            website_model_doc == website_model_without_resolvers,
            "title: standard_v3_3 model" in website_model_doc,
            "[Supported Content Types by Version](/magika/content-types)" in website_model_doc,
            "title: Supported Content Types (main)" in main_content_types_doc,
            "| `3gp` | 3GPP multimedia file | `video/3gpp` | `video` | `3gp`, `3GPP` |  |  | ✓ | ✓ | ✓ |"
            in main_content_types_doc,
            "| `directory` | A directory | `inode/directory` | `inode` | — |  | ✓ |  |  |  |"
            in main_content_types_doc,
            "| `llvm_bitcode` | LLVM bitcode | `application/x-llvm` | `executable` | `bc` |  |  | ✓ |  |  |"
            in main_content_types_doc,
            "| `txt` | Generic text document | `text/plain` | `text` | `txt` | ✓ | ✓ |  | ✓ |  |"
            in main_content_types_doc,
            "`randomtxt`" not in main_content_types_doc,
            "unexposed_type" not in main_content_types_doc,
            snapshot_file_name("python", "1.1.0") == "python-v1_1_0.md",
            "title: Supported Content Types (Python 1.1.0)" in py_snap_doc,
            "declared in python/pyproject.toml" in py_snap_doc,
            "| `llvm_bitcode` |" in py_snap_doc,
            "| `directory` |" in py_snap_doc,
            "| `llvm_bitcode` |" not in js_snap_doc,
            "| `directory` |" not in js_snap_doc,
            "| `txt` | Generic text document | `text/plain` | `text` | `txt` | ✓ | ✓ |  | ✓ |  |"
            in js_snap_doc,
            # Only stable versions without a pinned snapshot are planned.
            list(planned) == ["python-v1_0_4.md"],
            # `main` first, then artifacts in RELEASE_ARTIFACTS order, newest first.
            index_doc.index("(/magika/content-types/main)")
            < index_doc.index("python-v1_1_0)")
            < index_doc.index("python-v1_0_4)")
            < index_doc.index("js-v1_0_0)"),
            "| Python `magika` `1.1.0` | Heuristics + Format Resolvers + ML Model |"
            in index_doc,
            "| JavaScript / TypeScript `magika` `1.0.0` | Byte Heuristics + ML Model (TF.js) | [`standard_v3_3`](/magika/models/standard_v3_3) | [2 content types](/magika/content-types/js-v1_0_0) |"
            in index_doc,
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

    failures.extend(_self_test_declared_versions())
    failures.extend(_self_test_frozen_snapshots())

    total = (
        5
        + len(_SELF_TEST_INVALID_CASES)
        + len(_SELF_TEST_INVALID_DERIVED_CASES)
        + len(_SELF_TEST_INVALID_EXPOSED_CASES)
        + len(_SELF_TEST_INVALID_MODEL_CODEGEN_CASES)
    )
    for failure in failures:
        click.echo(f"FAIL: {failure}", err=True)
    click.echo(f"{total - len(failures)}/{total} self-tests passed.")
    return 1 if failures else 0


def _self_test_declared_versions() -> list[str]:
    """Exercises version parsing and the stable vs. pre-release distinction."""
    checks: list[tuple[str, bool]] = [
        (
            "Cargo.toml",
            _parse_declared_version(
                "Cargo.toml", '[package]\nname = "magika"\nversion = "2.0.0-dev"\n', "c"
            )
            == "2.0.0-dev",
        ),
        (
            "pyproject.toml",
            _parse_declared_version(
                "pyproject.toml", '[project]\nversion = "2.0.0.dev0"\n', "p"
            )
            == "2.0.0.dev0",
        ),
        (
            "package.json",
            _parse_declared_version("package.json", '{"version": "1.1.0"}', "j")
            == "1.1.0",
        ),
        (
            "stable vs. pre-release",
            [
                bool(STABLE_VERSION_RE.match(v))
                for v in ("1.0.4", "2.0.0-dev", "2.0.0.dev0", "1.0.4rc1", "v1.0.4")
            ]
            == [True, False, False, False, False],
        ),
    ]
    try:
        _parse_declared_version("pyproject.toml", '[project]\nname = "x"\n', "p")
        checks.append(("missing version is rejected", False))
    except KbError as e:
        checks.append(
            ("missing version is rejected", "no literal package version" in str(e))
        )
    return [f"declared versions: {name}" for name, ok in checks if not ok]


def _self_test_frozen_snapshots() -> list[str]:
    """Exercises pinning, immutability, tamper detection, and the release gate."""
    kb = build_kb(
        _SELF_TEST_VALID_YAML, _SELF_TEST_VALID_RESOLVERS, _SELF_TEST_VALID_MODEL_CONFIG
    )
    python = RELEASE_ARTIFACTS_BY_KEY["python"]
    with tempfile.TemporaryDirectory() as tmp:
        snapshots_dir = Path(tmp) / "content-types"
        manifest_path = Path(tmp) / "content_types_snapshots.sha256"
        snapshots_dir.mkdir()
        for name in GENERATED_CONTENT_TYPES_DOC_NAMES:
            (snapshots_dir / name).write_text("live, not frozen\n", encoding="utf-8")

        def _verify() -> tuple[int, list[str]]:
            return verify_frozen_snapshots(snapshots_dir, manifest_path)

        def _gate(version: str, expected: str | None = None) -> str:
            try:
                verify_release_snapshot(
                    kb,
                    python,
                    version,
                    "standard_v3_3",
                    expected_version=expected,
                    snapshots_dir=snapshots_dir,
                    manifest_path=manifest_path,
                )
            except KbError as e:
                return str(e)
            return "OK"

        try:
            write_frozen_snapshot(
                snapshots_dir,
                manifest_path,
                "python-v1_0_4.md",
                generate_release_snapshot_doc(kb, python, "1.0.4", "standard_v3_3"),
            )
            write_frozen_snapshot(snapshots_dir, manifest_path, "js-v1_0_0.md", "b\n")
        except KbError as e:
            return [f"frozen snapshots: unexpected error: {e}"]

        checks: list[tuple[str, bool]] = [
            (
                "manifest is sorted",
                manifest_path.read_text().splitlines()[0].endswith("  js-v1_0_0.md"),
            ),
            ("clean state verifies", _verify() == (2, [])),
            ("release gate passes", _gate("1.0.4", "1.0.4") == "OK"),
            (
                "gate rejects pre-release",
                "not a stable release version" in _gate("1.0.5-dev"),
            ),
            (
                "gate rejects tag mismatch",
                "but the release expects '1.0.5'" in _gate("1.0.4", "1.0.5"),
            ),
            (
                "gate rejects missing snapshot",
                "missing or not pinned" in _gate("1.0.5"),
            ),
        ]
        try:
            write_frozen_snapshot(snapshots_dir, manifest_path, "js-v1_0_0.md", "c\n")
            checks.append(("overwrite is refused", False))
        except KbError as e:
            checks.append(
                ("overwrite is refused", "already exists and is frozen" in str(e))
            )

        # A snapshot that is pinned but no longer reproduced by the tree (stale release PR).
        stale = "stale\n"
        (snapshots_dir / "python-v1_0_4.md").write_text(stale, encoding="utf-8")
        lines = manifest_path.read_text().splitlines()
        lines[1] = f"{hashlib.sha256(stale.encode()).hexdigest()}  python-v1_0_4.md"
        manifest_path.write_text("\n".join(lines) + "\n", encoding="utf-8")
        checks.append(
            (
                "gate rejects stale snapshot",
                "does not match the supported content types" in _gate("1.0.4"),
            )
        )

        (snapshots_dir / "js-v1_0_0.md").write_text("tampered\n", encoding="utf-8")
        checks.append(
            (
                "modification is detected",
                any("modified after being frozen" in e for e in _verify()[1]),
            )
        )
        (snapshots_dir / "js-v1_0_0.md").unlink()
        checks.append(
            ("deletion is detected", any("but missing" in e for e in _verify()[1]))
        )
        (snapshots_dir / "rust-v9_9_9.md").write_text("x\n", encoding="utf-8")
        checks.append(
            (
                "unpinned snapshot is detected",
                any("is not pinned" in e for e in _verify()[1]),
            )
        )

        manifest_path.write_text("not-a-hash  js-v1_0_0.md\n", encoding="utf-8")
        try:
            _verify()
            checks.append(("malformed manifest is rejected", False))
        except KbError as e:
            checks.append(
                (
                    "malformed manifest is rejected",
                    "expected '<sha256>  <snapshot>.md'" in str(e),
                )
            )

    return [f"frozen snapshots: {name}" for name, ok in checks if not ok]


if __name__ == "__main__":
    main()
