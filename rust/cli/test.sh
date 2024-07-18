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

x cargo check
x cargo build --release
x cargo fmt -- --check
x cargo clippy -- --deny=warnings

PATH=$(dirname $PWD)/target/release:$PATH

info 'Test against the basic and mitra test suites'
( cd ../../tests_data
  magika --format='%p: %l' --recursive basic mitra | while read line; do
    file=${line%: *}
    directory=${file%/*}
    expected=${directory##*/}
    actual=${line#*: }
    [ "$expected" = "$actual" ] || error "$file is detected as $actual"
  done
)

info 'Test against expected output'
( set -x
  magika test.sh
  magika test.sh --colors
  magika test.sh --output-score
  magika test.sh --json
  magika test.sh README.md --json
  magika test.sh --jsonl
  magika test.sh README.md --jsonl
  magika test.sh --mime-type
) > output 2>&1
git diff --exit-code -- output || error 'Unexpected output'
