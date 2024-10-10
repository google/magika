#!/bin/bash
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

set -e
cd "$(dirname "$0")"
. ../color.sh

# This script sets up the maturin container for a manylinux build.

info "Build ONNX Runtime from source."
# We are root in maturin containers and ONNX Runtime doesn't like building as root by default.
export ONNX_RUNTIME_BUILD_FLAGS=--allow_running_as_root
./build.sh

info "Test Magika CLI in the container."
cd ../cli
rustup default stable
rustup component add rustfmt clippy
./test.sh
