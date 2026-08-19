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

set -e
# The workflow runs this from the repository root, but the commands below and color.sh are relative
# to the crate, so resolve where this script lives rather than assuming the caller's directory.
cd -- "$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
. ../color.sh

x cargo check
x cargo test
x cargo fmt -- --check
x cargo clippy --all-targets -- --deny=warnings
