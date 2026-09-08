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

TARGET="${1:-}"
ROOT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." &>/dev/null && pwd)
cd "$ROOT_DIR"

mkdir -p python/wheel_data/scripts
rm -f python/wheel_data/scripts/*

if [ -n "$TARGET" ]; then
    echo "Building magika CLI for target $TARGET..."
    cargo build --release --manifest-path rust/cli/Cargo.toml --target "$TARGET"
    SRC_DIR="rust/target/$TARGET/release"
else
    echo "Building magika CLI for default target..."
    cargo build --release --manifest-path rust/cli/Cargo.toml
    SRC_DIR="rust/target/release"
fi

if [ -f "$SRC_DIR/magika.exe" ]; then
    cp "$SRC_DIR/magika.exe" python/wheel_data/scripts/
    echo "Staged magika.exe into python/wheel_data/scripts/"
elif [ -f "$SRC_DIR/magika" ]; then
    cp "$SRC_DIR/magika" python/wheel_data/scripts/
    chmod +x python/wheel_data/scripts/magika
    echo "Staged magika into python/wheel_data/scripts/"
else
    echo "Error: CLI binary not found in $SRC_DIR" >&2
    exit 1
fi
