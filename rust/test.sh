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

TOOLCHAINS='stable nightly'
[ -z "$CI" ] || TOOLCHAINS=$(rustup show active-toolchain | sed 's/-.*//')

for toolchain in $TOOLCHAINS; do
  for dir in gen lib cli; do
    info "Running tests from $dir with $toolchain"
    ( cd $dir && rustup run $toolchain ./test.sh; )
  done
done

./sync.sh --check
