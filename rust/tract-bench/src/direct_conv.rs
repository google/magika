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

//! Benchmark-only direct Conv1D + bias + GELU operator for Magika's fixed shape.

use std::alloc::Layout;
use std::fmt::{Debug, Display};
use std::hash::{Hash, Hasher};
use std::sync::Arc;

use tract_core::internal::*;
use tract_linalg::BinOp;
use tract_linalg::WeightType;
use tract_linalg::mmm::{AsInputValue, FusedSpec, MMMInputFormat, MMMInputValue, MatMatMul};
use tract_linalg::pack::{PackedFormat, PackingWriter};

const INPUT_CHANNELS: usize = 256;
const INPUT_LENGTH: usize = 512;
const OUTPUT_CHANNELS: usize = 512;
const KERNEL_LENGTH: usize = 5;
const OUTPUT_LENGTH: usize = INPUT_LENGTH - KERNEL_LENGTH + 1;
const REDUCTION: usize = INPUT_CHANNELS * KERNEL_LENGTH;

const CONV_NODE: &str = "jax2tf_get_logits__pjit_get_logits__MagikaV2_Conv_0_Conv2D_conv";
const GELU_NODE: &str =
    "jax2tf_get_logits__pjit_get_logits__MagikaV2_ApplyActivation_1_Mul_5_fused_gelu";

/// Replace Magika's exact Conv1D -> transpose -> GELU chain with one direct operator.
pub(crate) fn fuse_magika_conv(model: &mut TypedModel, batch: usize) -> TractResult<()> {
    let conv = model.node_by_name(CONV_NODE)?;
    ensure!(conv.inputs.len() == 3, "unexpected Magika Conv1D input count");
    let input = conv.inputs[0];
    let input_shape = model
        .outlet_fact(input)?
        .shape
        .as_concrete()
        .context("direct Conv1D requires a concrete input shape")?;
    let channels_last = match input_shape {
        [n, INPUT_CHANNELS, INPUT_LENGTH] if *n == batch => false,
        [n, INPUT_LENGTH, INPUT_CHANNELS] if *n == batch => true,
        shape => bail!("unexpected direct Conv1D input shape {shape:?}"),
    };
    let kernel = model
        .outlet_fact(conv.inputs[1])?
        .konst
        .clone()
        .context("Magika Conv1D kernel is not constant")?;
    let bias = model
        .outlet_fact(conv.inputs[2])?
        .konst
        .clone()
        .context("Magika Conv1D bias is not constant")?;

    let gelu = model.node_by_name(GELU_NODE)?;
    let gelu_source = model.node(gelu.inputs[0].node);
    ensure!(
        gelu_source.id == conv.id || (gelu_source.inputs.as_slice() == [OutletId::new(conv.id, 0)]),
        "unexpected node {} between Magika Conv1D and GELU",
        gelu_source.name
    );

    let op = DirectFusedConv1D::new(batch, channels_last, kernel, bias)?;
    let mut patch = TypedModelPatch::default();
    let input = patch.tap_model(model, input)?;
    let output = patch.wire_node("magika.direct_fused_conv1d", op, &[input])?[0];
    patch.shunt_outside(model, OutletId::new(gelu.id, 0), output)?;
    patch.apply(model)
}

#[derive(Clone, Debug)]
struct DirectFusedConv1D {
    batch: usize,
    channels_last: bool,
    mmm: Box<dyn MatMatMul>,
    packing: usize,
    packed_kernel: Box<dyn MMMInputValue>,
    bias: Arc<Tensor>,
    input_format: Arc<DirectConvInputFormat>,
}

impl PartialEq for DirectFusedConv1D {
    fn eq(&self, other: &Self) -> bool {
        self.batch == other.batch
            && self.channels_last == other.channels_last
            && self.mmm.name() == other.mmm.name()
            && self.packing == other.packing
            && Arc::ptr_eq(&self.bias, &other.bias)
            && self.input_format == other.input_format
    }
}

impl Eq for DirectFusedConv1D {}

impl DirectFusedConv1D {
    fn new(
        batch: usize, channels_last: bool, kernel: Arc<Tensor>, bias: Arc<Tensor>,
    ) -> TractResult<Self> {
        ensure!(kernel.datum_type() == DatumType::F32, "Conv1D kernel must be f32");
        ensure!(kernel.shape() == [OUTPUT_CHANNELS, INPUT_CHANNELS, KERNEL_LENGTH]);
        ensure!(bias.datum_type() == DatumType::F32, "Conv1D bias must be f32");
        ensure!(bias.shape() == [OUTPUT_CHANNELS]);

        let columns = batch * OUTPUT_LENGTH;
        let mmm = tract_linalg::ops()
            .mmm(DatumType::F32, Some(OUTPUT_CHANNELS), Some(REDUCTION), Some(columns))
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
            &kernel.as_ref().clone().into_shape(&[OUTPUT_CHANNELS, REDUCTION])?,
            1,
            0,
        )?;
        let input_packer = mmm.packings()[packing]
            .1
            .downcast_ref::<PackedFormat>()
            .context("direct Conv1D input packer is not a PackedFormat")?
            .clone();
        let column_offsets = (0..columns)
            .map(|column| {
                let batch = column / OUTPUT_LENGTH;
                let position = column % OUTPUT_LENGTH;
                if channels_last {
                    (batch * INPUT_LENGTH + position) * INPUT_CHANNELS
                } else {
                    batch * INPUT_CHANNELS * INPUT_LENGTH + position
                }
            })
            .collect();
        let reduction_offsets = (0..REDUCTION)
            .map(|k| {
                if channels_last {
                    let channel = k / KERNEL_LENGTH;
                    let kernel_position = k % KERNEL_LENGTH;
                    kernel_position * INPUT_CHANNELS + channel
                } else {
                    let channel = k / KERNEL_LENGTH;
                    let kernel_position = k % KERNEL_LENGTH;
                    channel * INPUT_LENGTH + kernel_position
                }
            })
            .collect();
        let input_format = Arc::new(DirectConvInputFormat {
            packer: input_packer,
            batch,
            channels_last,
            column_offsets,
            reduction_offsets,
        });
        Ok(Self { batch, channels_last, mmm, packing, packed_kernel, bias, input_format })
    }
}

impl Op for DirectFusedConv1D {
    fn name(&self) -> StaticName {
        "DirectFusedConv1D".into()
    }

    fn info(&self) -> TractResult<Vec<String>> {
        Ok(vec![format!(
            "batch={} input_layout={} m={} k={} n={} kernel={} eager_im2col=false bias=mmm gelu=in_place",
            self.batch,
            if self.channels_last { "NLC" } else { "NCL" },
            OUTPUT_CHANNELS,
            REDUCTION,
            self.batch * OUTPUT_LENGTH,
            self.mmm.name()
        )])
    }

    op_as_typed_op!();
}

impl EvalOp for DirectFusedConv1D {
    fn is_stateless(&self) -> bool {
        true
    }

    fn eval(&self, inputs: TVec<TValue>) -> TractResult<TVec<TValue>> {
        let input = args_1!(inputs);
        let expected = if self.channels_last {
            [self.batch, INPUT_LENGTH, INPUT_CHANNELS]
        } else {
            [self.batch, INPUT_CHANNELS, INPUT_LENGTH]
        };
        ensure!(
            input.shape() == expected,
            "unexpected direct Conv1D input shape {:?}",
            input.shape()
        );
        let direct_input = DirectConvInput { tensor: input, format: self.input_format.clone() };
        // The MMM Store covers every output element before GELU reads it.
        let mut output = unsafe {
            Tensor::uninitialized_dt(DatumType::F32, &[self.batch, OUTPUT_LENGTH, OUTPUT_CHANNELS])?
        };
        let output_spec = unsafe {
            self.mmm.c_from_data_and_strides(
                std::mem::size_of::<f32>(),
                1,
                OUTPUT_CHANNELS as isize,
            )
        };
        {
            let output_view = output.view();
            let store = unsafe { output_spec.wrap(&output_view) };
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
                self.mmm.run(OUTPUT_CHANNELS, self.batch * OUTPUT_LENGTH, &specs)?;
            }
        }
        (tract_linalg::ops().gelu_f32)().run(output.try_as_plain_mut()?.as_slice_mut::<f32>()?)?;
        Ok(tvec!(output.into_tvalue()))
    }
}

impl TypedOp for DirectFusedConv1D {
    fn output_facts(&self, inputs: &[&TypedFact]) -> TractResult<TVec<TypedFact>> {
        ensure!(inputs.len() == 1);
        ensure!(inputs[0].datum_type == DatumType::F32);
        let expected = if self.channels_last {
            [self.batch, INPUT_LENGTH, INPUT_CHANNELS]
        } else {
            [self.batch, INPUT_CHANNELS, INPUT_LENGTH]
        };
        ensure!(inputs[0].shape.as_concrete() == Some(&expected[..]));
        Ok(tvec!(DatumType::F32.fact([self.batch, OUTPUT_LENGTH, OUTPUT_CHANNELS])))
    }

    as_op!();
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
struct DirectConvInputFormat {
    packer: PackedFormat,
    batch: usize,
    channels_last: bool,
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
        tvec!((REDUCTION * self.batch * OUTPUT_LENGTH * std::mem::size_of::<f32>()).to_dim())
    }
}

impl MMMInputFormat for DirectConvInputFormat {
    fn prepare_tensor(&self, _: &Tensor, _: usize, _: usize) -> TractResult<Tensor> {
        bail!("DirectConvInputFormat cannot eagerly prepare a tensor")
    }

    fn prepare_one(&self, _: &Tensor, _: usize, _: usize) -> TractResult<Box<dyn MMMInputValue>> {
        bail!("DirectConvInputFormat is created by DirectFusedConv1D")
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
    tensor: TValue,
    format: Arc<DirectConvInputFormat>,
}

impl Display for DirectConvInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "DirectConvInput({:?})", self.tensor.shape())
    }
}

impl Hash for DirectConvInput {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.tensor.as_bytes().hash(state);
        self.format.hash(state);
    }
}

unsafe impl Send for DirectConvInput {}
unsafe impl Sync for DirectConvInput {}

impl DirectConvInput {
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
        let mut writer =
            self.format.packer.write_with_k_outer(buffer.cast::<f32>(), REDUCTION, end - start);
        self.write_columns(&mut writer, start, end);
        buffer
    }

    #[inline]
    fn write_columns(&self, writer: &mut impl PackingWriter<f32>, start: usize, end: usize) {
        let input = unsafe { self.tensor.as_slice_unchecked::<f32>() };
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
        Some(self.format.packer.single_panel_layout(REDUCTION, std::mem::size_of::<f32>()))
    }

    fn panel_bytes(&self, panel: usize, buffer: Option<*mut u8>) -> TractResult<*const u8> {
        Ok(self.write_panel(panel, buffer.context("direct Conv1D needs a scratch panel")?))
    }

    fn mn(&self) -> usize {
        self.format.batch * OUTPUT_LENGTH
    }

    fn k(&self) -> usize {
        REDUCTION
    }

    fn exotic_fact(&self) -> &dyn ExoticFact {
        &*self.format
    }

    fn extract_at_mn_f16(&self, _: usize, _: &mut [f16]) -> TractResult<()> {
        bail!("f16 extraction is unsupported")
    }

    fn extract_at_mn_f32(&self, column: usize, output: &mut [f32]) -> TractResult<()> {
        ensure!(column < self.mn());
        ensure!(output.len() == REDUCTION);
        let input = unsafe { self.tensor.as_slice_unchecked::<f32>() };
        for (k, value) in output.iter_mut().enumerate() {
            let offset = self.format.column_offsets[column] + self.format.reduction_offsets[k];
            *value = input[offset];
        }
        Ok(())
    }
}
