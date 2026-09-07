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

# This script publishes all crates.

[ -z "$(git status -s)" ] || error "Repository is not clean"

info "Removing all -dev suffixes (if any)"
# Python is already required for source staging; avoid GNU/BSD sed -i differences.
python3 - <<'PYTHON'
import os
import subprocess
from pathlib import Path

for pattern, old, new in (
    ("*/Cargo.*", b'-dev"', b'"'),
    ("*/CHANGELOG.md", b"-dev", b""),
):
    for name in subprocess.check_output(["git", "ls-files", "-z", "--", pattern]).split(b"\0"):
        if name:
            path = Path(os.fsdecode(name))
            content = path.read_bytes()
            updated = b"".join(
                line.replace(old, new, 1) for line in content.splitlines(keepends=True)
            )
            path.write_bytes(updated)
PYTHON
if [ -n "$(git status -s)" ]; then
  info "Creating a commit with those changes"
  git commit -aqm'Release Rust crates'
  todo "Create a PR with this commit"
  success "Then re-run from the merged PR"
fi

info "Making sure we run from a merged PR"
git log -1 --pretty=%s | grep -q '^Release Rust crates (#[0-9]*)$' \
  || error "This is not a merged release PR"

[ "$1" = --no-dry-run ] || success "Run with --no-dry-run to actually publish"

info "Publishing the tract runtime"
( cd tract-runtime && cargo publish --locked )

info "Publishing the library"
source_stage=$(mktemp -d)
trap 'rm -rf "$source_stage"' EXIT HUP INT TERM
python3 ../rules/package.py source --output "$source_stage/source"
cargo publish --manifest-path "$source_stage/source/lib/Cargo.toml"

info "Publishing the CLI"
( cd cli && cargo publish )

success 'All crates have been published'
