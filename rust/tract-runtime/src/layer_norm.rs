// Copyright 2026 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//      https://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Safe fused true-LayerNorm operator for the exact Magika graph.

use std::sync::Arc;

use tract_core::internal::*;
use tract_core::ops::binary::TypedBinOp;
use tract_core::ops::change_axes::AxisOp;
use tract_core::ops::element_wise::ElementWiseOp;
use tract_core::ops::math::{Add, Max, Mul, Rsqrt, Square, Sub};
use tract_core::ops::nn::{Reduce, Reducer, RmsNorm};

/// Replace supported mean/variance/normalize/affine chains.
pub fn fuse_magika_layer_norm(model: &mut TypedModel) -> TractResult<usize> {
    let mut fused = 0;
    while let Some(pattern) = FusionPattern::find(model)? {
        let op = FusedLayerNorm::new(
            pattern.axis,
            pattern.epsilon,
            pattern.scale,
            pattern.bias,
            &pattern.input_shape,
        )?;
        let mut patch = TypedModelPatch::default();
        let input = patch.tap_model(model, pattern.input)?;
        let output = patch.wire_node("magika.fused_layer_norm", op, &[input])?[0];
        patch.shunt_outside(model, OutletId::new(pattern.output_node, 0), output)?;
        patch.apply(model)?;
        model.compact()?;
        fused += 1;
    }
    Ok(fused)
}

/// Rewrite true LayerNorm as mean-centering plus tract's GPU-fused RMSNorm.
pub fn fuse_magika_layer_norm_for_gpu(model: &mut TypedModel) -> TractResult<usize> {
    let mut fused = 0;
    while let Some(pattern) = FusionPattern::find(model)? {
        let epsilon = pattern.epsilon.as_ref().clone().into_shape(&[])?.into_arc_tensor();
        let mut patch = TypedModelPatch::default();
        let centered = patch.tap_model(model, pattern.centered)?;
        let normalized = patch.wire_node(
            "magika.gpu_rms_norm",
            RmsNorm { axis: pattern.axis, eps: epsilon },
            &[centered],
        )?[0];
        let scale = patch.add_const("magika.layer_norm_scale", pattern.scale)?;
        let scaled = patch.wire_node(
            "magika.layer_norm_scale_mul",
            TypedBinOp(Box::new(Mul), None),
            &[normalized, scale],
        )?[0];
        let bias = patch.add_const("magika.layer_norm_bias", pattern.bias)?;
        let output = patch.wire_node(
            "magika.layer_norm_bias_add",
            TypedBinOp(Box::new(Add), None),
            &[scaled, bias],
        )?[0];
        patch.shunt_outside(model, OutletId::new(pattern.output_node, 0), output)?;
        patch.apply(model)?;
        model.compact()?;
        fused += 1;
    }
    Ok(fused)
}

struct FusionPattern {
    input: OutletId,
    centered: OutletId,
    output_node: usize,
    axis: usize,
    epsilon: Arc<Tensor>,
    scale: Arc<Tensor>,
    bias: Arc<Tensor>,
    input_shape: TVec<usize>,
}

impl FusionPattern {
    fn find(model: &TypedModel) -> TractResult<Option<Self>> {
        for output in &model.nodes {
            if !is_add(output) {
                continue;
            }
            let Some((bias, normalized)) = const_and_dynamic(model, output)? else {
                continue;
            };
            let normalized = model.node(normalized.node);
            if !is_mul(normalized) || normalized.inputs.len() != 2 {
                continue;
            }
            for centered_slot in 0..2 {
                if let Some(pattern) =
                    Self::match_orientation(model, output, normalized, centered_slot, bias.clone())?
                {
                    return Ok(Some(pattern));
                }
            }
        }
        Ok(None)
    }

    fn match_orientation(
        model: &TypedModel, output: &TypedNode, normalized: &TypedNode, centered_slot: usize,
        bias: Arc<Tensor>,
    ) -> TractResult<Option<Self>> {
        let centered_outlet = normalized.inputs[centered_slot];
        let scaled_inverse_outlet = normalized.inputs[1 - centered_slot];
        let centered = model.node(centered_outlet.node);
        if !is_sub(centered) || centered.inputs.len() != 2 {
            return Ok(None);
        }
        let input = centered.inputs[0];
        let mean = centered.inputs[1];

        let scaled_inverse = model.node(scaled_inverse_outlet.node);
        if !is_mul(scaled_inverse) {
            return Ok(None);
        }
        let Some((scale, inverse_outlet)) = const_and_dynamic(model, scaled_inverse)? else {
            return Ok(None);
        };
        let inverse = model.node(inverse_outlet.node);
        if !is_rsqrt(inverse) || inverse.inputs.len() != 1 {
            return Ok(None);
        }
        let epsilon_add = model.node(inverse.inputs[0].node);
        if !is_add(epsilon_add) {
            return Ok(None);
        }
        let Some((epsilon, clamped_variance_outlet)) = const_and_dynamic(model, epsilon_add)?
        else {
            return Ok(None);
        };

        let statistics = if let Some(statistics) =
            Self::match_direct_statistics(model, input, mean, clamped_variance_outlet)?
        {
            statistics
        } else if let Some(statistics) =
            Self::match_squeezed_statistics(model, input, mean, clamped_variance_outlet)?
        {
            statistics
        } else {
            return Ok(None);
        };

        let axis = statistics.axis;
        let reciprocal = statistics.reciprocal;
        let input_fact = model.outlet_fact(input)?;
        let Some(input_shape) = input_fact.shape.as_concrete() else {
            return Ok(None);
        };
        if input_fact.datum_type != DatumType::F32
            || axis >= input_shape.len()
            || input_shape[axis] == 0
            || scalar_f32(&reciprocal)
                .is_none_or(|value| (value - 1.0 / input_shape[axis] as f32).abs() > 1.0e-7)
            || scalar_f32(&epsilon).is_none_or(|value| !value.is_finite() || value < 0.0)
            || !is_axis_parameter(&scale, input_shape, axis)
            || !is_axis_parameter(&bias, input_shape, axis)
            || model.outlet_fact(OutletId::new(output.id, 0))?.shape.as_concrete()
                != Some(input_shape)
        {
            return Ok(None);
        }

        Ok(Some(Self {
            input,
            centered: centered_outlet,
            output_node: output.id,
            axis,
            epsilon,
            scale,
            bias,
            input_shape: input_shape.into(),
        }))
    }

    fn match_direct_statistics(
        model: &TypedModel, input: OutletId, mean: OutletId, clamped_variance: OutletId,
    ) -> TractResult<Option<StatisticsPattern>> {
        let clamped_variance = model.node(clamped_variance.node);
        if !is_max(clamped_variance) {
            return Ok(None);
        }
        let Some((zero, variance_outlet)) = const_and_dynamic(model, clamped_variance)? else {
            return Ok(None);
        };
        if scalar_f32(&zero) != Some(0.0) {
            return Ok(None);
        }
        let variance = model.node(variance_outlet.node);
        if !is_sub(variance) || variance.inputs.len() != 2 {
            return Ok(None);
        }
        let mean_squares = model.node(variance.inputs[0].node);
        let mean_square = model.node(variance.inputs[1].node);
        let Some(mean_squares_op) = mean_squares.op_as::<Reduce>() else {
            return Ok(None);
        };
        if mean_squares_op.reducer != Reducer::MeanOfSquares
            || mean_squares_op.axes.len() != 1
            || mean_squares.inputs.as_slice() != [input]
            || !is_square(mean_square)
            || mean_square.inputs.as_slice() != [mean]
        {
            return Ok(None);
        }
        let axis = mean_squares_op.axes[0];

        let mean = model.node(mean.node);
        if !is_mul(mean) {
            return Ok(None);
        }
        let Some((reciprocal, sum_outlet)) = const_and_dynamic(model, mean)? else {
            return Ok(None);
        };
        let sum = model.node(sum_outlet.node);
        let Some(sum_op) = sum.op_as::<Reduce>() else {
            return Ok(None);
        };
        if sum_op.reducer != Reducer::Sum
            || sum_op.axes.as_slice() != [axis]
            || sum.inputs.as_slice() != [input]
        {
            return Ok(None);
        }

        Ok(Some(StatisticsPattern { axis, reciprocal }))
    }

    fn match_squeezed_statistics(
        model: &TypedModel, input: OutletId, broadcast_mean: OutletId,
        broadcast_clamped_variance: OutletId,
    ) -> TractResult<Option<StatisticsPattern>> {
        let Some((axis, mean)) = axis_op_input(model, broadcast_mean, AxisOpKind::Add) else {
            return Ok(None);
        };
        let Some((variance_axis, clamped_variance)) =
            axis_op_input(model, broadcast_clamped_variance, AxisOpKind::Add)
        else {
            return Ok(None);
        };
        if variance_axis != axis {
            return Ok(None);
        }

        let clamped_variance = model.node(clamped_variance.node);
        if !is_max(clamped_variance) {
            return Ok(None);
        }
        let Some((zero, variance)) = const_and_dynamic(model, clamped_variance)? else {
            return Ok(None);
        };
        if scalar_f32(&zero) != Some(0.0) {
            return Ok(None);
        }

        let variance = model.node(variance.node);
        if !is_sub(variance) || variance.inputs.len() != 2 {
            return Ok(None);
        }
        let mean_square = model.node(variance.inputs[1].node);
        if !is_square(mean_square) || mean_square.inputs.as_slice() != [mean] {
            return Ok(None);
        }

        let mean = model.node(mean.node);
        if !is_mul(mean) {
            return Ok(None);
        }
        let Some((reciprocal, squeezed_sum)) = const_and_dynamic(model, mean)? else {
            return Ok(None);
        };
        if !matches_squeezed_sum(model, squeezed_sum, input, axis, false) {
            return Ok(None);
        }

        let mean_squares = model.node(variance.inputs[0].node);
        if !is_mul(mean_squares) {
            return Ok(None);
        }
        let Some((squares_reciprocal, squeezed_sum_squares)) =
            const_and_dynamic(model, mean_squares)?
        else {
            return Ok(None);
        };
        if scalar_f32(&reciprocal) != scalar_f32(&squares_reciprocal)
            || !matches_squeezed_sum(model, squeezed_sum_squares, input, axis, true)
        {
            return Ok(None);
        }

        Ok(Some(StatisticsPattern { axis, reciprocal }))
    }
}

struct StatisticsPattern {
    axis: usize,
    reciprocal: Arc<Tensor>,
}

#[derive(Clone, Copy)]
enum AxisOpKind {
    Add,
    Remove,
}

fn axis_op_input(
    model: &TypedModel, outlet: OutletId, kind: AxisOpKind,
) -> Option<(usize, OutletId)> {
    let node = model.node(outlet.node);
    if node.inputs.len() != 1 {
        return None;
    }
    let axis = match (kind, node.op_as::<AxisOp>()) {
        (AxisOpKind::Add, Some(AxisOp::Add(axis)))
        | (AxisOpKind::Remove, Some(AxisOp::Rm(axis))) => *axis,
        _ => return None,
    };
    Some((axis, node.inputs[0]))
}

fn matches_squeezed_sum(
    model: &TypedModel, squeezed_sum: OutletId, input: OutletId, axis: usize, squares: bool,
) -> bool {
    let Some((removed_axis, sum)) = axis_op_input(model, squeezed_sum, AxisOpKind::Remove) else {
        return false;
    };
    if removed_axis != axis {
        return false;
    }
    let sum = model.node(sum.node);
    let Some(sum_op) = sum.op_as::<Reduce>() else {
        return false;
    };
    if sum_op.reducer != Reducer::Sum || sum_op.axes.as_slice() != [axis] {
        return false;
    }
    if !squares {
        return sum.inputs.as_slice() == [input];
    }
    let [square] = sum.inputs.as_slice() else {
        return false;
    };
    let square = model.node(square.node);
    is_square(square) && square.inputs.as_slice() == [input]
}

fn const_and_dynamic(
    model: &TypedModel, node: &TypedNode,
) -> TractResult<Option<(Arc<Tensor>, OutletId)>> {
    if node.inputs.len() != 2 {
        return Ok(None);
    }
    let left = model.outlet_fact(node.inputs[0])?.konst.clone();
    let right = model.outlet_fact(node.inputs[1])?.konst.clone();
    Ok(match (left, right) {
        (Some(value), None) => Some((value, node.inputs[1])),
        (None, Some(value)) => Some((value, node.inputs[0])),
        _ => None,
    })
}

fn scalar_f32(value: &Tensor) -> Option<f32> {
    (value.len() == 1).then(|| value.cast_to_scalar::<f32>().ok()).flatten()
}

fn is_axis_parameter(value: &Tensor, input_shape: &[usize], axis: usize) -> bool {
    value.datum_type() == DatumType::F32
        && value.rank() == input_shape.len()
        && value.len() == input_shape[axis]
        && value
            .shape()
            .iter()
            .enumerate()
            .all(|(index, dim)| *dim == if index == axis { input_shape[axis] } else { 1 })
}

fn is_add(node: &TypedNode) -> bool {
    node.op_as::<TypedBinOp>().is_some_and(|op| op.0.is::<Add>())
}

fn is_max(node: &TypedNode) -> bool {
    node.op_as::<TypedBinOp>().is_some_and(|op| op.0.is::<Max>())
}

fn is_mul(node: &TypedNode) -> bool {
    node.op_as::<TypedBinOp>().is_some_and(|op| op.0.is::<Mul>())
}

fn is_sub(node: &TypedNode) -> bool {
    node.op_as::<TypedBinOp>().is_some_and(|op| op.0.is::<Sub>())
}

fn is_rsqrt(node: &TypedNode) -> bool {
    node.op_as::<ElementWiseOp>().is_some_and(|op| op.0.is::<Rsqrt>())
}

fn is_square(node: &TypedNode) -> bool {
    node.op_as::<ElementWiseOp>().is_some_and(|op| op.0.is::<Square>())
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
struct FusedLayerNorm {
    axis: usize,
    epsilon: Arc<Tensor>,
    scale: Arc<Tensor>,
    bias: Arc<Tensor>,
    input_shape: TVec<usize>,
}

impl FusedLayerNorm {
    fn new(
        axis: usize, epsilon: Arc<Tensor>, scale: Arc<Tensor>, bias: Arc<Tensor>,
        input_shape: &[usize],
    ) -> TractResult<Self> {
        ensure!(axis < input_shape.len());
        ensure!(input_shape[axis] > 0);
        ensure!(
            scalar_f32(&epsilon).is_some_and(|value| value.is_finite() && value >= 0.0),
            "LayerNorm epsilon must be a finite non-negative f32 scalar"
        );
        ensure!(is_axis_parameter(&scale, input_shape, axis));
        ensure!(is_axis_parameter(&bias, input_shape, axis));
        Ok(Self { axis, epsilon, scale, bias, input_shape: input_shape.into() })
    }
}

impl Op for FusedLayerNorm {
    fn name(&self) -> StaticName {
        "FusedLayerNorm".into()
    }

    fn info(&self) -> TractResult<Vec<String>> {
        Ok(vec![format!("axis={} shape={:?}", self.axis, self.input_shape)])
    }

    op_as_typed_op!();
}

impl EvalOp for FusedLayerNorm {
    fn is_stateless(&self) -> bool {
        true
    }

    fn eval(&self, inputs: TVec<TValue>) -> TractResult<TVec<TValue>> {
        let input = args_1!(inputs);
        ensure!(input.datum_type() == DatumType::F32);
        ensure!(input.shape() == self.input_shape.as_slice());
        let axis_len = self.input_shape[self.axis];
        let outer = self.input_shape[..self.axis].iter().product::<usize>();
        let inner = self.input_shape[self.axis + 1..].iter().product::<usize>();
        let epsilon = self.epsilon.cast_to_scalar::<f32>()?;
        let scale_plain = self.scale.as_ref().try_as_plain()?;
        let scale = scale_plain.as_slice::<f32>()?;
        let bias_plain = self.bias.as_ref().try_as_plain()?;
        let bias = bias_plain.as_slice::<f32>()?;
        let mut output = input.into_tensor();
        let reciprocal = 1.0 / axis_len as f32;
        let mut means = vec![0.0_f32; inner];
        let mut inverse_stddevs = vec![0.0_f32; inner];
        {
            let mut plain = output.try_as_plain_mut()?;
            let values = plain.as_slice_mut::<f32>()?;
            for outer_index in 0..outer {
                means.fill(0.0);
                inverse_stddevs.fill(0.0);
                let outer_start = outer_index * axis_len * inner;
                for axis_index in 0..axis_len {
                    let start = outer_start + axis_index * inner;
                    for inner_index in 0..inner {
                        let value = values[start + inner_index];
                        means[inner_index] += value;
                        inverse_stddevs[inner_index] += value * value;
                    }
                }
                for inner_index in 0..inner {
                    let mean = means[inner_index] * reciprocal;
                    means[inner_index] = mean;
                    let variance =
                        (inverse_stddevs[inner_index] * reciprocal - mean * mean).max(0.0);
                    inverse_stddevs[inner_index] = (variance + epsilon).sqrt().recip();
                }
                for axis_index in 0..axis_len {
                    let start = outer_start + axis_index * inner;
                    for inner_index in 0..inner {
                        values[start + inner_index] = (values[start + inner_index]
                            - means[inner_index])
                            * (inverse_stddevs[inner_index] * scale[axis_index])
                            + bias[axis_index];
                    }
                }
            }
        }
        Ok(tvec!(output.into_tvalue()))
    }
}

impl TypedOp for FusedLayerNorm {
    fn output_facts(&self, inputs: &[&TypedFact]) -> TractResult<TVec<TypedFact>> {
        ensure!(inputs.len() == 1);
        ensure!(inputs[0].datum_type == DatumType::F32);
        ensure!(inputs[0].shape.as_concrete() == Some(self.input_shape.as_slice()));
        Ok(tvec!(inputs[0].clone()))
    }

    as_op!();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fused_layer_norm_matches_scalar_reference() {
        let input_shape = [2, 3, 2];
        let scale = tensor3(&[[[1.0_f32], [0.5], [2.0]]]).into_arc_tensor();
        let bias = tensor3(&[[[0.1_f32], [-0.2], [0.3]]]).into_arc_tensor();
        let op = FusedLayerNorm::new(
            1,
            tensor0(1.0e-6_f32).into_arc_tensor(),
            scale,
            bias,
            &input_shape,
        )
        .unwrap();
        let input = Tensor::from_shape(
            &input_shape,
            &[1.0_f32, 4.0, 2.0, 5.0, 6.0, 3.0, 8.0, 2.0, 4.0, 7.0, 1.0, 9.0],
        )
        .unwrap();
        let output = op.eval(tvec!(input.into_tvalue())).unwrap().remove(0).into_tensor();
        let output = output.try_as_plain().unwrap();
        let got = output.as_slice::<f32>().unwrap();

        for outer in 0..2 {
            for inner in 0..2 {
                let row = (0..3)
                    .map(|axis| {
                        [1.0_f32, 4.0, 2.0, 5.0, 6.0, 3.0, 8.0, 2.0, 4.0, 7.0, 1.0, 9.0]
                            [(outer * 3 + axis) * 2 + inner]
                    })
                    .collect::<Vec<_>>();
                let mean = row.iter().sum::<f32>() / 3.0;
                let variance =
                    row.iter().map(|value| value * value).sum::<f32>() / 3.0 - mean * mean;
                for axis in 0..3 {
                    let expected = (row[axis] - mean)
                        * ((variance.max(0.0) + 1.0e-6).sqrt().recip() * [1.0_f32, 0.5, 2.0][axis])
                        + [0.1_f32, -0.2, 0.3][axis];
                    let actual = got[(outer * 3 + axis) * 2 + inner];
                    assert!(
                        (actual - expected).abs() < 1.0e-5,
                        "outer={outer} inner={inner} axis={axis}: got {actual}, want {expected}"
                    );
                }
            }
        }
    }

    #[test]
    fn fused_layer_norm_rejects_invalid_epsilon() {
        let scale = tensor3(&[[[1.0_f32], [1.0], [1.0]]]).into_arc_tensor();
        let bias = tensor3(&[[[0.0_f32], [0.0], [0.0]]]).into_arc_tensor();
        assert!(
            FusedLayerNorm::new(
                1,
                tensor0(f32::NAN).into_arc_tensor(),
                scale.clone(),
                bias.clone(),
                &[1, 3, 2],
            )
            .is_err()
        );
        assert!(
            FusedLayerNorm::new(1, tensor0(-1.0_f32).into_arc_tensor(), scale, bias, &[1, 3, 2],)
                .is_err()
        );
    }
}
