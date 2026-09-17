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
. ../color.sh

PROFILE=release-fast

x cargo check
x cargo check --features=_trace
x cargo build --profile=$PROFILE
x cargo fmt -- --check
x cargo clippy -- --deny=warnings
x cargo clippy --features=_trace -- --deny=warnings

PATH=$(dirname $PWD)/target/$PROFILE:$PATH

TEST_SUITES='basic previous_missdetections'
info "Test against the test suites: $TEST_SUITES"
( cd ../../tests_data
  magika --format='%p: %l' --recursive $TEST_SUITES | while read line; do
    file=${line%: *}
    directory=${file%/*}
    expected=${directory##*/}
    actual=${line#*: }
    [ "$expected" = "$actual" ] || error "$file is detected as $actual"
  done
)

test_error() {
  files="$1"
  expected="$2"
  ( set +e
    actual="$(magika $files)"
    code=$?
    [ $code -eq 1 ] || error "invalid exit code for magika $files"
    [ "$actual" = "$expected" ] || error "invalid output for magika $files"
  )
}

if [ $(id -u) -ne 0 -a -e /run/systemd/inaccessible ]; then
  info "Test permission error and non-regular files"
  test_error '--jsonl -r /run/systemd/inaccessible' \
'{"path":"/run/systemd/inaccessible/blk","result":{"status":"not_a_regular_file"}}
{"path":"/run/systemd/inaccessible/chr","result":{"status":"not_a_regular_file"}}
{"path":"/run/systemd/inaccessible/dir","result":{"status":"permission_error"}}
{"path":"/run/systemd/inaccessible/fifo","result":{"status":"not_a_regular_file"}}
{"path":"/run/systemd/inaccessible/reg","result":{"status":"permission_error"}}
{"path":"/run/systemd/inaccessible/sock","result":{"status":"not_a_regular_file"}}'
fi

info "Test nonexistent files"
test_error 'non_existent src/main.rs' "\
non_existent: No such file or directory (os error 2) (error)
src/main.rs: Rust source (code)"

info "Test file names that are not UTF-8"
test_error "$(printf 'f\xff')" "\
f�: No such file or directory (os error 2) (error)"
test_error "--json $(printf 'f\xff')" '[
  {
    "path": "f�",
    "result": {
      "status": "file_does_not_exist"
    }
  }
]'

info "Test directory cycles"
( dir=$(mktemp -d)
  trap "rm -rf $dir" EXIT
  mkdir -p $dir/tree/sub
  touch $dir/tree/empty
  ln -s .. $dir/tree/sub/back
  ln -s tree $dir/sibling
  test_error "-r $dir" "\
$dir/sibling/empty: Empty file (inode)
$dir/sibling/sub/back: Directory cycle (error)
$dir/tree/empty: Empty file (inode)
$dir/tree/sub/back: Directory cycle (error)"
)

info "Test exit code with broken pipe"
magika -r ../../tests_data | head -n1 >/dev/null
[ "${PIPESTATUS[0]}" -eq 0 ] || error "non-zero exit code with broken pipe"

success "Exit shell"
