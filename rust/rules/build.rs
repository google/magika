// Copyright 2026 Google LLC
// SPDX-License-Identifier: Apache-2.0

//! Writes the bundled rules to `OUT_DIR/bundled.yar`.

fn main() {
    let out = std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap()).join("bundled.yar");
    std::fs::write(out, "").unwrap();
}
