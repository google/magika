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

//! Constant folding of a Magika embedding lookup's bias and activation.

use tract_core::internal::*;
use tract_core::ops::array::Gather;
use tract_core::ops::binary::TypedBinOp;
use tract_core::ops::element_wise::ElementWiseOp;
use tract_core::ops::math::Add;
use tract_core::ops::nn::GeluApproximate;

/// Fold `gelu(gather(table, tokens) + bias)` into `gather(gelu(table + bias), tokens)`.
///
/// The bias and the activation apply to the embedding width alone and do not depend on which row
/// was selected, so they can be evaluated once over the table's few hundred rows instead of once
/// per token of every file. On Magika's model that replaces two passes over a
/// `[batch, 2048, 64]` activation with a `[257, 64]` constant computed at model preparation.
///
/// The folded table is the identical expression applied to the identical values, so scores are
/// unchanged to the bit rather than approximated.
///
/// Returns the number of independent lookups folded.
pub(crate) fn fuse_magika_embedding(model: &mut TypedModel) -> TractResult<usize> {
    let mut fused = 0;
    while let Some(pattern) = FoldPattern::find(model)? {
        let table = fold_table(&pattern)?;
        let mut patch = TypedModelPatch::default();
        let indices = patch.tap_model(model, pattern.indices)?;
        let table = patch.add_const("magika.embedding.table", table)?;
        let gather = Gather { axis: 0, output_type: pattern.output_type };
        let output = patch.wire_node("magika.embedding.gather", gather, &[table, indices])?[0];
        patch.shunt_outside(model, OutletId::new(pattern.activation_node, 0), output)?;
        patch.apply(model)?;
        model.compact()?;
        fused += 1;
    }
    Ok(fused)
}

struct FoldPattern {
    activation_node: usize,
    indices: OutletId,
    output_type: Option<DatumType>,
    table: Arc<Tensor>,
    bias: Arc<Tensor>,
    fast_impl: bool,
}

impl FoldPattern {
    fn find(model: &TypedModel) -> TractResult<Option<Self>> {
        for activation in &model.nodes {
            let Some(element_wise) = activation.op.downcast_ref::<ElementWiseOp>() else {
                continue;
            };
            let Some(gelu) = element_wise.0.downcast_ref::<GeluApproximate>() else { continue };
            let [sum_input] = activation.inputs.as_slice() else { continue };
            if sum_input.slot != 0 {
                continue;
            }
            let sum = model.node(sum_input.node);
            let Some(binary) = sum.op.downcast_ref::<TypedBinOp>() else { continue };
            if !binary.0.is::<Add>() {
                continue;
            }
            let [left, right] = sum.inputs.as_slice() else { continue };
            // The bias may be written on either side of the sum.
            let Some((gather_outlet, bias_outlet)) = [(left, right), (right, left)]
                .into_iter()
                .find(|(gather, _)| model.node(gather.node).op.downcast_ref::<Gather>().is_some())
            else {
                continue;
            };
            let gather_node = model.node(gather_outlet.node);
            let Some(gather) = gather_node.op.downcast_ref::<Gather>() else { continue };
            if gather.axis != 0 || gather_outlet.slot != 0 {
                continue;
            }
            // Anything else reading the intermediate activations would still need them computed,
            // so folding would add the table without removing the passes it is meant to replace.
            if model.outlet_successors(*gather_outlet).len() != 1
                || model.outlet_successors(OutletId::new(sum.id, 0)).len() != 1
            {
                continue;
            }
            let [table_input, indices] = gather_node.inputs.as_slice() else { continue };
            let Some(table) = model.outlet_fact(*table_input)?.konst.clone() else { continue };
            let Some(bias) = model.outlet_fact(*bias_outlet)?.konst.clone() else { continue };
            let [_, width] = *table.shape() else { continue };
            // The bias has to address the embedding width and nothing else, or it is not a
            // property of the row that was selected.
            if table.datum_type() != DatumType::F32
                || bias.datum_type() != DatumType::F32
                || bias.len() != width
                || bias.shape().last() != Some(&width)
            {
                continue;
            }
            return Ok(Some(Self {
                activation_node: activation.id,
                indices: *indices,
                output_type: gather.output_type,
                table,
                bias,
                fast_impl: gelu.fast_impl,
            }));
        }
        Ok(None)
    }
}

/// The operator the graph itself uses for this activation.
fn gelu(fast_impl: bool) -> ElementWiseOp {
    ElementWiseOp(Box::new(GeluApproximate { fast_impl }), None)
}

fn fold_table(pattern: &FoldPattern) -> TractResult<Tensor> {
    let [rows, width] = *pattern.table.shape() else { bail!("embedding table is not a matrix") };
    let bias = pattern.bias.as_ref().try_as_plain()?;
    let bias = bias.as_slice::<f32>()?;
    let table = pattern.table.as_ref().try_as_plain()?;
    let table = table.as_slice::<f32>()?;
    let mut folded = Vec::with_capacity(rows * width);
    for row in table.chunks_exact(width) {
        folded.extend(row.iter().zip(bias).map(|(value, bias)| value + bias));
    }
    let folded = Tensor::from_shape(&[rows, width], &folded)?;
    // Evaluate the activation through the same operator the graph would have run, so the folded
    // table holds the values that operator produces rather than a second implementation of it.
    let mut activated = gelu(pattern.fast_impl).eval(tvec!(folded.into_tvalue()))?;
    Ok(activated.remove(0).into_tensor())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Builds `gelu(gather(table, tokens) + bias)` and checks the fold reproduces it exactly.
    fn folded_matches_reference(rows: usize, width: usize, tokens: usize) -> TractResult<()> {
        let table = (0..rows * width)
            .map(|index| ((index * 31 % 97) as f32 - 48.0) * 0.03)
            .collect::<Vec<_>>();
        let bias = (0..width).map(|index| (index as f32 % 7.0 - 3.0) * 0.05).collect::<Vec<_>>();
        let indices = (0..tokens).map(|index| (index * 17 % rows) as i64).collect::<Vec<_>>();

        let mut model = TypedModel::default();
        let source = model.add_source("tokens", i64::fact([tokens]))?;
        let table_node = model.add_const("table", Tensor::from_shape(&[rows, width], &table)?)?;
        let gather = model.wire_node(
            "gather",
            Gather { axis: 0, output_type: None },
            &[table_node, source],
        )?[0];
        let bias_node = model.add_const("bias", Tensor::from_shape(&[1, width], &bias)?)?;
        let sum = model.wire_node("sum", tract_core::ops::math::add(), &[gather, bias_node])?[0];
        let activation = model.wire_node("gelu", gelu(false), &[sum])?;
        model.select_output_outlets(&activation)?;
        let model = model.into_decluttered()?;

        let input = Tensor::from_shape(&[tokens], &indices)?;
        let reference =
            TypedSimplePlan::new(model.clone())?.run(tvec!(input.clone().into_tvalue()))?;

        let mut folded = model.clone();
        assert_eq!(fuse_magika_embedding(&mut folded)?, 1);
        assert!(
            !folded.nodes.iter().any(|node| node.op.is::<ElementWiseOp>()),
            "the activation should no longer run on the gathered rows"
        );
        let actual = TypedSimplePlan::new(folded.clone())?.run(tvec!(input.into_tvalue()))?;

        let reference = reference[0].to_plain_array_view::<f32>()?;
        let actual = actual[0].to_plain_array_view::<f32>()?;
        ensure!(reference == actual, "folding changed the scores");
        Ok(())
    }

    #[test]
    fn folding_reproduces_the_original_activations() -> TractResult<()> {
        folded_matches_reference(257, 64, 2048)
    }

    #[test]
    fn unmatched_model_reports_zero_folds() {
        let mut model = TypedModel::default();
        assert_eq!(fuse_magika_embedding(&mut model).unwrap(), 0);
    }
}
