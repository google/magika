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

# We rely below on the fact that we don't have permission on /etc/shadow.
[ $(id -u) -eq 0 ] && success "No more tests in Docker"

info "Test exit code with at least one error"
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
test_error '/etc/shadow' "\
/etc/shadow: Permission denied (os error 13) (error)"
test_error 'non_existent src/main.rs' "\
non_existent: No such file or directory (os error 2) (error)
src/main.rs: Rust source (code)"
