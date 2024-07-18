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

set -ex

cargo check
cargo check --features=serde
cargo test --features=_test
cargo fmt -- --check
cargo clippy -- --deny=warnings
if cargo --version | grep -q nightly; then
  RUSTDOCFLAGS=--deny=warnings cargo doc --features=_doc
fi

# Make sure we can build for the targets we care about.
TARGETS='
x86_64-unknown-linux-gnu
aarch64-apple-darwin
x86_64-pc-windows-msvc
'
for target in $TARGETS; do
  cargo build --release --target=$target
done
