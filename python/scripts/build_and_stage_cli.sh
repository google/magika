#!/bin/bash
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

# ==============================================================================
# Build and Stage Native Rust `magika` CLI Binary for Maturin Builds
# ==============================================================================
#
# Why this script is on the critical path:
# ----------------------------------------
# `python/pyproject.toml` configures Maturin with `data = "wheel_data"`.
# Under PEP 427 and Maturin's data directory layout, any files placed inside
# `python/wheel_data/scripts/` are bundled into the built wheel's `.data/scripts/`
# directory (and installed into the Python environment's `bin/` or `Scripts\`
# directory upon `pip install` / `uv sync`).
#
# Because the legacy pure-Python CLI (`magika-python-client`) has been replaced
# by the native Rust CLI (`rust/cli`), this script MUST be executed before
# building a wheel or running `uv sync` whenever the `magika` CLI binary is
# expected to be present in the Python environment:
#
#   1. Local dev & CI unit test suites (`python-test-suite.yml`, `sync_dev.sh`):
#      Run `./python/scripts/build_and_stage_cli.sh --profile release-fast` on
#      the host before `uv sync` so `magika` is compiled without fat LTO overhead
#      (matching `[tool.maturin] editable-profile = "release-fast"`) and installed
#      into `.venv/bin/magika`.
#
#   2. Local release wheel builds (`build_wheel.sh`):
#      Run `./python/scripts/build_and_stage_cli.sh` (defaults to `--profile release`)
#      before `uv build --wheel`.
#
#   3. macOS and Windows wheel builds (`python-build-test-and-release-package.yml`):
#      Run `./python/scripts/build_and_stage_cli.sh <target-triple>` on the
#      runner host immediately before `PyO3/maturin-action` packages the wheel.
#
#   4. Linux manylinux / musllinux wheel builds (`python-build-test-and-release-package.yml`):
#      Passed to `PyO3/maturin-action` via `before-script-linux` so the CLI is
#      compiled *inside* the manylinux/musllinux Docker container against the
#      exact same C library sysroot (`glibc` or `musl`) as the `_magika.abi3.so`
#      PyO3 extension.
#
# Usage:
#   ./python/scripts/build_and_stage_cli.sh [--profile <PROFILE>] [RUST_TARGET_TRIPLE]
# ==============================================================================

set -euo pipefail

PROFILE="release"
TARGET=""
while [[ $# -gt 0 ]]; do
    case "$1" in
        --profile)
            if [[ $# -lt 2 || -z "$2" ]]; then
                echo "Error: --profile requires a non-empty value." >&2
                exit 1
            fi
            PROFILE="$2"
            shift 2
            ;;
        --profile=*)
            PROFILE="${1#--profile=}"
            if [[ -z "$PROFILE" ]]; then
                echo "Error: --profile requires a non-empty value." >&2
                exit 1
            fi
            shift
            ;;
        -*)
            echo "Error: Unknown option '$1'. Usage: $0 [--profile <PROFILE>] [RUST_TARGET_TRIPLE]" >&2
            exit 1
            ;;
        *)
            if [[ -n "$TARGET" ]]; then
                echo "Error: Unexpected extra argument '$1'. Usage: $0 [--profile <PROFILE>] [RUST_TARGET_TRIPLE]" >&2
                exit 1
            fi
            TARGET="$1"
            shift
            ;;
    esac
done

# Fall back to $CARGO_BUILD_TARGET when no target triple was passed explicitly
# (set automatically by maturin-action inside cross-compilation containers).
TARGET="${TARGET:-${CARGO_BUILD_TARGET:-}}"

ROOT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." &>/dev/null && pwd)
cd "$ROOT_DIR"

# Ensure the staging directory exists and remove any previously staged binary so
# we never accidentally package a binary built for a different target architecture.
# Note: `rm -f python/wheel_data/scripts/*` does not delete hidden files like `.gitkeep`.
mkdir -p python/wheel_data/scripts
rm -f python/wheel_data/scripts/*

if [ -n "$TARGET" ]; then
    echo "Building magika CLI (profile: $PROFILE) for target $TARGET..."
    cargo build --profile "$PROFILE" --manifest-path rust/cli/Cargo.toml --target "$TARGET"
else
    echo "Building magika CLI (profile: $PROFILE) for default host target..."
    cargo build --profile "$PROFILE" --manifest-path rust/cli/Cargo.toml
fi

# Cargo places `--profile dev` artifacts in `debug/` and all other profiles in `<profile>/`.
if [ "$PROFILE" = "dev" ]; then
    PROFILE_DIR="debug"
else
    PROFILE_DIR="$PROFILE"
fi

# Honor $CARGO_TARGET_DIR when set (e.g. shared target cache across git worktrees),
# falling back to `.cargo/config.toml`'s `rust/target`.
TARGET_DIR="${CARGO_TARGET_DIR:-rust/target}"
TARGET_DIR="${TARGET_DIR%/}"

# Locate the compiled binary across candidate Cargo target directories.
# Depending on whether `--target` or `CARGO_BUILD_TARGET` was used (and whether
# we are on Unix or Windows), the binary lives under `<target-dir>/<triple>/<profile-dir>/`
# or `<target-dir>/<profile-dir>/` as either `magika` or `magika.exe`.
SRC_BIN=""
CANDIDATES=()
if [ -n "$TARGET" ]; then
    CANDIDATES+=("$TARGET_DIR/$TARGET/$PROFILE_DIR/magika" "$TARGET_DIR/$TARGET/$PROFILE_DIR/magika.exe")
fi
if [ -n "${CARGO_BUILD_TARGET:-}" ] && [ "${CARGO_BUILD_TARGET}" != "$TARGET" ]; then
    CANDIDATES+=("$TARGET_DIR/$CARGO_BUILD_TARGET/$PROFILE_DIR/magika" "$TARGET_DIR/$CARGO_BUILD_TARGET/$PROFILE_DIR/magika.exe")
fi
CANDIDATES+=("$TARGET_DIR/$PROFILE_DIR/magika" "$TARGET_DIR/$PROFILE_DIR/magika.exe")

for candidate in "${CANDIDATES[@]}"; do
    if [ -f "$candidate" ]; then
        SRC_BIN="$candidate"
        break
    fi
done

# Fallback search in case Cargo placed the binary under another target subdirectory.
if [ -z "$SRC_BIN" ]; then
    SRC_BIN=$(find "$TARGET_DIR" -type f \( -name "magika" -o -name "magika.exe" \) -path "*/$PROFILE_DIR/*" 2>/dev/null | head -n 1 || true)
fi

if [ -n "$SRC_BIN" ] && [ -f "$SRC_BIN" ]; then
    echo "Found CLI binary at $SRC_BIN"
    BIN_NAME=$(basename "$SRC_BIN")
    cp "$SRC_BIN" "python/wheel_data/scripts/$BIN_NAME"
    chmod +x "python/wheel_data/scripts/$BIN_NAME"
    echo "Staged $BIN_NAME into python/wheel_data/scripts/"
else
    echo "Error: CLI binary not found in any expected target directory." >&2
    echo "Checked candidates:" >&2
    for c in "${CANDIDATES[@]}"; do
        echo "  - $c" >&2
    done
    ls -la "$TARGET_DIR/" 2>/dev/null || true
    exit 1
fi
