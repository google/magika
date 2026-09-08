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

ROOT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." &>/dev/null && pwd)
cd "$ROOT_DIR"

echo "Building native CLI binary (magika)..."
cargo build --release --manifest-path rust/cli/Cargo.toml

echo "Staging binary into python/wheel_data/scripts/..."
rm -f python/wheel_data/scripts/*
mkdir -p python/wheel_data/scripts
cp rust/target/release/magika python/wheel_data/scripts/magika
chmod +x python/wheel_data/scripts/magika

echo "Building wheel via uv build..."
cd python
uv build --wheel
echo "Wheel build complete. Output in python/dist/"
