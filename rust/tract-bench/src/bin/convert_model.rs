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

//! Converts Magika's deployment ONNX model into deterministic NNEF/tract-OPL.

use std::fs::File;
use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use flate2::{Compression, GzBuilder};
use tract_onnx::prelude::*;

fn main() -> Result<()> {
    let mut args = std::env::args_os().skip(1).map(PathBuf::from);
    let Some(source) = args.next() else {
        bail!("usage: convert-model SOURCE.onnx DESTINATION.nnef.tgz");
    };
    let Some(destination) = args.next() else {
        bail!("usage: convert-model SOURCE.onnx DESTINATION.nnef.tgz");
    };
    if args.next().is_some() {
        bail!("usage: convert-model SOURCE.onnx DESTINATION.nnef.tgz");
    }

    let mut model = tract_onnx::onnx()
        .model_for_path(&source)
        .with_context(|| format!("loading {}", source.display()))?;
    model
        .set_input_fact(
            0,
            InferenceFact::dt_shape(i32::datum_type(), tvec!(1.to_dim(), 2048.to_dim())),
        )
        .context("setting the [1, 2048] i32 input fact")?;
    let model = model
        .into_typed()
        .context("resolving the ONNX graph")?
        .into_decluttered()
        .context("decluttering the typed graph")?;

    let file = File::create(&destination)
        .with_context(|| format!("creating {}", destination.display()))?;
    let gzip = GzBuilder::new().mtime(0).write(file, Compression::best());
    let gzip = tract_nnef::nnef()
        .write_to_tar_with_config(&model, gzip, false, true)
        .context("serializing the model as deterministic NNEF/tract-OPL")?;
    gzip.finish().context("finishing the NNEF gzip stream")?;

    Ok(())
}
