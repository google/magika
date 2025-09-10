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

# This script updates (or creates if it does not exist) the trampoline release for the website. The
# trampoline release is a "fake" release (which commits doesn't matter) containing a copy of the
# bash and powershell scripts from the latest cli/vX.Y.Z release.

REPO=google/magika
RELEASES=$(gh release list --repo=$REPO --json=tagName |
             tr -d '[{}]' | tr , '\n' | sed 's/^.*"\([^"]*\)"$/\1/')
info "Found the following releases"
echo "$RELEASES"

TAG=$(echo "$RELEASES" | grep cli/ | head -n1)
LATEST=cli-latest
[ -n "$TAG" ] || error "No CLI release found"
info "The latest CLI release is $TAG"

x gh release download --repo=$REPO $TAG --pattern='magika-installer.*'

info "Delete the previous trampoline release, if any"
echo "$RELEASES" | grep -q $LATEST && x gh release delete --repo=$REPO --yes $LATEST

cat >> notes.txt <<EOF
The latest CLI release is [$TAG](https://github.com/$REPO/releases/tag/$TAG).

This moving release is just a trampoline for the website, to provide stable URLs for the latest
install scripts.
EOF

info "Create the trampoline release"
x gh release create --repo=$REPO $LATEST --latest=false \
  --title="Trampoline to the latest CLI release" --notes-file=notes.txt \
  magika-installer.*
