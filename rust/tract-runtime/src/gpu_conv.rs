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

//! GPU-friendly lowering for Magika's valid Conv1D.

use std::sync::Arc;

use tract_core::internal::*;
use tract_core::ops::array::{Slice, TypedConcat};
use tract_core::ops::binary::TypedBinOp;
use tract_core::ops::change_axes::AxisOp;
use tract_core::ops::cnn::{Conv, KernelFormat};
use tract_core::ops::einsum::prefix_matmul::PrefixMatMul;
use tract_core::ops::math::Add;
use tract_core::ops::nn::DataFormat;

/// Lowers a supported channels-last Conv1D to a single matrix multiplication.
///
/// tract-metal's generic direct convolution gives every output element its own thread, which
/// repeatedly reads the same input and weights. Folding the kernel window into the reduction axis
/// instead leaves one large GEMM, for which the Metal transform selects its tuned matrix kernel.
pub(crate) fn lower_magika_conv_to_matmul(model: &mut TypedModel) -> TractResult<usize> {
    let mut lowered = 0;
    while let Some(pattern) = ConvPattern::find(model)? {
        pattern.apply(model)?;
        model.compact()?;
        lowered += 1;
    }
    Ok(lowered)
}

struct ConvPattern {
    channels_last_input: OutletId,
    channels_last_output: OutletId,
    kernel: Arc<Tensor>,
    bias: Arc<Tensor>,
    batch: usize,
    input_length: usize,
    input_channels: usize,
    output_channels: usize,
    kernel_length: usize,
}

impl ConvPattern {
    fn find(model: &TypedModel) -> TractResult<Option<Self>> {
        for output in &model.nodes {
            let Some(conv) = output.op_as::<Conv>() else { continue };
            if output.inputs.len() != 3
                || conv.group != 1
                || conv.q_params.is_some()
                || conv.kernel_fmt != KernelFormat::OIHW
                || conv.pool_spec.data_format != DataFormat::NHWC
                || conv.pool_spec.kernel_shape.len() != 1
                || conv.pool_spec.stride(0) != 1
                || conv.pool_spec.dilation(0) != 1
                || !conv.pool_spec.padding.valid_dim(0, true)
            {
                continue;
            }

            let channels_last_input = output.inputs[0];
            let Some(input_shape) = model.outlet_fact(channels_last_input)?.shape.as_concrete()
            else {
                continue;
            };
            let [batch, input_length, input_channels] = input_shape else { continue };
            let Some(kernel) = model.outlet_fact(output.inputs[1])?.konst.clone() else {
                continue;
            };
            let Some(bias) = model.outlet_fact(output.inputs[2])?.konst.clone() else {
                continue;
            };
            let [output_channels, kernel_input_channels, kernel_length] = kernel.shape() else {
                continue;
            };
            let (output_channels, kernel_input_channels, kernel_length) =
                (*output_channels, *kernel_input_channels, *kernel_length);
            if *input_channels != kernel_input_channels
                || *input_channels != conv.pool_spec.input_channels
                || output_channels != conv.pool_spec.output_channels
                || kernel_length != conv.pool_spec.kernel_shape[0]
                || bias.shape() != [output_channels]
                || *input_length < kernel_length
                || kernel.datum_type() != DatumType::F32
                || bias.datum_type() != DatumType::F32
            {
                continue;
            }
            let output_length = *input_length - kernel_length + 1;
            if model.outlet_fact(OutletId::new(output.id, 0))?.shape.as_concrete()
                != Some(&[*batch, output_length, output_channels])
            {
                continue;
            }
            return Ok(Some(Self {
                channels_last_input,
                channels_last_output: OutletId::new(output.id, 0),
                kernel,
                bias,
                batch: *batch,
                input_length: *input_length,
                input_channels: *input_channels,
                output_channels,
                kernel_length,
            }));
        }
        Ok(None)
    }

    fn apply(self, model: &mut TypedModel) -> TractResult<()> {
        let output_length = self.input_length - self.kernel_length + 1;
        let rows = self.batch * output_length;

        let mut patch = TypedModelPatch::default();
        let input = patch.tap_model(model, self.channels_last_input)?;

        // Every kernel position is the input shifted along the position axis. Concatenating the
        // shifts on the channel axis turns the whole convolution into one matrix multiplication,
        // which is the classic im2col expansion written with ordinary operators.
        //
        // One product per position and a sum tree computes the same thing, but each product
        // materializes a full `[batch, output_length, output_channels]` activation that the tree
        // then reads back. At batch 32 that is about 33 MB written and re-read five times over,
        // against a single 83 MB concatenation here, and it measured about 30% of throughput.
        let mut shifts = TVec::with_capacity(self.kernel_length);
        for kernel_index in 0..self.kernel_length {
            shifts.push(
                patch.wire_node(
                    format!("magika.gpu_conv.slice_{kernel_index}"),
                    Slice::new(1, kernel_index, kernel_index + output_length),
                    &[input],
                )?[0],
            );
        }
        let gathered =
            patch.wire_node("magika.gpu_conv.gather", TypedConcat { axis: 2 }, &shifts)?[0];
        let matrix = patch.wire_node(
            "magika.gpu_conv.flatten",
            AxisOp::Reshape(
                0,
                tvec![self.batch.to_dim(), output_length.to_dim()],
                tvec![rows.to_dim()],
            ),
            &[gathered],
        )?[0];
        let weights = patch.add_const("magika.gpu_conv.weights", self.kernel_matrix_all()?)?;
        let product = patch.wire_node(
            "magika.gpu_conv.gemm",
            PrefixMatMul {
                transpose_a: false,
                transpose_b: true,
                transpose_c: false,
                quantize_output: None,
                operating_dt: Some(DatumType::F32),
            },
            &[matrix, weights],
        )?[0];
        let product = patch.wire_node(
            "magika.gpu_conv.unflatten",
            AxisOp::Reshape(
                0,
                tvec![rows.to_dim()],
                tvec![self.batch.to_dim(), output_length.to_dim()],
            ),
            &[product],
        )?[0];

        let bias = self.bias.as_ref().clone().into_shape(&[1, 1, self.output_channels])?;
        let bias = patch.add_const("magika.gpu_conv.bias", bias)?;
        let output = patch.wire_node(
            "magika.gpu_conv.bias_add",
            TypedBinOp(Box::new(Add), None),
            &[product, bias],
        )?[0];
        patch.shunt_outside(model, self.channels_last_output, output)?;
        patch.apply(model)
    }

    /// Returns the kernel as one `[output_channels, kernel_length * input_channels]` matrix.
    ///
    /// The column order is position major, matching the concatenation of the shifted inputs.
    fn kernel_matrix_all(&self) -> TractResult<Tensor> {
        let source_plain = self.kernel.as_ref().try_as_plain()?;
        let source = source_plain.as_slice::<f32>()?;
        let reduction = self.kernel_length * self.input_channels;
        let mut values = Vec::with_capacity(self.output_channels * reduction);
        for output_channel in 0..self.output_channels {
            for kernel_index in 0..self.kernel_length {
                for input_channel in 0..self.input_channels {
                    let offset = (output_channel * self.input_channels + input_channel)
                        * self.kernel_length
                        + kernel_index;
                    values.push(source[offset]);
                }
            }
        }
        Tensor::from_shape(&[self.output_channels, reduction], &values)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flattens_oik_kernel_into_one_position_major_matrix() -> TractResult<()> {
        let kernel =
            Tensor::from_shape(&[2, 2, 3], &(0..12).map(|value| value as f32).collect::<Vec<_>>())?
                .into_arc_tensor();
        let pattern = ConvPattern {
            channels_last_input: OutletId::new(0, 0),
            channels_last_output: OutletId::new(1, 0),
            kernel,
            bias: tensor1(&[0.0_f32, 0.0]).into_arc_tensor(),
            batch: 1,
            input_length: 4,
            input_channels: 2,
            output_channels: 2,
            kernel_length: 3,
        };

        // Rows are output channels; columns are position major, matching the concatenated shifts.
        let matrix = pattern.kernel_matrix_all()?;
        assert_eq!(matrix.shape(), &[2, 6]);
        assert_eq!(
            matrix.try_as_plain()?.as_slice::<f32>()?,
            &[0.0, 3.0, 1.0, 4.0, 2.0, 5.0, 6.0, 9.0, 7.0, 10.0, 8.0, 11.0]
        );
        Ok(())
    }
}
