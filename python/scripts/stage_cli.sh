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
# Stage Native Rust `magika` CLI Binary for Maturin Wheel & Editable Builds
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
#   1. Local builds & CI unit test suites (`python-test-suite.yml`, `build_wheel.sh`):
#      Run `./python/scripts/stage_cli.sh` (no arguments) on the host before
#      `uv sync` or `uv build` so `magika` is compiled for the host target and
#      installed into `.venv/bin/magika`.
#
#   2. macOS and Windows wheel builds (`python-build-and-release-package.yml`):
#      Run `./python/scripts/stage_cli.sh <target-triple>` on the runner host
#      immediately before `PyO3/maturin-action` packages the wheel.
#
#   3. Linux manylinux / musllinux wheel builds (`python-build-and-release-package.yml`):
#      Passed to `PyO3/maturin-action` via `before-script-linux` so the CLI is
#      compiled *inside* the manylinux/musllinux Docker container against the
#      exact same C library sysroot (`glibc` or `musl`) as the `_magika.abi3.so`
#      PyO3 extension.
#
# Usage:
#   ./python/scripts/stage_cli.sh [RUST_TARGET_TRIPLE]
# ==============================================================================

set -euo pipefail

# Accept an explicit target triple as $1, falling back to $CARGO_BUILD_TARGET
# (which is set automatically by maturin-action inside cross-compilation containers).
TARGET="${1:-${CARGO_BUILD_TARGET:-}}"
ROOT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." &>/dev/null && pwd)
cd "$ROOT_DIR"

# Ensure the staging directory exists and remove any previously staged binary so
# we never accidentally package a binary built for a different target architecture.
# Note: `rm -f python/wheel_data/scripts/*` does not delete hidden files like `.gitkeep`.
mkdir -p python/wheel_data/scripts
rm -f python/wheel_data/scripts/*

if [ -n "$TARGET" ]; then
    echo "Building magika CLI for target $TARGET..."
    cargo build --release --manifest-path rust/cli/Cargo.toml --target "$TARGET"
else
    echo "Building magika CLI for default host target..."
    cargo build --release --manifest-path rust/cli/Cargo.toml
fi

# Locate the compiled binary across candidate Cargo target directories.
# Depending on whether `--target` or `CARGO_BUILD_TARGET` was used (and whether
# we are on Unix or Windows), the binary lives under `rust/target/<triple>/release/`
# or `rust/target/release/` as either `magika` or `magika.exe`.
SRC_BIN=""
CANDIDATES=()
if [ -n "$TARGET" ]; then
    CANDIDATES+=("rust/target/$TARGET/release/magika" "rust/target/$TARGET/release/magika.exe")
fi
if [ -n "${CARGO_BUILD_TARGET:-}" ]; then
    CANDIDATES+=("rust/target/$CARGO_BUILD_TARGET/release/magika" "rust/target/$CARGO_BUILD_TARGET/release/magika.exe")
fi
CANDIDATES+=("rust/target/release/magika" "rust/target/release/magika.exe")

for candidate in "${CANDIDATES[@]}"; do
    if [ -f "$candidate" ]; then
        SRC_BIN="$candidate"
        break
    fi
done

# Fallback search in case Cargo placed the release binary under another target subdirectory.
if [ -z "$SRC_BIN" ]; then
    SRC_BIN=$(find rust/target -type f \( -name "magika" -o -name "magika.exe" \) -path "*/release/*" 2>/dev/null | head -n 1 || true)
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
    ls -la rust/target/ 2>/dev/null || true
    exit 1
fi
