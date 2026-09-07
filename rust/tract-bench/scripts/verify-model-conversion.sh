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
if [ "$#" -gt 1 ]; then
  echo "usage: $0 [OUTPUT_DIRECTORY]" >&2
  exit 2
fi
mkdir -p "$repo_dir/tmp"
candidate_dir=${1:-$(mktemp -d "$repo_dir/tmp/model-conversion.XXXXXX")}
mkdir -p "$candidate_dir"
candidate_dir=$(CDPATH= cd -- "$candidate_dir" && pwd)
printf 'conversion_intermediates\t%s\n' "$candidate_dir"
cd "$repo_dir"

executable_suffix=""
case "$(uname -s)" in
  MINGW* | MSYS* | CYGWIN*) executable_suffix=".exe" ;;
esac

cargo build --quiet --manifest-path "$bench_dir/Cargo.toml" --no-default-features --features convert --bin convert-model
converter="$repo_dir/rust/target/debug/convert-model$executable_suffix"
for source_model in "$repo_dir"/assets/models/*/model.onnx; do
  model_name=$(basename "$(dirname "$source_model")")
  first="$candidate_dir/$model_name.first.nnef.tgz"
  second="$candidate_dir/$model_name.second.nnef.tgz"
  if [ "$model_name" = "standard_v3_3" ]; then
    "$converter" "$source_model" "$first" "$candidate_dir/model.probe.f32le" "$candidate_dir/portable"
  else
    "$converter" "$source_model" "$first"
  fi
  if [ "$model_name" = "standard_v3_3" ]; then
    "$converter" "$source_model" "$second" "$candidate_dir/model.second.probe.f32le" "$candidate_dir/portable-second"
    cmp "$candidate_dir/model.probe.f32le" "$candidate_dir/model.second.probe.f32le"
    cmp "$candidate_dir/portable/model.graph.json" "$candidate_dir/portable-second/model.graph.json"
    cmp "$candidate_dir/portable/model.weights" "$candidate_dir/portable-second/model.weights"
  else
    "$converter" "$source_model" "$second"
  fi
  cmp "$first" "$second"
  gzip -t "$first"
  tar -tzf "$first" | grep -qx 'graph.nnef'
  printf 'round_trip\t%s\t%s bytes\n' "$model_name" "$(wc -c <"$first" | tr -d ' ')"
done

cargo build --quiet --manifest-path "$bench_dir/Cargo.toml" --features verify --bin verify-model
verifier="$repo_dir/rust/target/debug/verify-model$executable_suffix"
for source_model in "$repo_dir"/assets/models/*/model.onnx; do
  model_name=$(basename "$(dirname "$source_model")")
  candidate="$candidate_dir/$model_name.first.nnef.tgz"
  case "$model_name" in
    fast_v2_1) feature_size=1024 ;;
    standard_v2_0|standard_v2_1) feature_size=4096 ;;
    begonly_v2_1|standard_v3_0|standard_v3_1|standard_v3_2|standard_v3_3) feature_size=2048 ;;
    *) printf 'unknown feature size for %s\n' "$model_name" >&2; exit 1 ;;
  esac
  for batch in 1 8 16 32 64; do
    "$verifier" --onnx-model "$source_model" --nnef-model "$candidate" --feature-size "$feature_size" --batch "$batch"
  done
  printf 'verified_model\t%s\t%s bytes\n' "$model_name" "$(wc -c <"$candidate" | tr -d ' ')"
done

current="$candidate_dir/standard_v3_3.first.nnef.tgz"
cmp "$repo_dir/rust/tract-runtime/models/model.nnef.tgz" "$current"
cargo test --quiet --manifest-path "$repo_dir/rust/tract-runtime/Cargo.toml" release_cpu_graph_has_every_required_fusion
MAGIKA_RELEASE_PROBE="$candidate_dir/model.probe.f32le" cargo test --quiet --manifest-path "$repo_dir/rust/tract-runtime/Cargo.toml" embedded_gpu_probe_matches_the_release_cpu_model
# Neither gate above reaches the fused convolution: the verifier runs the unfused NNEF, the fusion
# contract only counts matches, and the score probe is batch one, below the batch the fusion needs.
# These run the fused graph and check the numbers it produces.
cargo test --quiet --manifest-path "$repo_dir/rust/tract-runtime/Cargo.toml" the_fallback_packing_path_scores_the_release_model_the_same
cargo test --quiet --manifest-path "$repo_dir/rust/tract-runtime/Cargo.toml" both_packing_paths_agree
cmp "$repo_dir/rust/tract-runtime/models/model.graph.json" "$candidate_dir/portable/model.graph.json"
cmp "$repo_dir/rust/tract-runtime/models/model.weights" "$candidate_dir/portable/model.weights"
cargo test --quiet --manifest-path "$repo_dir/rust/tract-runtime/Cargo.toml" artifact::tests -- --test-threads=1
printf 'verified_release_artifacts\t%s\t%s\n' "$current" "$repo_dir/rust/tract-runtime/models/model.probe.f32le"

if [ "$(uname -s)" = "Darwin" ]; then
  # This is the production runtime, including its fail-closed score probe and graph contracts.
  cargo run --quiet --manifest-path "$bench_dir/Cargo.toml" --no-default-features -- --backend gpu --batch 8 --threads 1 --iterations 1
  printf 'verified_production_gpu\t%s\n' "$current"
fi
