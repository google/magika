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

fail() {
  kind=$1
  dir=$2
  case $kind in
    stale) message="Some changes have not been logged." ;;
    format) message="This line should be an H2 version." ;;
    diff) message="This version differs from the Cargo.toml file." ;;
    *) error "Unsupported kind '$kind'" ;;
  esac
  if [ -z "$CI" ]
  then error "$message"
  else echo "::warning file=rust/$dir/CHANGELOG.md,line=3::$message"
  fi
}

for dir in lib cli; do
  ( cd $dir
    info "Checking $dir"
    ref=$(git log -n1 --pretty=format:%H origin/main.. -- CHANGELOG.md)
    [ -n "$ref" ] || ref=origin/main
    git diff --quiet $ref -- Cargo.toml src || fail stale $dir
    cver="$(sed -n '3s/^## //p' CHANGELOG.md)"
    [ -n "$cver" ] || fail format $dir
    pver="$(sed -n '/^\[package]$/,/^$/{s/^version = "\(.*\)"$/\1/p}' Cargo.toml)"
    [ "$pver" = "$cver" ] || fail diff $dir
  )
done
