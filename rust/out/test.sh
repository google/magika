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

cd ../..
run() {
  info "Run with $*"
  local file
  local arg
  for arg in "$@"; do
    [ -z "$file" ] || file="$file"_
    file="$file${arg#--}"
  done
  file=rust/out/"$file"
  local paths='tests_data/basic tests_data/mitra'
  rust/target/release/magika --recursive $paths "$@" > "$file".out 2> "$file".err
  [ -n "$(cat "$file".err)" ] || rm "$file".err
}

run --colors
run --json
run --jsonl
run --generate-report
run --label
run --mime-type --output-score
git diff --exit-code || error "Difference in CLI output"
