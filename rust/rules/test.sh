#!/bin/sh
# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0

set -e
. ../color.sh

x cargo check
x cargo check --no-default-features
x cargo test
x cargo test --no-default-features
x cargo test --release --test=perf
x cargo fmt -- --check
x cargo clippy -- --deny=warnings
x env RUSTDOCFLAGS=--deny=warnings cargo doc --no-deps
