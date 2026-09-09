// Copyright 2026 Google LLC
// SPDX-License-Identifier: Apache-2.0
//! Regenerate portable runtime graphs from an already verified NNEF model.

use std::path::PathBuf;

use anyhow::{Context, Result, ensure};

fn main() -> Result<()> {
    let mut args = std::env::args_os().skip(1).map(PathBuf::from);
    let source = args.next().context("usage: export-runtime MODEL.nnef.tgz OUTPUT_DIRECTORY")?;
    let destination = args.next().context("missing output directory")?;
    ensure!(args.next().is_none(), "unexpected arguments");
    magika_tract_runtime::export_model_artifact(&source, &destination)
}
