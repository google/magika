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
source_model="$repo_dir/assets/models/standard_v3_3/model.onnx"
checked_model="$repo_dir/rust/tract-runtime/models/model.nnef.tgz"
checked_probe="$repo_dir/rust/tract-runtime/models/model.probe.f32le"

convert_model() {
  cargo run --quiet --manifest-path "$bench_dir/Cargo.toml" --no-default-features --features convert --bin convert-model -- "$source_model" "$1" "$2" "$3"
}

if [ "${1:-}" = "--check" ]; then
  if [ "$#" -gt 2 ]; then
    echo "usage: $0 --check [OUTPUT_DIRECTORY]" >&2
    exit 2
  fi
  mkdir -p "$repo_dir/tmp"
  candidate_dir=${2:-$(mktemp -d "$repo_dir/tmp/model-check.XXXXXX")}
  mkdir -p "$candidate_dir"
  candidate_dir=$(CDPATH= cd -- "$candidate_dir" && pwd)
  printf 'conversion_intermediates\t%s\n' "$candidate_dir"
  candidate_model="$candidate_dir/model.nnef.tgz"
  convert_model "$candidate_model" "$candidate_dir/model.probe.f32le" "$candidate_dir"
  cmp "$checked_model" "$candidate_model"
  cmp "$repo_dir/rust/tract-runtime/models/model.graph.json" "$candidate_dir/model.graph.json"
  cmp "$repo_dir/rust/tract-runtime/models/model.weights" "$candidate_dir/model.weights"
  MAGIKA_RELEASE_PROBE="$candidate_dir/model.probe.f32le" cargo test --quiet --manifest-path "$repo_dir/rust/tract-runtime/Cargo.toml" embedded_gpu_probe_matches_the_release_cpu_model
  exit 0
fi

if [ "$#" -ne 0 ]; then
  echo "usage: $0 [--check]" >&2
  exit 2
fi

convert_model "$checked_model" "$checked_probe" "$repo_dir/rust/tract-runtime/models"
