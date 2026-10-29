#!/bin/sh
# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0

set -e
. ../color.sh

x cargo check
x cargo test
x cargo fmt -- --check
x cargo clippy -- --deny=warnings
x env RUSTDOCFLAGS=--deny=warnings cargo doc --no-deps
