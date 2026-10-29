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

command -v zstd >/dev/null 2>&1 || {
  echo "zstd is required" >&2
  exit 1
}

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
bench_dir=$(dirname -- "$script_dir")
repo_dir=$(CDPATH= cd -- "$bench_dir/../.." && pwd)
onnx="$repo_dir/assets/models/standard_v3_3/model.onnx"
nnef_gzip="$repo_dir/rust/tract-runtime/models/model.nnef.tgz"
candidate_dir=$(mktemp -d)
trap 'rm -rf "$candidate_dir"' EXIT HUP INT TERM

nnef_tar="$candidate_dir/model.nnef.tar"
onnx_gzip="$candidate_dir/model.onnx.gz"
onnx_zstd="$candidate_dir/model.onnx.zst"
nnef_zstd="$candidate_dir/model.nnef.tar.zst"

gzip -dc "$nnef_gzip" >"$nnef_tar"
gzip -9 -c "$onnx" >"$onnx_gzip"
zstd -q -19 -f "$onnx" -o "$onnx_zstd"
zstd -q -19 -f "$nnef_tar" -o "$nnef_zstd"

size() {
  wc -c <"$1" | tr -d ' '
}

report() {
  codec=$1
  onnx_bytes=$(size "$2")
  nnef_bytes=$(size "$3")
  delta=$(awk -v onnx="$onnx_bytes" -v nnef="$nnef_bytes" \
    'BEGIN { printf "%.4f%%", (nnef / onnx - 1) * 100 }')
  printf '%s\t%s\t%s\t%s\n' "$codec" "$onnx_bytes" "$nnef_bytes" "$delta"
}

printf 'codec\tonnx_bytes\tnnef_bytes\tnnef_change\n'
report raw "$onnx" "$nnef_tar"
report gzip-9 "$onnx_gzip" "$nnef_gzip"
report zstd-19 "$onnx_zstd" "$nnef_zstd"
