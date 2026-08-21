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

x cargo check
x cargo build --release
x cargo fmt -- --check
x cargo clippy -- --deny=warnings

PATH=$(dirname $PWD)/target/release:$PATH

info "Test inference backend CLI"
help=$(magika --help)
grep -F '[possible values: auto, cpu, gpu]' <<<"$help" >/dev/null
grep -F '[default: auto]' <<<"$help" >/dev/null
backend=$(magika --backend cpu -v src/main.rs 2>&1)
grep -F 'backend: cpu (tract-cpu)' <<<"$backend" >/dev/null
( set +e
  magika --backend metal src/main.rs >/dev/null 2>&1
  [ $? -eq 2 ] || error "invalid backend should be rejected by argument parsing"
)

TEST_SUITES='basic previous_missdetections'
# Both backends, not just whichever one auto picks here. They run different graphs through
# different kernels, and a backend that computes the wrong thing still exits successfully, so
# checking only one leaves the other free to be silently wrong.
for BACKEND in auto cpu; do
  info "Test against the test suites with --backend $BACKEND: $TEST_SUITES"
  ( cd ../../tests_data
    magika --backend $BACKEND --format='%p: %l' --recursive $TEST_SUITES | while read line; do
      file=${line%: *}
      directory=${file%/*}
      expected=${directory##*/}
      actual=${line#*: }
      [ "$expected" = "$actual" ] || error "$file is detected as $actual with --backend $BACKEND"
    done
  )
done

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
unreadable=$(mktemp)
trap 'chmod u+rw "$unreadable"; rm -f "$unreadable"' EXIT
chmod 000 "$unreadable"
test_error "$unreadable" "\
$unreadable: Permission denied (os error 13) (error)"
test_error 'non_existent src/main.rs' "\
non_existent: No such file or directory (os error 2) (error)
src/main.rs: Rust source (code)"

info "Test exit code with broken pipe"
magika -r ../../tests_data > >(head -n1 >/dev/null) &
magika_pid=$!
( sleep 10; kill -TERM "$magika_pid" 2>/dev/null ) &
watchdog_pid=$!
set +e
wait "$magika_pid"
code=$?
set -e
kill -TERM "$watchdog_pid" 2>/dev/null || true
wait "$watchdog_pid" 2>/dev/null || true
[ "$code" -eq 0 ] || error "non-zero exit code or timeout with broken pipe"
