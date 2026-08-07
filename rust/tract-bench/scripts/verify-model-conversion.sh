#!/bin/sh
# Copyright 2026 Google LLC
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     https://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

set -eu

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
bench_dir=$(dirname -- "$script_dir")
repo_dir=$(CDPATH= cd -- "$bench_dir/../.." && pwd)
candidate_dir=$(mktemp -d)
trap 'rm -rf "$candidate_dir"' EXIT HUP INT TERM

cargo build --quiet --manifest-path "$bench_dir/Cargo.toml" \
  --no-default-features --features convert --bin convert-model
converter="$repo_dir/rust/target/debug/convert-model"
for source_model in "$repo_dir"/assets/models/*/model.onnx; do
  model_name=$(basename "$(dirname "$source_model")")
  first="$candidate_dir/$model_name.first.nnef.tgz"
  second="$candidate_dir/$model_name.second.nnef.tgz"
  "$converter" "$source_model" "$first"
  "$converter" "$source_model" "$second"
  cmp "$first" "$second"
  gzip -t "$first"
  tar -tzf "$first" | grep -qx 'graph.nnef'
  printf 'round_trip\t%s\t%s bytes\n' "$model_name" "$(wc -c <"$first" | tr -d ' ')"
done

first="$candidate_dir/standard_v3_3.first.nnef.tgz"

cargo build --quiet --manifest-path "$bench_dir/Cargo.toml" --bin magika-runtime-bench
benchmark="$repo_dir/rust/target/debug/magika-runtime-bench"
for batch in 1 8 16 32 64; do
  "$benchmark" --nnef-model "$first" --fixed-batch --batch "$batch" --verify
done

printf 'verified_current_model\t%s\t%s bytes\n' "$first" "$(wc -c <"$first" | tr -d ' ')"
