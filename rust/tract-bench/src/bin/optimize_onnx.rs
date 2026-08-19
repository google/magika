// Copyright 2026 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     https://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Exports ONNX Runtime's CPU-optimized form of an ONNX model for inspection.

use std::path::PathBuf;

use anyhow::{Context, Result, anyhow, bail};
use ort::session::Session;
use ort::session::builder::GraphOptimizationLevel;

fn main() -> Result<()> {
    let mut args = std::env::args_os().skip(1).map(PathBuf::from);
    let Some(source) = args.next() else {
        bail!("usage: optimize-onnx SOURCE.onnx DESTINATION.onnx");
    };
    let Some(destination) = args.next() else {
        bail!("usage: optimize-onnx SOURCE.onnx DESTINATION.onnx");
    };
    if args.next().is_some() {
        bail!("usage: optimize-onnx SOURCE.onnx DESTINATION.onnx");
    }

    ort::init().with_telemetry(false).commit();
    let _session = Session::builder()?
        .with_optimization_level(GraphOptimizationLevel::Level3)
        .map_err(|error| anyhow!(error.to_string()))?
        .with_optimized_model_path(&destination)
        .map_err(|error| anyhow!(error.to_string()))?
        .commit_from_file(&source)
        .with_context(|| format!("optimizing {} with ONNX Runtime", source.display()))?;
    Ok(())
}
