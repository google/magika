#!/bin/sh
# Copyright 2025 Google LLC
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

# This script rename the package to magika-cli to publish to crates.io.

[ -z "$(git status -s)" ] || error "Repository is not clean"

info "Patch Cargo.toml"
sed -i '2s/"$/-cli"/;s/^magika_lib = { package = "magika",/magika = {/' Cargo.toml
echo >> Cargo.toml
echo '[[bin]]' >> Cargo.toml
echo 'name = "magika"' >> Cargo.toml
echo 'path = "src/main.rs"' >> Cargo.toml

info "Patch src/main.rs"
sed -i 's/^use magika_lib/use magika/;s/self as magika,//' src/main.rs
cargo fmt -- src/main.rs

info "Run tests"
./test.sh

todo "Create a temporary commit and run cargo publish"
