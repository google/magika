#!/usr/bin/env sh

set -eu

input_dir=${1:-.}
total=0

for path in "$input_dir"/*; do
  if [ -f "$path" ]; then
    total=$((total + 1))
    printf '%s\n' "$path"
  fi
done

printf 'files=%s\n' "$total"
