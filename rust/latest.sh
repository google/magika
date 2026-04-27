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
. ./color.sh

# This script updates (or creates if it does not exist) the `cli-latest` release.
#
# The `cli-latest` release replicates the assets and commit hash of the latest `cli/vX.Y.Z` release.
# The website uses this release for the install scripts. Other tools may also use this release to
# reference the latest CLI release. This is necessary because the `latest` release is not
# necessarily a CLI release due to Python.
#
# This script relies on the repository disabling release immutability since this release is mutated.

REPO=google/magika
RELEASES=$(gh release list --repo=$REPO --json=tagName |
             tr -d '[{}]' | tr , '\n' | sed 's/^.*"\([^"]*\)"$/\1/')
info "Found the following releases"
echo "$RELEASES"

TAG=$(echo "$RELEASES" | grep cli/ | head -n1)
COMMIT=$(git rev-parse $TAG)
LATEST=cli-latest
[ -n "$TAG" ] || error "No CLI release found"
info "The latest CLI release is $TAG"

x gh release download $TAG --repo=$REPO --dir=assets

info "Delete the previous $LATEST release, if any"
echo "$RELEASES" | grep -q $LATEST && x gh release delete $LATEST --repo=$REPO --yes --cleanup-tag

cat >> notes.txt <<EOF
This moving release simply replicates the assets and commit hash of the latest CLI release.

The latest CLI release is [$TAG](https://github.com/$REPO/releases/tag/$TAG).
EOF

info "Create the $LATEST release"
x gh release create $LATEST --repo=$REPO --target=$COMMIT --latest=false \
  --title="Latest CLI release" --notes-file=notes.txt assets/*

x rm -r notes.txt assets
