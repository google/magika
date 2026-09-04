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
use tract_linalg::mmm::{AsInputValue, FusedSpec, MMMInputFormat, MMMInputValue, MatMatMul};
#[cfg(target_arch = "x86_64")]
use tract_linalg::mmm::{EagerPackedInput, PackedExoticFact};
use tract_linalg::pack::{PackedFormat, PackingWriter};
use tract_linalg::{BinOp, WeightType};

/// Default number of complete file batches per tile on the portable virtual-input path.
const DEFAULT_TILE_BATCHES: usize = 4;
const TILE_BATCHES_ENV: &str = "MAGIKA_DIRECT_TILE_BATCHES";
/// Overrides the tile below, which is sized for a cache this project cannot measure everywhere.
const TILE_COLUMNS_ENV: &str = "MAGIKA_DIRECT_TILE_COLUMNS";
/// Output columns packed into the matmul's final B-panel layout at a time on x86_64.
///
/// Throughput measured flat across 24 to 96 columns and fell away past 144, so this takes the small
/// end of the plateau rather than its middle. A tile costs `reduction * columns * 4` bytes of packed
/// input, which at 48 is a quarter megabyte: within a 256KiB L2, and still within half of a 1MiB L2
/// when two hyperthreads share one. The 96 this replaces was three times that, sized on a machine
/// whose L2 happened to absorb it.
#[cfg(target_arch = "x86_64")]
const X86_64_TILE_COLUMNS: usize = 48;
/// Panel width the vectorized packer below transposes into, matching the AVX-512 kernel's `nr`.
const PACK_PANEL_COLUMNS: usize = 12;

/// Whether this core can run the vectorized packer.
fn has_transposing_packer() -> bool {
    #[cfg(target_arch = "x86_64")]
    {
        is_x86_feature_detected!("avx2")
    }
    #[cfg(not(target_arch = "x86_64"))]
    {
        false
    }
}

/// Replace a supported Conv1D -> transpose -> GELU -> max-over-position chain.
///
/// Returns the number of independent chains replaced.
pub(crate) fn fuse_magika_conv_max(model: &mut TypedModel, batch: usize) -> TractResult<usize> {
    fuse_magika_conv_max_inner(model, batch, false)
}

/// Fuses the chain onto the packing path of a target without a vectorized packer, so a test can
/// run the real model through the fallback on a machine that would not otherwise take it.
#[cfg(test)]
pub(crate) fn fuse_magika_conv_max_portable(
    model: &mut TypedModel, batch: usize,
) -> TractResult<usize> {
    fuse_magika_conv_max_inner(model, batch, true)
}

fn fuse_magika_conv_max_inner(
    model: &mut TypedModel, batch: usize, force_portable: bool,
) -> TractResult<usize> {
    let mut fused = 0;
    while let Some(pattern) = FusionPattern::find(model, batch)? {
        let op = DirectFusedConvMax1D::new(
            pattern.dimensions,
            pattern.kernel,
            pattern.bias,
            force_portable,
        )?;
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
    tile_columns: usize,
    eager_pack: bool,
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
            && self.tile_columns == other.tile_columns
            && self.eager_pack == other.eager_pack
            && self.mmm.name() == other.mmm.name()
            && self.packing == other.packing
            && self.kernel == other.kernel
            && self.bias == other.bias
            && self.input_format == other.input_format
    }
}

impl Eq for DirectFusedConvMax1D {}

impl DirectFusedConvMax1D {
    /// `force_portable` makes the operator take the packing path of a target without a vectorized
    /// packer, so a test can compare the two on one machine. It can only ever turn the eager path
    /// off; turning it on where the host cannot run it would be unsound.
    fn new(
        dimensions: ConvDimensions, kernel: Arc<Tensor>, bias: Arc<Tensor>, force_portable: bool,
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

        let configured_tile_batches = std::env::var(TILE_BATCHES_ENV)
            .ok()
            .map(|value| value.parse::<usize>())
            .transpose()
            .with_context(|| format!("{TILE_BATCHES_ENV} must be a positive integer"))?;
        let tile_batches =
            configured_tile_batches.unwrap_or(DEFAULT_TILE_BATCHES).min(dimensions.batch);
        ensure!(tile_batches > 0, "{TILE_BATCHES_ENV} must be greater than zero");
        let default_tile_columns = dimensions.output_length * tile_batches;
        #[cfg(target_arch = "x86_64")]
        let tile_columns = if dimensions.channels_last && configured_tile_batches.is_none() {
            X86_64_TILE_COLUMNS.min(dimensions.columns())
        } else {
            default_tile_columns
        };
        #[cfg(not(target_arch = "x86_64"))]
        let tile_columns = default_tile_columns;
        let tile_columns = std::env::var(TILE_COLUMNS_ENV)
            .ok()
            .map(|value| value.parse::<usize>())
            .transpose()
            .with_context(|| format!("{TILE_COLUMNS_ENV} must be a positive integer"))?
            .map(|columns| columns.min(dimensions.columns()))
            .unwrap_or(tile_columns);
        ensure!(tile_columns > 0, "{TILE_COLUMNS_ENV} must be greater than zero");
        // Let tract score its own kernel pool for this tile. The bounded x86_64 tile is what makes
        // that choice good: at this size the scorer lands on the 16x12 AVX-512 kernel, whereas the
        // unbounded tile the portable path uses scores a narrower one. A whole-batch tile scores
        // the 128x1 kernel, which measured five times slower.
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
        let input_packer = mmm.packings()[packing]
            .1
            .downcast_ref::<PackedFormat>()
            .context("direct Conv1D input packer is not a PackedFormat")?
            .clone();
        // Pack each tile up front only where there is a vectorized packer for the panel width the
        // scorer chose. That is the AVX-512 kernel's twelve columns; a core without AVX-512 scores
        // a six-column kernel instead, and packing that one element by element would be slower
        // than the portable path below, which lets the matmul pull columns as it needs them.
        let eager_pack = !force_portable
            && dimensions.channels_last
            && input_packer.r == PACK_PANEL_COLUMNS
            && dimensions.input_channels.is_multiple_of(8)
            && has_transposing_packer();
        // The eager packer consumes each [position, input] window contiguously, so it needs the
        // kernel compiled into a matching [output, position, input] layout. The portable path keeps
        // the model's [output, input, position] layout and the DirectConvInput graph unchanged.
        // These two must agree: reordering the kernel without the matching packer is not a failure,
        // it is silently wrong scores.
        let reorder_kernel = eager_pack;
        let kernel_matrix = if reorder_kernel {
            let source = kernel.to_plain_array_view::<f32>()?;
            let mut reordered = Vec::with_capacity(kernel.len());
            for output in 0..dimensions.output_channels {
                for position in 0..dimensions.kernel_length {
                    for input in 0..dimensions.input_channels {
                        reordered.push(source[[output, input, position]]);
                    }
                }
            }
            Tensor::from_shape(&[dimensions.output_channels, dimensions.reduction()], &reordered)?
        } else {
            kernel
                .as_ref()
                .clone()
                .into_shape(&[dimensions.output_channels, dimensions.reduction()])?
        };
        let packed_kernel = mmm.packings()[packing].0.prepare_one(&kernel_matrix, 1, 0)?;
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
                if reorder_kernel {
                    // The reordered kernel walks [position, input], exactly as NLC input does.
                    k
                } else if dimensions.channels_last {
                    let channel = k / dimensions.kernel_length;
                    let kernel_position = k % dimensions.kernel_length;
                    kernel_position * dimensions.input_channels + channel
                } else {
                    let channel = k / dimensions.kernel_length;
                    let kernel_position = k % dimensions.kernel_length;
                    channel * dimensions.input_length + kernel_position
                }
            })
            .collect();
        let input_format = Arc::new(DirectConvInputFormat {
            packer: input_packer,
            dimensions,
            tile_columns,
            column_offsets,
            reduction_offsets,
        });
        Ok(Self {
            dimensions,
            tile_batches,
            tile_columns,
            eager_pack,
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
        let packing = if self.eager_pack { "x86_64-eager" } else { "portable-virtual" };
        Ok(vec![format!(
            "batch={} input_layout={} m={} k={} n={} tile_batches={} tile_columns={} kernel={} input_packing={} bias=mmm gelu=in_place max=tiled",
            self.dimensions.batch,
            if self.dimensions.channels_last { "NLC" } else { "NCL" },
            self.dimensions.output_channels,
            self.dimensions.reduction(),
            self.dimensions.columns(),
            self.tile_batches,
            self.tile_columns,
            self.mmm.name(),
            packing
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
            #[cfg(target_arch = "x86_64")]
            let mut segment_maxima = vec![f32::NEG_INFINITY; d.output_channels];
            #[cfg(target_arch = "x86_64")]
            let mut negative_channels = Vec::new();
            #[cfg(target_arch = "x86_64")]
            let mut negative_values = Vec::new();
            for column_start in (0..d.columns()).step_by(self.tile_columns) {
                let tile_columns = (d.columns() - column_start).min(self.tile_columns);
                #[cfg(target_arch = "x86_64")]
                let packed_input: Box<dyn MMMInputValue> = if self.eager_pack {
                    let input = unsafe { std::slice::from_raw_parts(input_ptr, input_len) };
                    pack_x86_64_nlc(
                        &self.input_format.packer,
                        input,
                        &self.input_format.column_offsets,
                        column_start,
                        tile_columns,
                        d.input_channels,
                        d.kernel_length,
                        d.reduction(),
                    )?
                } else {
                    // SAFETY: `input` remains alive and immutable on this eval stack until the
                    // synchronous MMM call below returns. The input value is borrowed only by
                    // that call, whose worker scratch buffers are joined before return.
                    Box::new(unsafe {
                        DirectConvInput::new(
                            input_ptr,
                            input_len,
                            self.input_format.clone(),
                            column_start,
                            tile_columns,
                        )
                    })
                };
                #[cfg(not(target_arch = "x86_64"))]
                let packed_input: Box<dyn MMMInputValue> = Box::new(unsafe {
                    DirectConvInput::new(
                        input_ptr,
                        input_len,
                        self.input_format.clone(),
                        column_start,
                        tile_columns,
                    )
                });
                // Store covers every tile element before GELU and max read it.
                let mut tile = unsafe {
                    Tensor::uninitialized_dt(DatumType::F32, &[tile_columns, d.output_channels])?
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
                            b: AsInputValue::Borrowed(&*packed_input),
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
                #[cfg(target_arch = "x86_64")]
                reduce_x86_64_gelu_max(
                    maxima_values,
                    &mut segment_maxima,
                    &mut negative_channels,
                    &mut negative_values,
                    tile_values,
                    column_start,
                    tile_columns,
                    d.output_length,
                    d.output_channels,
                )?;
                #[cfg(not(target_arch = "x86_64"))]
                {
                    (tract_linalg::ops().gelu_f32)().run(tile_values)?;
                    for column in 0..tile_columns {
                        let batch = (column_start + column) / d.output_length;
                        let maxima_start = batch * d.output_channels;
                        let item_maxima =
                            &mut maxima_values[maxima_start..maxima_start + d.output_channels];
                        let row_start = column * d.output_channels;
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

#[cfg(target_arch = "x86_64")]
#[allow(clippy::too_many_arguments)] // Flat hot-path slices preserve LLVM alias information.
fn reduce_x86_64_gelu_max(
    maxima: &mut [f32], segment_maxima: &mut [f32], negative_channels: &mut Vec<usize>,
    negative_values: &mut Vec<f32>, tile: &mut [f32], column_start: usize, tile_columns: usize,
    output_length: usize, output_channels: usize,
) -> TractResult<()> {
    let mut column = 0;
    while column < tile_columns {
        let global_column = column_start + column;
        let batch = global_column / output_length;
        let remaining_in_batch = output_length - global_column % output_length;
        let segment_columns = remaining_in_batch.min(tile_columns - column);
        segment_maxima.fill(f32::NEG_INFINITY);
        for segment_column in 0..segment_columns {
            let row_start = (column + segment_column) * output_channels;
            update_maxima(segment_maxima, &tile[row_start..row_start + output_channels]);
        }

        let maxima_start = batch * output_channels;
        let item_maxima = &mut maxima[maxima_start..maxima_start + output_channels];
        if segment_maxima.iter().all(|value| value.is_finite()) {
            // GELU is monotonic and non-negative from zero upward, while negative inputs produce
            // non-positive outputs. Evaluate it once for positive-channel maxima, and gather only
            // the unusual all-negative channels for exact element-wise evaluation.
            negative_channels.clear();
            negative_channels.extend(
                segment_maxima
                    .iter()
                    .enumerate()
                    .filter_map(|(channel, value)| (*value < 0.0).then_some(channel)),
            );
            negative_values.clear();
            negative_values.reserve(negative_channels.len() * segment_columns);
            for segment_column in 0..segment_columns {
                let row_start = (column + segment_column) * output_channels;
                for &channel in negative_channels.iter() {
                    negative_values.push(tile[row_start + channel]);
                }
            }
            (tract_linalg::ops().gelu_f32)().run(segment_maxima)?;
            let mut next_negative = negative_channels.iter().copied().peekable();
            for channel in 0..output_channels {
                if next_negative.peek() == Some(&channel) {
                    next_negative.next();
                } else {
                    item_maxima[channel] = item_maxima[channel].max(segment_maxima[channel]);
                }
            }
            if !negative_channels.is_empty() {
                (tract_linalg::ops().gelu_f32)().run(negative_values)?;
                for values in negative_values.chunks_exact(negative_channels.len()) {
                    for (&channel, &value) in negative_channels.iter().zip(values) {
                        item_maxima[channel] = item_maxima[channel].max(value);
                    }
                }
            }
        } else {
            // Preserve exact behavior for an unusual all-negative channel (and for non-finite
            // values) by evaluating every activation in this segment.
            let start = column * output_channels;
            let end = (column + segment_columns) * output_channels;
            let segment = &mut tile[start..end];
            (tract_linalg::ops().gelu_f32)().run(segment)?;
            for row in segment.chunks_exact(output_channels) {
                update_maxima(item_maxima, row);
            }
        }
        column += segment_columns;
    }
    Ok(())
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
    tile_columns: usize,
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
        tvec!((d.reduction() * self.tile_columns * std::mem::size_of::<f32>()).to_dim())
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

#[cfg(target_arch = "x86_64")]
#[allow(clippy::too_many_arguments)] // Flat hot-path slices preserve LLVM alias information.
fn pack_x86_64_nlc(
    format: &PackedFormat, input: &[f32], column_offsets: &[usize], column_start: usize,
    columns: usize, input_channels: usize, kernel_length: usize, reduction: usize,
) -> TractResult<Box<dyn MMMInputValue>> {
    let offsets = &column_offsets[column_start..column_start + columns];
    ensure!(offsets.last().is_none_or(|offset| offset + reduction <= input.len()));
    let panel_len = format.single_panel_len(reduction);
    let packed_len = format.len(reduction, columns);
    let panel_bytes = panel_len * std::mem::size_of::<f32>();
    unsafe {
        let mut packed = Blob::new_for_size_and_align(
            packed_len * std::mem::size_of::<f32>(),
            format.alignment_bytes,
        );
        if cfg!(debug_assertions) {
            packed.as_bytes_mut().fill(0);
        }
        let output = packed.as_mut_ptr().cast::<f32>();
        for panel in 0..columns.div_ceil(format.r) {
            let first = panel * format.r;
            let width = (columns - first).min(format.r);
            let panel_output = output.add(panel * panel_len);
            let panel_offsets = &offsets[first..first + width];
            // The caller only takes this path where a full panel is twelve columns wide and the
            // vectorized packer is available; the short arm is the last panel of a tile.
            if panel_offsets.len() == PACK_PANEL_COLUMNS {
                pack_panel_12_transposed_avx2(
                    input.as_ptr(),
                    panel_output,
                    panel_offsets,
                    input_channels,
                    kernel_length,
                );
            } else {
                for k in 0..reduction {
                    let row = panel_output.add(k * format.r);
                    for (column, &offset) in panel_offsets.iter().enumerate() {
                        row.add(column).write(*input.get_unchecked(offset + k));
                    }
                }
            }
        }
        Ok(Box::new(EagerPackedInput {
            fact: PackedExoticFact {
                format: Box::new(format.clone()),
                mn: columns.to_dim(),
                k: reduction,
            },
            packed: packed.into(),
            panel_bytes,
            mn: columns,
        }))
    }
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn pack_panel_12_transposed_avx2(
    input: *const f32, output: *mut f32, offsets: &[usize], input_channels: usize,
    kernel_length: usize,
) {
    use std::arch::x86_64::*;

    #[inline(always)]
    unsafe fn transpose8(rows: [__m256; 8]) -> [__m256; 8] {
        unsafe {
            let t0 = _mm256_unpacklo_ps(rows[0], rows[1]);
            let t1 = _mm256_unpackhi_ps(rows[0], rows[1]);
            let t2 = _mm256_unpacklo_ps(rows[2], rows[3]);
            let t3 = _mm256_unpackhi_ps(rows[2], rows[3]);
            let t4 = _mm256_unpacklo_ps(rows[4], rows[5]);
            let t5 = _mm256_unpackhi_ps(rows[4], rows[5]);
            let t6 = _mm256_unpacklo_ps(rows[6], rows[7]);
            let t7 = _mm256_unpackhi_ps(rows[6], rows[7]);
            let s0 = _mm256_shuffle_ps::<0x44>(t0, t2);
            let s1 = _mm256_shuffle_ps::<0xee>(t0, t2);
            let s2 = _mm256_shuffle_ps::<0x44>(t1, t3);
            let s3 = _mm256_shuffle_ps::<0xee>(t1, t3);
            let s4 = _mm256_shuffle_ps::<0x44>(t4, t6);
            let s5 = _mm256_shuffle_ps::<0xee>(t4, t6);
            let s6 = _mm256_shuffle_ps::<0x44>(t5, t7);
            let s7 = _mm256_shuffle_ps::<0xee>(t5, t7);
            [
                _mm256_permute2f128_ps::<0x20>(s0, s4),
                _mm256_permute2f128_ps::<0x20>(s1, s5),
                _mm256_permute2f128_ps::<0x20>(s2, s6),
                _mm256_permute2f128_ps::<0x20>(s3, s7),
                _mm256_permute2f128_ps::<0x31>(s0, s4),
                _mm256_permute2f128_ps::<0x31>(s1, s5),
                _mm256_permute2f128_ps::<0x31>(s2, s6),
                _mm256_permute2f128_ps::<0x31>(s3, s7),
            ]
        }
    }

    #[inline(always)]
    unsafe fn transpose4(rows: [__m128; 4]) -> [__m128; 4] {
        unsafe {
            let t0 = _mm_unpacklo_ps(rows[0], rows[1]);
            let t1 = _mm_unpacklo_ps(rows[2], rows[3]);
            let t2 = _mm_unpackhi_ps(rows[0], rows[1]);
            let t3 = _mm_unpackhi_ps(rows[2], rows[3]);
            [
                _mm_movelh_ps(t0, t1),
                _mm_movehl_ps(t1, t0),
                _mm_movelh_ps(t2, t3),
                _mm_movehl_ps(t3, t2),
            ]
        }
    }

    unsafe {
        debug_assert_eq!(offsets.len(), 12);
        for position in 0..kernel_length {
            let position_offset = position * input_channels;
            for channel in (0..input_channels).step_by(8) {
                let mut first = [_mm256_setzero_ps(); 8];
                for column in 0..8 {
                    first[column] =
                        _mm256_loadu_ps(input.add(offsets[column] + position_offset + channel));
                }
                let first = transpose8(first);
                for (lane, vector) in first.iter().enumerate() {
                    _mm256_storeu_ps(output.add((position_offset + channel + lane) * 12), *vector);
                }

                let mut last_low = [_mm_setzero_ps(); 4];
                let mut last_high = [_mm_setzero_ps(); 4];
                for column in 0..4 {
                    let source = input.add(offsets[column + 8] + position_offset + channel);
                    last_low[column] = _mm_loadu_ps(source);
                    last_high[column] = _mm_loadu_ps(source.add(4));
                }
                let last_low = transpose4(last_low);
                let last_high = transpose4(last_high);
                for lane in 0..4 {
                    _mm_storeu_ps(
                        output.add((position_offset + channel + lane) * 12 + 8),
                        last_low[lane],
                    );
                    _mm_storeu_ps(
                        output.add((position_offset + channel + lane + 4) * 12 + 8),
                        last_high[lane],
                    );
                }
            }
        }
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
        let mut op = DirectFusedConvMax1D::new(dimensions, kernel_tensor, bias_tensor, false)?;
        op.tile_batches = 2;
        op.tile_columns = 2 * dimensions.output_length;
        Arc::make_mut(&mut op.input_format).tile_columns = op.tile_columns;
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

    /// The eager packer and the reordered kernel layout have to be chosen together: reordering the
    /// kernel while packing the model layout, or the reverse, produces plausible wrong scores
    /// rather than an error. This runs one input through both packing paths on this machine.
    ///
    /// They agree closely rather than exactly: the eager path walks the reduction in
    /// [position, channel] order and the portable one in [channel, position] order, and float
    /// addition is not associative.
    #[test]
    fn both_packing_paths_agree() -> TractResult<()> {
        let dimensions = ConvDimensions {
            // 3 * 38 = 114 columns over 48-column tiles leaves a final tile of 18, which is one
            // full twelve-column panel and a partial one. Both packers have a separate path for
            // the short panel, so a shape that divides evenly would skip half of what this checks.
            batch: 3,
            input_channels: 64,
            input_length: 42,
            output_channels: 32,
            kernel_length: 5,
            output_length: 38,
            channels_last: true,
        };
        let input = (0..dimensions.batch * dimensions.input_length * dimensions.input_channels)
            .map(|index| ((index * 37 % 101) as f32 - 50.0) * 0.02)
            .collect::<Vec<_>>();
        let kernel =
            (0..dimensions.output_channels * dimensions.input_channels * dimensions.kernel_length)
                .map(|index| ((index * 53 % 79) as f32 - 39.0) * 0.01)
                .collect::<Vec<_>>();
        let bias = (0..dimensions.output_channels)
            .map(|index| (index as f32 % 5.0 - 2.0) * 0.1)
            .collect::<Vec<_>>();
        let kernel_tensor = Arc::new(Tensor::from_shape(
            &[dimensions.output_channels, dimensions.input_channels, dimensions.kernel_length],
            &kernel,
        )?);
        let bias_tensor = Arc::new(Tensor::from_shape(&[dimensions.output_channels], &bias)?);
        let input_tensor = Tensor::from_shape(
            &[dimensions.batch, dimensions.input_length, dimensions.input_channels],
            &input,
        )?;

        let run = |force_portable: bool| -> TractResult<(Tensor, bool, usize)> {
            let op = DirectFusedConvMax1D::new(
                dimensions,
                kernel_tensor.clone(),
                bias_tensor.clone(),
                force_portable,
            )?;
            let eager_pack = op.eager_pack;
            let panel_columns = op.mmm.nr();
            ensure!(
                eager_pack
                    == (!force_portable
                        && has_transposing_packer()
                        && panel_columns == PACK_PANEL_COLUMNS),
                "the eager path did not engage as this machine's packer and kernel imply"
            );
            let mut outputs = op.eval(tvec!(input_tensor.clone().into_tvalue()))?;
            Ok((outputs.remove(0).into_tensor(), eager_pack, panel_columns))
        };
        let (natural, natural_eager, natural_panel_columns) = run(false)?;
        let (portable, portable_eager, _) = run(true)?;
        ensure!(!portable_eager);
        // AVX2-only machines generally select a six-column kernel. The twelve-column eager packer
        // is not a production path there, so leave its equivalence to a machine whose selected
        // kernel can actually consume it instead of failing a portable target's test suite.
        if !natural_eager {
            ensure!(
                !has_transposing_packer() || natural_panel_columns != PACK_PANEL_COLUMNS,
                "the eligible eager packing path was not selected"
            );
            return Ok(());
        }
        let natural = natural.to_plain_array_view::<f32>()?;
        let portable = portable.to_plain_array_view::<f32>()?;
        for (left, right) in natural.iter().zip(portable.iter()) {
            ensure!((left - right).abs() <= 1e-5, "{left} != {right}");
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

    #[cfg(target_arch = "x86_64")]
    #[test]
    fn reordered_gelu_max_handles_negative_channels_and_batch_boundaries() -> TractResult<()> {
        let output_length = 3;
        let output_channels = 4;
        let column_start = 1;
        let tile_columns = 4;
        let mut tile = vec![
            -3.0, -0.5, 1.0, -1.0, -1.0, -2.0, 2.0, -0.1, -4.0, 3.0, -0.8, -2.0, -2.0, 1.0, -1.2,
            -3.0,
        ];
        let mut reference_tile = tile.clone();
        (tract_linalg::ops().gelu_f32)().run(&mut reference_tile)?;
        let mut reference = vec![f32::NEG_INFINITY; 2 * output_channels];
        for column in 0..tile_columns {
            let batch = (column_start + column) / output_length;
            update_maxima(
                &mut reference[batch * output_channels..(batch + 1) * output_channels],
                &reference_tile[column * output_channels..(column + 1) * output_channels],
            );
        }

        let mut actual = vec![f32::NEG_INFINITY; 2 * output_channels];
        reduce_x86_64_gelu_max(
            &mut actual,
            &mut vec![f32::NEG_INFINITY; output_channels],
            &mut Vec::new(),
            &mut Vec::new(),
            &mut tile,
            column_start,
            tile_columns,
            output_length,
            output_channels,
        )?;
        assert_eq!(actual, reference);
        Ok(())
    }

    #[test]
    fn direct_operator_matches_reference_for_both_layouts() -> TractResult<()> {
        direct_matches_reference(false)?;
        direct_matches_reference(true)
    }
}
