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

set -euo pipefail

TARGET="${1:-${CARGO_BUILD_TARGET:-}}"
ROOT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." &>/dev/null && pwd)
cd "$ROOT_DIR"

mkdir -p python/wheel_data/scripts
rm -f python/wheel_data/scripts/*

if [ -n "$TARGET" ]; then
    echo "Building magika CLI for target $TARGET..."
    cargo build --release --manifest-path rust/cli/Cargo.toml --target "$TARGET"
else
    echo "Building magika CLI for default target..."
    cargo build --release --manifest-path rust/cli/Cargo.toml
fi

# Locate the compiled binary across candidate output locations
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

# If still not found, search rust/target for recently built magika binary
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
