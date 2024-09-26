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
. ./color.sh

# This script removes all -dev version suffixes and creates a commit.

[ -z "$(git status -s)" ] || error "Repository is not clean"

info "Removing all -dev suffixes (if any)"
sed -i 's/-dev"/"/' $(git ls-files '*/Cargo.*')
sed -i 's/-dev//' $(git ls-files '*/CHANGELOG.md')
if [ -n "$(git status -s)" ]; then
  info "Creating a commit with those changes"
  git commit -aqm'Release Rust crates'
  todo "Create a PR with this commit, merge it, and publish from the PR commit"
else
  success 'No change since last release'
fi
