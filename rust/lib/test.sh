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
x cargo check --features=serde
x cargo test --features=_test
x cargo fmt -- --check
x cargo clippy -- --deny=warnings
if cargo --version | grep -q nightly; then
  x env RUSTDOCFLAGS=--deny=warnings cargo doc --features=_doc
fi
