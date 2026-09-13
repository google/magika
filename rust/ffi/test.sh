#!/bin/sh
# Copyright 2026 Google LLC
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
x cargo fmt -- --check
x cargo clippy -- --deny=warnings

# The static library leaves its dependencies' native libraries to the linker, and they depend on the
# target: macOS needs the Metal and Objective-C runtime frameworks. rustc reports them only when it
# compiles the crate, so rebuild it rather than hard-code a list.
x cargo clean --release --package ffi
build=$(cargo rustc --release --lib -- --print native-static-libs 2>&1) ||
  { printf '%s\n' "$build"; error 'building the C library failed'; }
native_libs=$(printf '%s\n' "$build" | sed -n 's/^note: native-static-libs: //p')
[ -n "$native_libs" ] || error 'rustc did not report the native libraries of the static library'
info "Native libraries of the static library: $native_libs"

compile_test() {
  x $cc -fsanitize=address,undefined -fno-omit-frame-pointer "$@"
  x ./test
  x rm test
}

TARGET_DIR=../target/release
for cc in gcc clang; do
  which $cc >/dev/null 2>&1 || continue
  # Word splitting is intended: the native libraries are separate linker arguments.
  compile_test -Iinclude test.c $TARGET_DIR/libmagika.a $native_libs -o test
  compile_test -Iinclude test.c -L$TARGET_DIR -lmagika -Wl,-rpath,$TARGET_DIR -o test
done
