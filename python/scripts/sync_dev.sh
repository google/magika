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

echo "Building and staging native CLI binary (magika)..."
./python/scripts/build_and_stage_cli.sh

echo "Syncing local Python environment and reinstalling editable magika package..."
cd python
uv sync --all-extras --dev --reinstall-package magika
echo "Local Python dev environment is synced (both _magika PyO3 extension and magika CLI are ready)."
