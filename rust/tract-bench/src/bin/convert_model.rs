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

//! Converts an ONNX model into a deterministic, optimized, compressed NNEF archive.

use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail, ensure};
use flate2::{Compression, GzBuilder};
use tract_core::internal::DimLike as _;
use tract_core::ops::binary::TypedBinOp;
use tract_core::ops::element_wise::ElementWiseOp;
use tract_core::ops::math::{Add, Mul, Pow, Tanh};
use tract_core::runtime::{DefaultRuntime, Runtime as _};
use tract_hir::infer::{Factoid, InferenceModelPatch};
use tract_hir::ops::array::Gather;
use tract_hir::ops::binary::BinIntoHir;
use tract_hir::ops::element_wise::ElementWiseIntoHir;
use tract_hir::ops::expandable::expand;
use tract_onnx::prelude::*;

fn main() -> Result<()> {
    let mut args = std::env::args_os().skip(1).map(PathBuf::from);
    let Some(source) = args.next() else {
        bail!("usage: convert-model SOURCE.onnx DESTINATION.nnef.tgz [PROBE.f32le]");
    };
    let Some(destination) = args.next() else {
        bail!("usage: convert-model SOURCE.onnx DESTINATION.nnef.tgz [PROBE.f32le]");
    };
    let probe = args.next();
    if args.next().is_some() {
        bail!("usage: convert-model SOURCE.onnx DESTINATION.nnef.tgz [PROBE.f32le]");
    }

    let onnx = tract_onnx::onnx();
    let proto = onnx
        .proto_model_for_path(&source)
        .with_context(|| format!("reading ONNX attributes from {}", source.display()))?;
    let batch_norm_epsilons = batch_norm_epsilons(&proto)?;
    let model = onnx
        .model_for_path(&source)
        .with_context(|| format!("loading ONNX model {}", source.display()))?;
    let model = prepare_nnef(model, &batch_norm_epsilons)
        .with_context(|| format!("optimizing ONNX model {}", source.display()))?;

    write_nnef(&model, &destination)?;
    verify_rust_round_trip(&destination)?;
    if let Some(probe) = probe {
        write_probe_reference(&destination, &probe)?;
    }

    Ok(())
}

/// Writes the release probe's batch-one CPU scores as deterministic little-endian f32 values.
fn write_probe_reference(model: &Path, destination: &Path) -> Result<()> {
    let model = tract_nnef::nnef()
        .model_for_path(model)
        .with_context(|| format!("reloading NNEF archive for probe {}", model.display()))?;
    let batch = model.symbols.get("N").context("converted NNEF has no batch symbol N")?;
    let symbols = HashMap::from([(batch, 1.to_dim())]);
    let model = model.set_symbols(&symbols).context("binding batch one for the release probe")?;
    let feature_size = model.input_fact(0)?.shape[1].to_usize()?;
    let model = model.into_optimized().context("preparing the release probe model")?;
    let runnable = DefaultRuntime.prepare(model).context("preparing the CPU release probe")?;
    let mut state = runnable.spawn().context("spawning the CPU release probe")?;
    let input = (0..feature_size).map(|index| (index % 257) as i32).collect::<Vec<_>>();
    let input = Tensor::from_shape(&[1, feature_size], &input)?.into_tvalue();
    let mut outputs: TVec<TValue> = state.run(tvec!(input))?;
    ensure!(outputs.len() == 1, "release probe model must have one output");
    let output = outputs.remove(0).into_tensor();
    ensure!(output.rank() == 2 && output.shape()[0] == 1, "invalid release probe output shape");
    let output = output.to_plain_array_view::<f32>()?;
    let mut bytes = Vec::with_capacity(output.len() * std::mem::size_of::<f32>());
    for score in output.iter() {
        bytes.extend_from_slice(&score.to_le_bytes());
    }
    std::fs::write(destination, bytes)
        .with_context(|| format!("writing release probe scores {}", destination.display()))
}

fn prepare_nnef(
    mut model: InferenceModel, batch_norm_epsilons: &HashMap<String, f32>,
) -> Result<TypedModel> {
    set_conversion_batch(&mut model)?;
    rewrite_one_hot_embeddings(&mut model).context("replacing one-hot embeddings with gathers")?;
    rewrite_dynamic_batch_norms(&mut model, batch_norm_epsilons)
        .context("lowering dynamic batch normalization")?;
    let mut model = model.into_typed().context("resolving the ONNX model")?;
    fuse_gelu_approximations(&mut model).context("fusing canonical GELU approximations")?;
    let model = model.into_decluttered().context("optimizing the portable model")?;
    symbolize_batch(model).context("restoring a symbolic batch dimension")
}

fn set_conversion_batch(model: &mut InferenceModel) -> TractResult<()> {
    for input in 0..model.input_outlets()?.len() {
        let mut fact = model.input_fact(input)?.clone();
        // A non-singleton sentinel prevents decluttering from folding the batch axis into
        // reductions. The typed graph is made symbolic again before it is serialized.
        fact.shape.set_dim(0, 2.to_dim());
        model.set_input_fact(input, fact)?;
    }
    model.analyse(false)?;
    Ok(())
}

fn write_nnef(model: &TypedModel, destination: &Path) -> Result<()> {
    let file = File::create(destination)
        .with_context(|| format!("creating NNEF archive {}", destination.display()))?;
    let gzip = GzBuilder::new().mtime(0).write(file, Compression::best());
    let gzip = tract_nnef::nnef()
        .write_to_tar_with_config(model, gzip, false, true)
        .context("serializing deterministic optimized NNEF")?;
    gzip.finish().context("finishing compressed NNEF archive")?;
    Ok(())
}

fn verify_rust_round_trip(destination: &Path) -> Result<()> {
    let round_trip = tract_nnef::nnef()
        .model_for_path(destination)
        .with_context(|| format!("reloading NNEF archive {}", destination.display()))?;
    let batch = round_trip.symbols.get("N").context("converted NNEF has no batch symbol N")?;
    let symbols = HashMap::from([(batch, 1.to_dim())]);
    round_trip
        .set_symbols(&symbols)
        .context("binding batch one in converted NNEF")?
        .into_optimized()
        .context("preparing converted NNEF for the Rust CPU runtime")?;
    Ok(())
}

fn batch_norm_epsilons(proto: &tract_onnx::pb::ModelProto) -> TractResult<HashMap<String, f32>> {
    let graph = proto.graph.as_ref().context("ONNX model has no graph")?;
    let mut epsilons = HashMap::new();
    for node in graph.node.iter().filter(|node| node.op_type == "BatchNormalization") {
        let name = if !node.name.is_empty() {
            node.name.clone()
        } else {
            node.output.first().cloned().context("unnamed BatchNormalization has no output")?
        };
        let epsilon = node.get_attr_opt::<f32>("epsilon")?.unwrap_or(1.0e-5);
        ensure!(
            epsilons.insert(name.clone(), epsilon).is_none(),
            "duplicate BatchNormalization node name {name}"
        );
    }
    Ok(epsilons)
}

fn rewrite_dynamic_batch_norms(
    model: &mut InferenceModel, batch_norm_epsilons: &HashMap<String, f32>,
) -> TractResult<()> {
    while let Some(node) = model.nodes().iter().find(|node| {
        node.op.name().as_ref() == "BatchNorm"
            && node.inputs[1..]
                .iter()
                .any(|input| model.outlet_fact(*input).ok().and_then(Factoid::concretize).is_none())
    }) {
        ensure!(node.inputs.len() == 5, "BatchNorm must have five inputs");
        let node_id = node.id;
        let inputs = node.inputs.clone();
        let node_name = node.name.clone();
        let input_rank = model
            .outlet_fact(inputs[0])?
            .shape
            .rank()
            .concretize()
            .with_context(|| format!("dynamic BatchNorm {node_name} input rank is unknown"))?;
        ensure!(input_rank >= 2, "dynamic BatchNorm {node_name} input rank must be at least 2");
        let input_rank = usize::try_from(input_rank)?;
        for input in &inputs[1..] {
            ensure!(
                model.outlet_fact(*input)?.shape.rank().concretize() == Some(1),
                "dynamic BatchNorm {node_name} parameters must be rank 1"
            );
        }
        let epsilon = *batch_norm_epsilons
            .get(&node_name)
            .with_context(|| format!("ONNX epsilon is missing for BatchNorm {node_name}"))?;

        let mut patch = InferenceModelPatch::new("lower dynamic BatchNorm to portable arithmetic");
        let inputs = patch.taps(model, &inputs)?;
        let scale =
            broadcast_batch_norm_param(&mut patch, &node_name, "scale", inputs[1], input_rank)?;
        let beta =
            broadcast_batch_norm_param(&mut patch, &node_name, "beta", inputs[2], input_rank)?;
        let mean =
            broadcast_batch_norm_param(&mut patch, &node_name, "mean", inputs[3], input_rank)?;
        let variance =
            broadcast_batch_norm_param(&mut patch, &node_name, "variance", inputs[4], input_rank)?;
        let epsilon = patch.add_const(format!("{node_name}.epsilon"), tensor0(epsilon))?;
        let centered = patch.wire_node(
            format!("{node_name}.center"),
            tract_core::ops::math::Sub.into_hir(),
            &[inputs[0], mean],
        )?;
        let variance_epsilon = patch.wire_node(
            format!("{node_name}.variance_epsilon"),
            tract_core::ops::math::Add.into_hir(),
            &[variance, epsilon],
        )?;
        let inverse_stddev = patch.wire_node(
            format!("{node_name}.inverse_stddev"),
            tract_core::ops::math::rsqrt().into_hir(),
            &variance_epsilon,
        )?;
        let normalized = patch.wire_node(
            format!("{node_name}.normalize"),
            tract_core::ops::math::Mul.into_hir(),
            &[centered[0], inverse_stddev[0]],
        )?;
        let scaled = patch.wire_node(
            format!("{node_name}.scale"),
            tract_core::ops::math::Mul.into_hir(),
            &[normalized[0], scale],
        )?;
        let shifted = patch.wire_node(
            format!("{node_name}.shift"),
            tract_core::ops::math::Add.into_hir(),
            &[scaled[0], beta],
        )?;
        patch.shunt_outside(model, node_id.into(), shifted[0])?;
        patch.apply(model)?;
        model.compact()?;
    }
    Ok(())
}

fn broadcast_batch_norm_param(
    patch: &mut InferenceModelPatch, node_name: &str, param_name: &str, input: OutletId,
    input_rank: usize,
) -> TractResult<OutletId> {
    let axes =
        std::iter::once(0).chain(2..input_rank).map(|axis| axis as isize).collect::<Vec<_>>();
    Ok(patch.wire_node(
        format!("{node_name}.{param_name}.broadcast"),
        expand(tract_hir::ops::array::AddDims::new(axes)),
        &[input],
    )?[0])
}

fn rewrite_one_hot_embeddings(model: &mut InferenceModel) -> TractResult<()> {
    loop {
        let Some((node_id, source, weights)) = find_one_hot_embedding(model)? else {
            break;
        };
        let mut patch = InferenceModelPatch::new("replace one-hot embedding with gather");
        let source = patch.tap_model(model, source)?;
        let weights = patch.tap_model(model, weights)?;
        let gathered =
            patch.wire_node("embedding.gather", expand(Gather::new(0)), &[weights, source])?;
        patch.shunt_outside(model, node_id.into(), gathered[0])?;
        patch.apply(model)?;
        model.compact()?;
    }
    Ok(())
}

fn find_one_hot_embedding(
    model: &InferenceModel,
) -> TractResult<Option<(usize, OutletId, OutletId)>> {
    for node in model.nodes() {
        if node.op.name().as_ref() != "MatMulInference" || node.inputs.len() != 2 {
            continue;
        }
        let encoded = node.inputs[0];
        let weights = node.inputs[1];
        let Some(weight_values) = model.outlet_fact(weights)?.concretize() else {
            continue;
        };
        if weight_values.rank() != 2 {
            continue;
        }
        if let Some(source) = onnx_one_hot_source(model, encoded, weight_values.shape()[0])?
            && model.input_outlets()?.contains(&source)
            && matches!(
                model.outlet_fact(source)?.datum_type.concretize(),
                Some(dt) if dt.is_integer()
            )
        {
            return Ok(Some((node.id, source, weights)));
        }
        let cast = model.node(encoded.node);
        if cast.op.name().as_ref() != "onnx.Cast" || cast.inputs.len() != 1 {
            continue;
        }
        let equal = model.node(cast.inputs[0].node);
        if equal.op.name().as_ref() != "Eq" || equal.inputs.len() != 2 {
            continue;
        }
        for constant_slot in 0..2 {
            let Some(classes) = model.outlet_fact(equal.inputs[constant_slot])?.concretize() else {
                continue;
            };
            let Ok(classes) = classes.try_as_plain() else {
                continue;
            };
            let Ok(classes) = classes.as_slice::<i32>() else {
                continue;
            };
            if classes.len() != weight_values.shape()[0]
                || !classes.iter().enumerate().all(|(index, value)| *value == index as i32)
            {
                continue;
            }
            let encoded_indices = equal.inputs[1 - constant_slot];
            for source in model.input_outlets()? {
                let fact = model.outlet_fact(*source)?;
                if matches!(fact.datum_type.concretize(), Some(dt) if dt.is_integer())
                    && is_one_hot_index_expansion(model, encoded_indices, *source)
                {
                    return Ok(Some((node.id, *source, weights)));
                }
            }
        }
    }
    Ok(None)
}

fn onnx_one_hot_source(
    model: &InferenceModel, encoded: OutletId, classes: usize,
) -> TractResult<Option<OutletId>> {
    let reshape = model.node(encoded.node);
    if reshape.op.name().as_ref() != "Reshape" || reshape.inputs.len() != 2 {
        return Ok(None);
    }
    let one_hot_outlet = reshape.inputs[0];
    let one_hot = model.node(one_hot_outlet.node);
    if one_hot.op.name().as_ref() != "OneHot" || one_hot.inputs.len() != 3 {
        return Ok(None);
    }
    let Some(depth) = model.outlet_fact(one_hot.inputs[1])?.concretize() else {
        return Ok(None);
    };
    let Some(values) = model.outlet_fact(one_hot.inputs[2])?.concretize() else {
        return Ok(None);
    };
    if depth.cast_to_scalar::<i64>()? != classes as i64
        || values.nth(0)?.cast_to_scalar::<f32>()? != 0.0
        || values.nth(1)?.cast_to_scalar::<f32>()? != 1.0
    {
        return Ok(None);
    }
    let source = one_hot.inputs[0];
    let source_shape = &model.outlet_fact(source)?.shape;
    let one_hot_shape = &model.outlet_fact(one_hot_outlet)?.shape;
    let encoded_shape = &model.outlet_fact(encoded)?.shape;
    let Some(source_rank) = source_shape.rank().concretize() else {
        return Ok(None);
    };
    let Some(one_hot_rank) = one_hot_shape.rank().concretize() else {
        return Ok(None);
    };
    let Some(encoded_rank) = encoded_shape.rank().concretize() else {
        return Ok(None);
    };
    let source_rank = source_rank as usize;
    if one_hot_rank != source_rank as i64 + 1
        || (0..source_rank).any(|axis| source_shape.dim(axis) != one_hot_shape.dim(axis))
        || concrete_dim(one_hot_shape, source_rank) != Some(classes)
        || encoded_rank != 2
        || concrete_dim(encoded_shape, 1) != Some(classes)
    {
        return Ok(None);
    }
    Ok(Some(source))
}

fn is_one_hot_index_expansion(
    model: &InferenceModel, encoded_indices: OutletId, source: OutletId,
) -> bool {
    if encoded_indices == source {
        return true;
    }
    if !adds_trailing_singleton(model, source, encoded_indices) {
        return false;
    }
    let node = model.node(encoded_indices.node);
    match node.op.name().as_ref() {
        "AddDims" | "AddAxis" => node.inputs.as_slice() == [source],
        "MultiBroadcastTo" => {
            let Some(data) = node.inputs.first() else {
                return false;
            };
            let reshape = model.node(data.node);
            reshape.op.name().as_ref() == "Reshape"
                && reshape.inputs.first() == Some(&source)
                && same_concrete_shape(model, *data, encoded_indices)
        }
        _ => false,
    }
}

fn adds_trailing_singleton(model: &InferenceModel, source: OutletId, expanded: OutletId) -> bool {
    let Ok(source_fact) = model.outlet_fact(source) else {
        return false;
    };
    let Ok(expanded_fact) = model.outlet_fact(expanded) else {
        return false;
    };
    let Some(source_rank) = source_fact.shape.rank().concretize() else {
        return false;
    };
    let Some(expanded_rank) = expanded_fact.shape.rank().concretize() else {
        return false;
    };
    let source_rank = source_rank as usize;
    expanded.slot == 0
        && expanded_rank == source_rank as i64 + 1
        && (0..source_rank).all(|axis| source_fact.shape.dim(axis) == expanded_fact.shape.dim(axis))
        && concrete_dim(&expanded_fact.shape, source_rank) == Some(1)
}

fn same_concrete_shape(model: &InferenceModel, left: OutletId, right: OutletId) -> bool {
    let Ok(left) = model.outlet_fact(left) else {
        return false;
    };
    let Ok(right) = model.outlet_fact(right) else {
        return false;
    };
    left.shape == right.shape
}

fn concrete_dim(shape: &tract_hir::infer::ShapeFactoid, axis: usize) -> Option<usize> {
    shape.dim(axis)?.concretize()?.to_usize().ok()
}

fn symbolize_batch(model: TypedModel) -> TractResult<TypedModel> {
    let input_outlets = model.input_outlets()?.to_vec();
    let input_nodes = input_outlets.iter().map(|outlet| outlet.node).collect::<HashSet<_>>();
    let mut target = TypedModel::default();
    let batch = target.symbols.sym("N").to_dim();
    let mut mapping = HashMap::<OutletId, OutletId>::new();

    for input in &input_outlets {
        let source = model.node(input.node);
        ensure!(source.outputs.len() == 1, "model inputs must have one output");
        let mut fact = model.outlet_fact(*input)?.clone();
        ensure!(fact.rank() > 0, "model inputs must have a batch dimension");
        ensure!(fact.shape[0] == 2.to_dim(), "temporary conversion batch must be two");
        fact.shape.set(0, batch.clone());
        let output = target.add_source(source.name.clone(), fact)?;
        mapping.insert(*input, output);
    }

    for node_id in model.eval_order()? {
        if input_nodes.contains(&node_id) {
            continue;
        }
        let node = model.node(node_id);
        let inputs = node
            .inputs
            .iter()
            .map(|input| {
                mapping.get(input).copied().with_context(|| {
                    format!("missing translated input {input:?} for {}", node.name)
                })
            })
            .collect::<TractResult<TVec<_>>>()?;
        let outputs = target.wire_node(node.name.clone(), node.op.clone(), &inputs)?;
        ensure!(outputs.len() == node.outputs.len());
        for (slot, output) in outputs.into_iter().enumerate() {
            mapping.insert(OutletId::new(node_id, slot), output);
        }
    }

    let outputs = model
        .output_outlets()?
        .iter()
        .map(|output| {
            mapping
                .get(output)
                .copied()
                .with_context(|| format!("missing translated model output {output:?}"))
        })
        .collect::<TractResult<TVec<_>>>()?;
    target.select_output_outlets(&outputs)?;
    Ok(target)
}

fn fuse_gelu_approximations(model: &mut TypedModel) -> TractResult<()> {
    loop {
        let mut patch = None;
        for node in model.nodes() {
            if let Some(pow) = node.op_as::<TypedBinOp>().and_then(|op| op.0.downcast_ref::<Pow>())
                && let Some(candidate) =
                    tract_core::ops::nn::gelu_approximate::detect_gelu_approx(pow, model, node)?
            {
                patch = Some(candidate);
                break;
            }
            if let Some(input) = match_expanded_gelu(model, node) {
                let mut candidate = TypedModelPatch::default();
                let input = candidate.tap_model(model, input)?;
                let output = candidate.wire_node(
                    format!("{}.gelu_approx", node.name),
                    tract_core::ops::nn::gelu_approximate::gelu_approximate(false),
                    &[input],
                )?;
                candidate.shunt_outside(model, node.id.into(), output[0])?;
                patch = Some(candidate);
                break;
            }
        }
        let Some(patch) = patch else { break };
        patch.apply(model)?;
        model.compact()?;
    }
    Ok(())
}

fn match_expanded_gelu(model: &TypedModel, node: &TypedNode) -> Option<OutletId> {
    if !is_mul(node) || node.inputs.len() != 2 {
        return None;
    }
    for input_slot in 0..2 {
        let input = node.inputs[input_slot];
        let scaled = node.inputs[1 - input_slot];
        if matches_expanded_gelu_orientation(model, input, scaled) {
            return Some(input);
        }
    }
    None
}

fn matches_expanded_gelu_orientation(
    model: &TypedModel, input: OutletId, scaled: OutletId,
) -> bool {
    let Some(one_plus_tanh) = bin_with_const_input(model, scaled, 0.5, is_mul) else {
        return false;
    };
    let Some(tanh) = bin_with_const_input(model, one_plus_tanh, 1.0, is_add) else {
        return false;
    };
    let tanh = model.node(tanh.node);
    if !is_tanh(tanh) || tanh.inputs.len() != 1 {
        return false;
    }
    let Some(sum) = bin_with_const_input(
        model,
        tanh.inputs[0],
        (2.0_f32 / std::f32::consts::PI).sqrt(),
        is_mul,
    ) else {
        return false;
    };
    let sum = model.node(sum.node);
    if !is_add(sum) || sum.inputs.len() != 2 || !sum.inputs.contains(&input) {
        return false;
    }
    let Some(cubic) = sum.inputs.iter().copied().find(|candidate| *candidate != input) else {
        return false;
    };
    let Some((powers, coefficient)) = collect_mul_factors(model, cubic, input, 0) else {
        return false;
    };
    powers == 3 && close(coefficient, 0.044715)
}

fn collect_mul_factors(
    model: &TypedModel, outlet: OutletId, input: OutletId, depth: usize,
) -> Option<(usize, f32)> {
    if outlet == input {
        return Some((1, 1.0));
    }
    if let Some(value) = uniform_f32(model, outlet) {
        return Some((0, value));
    }
    if depth >= 4 {
        return None;
    }
    let node = model.node(outlet.node);
    if !is_mul(node) || node.inputs.len() != 2 {
        return None;
    }
    let left = collect_mul_factors(model, node.inputs[0], input, depth + 1)?;
    let right = collect_mul_factors(model, node.inputs[1], input, depth + 1)?;
    Some((left.0 + right.0, left.1 * right.1))
}

fn bin_with_const_input(
    model: &TypedModel, outlet: OutletId, value: f32, predicate: fn(&TypedNode) -> bool,
) -> Option<OutletId> {
    let node = model.node(outlet.node);
    if !predicate(node) || node.inputs.len() != 2 {
        return None;
    }
    for slot in 0..2 {
        if uniform_f32(model, node.inputs[slot]).is_some_and(|candidate| close(candidate, value)) {
            return Some(node.inputs[1 - slot]);
        }
    }
    None
}

fn uniform_f32(model: &TypedModel, outlet: OutletId) -> Option<f32> {
    model.outlet_fact(outlet).ok()?.uniform.as_ref()?.cast_to_scalar::<f32>().ok()
}

fn close(left: f32, right: f32) -> bool {
    (left - right).abs() <= 1e-6
}

fn is_add(node: &TypedNode) -> bool {
    node.op_as::<TypedBinOp>().is_some_and(|op| op.0.is::<Add>())
}

fn is_mul(node: &TypedNode) -> bool {
    node.op_as::<TypedBinOp>().is_some_and(|op| op.0.is::<Mul>())
}

fn is_tanh(node: &TypedNode) -> bool {
    node.op_as::<ElementWiseOp>().is_some_and(|op| op.0.is::<Tanh>())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bundled_model(name: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join("assets/models")
            .join(name)
            .join("model.onnx")
    }

    fn load_bundled(name: &str) -> (InferenceModel, HashMap<String, f32>) {
        let path = bundled_model(name);
        let onnx = tract_onnx::onnx();
        let proto = onnx.proto_model_for_path(&path).unwrap();
        let epsilons = batch_norm_epsilons(&proto).unwrap();
        (onnx.model_for_path(path).unwrap(), epsilons)
    }

    #[test]
    fn v3_semantic_rewrites_survive_nnef_preparation() {
        let (model, epsilons) = load_bundled("standard_v3_3");
        let model = prepare_nnef(model, &epsilons).unwrap();

        let names = model.nodes().iter().map(|node| node.op.name().to_string()).collect::<Vec<_>>();
        assert_eq!(
            model.nodes().iter().filter(|node| node.op.name() == "Gather").count(),
            1,
            "optimized ops: {names:?}"
        );
        assert_eq!(
            model.nodes().iter().filter(|node| node.op.name() == "GeluApproximate").count(),
            2
        );

        let batch = model.symbols.get("N").unwrap();
        for size in [1, 8, 16, 32, 64] {
            let symbols = HashMap::from([(batch.clone(), size.to_dim())]);
            model.clone().set_symbols(&symbols).unwrap().into_optimized().unwrap();
        }
    }

    #[test]
    fn fast_v2_dynamic_batch_norm_lowers_to_portable_math() {
        let (mut model, epsilons) = load_bundled("fast_v2_1");
        assert!(model.nodes().iter().any(|node| node.op.name().as_ref() == "BatchNorm"));
        set_conversion_batch(&mut model).unwrap();
        let batch_norm = model.nodes().iter().find(|node| node.op.name() == "BatchNorm").unwrap();
        assert_eq!(
            model.outlet_fact(batch_norm.inputs[0]).unwrap().shape.rank().concretize(),
            Some(4)
        );

        rewrite_dynamic_batch_norms(&mut model, &epsilons).unwrap();

        assert!(!model.nodes().iter().any(|node| node.op.name().as_ref() == "BatchNorm"));
        let model = prepare_nnef(model, &epsilons).unwrap();
        assert_eq!(model.nodes().iter().filter(|node| node.op.name() == "Gather").count(), 1);
    }

    #[test]
    fn batch_norm_epsilon_comes_from_onnx() {
        let (_, epsilons) = load_bundled("fast_v2_1");
        assert!(!epsilons.is_empty());
        assert!(epsilons.values().all(|epsilon| *epsilon == 0.001));
    }
}
