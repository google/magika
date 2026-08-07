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

use anyhow::{Context, Result, bail, ensure};
use flate2::{Compression, GzBuilder};
use tract_hir::infer::InferenceModelPatch;
use tract_hir::ops::array::Gather;
use tract_hir::ops::expandable::expand;
use tract_onnx::prelude::*;

const EMBEDDING_MATMUL: &str = "jax2tf_get_logits_/pjit_get_logits_/MagikaV2/Dense_0/einsum/Einsum";

fn main() -> Result<()> {
    let mut args = std::env::args_os().skip(1).map(PathBuf::from).peekable();
    let batch = if args.peek().is_some_and(|arg| arg == std::path::Path::new("--batch")) {
        let _flag = args.next();
        let value = args.next().context("--batch needs a positive integer")?;
        let value = value
            .to_str()
            .context("--batch must be valid UTF-8")?
            .parse::<usize>()
            .context("--batch must be a positive integer")?;
        ensure!(value > 0, "--batch must be greater than zero");
        Some(value)
    } else {
        None
    };
    let Some(source) = args.next() else {
        bail!("usage: convert-model [--batch N] SOURCE.onnx DESTINATION.nnef.tgz");
    };
    let Some(destination) = args.next() else {
        bail!("usage: convert-model [--batch N] SOURCE.onnx DESTINATION.nnef.tgz");
    };
    if args.next().is_some() {
        bail!("usage: convert-model [--batch N] SOURCE.onnx DESTINATION.nnef.tgz");
    }

    let mut model = tract_onnx::onnx()
        .model_for_path(&source)
        .with_context(|| format!("loading {}", source.display()))?;
    let batch = batch.map_or_else(|| model.sym("N").to_dim(), |batch| batch.to_dim());
    model
        .set_input_fact(
            0,
            InferenceFact::dt_shape(i32::datum_type(), tvec!(batch.clone(), 2048.to_dim())),
        )
        .context("setting the [N, 2048] i32 input fact")?;
    rewrite_one_hot_embedding(&mut model)?;
    remove_full_slice(
        &mut model,
        "jax2tf_get_logits_/pjit_get_logits_/MagikaV2/Conv_0/strided_slice",
    )?;
    remove_full_slice(
        &mut model,
        "jax2tf_get_logits_/pjit_get_logits_/MagikaV2/Conv_0/strided_slice_2",
    )?;
    rewrite_symbolic_shapes(&mut model, batch)?;
    let mut model = model.into_typed().context("resolving the ONNX graph")?;
    rewrite_gelu_approximations(&mut model)?;
    let model = model.into_decluttered().context("decluttering the typed graph")?;

    let file = File::create(&destination)
        .with_context(|| format!("creating {}", destination.display()))?;
    let gzip = GzBuilder::new().mtime(0).write(file, Compression::best());
    let gzip = tract_nnef::nnef()
        .write_to_tar_with_config(&model, gzip, false, true)
        .context("serializing the model as deterministic NNEF/tract-OPL")?;
    gzip.finish().context("finishing the NNEF gzip stream")?;

    Ok(())
}

fn rewrite_gelu_approximations(model: &mut TypedModel) -> Result<()> {
    const GELUS: &[&str] = &[
        "jax2tf_get_logits_/pjit_get_logits_/MagikaV2/ApplyActivation_0/Mul_5",
        "jax2tf_get_logits_/pjit_get_logits_/MagikaV2/ApplyActivation_1/Mul_5",
    ];

    for output_name in GELUS {
        let output_node =
            model.node_by_name(output_name).with_context(|| format!("finding {output_name}"))?;
        ensure!(output_node.inputs.len() == 2, "{output_name} is not the expected final GELU mul");
        let input = output_node.inputs[0];
        let output = OutletId::new(output_node.id, 0);
        let mut patch = TypedModelPatch::default();
        let input = patch.tap_model(model, input)?;
        let gelu = patch.wire_node(
            format!("{output_name}.fused-gelu"),
            tract_core::ops::nn::gelu_approximate::gelu_approximate(false),
            &[input],
        )?;
        patch.shunt_outside(model, output, gelu[0])?;
        patch.apply(model).with_context(|| format!("fusing GELU ending at {output_name}"))?;
        model.compact().with_context(|| format!("compacting after fusing {output_name}"))?;
    }
    Ok(())
}

fn remove_full_slice(model: &mut InferenceModel, node_name: &str) -> Result<()> {
    let node = model.node_by_name(node_name).with_context(|| format!("finding {node_name}"))?;
    let input = node.inputs[0];
    let output = OutletId::new(node.id, 0);
    let mut patch = InferenceModelPatch::new("remove full-range slice");
    let input = patch.tap_model(model, input)?;
    patch.shunt_outside(model, output, input)?;
    patch.apply(model).with_context(|| format!("removing {node_name}"))?;
    model.compact().with_context(|| format!("compacting after removing {node_name}"))?;
    Ok(())
}

fn rewrite_symbolic_shapes(model: &mut InferenceModel, batch: TDim) -> Result<()> {
    replace_shape_input(
        model,
        "jax2tf_get_logits_/pjit_get_logits_/MagikaV2/Reshape",
        tvec!(batch.clone(), 512.to_dim(), 256.to_dim()),
    )?;
    for name in [
        "jax2tf_get_logits_/pjit_get_logits_/MagikaV2/LayerNorm_0/Reshape",
        "jax2tf_get_logits_/pjit_get_logits_/MagikaV2/LayerNorm_0/BroadcastTo",
        "jax2tf_get_logits_/pjit_get_logits_/MagikaV2/LayerNorm_0/Reshape_1",
        "jax2tf_get_logits_/pjit_get_logits_/MagikaV2/LayerNorm_0/BroadcastTo_1",
    ] {
        replace_shape_input(model, name, tvec!(batch.clone(), 1.to_dim(), 256.to_dim()))?;
    }
    for name in [
        "jax2tf_get_logits_/pjit_get_logits_/MagikaV2/LayerNorm_1/Reshape",
        "jax2tf_get_logits_/pjit_get_logits_/MagikaV2/LayerNorm_1/BroadcastTo",
        "jax2tf_get_logits_/pjit_get_logits_/MagikaV2/LayerNorm_1/Reshape_1",
        "jax2tf_get_logits_/pjit_get_logits_/MagikaV2/LayerNorm_1/BroadcastTo_1",
        "jax2tf_get_logits_/pjit_get_logits_/Reshape",
        "jax2tf_get_logits_/pjit_get_logits_/BroadcastTo",
        "jax2tf_get_logits_/pjit_get_logits_/Reshape_1",
        "jax2tf_get_logits_/pjit_get_logits_/BroadcastTo_1",
    ] {
        replace_shape_input(model, name, tvec!(batch.clone(), 1.to_dim()))?;
    }
    Ok(())
}

fn replace_shape_input(
    model: &mut InferenceModel, node_name: &str, shape: TVec<TDim>,
) -> Result<()> {
    let node = model.node_by_name(node_name).with_context(|| format!("finding {node_name}"))?;
    ensure!(node.inputs.len() == 2, "{node_name} does not have a shape input");
    let inlet = InletId::new(node.id, 1);
    let shape = model.add_const(format!("{node_name}.symbolic-shape"), tensor1(&shape))?;
    model
        .add_edge(shape, inlet)
        .with_context(|| format!("replacing the shape input for {node_name}"))?;
    Ok(())
}

fn rewrite_one_hot_embedding(model: &mut InferenceModel) -> Result<()> {
    let bytes = model.input_outlets().context("finding the model input")?[0];
    let embedding =
        model.node_by_name(EMBEDDING_MATMUL).context("finding the one-hot embedding matmul")?;
    let embedding_output = OutletId::new(embedding.id, 0);
    let weights = embedding.inputs[1];

    let mut patch = InferenceModelPatch::new("replace one-hot embedding with gather");
    let bytes = patch.tap_model(model, bytes)?;
    let weights = patch.tap_model(model, weights)?;
    let gathered =
        patch.wire_node("embedding.gather", expand(Gather::new(0)), &[weights, bytes])?[0];
    patch.shunt_outside(model, embedding_output, gathered)?;
    patch.apply(model).context("applying the embedding gather rewrite")?;
    model.compact().context("removing the replaced one-hot subgraph")?;
    Ok(())
}
