#!/bin/sh
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
. ../color.sh

info "Build CLI"
cd ../cli
cargo build --release

info "Generate labels"
cd ../../tests_data
../rust/target/release/magika --format='%p: %l' --recursive basic mitra > ../rust/out/labels

info "Generate flags"
cd ../rust/out
( set -x
  ../target/release/magika test.sh
  ../target/release/magika test.sh --colors
  ../target/release/magika test.sh --output-score
  ../target/release/magika test.sh --json
  ../target/release/magika test.sh README.md --json
  ../target/release/magika test.sh --jsonl
  ../target/release/magika test.sh README.md --jsonl
  ../target/release/magika test.sh --mime-type
) > flags 2>&1

git diff --exit-code || error "Difference in CLI output"