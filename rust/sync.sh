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

info "Generating files from gen"
( cd gen; cargo run; )
info "Generating files from out"
( cd out; ./run.sh; )

if [ "$1" = --check ]; then
  if ! git diff --exit-code; then
    [ -n "$CI" ] && todo 'Execute ./sync.sh from the rust directory'
    error 'Generated files are not in sync'
  fi
fi
success "Generated files are synced"
