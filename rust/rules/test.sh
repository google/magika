#!/bin/sh
# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
set -e
. ../color.sh
x cargo check --locked
x cargo check --locked --all-features --all-targets
x cargo test --locked
x cargo test --locked --no-default-features
x cargo test --locked --release --test corpus --test perf
x cargo fmt -- --check
x cargo clippy --locked --all-features --all-targets -- --deny=warnings
x env RUSTDOCFLAGS=--deny=warnings cargo doc --locked --all-features --no-deps
