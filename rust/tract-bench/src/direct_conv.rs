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

//! Tiled Conv1D + bias + GELU + max operator for Magika models.

use std::alloc::Layout;
use std::fmt::{Debug, Display};
use std::hash::{Hash, Hasher};
use std::sync::Arc;

use tract_core::internal::*;
use tract_linalg::BinOp;
use tract_linalg::WeightType;
use tract_linalg::mmm::{AsInputValue, FusedSpec, MMMInputFormat, MMMInputValue, MatMatMul};
use tract_linalg::pack::{PackedFormat, PackingWriter};

/// Keep the default temporary activation bounded while preserving enough columns for an efficient
/// MMM. The benchmark can override this at model-preparation time to tune the memory/dispatch
/// tradeoff without rebuilding.
const DEFAULT_TILE_BATCHES: usize = 4;
const TILE_BATCHES_ENV: &str = "MAGIKA_DIRECT_TILE_BATCHES";

/// Replace a supported Conv1D -> transpose -> GELU -> max-over-position chain.
///
/// Returns the number of independent chains replaced.
pub(crate) fn fuse_magika_conv_max(model: &mut TypedModel, batch: usize) -> TractResult<usize> {
    let mut fused = 0;
    while let Some(pattern) = FusionPattern::find(model, batch)? {
        let op = DirectFusedConvMax1D::new(pattern.dimensions, pattern.kernel, pattern.bias)?;
        let mut patch = TypedModelPatch::default();
        let input = patch.tap_model(model, pattern.input)?;
        let output = patch.wire_node("magika.direct_fused_conv_max1d", op, &[input])?[0];
        patch.shunt_outside(model, OutletId::new(pattern.max_node, 0), output)?;
        patch.apply(model)?;
        model.compact()?;
        fused += 1;
    }
    Ok(fused)
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
struct ConvDimensions {
    batch: usize,
    input_channels: usize,
    input_length: usize,
    output_channels: usize,
    kernel_length: usize,
    output_length: usize,
    channels_last: bool,
}

impl ConvDimensions {
    fn reduction(self) -> usize {
        self.input_channels * self.kernel_length
    }

    fn columns(self) -> usize {
        self.batch * self.output_length
    }
}

struct FusionPattern {
    input: OutletId,
    max_node: usize,
    kernel: Arc<Tensor>,
    bias: Arc<Tensor>,
    dimensions: ConvDimensions,
}

impl FusionPattern {
    fn find(model: &TypedModel, batch: usize) -> TractResult<Option<Self>> {
        use tract_core::ops::change_axes::AxisOp;
        use tract_core::ops::cnn::{Conv, KernelFormat};
        use tract_core::ops::element_wise::ElementWiseOp;
        use tract_core::ops::nn::{DataFormat, GeluApproximate, Reduce, Reducer};

        for max in &model.nodes {
            let Some(reduce) = max.op.downcast_ref::<Reduce>() else { continue };
            if reduce.reducer != Reducer::Max || reduce.axes.as_slice() != [1] {
                continue;
            }
            let [gelu_input] = max.inputs.as_slice() else { continue };
            if gelu_input.slot != 0 {
                continue;
            }
            let gelu = model.node(gelu_input.node);
            let Some(element_wise) = gelu.op.downcast_ref::<ElementWiseOp>() else { continue };
            let Some(gelu_op) = element_wise.0.downcast_ref::<GeluApproximate>() else { continue };
            if gelu_op.fast_impl {
                continue;
            }
            let [transpose_input] = gelu.inputs.as_slice() else { continue };
            if transpose_input.slot != 0 {
                continue;
            }
            let activation_source = model.node(transpose_input.node);
            let conv = if activation_source.op.downcast_ref::<Conv>().is_some() {
                activation_source
            } else {
                let Some(axis_op) = activation_source.op.downcast_ref::<AxisOp>() else {
                    continue;
                };
                if !matches!(axis_op, AxisOp::Move(1, 2) | AxisOp::Move(2, 1)) {
                    continue;
                }
                let [conv_input] = activation_source.inputs.as_slice() else { continue };
                if conv_input.slot != 0 {
                    continue;
                }
                model.node(conv_input.node)
            };
            let Some(conv_op) = conv.op.downcast_ref::<Conv>() else { continue };
            if conv.inputs.len() != 3
                || conv_op.group != 1
                || conv_op.q_params.is_some()
                || conv_op.kernel_fmt != KernelFormat::OIHW
                || !matches!(conv_op.pool_spec.data_format, DataFormat::NCHW | DataFormat::NHWC)
                || conv_op.pool_spec.kernel_shape.len() != 1
                || conv_op.pool_spec.stride(0) != 1
                || conv_op.pool_spec.dilation(0) != 1
                || !conv_op.pool_spec.padding.valid_dim(0, true)
            {
                continue;
            }

            let input = conv.inputs[0];
            let Some(input_shape) = model.outlet_fact(input)?.shape.as_concrete() else {
                continue;
            };
            let (n, input_channels, input_length, channels_last) =
                match (conv_op.pool_spec.data_format, input_shape) {
                    (DataFormat::NCHW, [n, input_channels, input_length]) => {
                        (*n, *input_channels, *input_length, false)
                    }
                    (DataFormat::NHWC, [n, input_length, input_channels]) => {
                        (*n, *input_channels, *input_length, true)
                    }
                    _ => continue,
                };
            if n != batch || input_channels != conv_op.pool_spec.input_channels {
                continue;
            }
            let Some(kernel) = model.outlet_fact(conv.inputs[1])?.konst.clone() else {
                continue;
            };
            let Some(bias) = model.outlet_fact(conv.inputs[2])?.konst.clone() else {
                continue;
            };
            let kernel_shape = kernel.shape();
            let [output_channels, kernel_input_channels, kernel_length] = kernel_shape else {
                continue;
            };
            if *kernel_input_channels != input_channels
                || *output_channels != conv_op.pool_spec.output_channels
                || bias.shape() != [*output_channels]
                || input_length < *kernel_length
            {
                continue;
            }
            let output_length = input_length - *kernel_length + 1;
            let dimensions = ConvDimensions {
                batch,
                input_channels,
                input_length,
                output_channels: *output_channels,
                kernel_length: *kernel_length,
                output_length,
                channels_last,
            };
            let expected_conv_output = if channels_last {
                [batch, output_length, *output_channels]
            } else {
                [batch, *output_channels, output_length]
            };
            if model.outlet_fact(OutletId::new(conv.id, 0))?.shape.as_concrete()
                != Some(&expected_conv_output)
                || model.outlet_fact(OutletId::new(gelu.id, 0))?.shape.as_concrete()
                    != Some(&[batch, output_length, *output_channels])
                || model.outlet_fact(OutletId::new(max.id, 0))?.shape.as_concrete()
                    != Some(&[batch, 1, *output_channels])
            {
                continue;
            }
            return Ok(Some(Self { input, max_node: max.id, kernel, bias, dimensions }));
        }
        Ok(None)
    }
}

#[derive(Clone, Debug)]
struct DirectFusedConvMax1D {
    dimensions: ConvDimensions,
    tile_batches: usize,
    mmm: Box<dyn MatMatMul>,
    packing: usize,
    packed_kernel: Box<dyn MMMInputValue>,
    kernel: Arc<Tensor>,
    bias: Arc<Tensor>,
    input_format: Arc<DirectConvInputFormat>,
}

impl PartialEq for DirectFusedConvMax1D {
    fn eq(&self, other: &Self) -> bool {
        self.dimensions == other.dimensions
            && self.tile_batches == other.tile_batches
            && self.mmm.name() == other.mmm.name()
            && self.packing == other.packing
            && self.kernel == other.kernel
            && self.bias == other.bias
            && self.input_format == other.input_format
    }
}

impl Eq for DirectFusedConvMax1D {}

impl DirectFusedConvMax1D {
    fn new(
        dimensions: ConvDimensions, kernel: Arc<Tensor>, bias: Arc<Tensor>,
    ) -> TractResult<Self> {
        ensure!(kernel.datum_type() == DatumType::F32, "Conv1D kernel must be f32");
        ensure!(
            kernel.shape()
                == [
                    dimensions.output_channels,
                    dimensions.input_channels,
                    dimensions.kernel_length,
                ]
        );
        ensure!(bias.datum_type() == DatumType::F32, "Conv1D bias must be f32");
        ensure!(bias.shape() == [dimensions.output_channels]);

        let tile_batches = std::env::var(TILE_BATCHES_ENV)
            .ok()
            .map(|value| value.parse::<usize>())
            .transpose()
            .with_context(|| format!("{TILE_BATCHES_ENV} must be a positive integer"))?
            .unwrap_or(DEFAULT_TILE_BATCHES)
            .min(dimensions.batch);
        ensure!(tile_batches > 0, "{TILE_BATCHES_ENV} must be greater than zero");
        let tile_columns = dimensions.output_length * tile_batches;
        let mmm = tract_linalg::ops()
            .mmm(
                DatumType::F32,
                Some(dimensions.output_channels),
                Some(dimensions.reduction()),
                Some(tile_columns),
            )
            .context("no f32 matmul implementation for direct Magika Conv1D")?;
        let packing = mmm
            .packings()
            .iter()
            .position(|(a, b)| {
                a.precursor() == WeightType::Plain(DatumType::F32)
                    && b.precursor() == WeightType::Plain(DatumType::F32)
                    && b.is::<PackedFormat>()
            })
            .context("matmul implementation has no plain-f32 packed input pair")?;
        let packed_kernel = mmm.packings()[packing].0.prepare_one(
            &kernel
                .as_ref()
                .clone()
                .into_shape(&[dimensions.output_channels, dimensions.reduction()])?,
            1,
            0,
        )?;
        let input_packer = mmm.packings()[packing]
            .1
            .downcast_ref::<PackedFormat>()
            .context("direct Conv1D input packer is not a PackedFormat")?
            .clone();
        let column_offsets = (0..dimensions.columns())
            .map(|column| {
                let batch = column / dimensions.output_length;
                let position = column % dimensions.output_length;
                if dimensions.channels_last {
                    (batch * dimensions.input_length + position) * dimensions.input_channels
                } else {
                    batch * dimensions.input_channels * dimensions.input_length + position
                }
            })
            .collect();
        let reduction_offsets = (0..dimensions.reduction())
            .map(|k| {
                let channel = k / dimensions.kernel_length;
                let kernel_position = k % dimensions.kernel_length;
                if dimensions.channels_last {
                    kernel_position * dimensions.input_channels + channel
                } else {
                    channel * dimensions.input_length + kernel_position
                }
            })
            .collect();
        let input_format = Arc::new(DirectConvInputFormat {
            packer: input_packer,
            dimensions,
            tile_batches,
            column_offsets,
            reduction_offsets,
        });
        Ok(Self {
            dimensions,
            tile_batches,
            mmm,
            packing,
            packed_kernel,
            kernel,
            bias,
            input_format,
        })
    }
}

impl Op for DirectFusedConvMax1D {
    fn name(&self) -> StaticName {
        "DirectFusedConvMax1D".into()
    }

    fn info(&self) -> TractResult<Vec<String>> {
        Ok(vec![format!(
            "batch={} input_layout={} m={} k={} n={} tile_batches={} kernel={} eager_im2col=false bias=mmm gelu=in_place max=tiled",
            self.dimensions.batch,
            if self.dimensions.channels_last { "NLC" } else { "NCL" },
            self.dimensions.output_channels,
            self.dimensions.reduction(),
            self.dimensions.columns(),
            self.tile_batches,
            self.mmm.name()
        )])
    }

    op_as_typed_op!();
}

impl EvalOp for DirectFusedConvMax1D {
    fn is_stateless(&self) -> bool {
        true
    }

    fn eval(&self, inputs: TVec<TValue>) -> TractResult<TVec<TValue>> {
        let input = args_1!(inputs);
        let d = self.dimensions;
        let expected = if d.channels_last {
            [d.batch, d.input_length, d.input_channels]
        } else {
            [d.batch, d.input_channels, d.input_length]
        };
        ensure!(
            input.shape() == expected,
            "unexpected direct Conv1D input shape {:?}",
            input.shape()
        );
        ensure!(input.datum_type() == DatumType::F32, "direct Conv1D input must be f32");
        let input_ptr = input.as_ptr::<f32>()?;
        let input_len = input.len();
        let mut maxima = Tensor::from_shape(
            &[d.batch, 1, d.output_channels],
            &vec![f32::NEG_INFINITY; d.batch * d.output_channels],
        )?;
        {
            let mut maxima_plain = maxima.try_as_plain_mut()?;
            let maxima_values = maxima_plain.as_slice_mut::<f32>()?;
            for batch_start in (0..d.batch).step_by(self.tile_batches) {
                let tile_batches = (d.batch - batch_start).min(self.tile_batches);
                let tile_columns = tile_batches * d.output_length;
                // SAFETY: `input` remains alive and immutable on this eval stack until the
                // synchronous MMM call below returns. The input value is borrowed only by that
                // call, whose worker scratch buffers are thread-local and joined before return.
                let direct_input = unsafe {
                    DirectConvInput::new(
                        input_ptr,
                        input_len,
                        self.input_format.clone(),
                        batch_start * d.output_length,
                        tile_columns,
                    )
                };
                // Store covers every tile element before GELU and max read it.
                let mut tile = unsafe {
                    Tensor::uninitialized_dt(
                        DatumType::F32,
                        &[tile_batches, d.output_length, d.output_channels],
                    )?
                };
                let output_spec = unsafe {
                    self.mmm.c_from_data_and_strides(
                        std::mem::size_of::<f32>(),
                        1,
                        d.output_channels as isize,
                    )
                };
                {
                    let tile_view = tile.view();
                    let store = unsafe { output_spec.wrap(&tile_view) };
                    let specs = [
                        FusedSpec::AddMatMul {
                            a: AsInputValue::Borrowed(&*self.packed_kernel),
                            b: AsInputValue::Borrowed(&direct_input),
                            packing: self.packing,
                        },
                        FusedSpec::BinPerRow(self.bias.view(), BinOp::Add),
                        FusedSpec::Store(store),
                    ];
                    unsafe {
                        self.mmm.run(d.output_channels, tile_columns, &specs)?;
                    }
                }
                let mut tile_plain = tile.try_as_plain_mut()?;
                let tile_values = tile_plain.as_slice_mut::<f32>()?;
                (tract_linalg::ops().gelu_f32)().run(tile_values)?;
                for tile_batch in 0..tile_batches {
                    let maxima_start = (batch_start + tile_batch) * d.output_channels;
                    let item_maxima =
                        &mut maxima_values[maxima_start..maxima_start + d.output_channels];
                    let item_start = tile_batch * d.output_length * d.output_channels;
                    for position in 0..d.output_length {
                        let row_start = item_start + position * d.output_channels;
                        update_maxima(
                            item_maxima,
                            &tile_values[row_start..row_start + d.output_channels],
                        );
                    }
                }
            }
        }
        Ok(tvec!(maxima.into_tvalue()))
    }
}

#[inline]
fn update_maxima(maxima: &mut [f32], row: &[f32]) {
    for (maximum, value) in maxima.iter_mut().zip(row) {
        *maximum = maximum.max(*value);
    }
}

impl TypedOp for DirectFusedConvMax1D {
    fn output_facts(&self, inputs: &[&TypedFact]) -> TractResult<TVec<TypedFact>> {
        ensure!(inputs.len() == 1);
        ensure!(inputs[0].datum_type == DatumType::F32);
        let d = self.dimensions;
        let expected = if d.channels_last {
            [d.batch, d.input_length, d.input_channels]
        } else {
            [d.batch, d.input_channels, d.input_length]
        };
        ensure!(inputs[0].shape.as_concrete() == Some(&expected[..]));
        Ok(tvec!(DatumType::F32.fact([d.batch, 1, d.output_channels])))
    }

    as_op!();
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
struct DirectConvInputFormat {
    packer: PackedFormat,
    dimensions: ConvDimensions,
    tile_batches: usize,
    column_offsets: Vec<usize>,
    reduction_offsets: Vec<usize>,
}

impl Display for DirectConvInputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "DirectConv1D({})", self.packer)
    }
}

impl ExoticFact for DirectConvInputFormat {
    fn buffer_sizes(&self) -> TVec<TDim> {
        let d = self.dimensions;
        tvec!(
            (d.reduction() * d.output_length * self.tile_batches * std::mem::size_of::<f32>())
                .to_dim()
        )
    }
}

impl MMMInputFormat for DirectConvInputFormat {
    fn prepare_tensor(&self, _: &Tensor, _: usize, _: usize) -> TractResult<Tensor> {
        bail!("DirectConvInputFormat cannot eagerly prepare a tensor")
    }

    fn prepare_one(&self, _: &Tensor, _: usize, _: usize) -> TractResult<Box<dyn MMMInputValue>> {
        bail!("DirectConvInputFormat is created by DirectFusedConvMax1D")
    }

    fn precursor(&self) -> WeightType {
        self.packer.precursor()
    }

    fn r(&self) -> usize {
        self.packer.r
    }

    fn k_alignment(&self) -> usize {
        1
    }

    fn mem_size(&self, k: TDim, mn: TDim) -> TDim {
        k * mn * std::mem::size_of::<f32>()
    }

    fn extract_at_mn_f16(
        &self, _: &tract_linalg::mmm::EagerPackedInput, _: usize, _: &mut [f16],
    ) -> TractResult<()> {
        bail!("f16 extraction is unsupported")
    }

    fn extract_at_mn_f32(
        &self, _: &tract_linalg::mmm::EagerPackedInput, _: usize, _: &mut [f32],
    ) -> TractResult<()> {
        bail!("eager extraction is unsupported")
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct DirectConvInput {
    input: *const f32,
    input_len: usize,
    format: Arc<DirectConvInputFormat>,
    column_start: usize,
    columns: usize,
}

impl Display for DirectConvInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "DirectConvInput(len={})", self.input_len)
    }
}

impl Hash for DirectConvInput {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.input.hash(state);
        self.input_len.hash(state);
        self.format.hash(state);
        self.column_start.hash(state);
        self.columns.hash(state);
    }
}

// SAFETY: values are created only from an immutable f32 slice that remains live until the
// synchronous MMM invocation and all of its worker threads return. Clones share only that immutable
// allocation; panel writes target caller-provided thread-local scratch buffers.
unsafe impl Send for DirectConvInput {}
// SAFETY: see the `Send` justification above; shared access only reads the live input allocation.
unsafe impl Sync for DirectConvInput {}

impl DirectConvInput {
    /// The caller must keep `input` alive and immutable until this value and all clones are dropped.
    unsafe fn new(
        input: *const f32, input_len: usize, format: Arc<DirectConvInputFormat>,
        column_start: usize, columns: usize,
    ) -> Self {
        Self { input, input_len, format, column_start, columns }
    }

    #[inline]
    fn input(&self) -> &[f32] {
        // SAFETY: upheld by `new`; the MMM call cannot outlive the eval-stack tensor owner.
        unsafe { std::slice::from_raw_parts(self.input, self.input_len) }
    }

    fn write_panel(&self, panel: usize, buffer: *mut u8) -> *const u8 {
        let r = self.format.packer.r;
        let column_start = panel * r;
        let column_end = (column_start + r).min(self.mn());
        let mut writer = if column_end - column_start == r {
            self.format.packer.write_single_panel_with_k_outer(buffer.cast::<f32>())
        } else {
            // The two writer types differ, so handle the tail in a helper.
            return self.write_tail_panel(column_start, column_end, buffer);
        };
        self.write_columns(&mut writer, column_start, column_end);
        buffer
    }

    fn write_tail_panel(&self, start: usize, end: usize, buffer: *mut u8) -> *const u8 {
        let mut writer = self.format.packer.write_with_k_outer(
            buffer.cast::<f32>(),
            self.format.dimensions.reduction(),
            end - start,
        );
        self.write_columns(&mut writer, start, end);
        buffer
    }

    #[inline]
    fn write_columns(&self, writer: &mut impl PackingWriter<f32>, start: usize, end: usize) {
        let input = self.input();
        let start = self.column_start + start;
        let end = self.column_start + end;
        let columns = &self.format.column_offsets[start..end];
        match columns {
            &[c0, c1, c2, c3, c4, c5, c6, c7] => {
                for &k in &self.format.reduction_offsets {
                    writer.write(input[c0 + k]);
                    writer.write(input[c1 + k]);
                    writer.write(input[c2 + k]);
                    writer.write(input[c3 + k]);
                    writer.write(input[c4 + k]);
                    writer.write(input[c5 + k]);
                    writer.write(input[c6 + k]);
                    writer.write(input[c7 + k]);
                }
            }
            &[c0, c1, c2, c3, c4, c5] => {
                for &k in &self.format.reduction_offsets {
                    writer.write(input[c0 + k]);
                    writer.write(input[c1 + k]);
                    writer.write(input[c2 + k]);
                    writer.write(input[c3 + k]);
                    writer.write(input[c4 + k]);
                    writer.write(input[c5 + k]);
                }
            }
            &[c0, c1, c2, c3] => {
                for &k in &self.format.reduction_offsets {
                    writer.write(input[c0 + k]);
                    writer.write(input[c1 + k]);
                    writer.write(input[c2 + k]);
                    writer.write(input[c3 + k]);
                }
            }
            &[c0, c1] => {
                for &k in &self.format.reduction_offsets {
                    writer.write(input[c0 + k]);
                    writer.write(input[c1 + k]);
                }
            }
            columns => {
                for &k in &self.format.reduction_offsets {
                    for &column in columns {
                        writer.write(input[column + k]);
                    }
                }
            }
        }
    }
}

impl MMMInputValue for DirectConvInput {
    fn format(&self) -> &dyn MMMInputFormat {
        &*self.format
    }

    fn scratch_panel_buffer_layout(&self) -> Option<Layout> {
        Some(
            self.format.packer.single_panel_layout(
                self.format.dimensions.reduction(),
                std::mem::size_of::<f32>(),
            ),
        )
    }

    fn panel_bytes(&self, panel: usize, buffer: Option<*mut u8>) -> TractResult<*const u8> {
        Ok(self.write_panel(panel, buffer.context("direct Conv1D needs a scratch panel")?))
    }

    fn mn(&self) -> usize {
        self.columns
    }

    fn k(&self) -> usize {
        self.format.dimensions.reduction()
    }

    fn exotic_fact(&self) -> &dyn ExoticFact {
        &*self.format
    }

    fn extract_at_mn_f16(&self, _: usize, _: &mut [f16]) -> TractResult<()> {
        bail!("f16 extraction is unsupported")
    }

    fn extract_at_mn_f32(&self, column: usize, output: &mut [f32]) -> TractResult<()> {
        ensure!(column < self.mn());
        ensure!(output.len() == self.format.dimensions.reduction());
        let input = self.input();
        for (k, value) in output.iter_mut().enumerate() {
            let offset = self.format.column_offsets[self.column_start + column]
                + self.format.reduction_offsets[k];
            *value = input[offset];
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn gelu(value: f32) -> f32 {
        0.5 * value
            * (1.0
                + f32::tanh(
                    (2.0 / std::f32::consts::PI).sqrt() * (value + 0.044715 * value.powi(3)),
                ))
    }

    fn direct_matches_reference(channels_last: bool) -> TractResult<()> {
        let dimensions = ConvDimensions {
            batch: 5,
            input_channels: 2,
            input_length: 6,
            output_channels: 3,
            kernel_length: 3,
            output_length: 4,
            channels_last,
        };
        let input = (0..dimensions.batch * dimensions.input_channels * dimensions.input_length)
            .map(|index| (index as f32 % 17.0 - 8.0) * 0.1)
            .collect::<Vec<_>>();
        let kernel =
            (0..dimensions.output_channels * dimensions.input_channels * dimensions.kernel_length)
                .map(|index| (index as f32 % 7.0 - 3.0) * 0.05)
                .collect::<Vec<_>>();
        let bias = vec![0.1, -0.2, 0.3];
        let kernel_tensor = Arc::new(Tensor::from_shape(
            &[dimensions.output_channels, dimensions.input_channels, dimensions.kernel_length],
            &kernel,
        )?);
        let bias_tensor = Arc::new(Tensor::from_shape(&[dimensions.output_channels], &bias)?);
        let mut op = DirectFusedConvMax1D::new(dimensions, kernel_tensor, bias_tensor)?;
        op.tile_batches = 2;
        Arc::make_mut(&mut op.input_format).tile_batches = 2;
        let input_shape = if channels_last {
            [dimensions.batch, dimensions.input_length, dimensions.input_channels]
        } else {
            [dimensions.batch, dimensions.input_channels, dimensions.input_length]
        };
        let mut outputs =
            op.eval(tvec!(Tensor::from_shape(&input_shape, &input)?.into_tvalue()))?;
        let output = outputs.remove(0).into_tensor();
        let output = output.to_plain_array_view::<f32>()?;

        for batch in 0..dimensions.batch {
            for output_channel in 0..dimensions.output_channels {
                let mut expected = f32::NEG_INFINITY;
                for position in 0..dimensions.output_length {
                    let mut convolution = bias[output_channel];
                    for input_channel in 0..dimensions.input_channels {
                        for kernel_position in 0..dimensions.kernel_length {
                            let input_position = position + kernel_position;
                            let input_index = if channels_last {
                                (batch * dimensions.input_length + input_position)
                                    * dimensions.input_channels
                                    + input_channel
                            } else {
                                (batch * dimensions.input_channels + input_channel)
                                    * dimensions.input_length
                                    + input_position
                            };
                            let kernel_index = (output_channel * dimensions.input_channels
                                + input_channel)
                                * dimensions.kernel_length
                                + kernel_position;
                            convolution += input[input_index] * kernel[kernel_index];
                        }
                    }
                    expected = expected.max(gelu(convolution));
                }
                let actual = output[[batch, 0, output_channel]];
                ensure!((actual - expected).abs() <= 1e-5, "{actual} != {expected}");
            }
        }
        Ok(())
    }

    #[test]
    fn unmatched_model_reports_zero_fusions() {
        let mut model = TypedModel::default();
        assert_eq!(fuse_magika_conv_max(&mut model, 8).unwrap(), 0);
    }

    #[test]
    fn tiled_max_accumulates_each_channel() {
        let mut maxima = [f32::NEG_INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY];
        update_maxima(&mut maxima, &[1.0, 4.0, -2.0]);
        update_maxima(&mut maxima, &[3.0, 2.0, -1.0]);
        assert_eq!(maxima, [3.0, 4.0, -1.0]);
    }

    #[test]
    fn direct_operator_matches_reference_for_both_layouts() -> TractResult<()> {
        direct_matches_reference(false)?;
        direct_matches_reference(true)
    }
}
