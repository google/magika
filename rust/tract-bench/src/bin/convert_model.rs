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
use tract_onnx::tract_core::ops::array::Gather;

const EMBEDDING_MATMUL: &str = "jax2tf_get_logits_/pjit_get_logits_/MagikaV2/Dense_0/einsum/Einsum";

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
    let mut model = model
        .into_typed()
        .context("resolving the ONNX graph")?
        .into_decluttered()
        .context("decluttering the typed graph")?;
    rewrite_one_hot_embedding(&mut model)?;
    let model = model.into_decluttered().context("decluttering the embedding rewrite")?;

    let file = File::create(&destination)
        .with_context(|| format!("creating {}", destination.display()))?;
    let gzip = GzBuilder::new().mtime(0).write(file, Compression::best());
    let gzip = tract_nnef::nnef()
        .write_to_tar_with_config(&model, gzip, false, true)
        .context("serializing the model as deterministic NNEF/tract-OPL")?;
    gzip.finish().context("finishing the NNEF gzip stream")?;

    Ok(())
}

fn rewrite_one_hot_embedding(model: &mut TypedModel) -> Result<()> {
    let bytes = model.input_outlets().context("finding the model input")?[0];
    let embedding =
        model.node_by_name(EMBEDDING_MATMUL).context("finding the one-hot embedding matmul")?;
    let embedding_output = OutletId::new(embedding.id, 0);
    let weights = embedding.inputs[1];

    let mut patch = TypedModelPatch::new("replace one-hot embedding with gather");
    let bytes = patch.tap_model(model, bytes)?;
    let bytes = patch.wire_node(
        "embedding.indices",
        tract_onnx::tract_core::ops::cast::cast(i64::datum_type()),
        &[bytes],
    )?[0];
    let weights = patch.tap_model(model, weights)?;
    let gathered = patch.wire_node("embedding.gather", Gather::new(0), &[weights, bytes])?[0];
    patch.shunt_outside(model, embedding_output, gathered)?;
    patch.apply(model).context("applying the embedding gather rewrite")?;
    Ok(())
}
