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
use tract_core::ops::binary::TypedBinOp;
use tract_core::ops::element_wise::ElementWiseOp;
use tract_core::ops::math::{Add, Mul, Pow, Tanh};
use tract_hir::infer::{Factoid, InferenceModelPatch};
use tract_hir::ops::array::Gather;
use tract_hir::ops::binary::BinIntoHir;
use tract_hir::ops::element_wise::ElementWiseIntoHir;
use tract_hir::ops::expandable::expand;
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

    let model = tract_onnx::onnx()
        .model_for_path(&source)
        .with_context(|| format!("loading ONNX model {}", source.display()))?;
    let model = prepare_nnef(model)
        .with_context(|| format!("optimizing ONNX model {}", source.display()))?;

    write_nnef(&model, &destination)?;
    verify_rust_round_trip(&destination)?;

    Ok(())
}

fn prepare_nnef(mut model: InferenceModel) -> Result<TypedModel> {
    rewrite_one_hot_embeddings(&mut model).context("replacing one-hot embeddings with gathers")?;
    rewrite_dynamic_batch_norms(&mut model).context("lowering dynamic batch normalization")?;
    for input in 0..model.input_outlets()?.len() {
        let mut fact = model.input_fact(input)?.clone();
        fact.shape.set_dim(0, 1.to_dim());
        model.set_input_fact(input, fact)?;
    }
    let mut model = model.into_typed().context("resolving the ONNX model")?;
    fuse_gelu_approximations(&mut model).context("fusing canonical GELU approximations")?;
    let model = model.into_decluttered().context("optimizing the portable model")?;
    symbolize_batch(model).context("restoring a symbolic batch dimension")
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

fn rewrite_dynamic_batch_norms(model: &mut InferenceModel) -> TractResult<()> {
    while let Some(node) = model.nodes().iter().find(|node| {
        node.op.name().as_ref() == "BatchNorm"
            && node.inputs[1..]
                .iter()
                .any(|input| model.outlet_fact(*input).ok().and_then(Factoid::concretize).is_none())
    }) {
        ensure!(node.inputs.len() == 5, "BatchNorm must have five inputs");
        let epsilon = batch_norm_epsilon(&format!("{:?}", node.op))?;
        let node_id = node.id;
        let inputs = node.inputs.clone();
        let node_name = node.name.clone();

        let mut patch = InferenceModelPatch::new("lower dynamic BatchNorm to portable arithmetic");
        let inputs = patch.taps(model, &inputs)?;
        let epsilon = patch.add_const(format!("{node_name}.epsilon"), tensor0(epsilon))?;
        let centered = patch.wire_node(
            format!("{node_name}.center"),
            tract_core::ops::math::Sub.into_hir(),
            &[inputs[0], inputs[3]],
        )?;
        let variance = patch.wire_node(
            format!("{node_name}.variance_epsilon"),
            tract_core::ops::math::Add.into_hir(),
            &[inputs[4], epsilon],
        )?;
        let inverse_stddev = patch.wire_node(
            format!("{node_name}.inverse_stddev"),
            tract_core::ops::math::rsqrt().into_hir(),
            &variance,
        )?;
        let normalized = patch.wire_node(
            format!("{node_name}.normalize"),
            tract_core::ops::math::Mul.into_hir(),
            &[centered[0], inverse_stddev[0]],
        )?;
        let scaled = patch.wire_node(
            format!("{node_name}.scale"),
            tract_core::ops::math::Mul.into_hir(),
            &[normalized[0], inputs[1]],
        )?;
        let shifted = patch.wire_node(
            format!("{node_name}.shift"),
            tract_core::ops::math::Add.into_hir(),
            &[scaled[0], inputs[2]],
        )?;
        patch.shunt_outside(model, node_id.into(), shifted[0])?;
        patch.apply(model)?;
        model.compact()?;
    }
    Ok(())
}

fn batch_norm_epsilon(debug: &str) -> TractResult<f32> {
    // tract-onnx keeps BatchNorm's fields private. Its exact version is pinned, so parse the
    // operator's stable Debug representation rather than hard-coding Magika's epsilon.
    let marker = "epsilon: ";
    let start = debug.find(marker).context("BatchNorm epsilon is missing")? + marker.len();
    let value = debug[start..].split([',', ' ']).next().context("BatchNorm epsilon is empty")?;
    value.parse().context("BatchNorm epsilon is not a float")
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
                if !matches!(fact.datum_type.concretize(), Some(dt) if dt.is_integer())
                    || !reaches_source(model, encoded_indices, *source, &mut HashSet::new())
                {
                    continue;
                }
                return Ok(Some((node.id, *source, weights)));
            }
        }
    }
    Ok(None)
}

fn reaches_source(
    model: &InferenceModel, outlet: OutletId, source: OutletId, seen: &mut HashSet<usize>,
) -> bool {
    if outlet == source {
        return true;
    }
    if !seen.insert(outlet.node) {
        return false;
    }
    model.node(outlet.node).inputs.iter().any(|input| reaches_source(model, *input, source, seen))
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
        ensure!(fact.shape[0] == 1.to_dim(), "temporary conversion batch must be one");
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

    #[test]
    fn v3_semantic_rewrites_survive_nnef_preparation() {
        let model = tract_onnx::onnx().model_for_path(bundled_model("standard_v3_3")).unwrap();
        let model = prepare_nnef(model).unwrap();

        assert_eq!(model.nodes().iter().filter(|node| node.op.name() == "Gather").count(), 1);
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
        let mut model = tract_onnx::onnx().model_for_path(bundled_model("fast_v2_1")).unwrap();
        assert!(model.nodes().iter().any(|node| node.op.name().as_ref() == "BatchNorm"));

        rewrite_dynamic_batch_norms(&mut model).unwrap();

        assert!(!model.nodes().iter().any(|node| node.op.name().as_ref() == "BatchNorm"));
        prepare_nnef(model).unwrap();
    }

    #[test]
    fn batch_norm_epsilon_comes_from_the_pinned_operator() {
        let debug = "BatchNorm { data_format: NCHW, epsilon: 0.001, spatial: true }";
        assert_eq!(batch_norm_epsilon(debug).unwrap(), 0.001);
    }
}
