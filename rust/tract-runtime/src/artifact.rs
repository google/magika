// Copyright 2026 Google LLC
// SPDX-License-Identifier: Apache-2.0

//! Release-generated portable graph and shared little-endian tensor payload.
use std::collections::HashMap;
#[cfg(feature = "_model-release")]
use std::path::Path;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tract_core::internal::*;
use tract_core::ops::array::Gather;
use tract_core::ops::binary::TypedBinOp;
use tract_core::ops::cast::Cast;
use tract_core::ops::change_axes::AxisOp;
use tract_core::ops::cnn::{Conv, KernelFormat, PaddingSpec, PoolSpec};
use tract_core::ops::einsum::EinSum;
use tract_core::ops::element_wise::ElementWiseOp;
use tract_core::ops::math;
use tract_core::ops::nn::{DataFormat, GeluApproximate, Reduce, Reducer};

use crate::direct_conv::{ConvDimensions, DirectFusedConvMax1D};
use crate::layer_norm::FusedLayerNorm;

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq)]
struct Parameter {
    shape: Vec<usize>,
    offset: usize,
    bytes: usize,
}
#[derive(Serialize, Deserialize)]
enum Operator {
    Source,
    Cast,
    Const(usize),
    Gather(usize),
    Reshape(usize, Vec<usize>, Vec<usize>),
    Remove(usize),
    Add(usize),
    Conv {
        kernel: Vec<usize>,
        before: Vec<usize>,
        after: Vec<usize>,
        dilations: Option<Vec<usize>>,
        strides: Option<Vec<usize>>,
        input_channels: usize,
        output_channels: usize,
        group: usize,
    },
    Norm {
        axis: usize,
        epsilon: usize,
        scale: usize,
        bias: usize,
        shape: Vec<usize>,
    },
    FusedConv {
        dims: [usize; 6],
        channels_last: bool,
        kernel: usize,
        bias: usize,
    },
    Gelu(bool),
    Einsum(String),
    Binary(String),
    Unary(String),
    Reduce(Vec<usize>, String),
}
#[derive(Serialize, Deserialize)]
struct Node {
    id: usize,
    name: String,
    inputs: Vec<(usize, usize)>,
    shape: Vec<usize>,
    dt: String,
    op: Operator,
}
#[derive(Serialize, Deserialize)]
struct Graph {
    batch: usize,
    nodes: Vec<Node>,
    inputs: Vec<(usize, usize)>,
    outputs: Vec<(usize, usize)>,
}

#[derive(Serialize, Deserialize)]
struct Manifest {
    version: u32,
    tract: String,
    graphs: Vec<Graph>,
    cpu_graphs: Vec<Graph>,
    parameters: Vec<Parameter>,
    weights_len: usize,
}

pub(super) struct Bundle {
    manifest: Manifest,
    tensors: Vec<Option<Arc<Tensor>>>,
    bytes: &'static [u8],
}
impl Bundle {
    pub(super) fn embedded() -> TractResult<Self> {
        Self::read(
            include_bytes!("../models/model.graph.json"),
            include_bytes!("../models/model.weights"),
        )
    }
    fn read(graph: &[u8], bytes: &'static [u8]) -> TractResult<Self> {
        let manifest: Manifest = serde_json::from_slice(graph)?;
        ensure!(manifest.version == 2 && manifest.tract == "0.23.4", "incompatible model artifact");
        ensure!(manifest.weights_len == bytes.len(), "model weight length mismatch");
        ensure!(
            manifest.graphs.iter().map(|g| g.batch).collect::<Vec<_>>() == crate::BATCH_CLASSES,
            "model batch classes mismatch"
        );
        ensure!(
            manifest.cpu_graphs.iter().map(|g| g.batch).collect::<Vec<_>>() == crate::BATCH_CLASSES,
            "CPU model batch classes mismatch"
        );
        for p in &manifest.parameters {
            validate_parameter(p, bytes.len())?;
        }
        let tensors = vec![None; manifest.parameters.len()];
        Ok(Self { manifest, tensors, bytes })
    }
}
fn validate_parameter(p: &Parameter, bytes: usize) -> TractResult<()> {
    let len = p
        .shape
        .iter()
        .try_fold(1usize, |a, b| a.checked_mul(*b))
        .context("tensor shape overflow")?;
    ensure!(len.checked_mul(4) == Some(p.bytes), "tensor length mismatch");
    ensure!(p.offset.checked_add(p.bytes).is_some_and(|end| end <= bytes), "truncated tensor");
    Ok(())
}

#[cfg(feature = "_model-release")]
fn parameter(t: &Tensor, bytes: &mut Vec<u8>) -> TractResult<Parameter> {
    ensure!(t.datum_type() == DatumType::F32, "release artifact supports only F32 constants");
    let offset = bytes.len();
    for value in t.try_as_plain()?.as_slice::<f32>()? {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    Ok(Parameter { shape: t.shape().to_vec(), offset, bytes: bytes.len() - offset })
}
#[cfg(feature = "_model-release")]
fn intern_parameter(
    t: &Tensor, bytes: &mut Vec<u8>, parameters: &mut Vec<Parameter>,
) -> TractResult<usize> {
    let p = parameter(t, bytes)?;
    if let Some(i) = parameters.iter().position(|old| {
        old.shape == p.shape
            && bytes[old.offset..old.offset + old.bytes] == bytes[p.offset..p.offset + p.bytes]
    }) {
        bytes.truncate(p.offset);
        Ok(i)
    } else {
        parameters.push(p);
        Ok(parameters.len() - 1)
    }
}
fn tensor(p: &Parameter, bytes: &[u8]) -> TractResult<Arc<Tensor>> {
    let len = p
        .shape
        .iter()
        .try_fold(1usize, |a, b| a.checked_mul(*b))
        .context("tensor shape overflow")?;
    ensure!(len.checked_mul(4) == Some(p.bytes), "tensor length mismatch");
    let end = p.offset.checked_add(p.bytes).context("tensor offset overflow")?;
    let data = bytes.get(p.offset..end).context("truncated tensor")?;
    let values: Vec<f32> =
        data.chunks_exact(4).map(|b| f32::from_le_bytes(b.try_into().unwrap())).collect();
    Ok(Arc::new(Tensor::from_shape(&p.shape, &values)?))
}
fn shape(f: &TypedFact) -> TractResult<Vec<usize>> {
    f.shape.iter().map(|d| d.to_usize()).collect()
}
#[cfg(feature = "_model-release")]
fn outlets(os: &[OutletId]) -> Vec<(usize, usize)> {
    os.iter().map(|o| (o.node, o.slot)).collect()
}

#[cfg(feature = "_model-release")]
fn export_graph(
    model: &TypedModel, batch: usize, bytes: &mut Vec<u8>, parameters: &mut Vec<Parameter>,
) -> TractResult<Graph> {
    let mut nodes = Vec::new();
    for id in model.eval_order()? {
        let n = model.node(id);
        ensure!(n.outputs.len() == 1, "unsupported output arity");
        let op = if n.op.name() == "Source" {
            Operator::Source
        } else if n.op.name() == "Const" {
            Operator::Const(intern_parameter(
                n.outputs[0].fact.konst.as_ref().context("missing constant")?,
                bytes,
                parameters,
            )?)
        } else if let Some(c) = n.op_as::<Cast>() {
            ensure!(c.to == DatumType::I64);
            Operator::Cast
        } else if let Some(g) = n.op_as::<Gather>() {
            ensure!(g.output_type.is_none());
            Operator::Gather(g.axis)
        } else if let Some(a) = n.op_as::<AxisOp>() {
            match a {
                AxisOp::Reshape(axis, from, to) => Operator::Reshape(
                    *axis,
                    from.iter().map(|d| d.to_usize()).collect::<TractResult<_>>()?,
                    to.iter().map(|d| d.to_usize()).collect::<TractResult<_>>()?,
                ),
                AxisOp::Rm(axis) => Operator::Remove(*axis),
                AxisOp::Add(axis) => Operator::Add(*axis),
                _ => bail!("unsupported axis operation"),
            }
        } else if let Some(c) = n.op_as::<Conv>() {
            ensure!(
                c.kernel_fmt == KernelFormat::OIHW
                    && c.q_params.is_none()
                    && c.pool_spec.data_format == DataFormat::NHWC,
                "unsupported convolution"
            );
            let PaddingSpec::Explicit(before, after) = &c.pool_spec.padding else {
                bail!("unsupported convolution padding")
            };
            let p = &c.pool_spec;
            Operator::Conv {
                kernel: p.kernel_shape.to_vec(),
                before: before.to_vec(),
                after: after.to_vec(),
                dilations: p.dilations.as_ref().map(|v| v.to_vec()),
                strides: p.strides.as_ref().map(|v| v.to_vec()),
                input_channels: p.input_channels,
                output_channels: p.output_channels,
                group: c.group,
            }
        } else if let Some(l) = n.op_as::<FusedLayerNorm>() {
            Operator::Norm {
                axis: l.axis,
                epsilon: intern_parameter(&l.epsilon, bytes, parameters)?,
                scale: intern_parameter(&l.scale, bytes, parameters)?,
                bias: intern_parameter(&l.bias, bytes, parameters)?,
                shape: l.input_shape.to_vec(),
            }
        } else if let Some(c) = n.op_as::<DirectFusedConvMax1D>() {
            let d = c.dimensions;
            Operator::FusedConv {
                dims: [
                    d.batch,
                    d.input_channels,
                    d.input_length,
                    d.output_channels,
                    d.kernel_length,
                    d.output_length,
                ],
                channels_last: d.channels_last,
                kernel: intern_parameter(&c.kernel, bytes, parameters)?,
                bias: intern_parameter(&c.bias, bytes, parameters)?,
            }
        } else if let Some(e) = n.op_as::<EinSum>() {
            ensure!(e.operating_dt == DatumType::F32 && e.q_params.is_none());
            Operator::Einsum(e.axes.to_string())
        } else if let Some(b) = n.op_as::<TypedBinOp>() {
            ensure!(b.1.is_none());
            let name = n.op.name().to_string();
            ensure!(["Add", "Max", "Sub", "Mul"].contains(&name.as_str()));
            Operator::Binary(name)
        } else if let Some(e) = n.op_as::<ElementWiseOp>() {
            ensure!(e.1.is_none());
            let name = n.op.name().to_string();
            if let Some(g) = e.0.downcast_ref::<GeluApproximate>() {
                Operator::Gelu(g.fast_impl)
            } else {
                ensure!(["Exp", "Recip", "Square", "Rsqrt"].contains(&name.as_str()));
                Operator::Unary(name)
            }
        } else if let Some(r) = n.op_as::<Reduce>() {
            ensure!(matches!(r.reducer, Reducer::Max | Reducer::Sum | Reducer::MeanOfSquares));
            Operator::Reduce(r.axes.to_vec(), format!("{:?}", r.reducer))
        } else {
            bail!("unsupported operator {}", n.op.name())
        };
        nodes.push(Node {
            id,
            name: n.name.clone(),
            inputs: outlets(&n.inputs),
            shape: shape(&n.outputs[0].fact)?,
            dt: format!("{:?}", n.outputs[0].fact.datum_type),
            op,
        });
    }
    Ok(Graph {
        batch,
        nodes,
        inputs: outlets(model.input_outlets()?),
        outputs: outlets(model.output_outlets()?),
    })
}

/// Generates all batch graphs and their shared weights from the release NNEF model.
#[cfg(feature = "_model-release")]
pub fn export(source: &Path, destination: &Path) -> TractResult<()> {
    let source = tract_nnef::nnef().model_for_path(source)?;
    let mut bytes = Vec::new();
    let mut parameters = Vec::new();
    let mut graphs = Vec::new();
    let mut cpu_graphs = Vec::new();
    for batch in crate::BATCH_CLASSES {
        let mut model = crate::prepare_nnef_model(source.clone(), batch)?;
        graphs.push(export_graph(&model, batch, &mut bytes, &mut parameters)?);
        crate::prepare_cpu_graph(&mut model, batch)?;
        cpu_graphs.push(export_graph(&model, batch, &mut bytes, &mut parameters)?);
    }
    let manifest = Manifest {
        version: 2,
        tract: "0.23.4".into(),
        graphs,
        cpu_graphs,
        parameters,
        weights_len: bytes.len(),
    };
    std::fs::create_dir_all(destination)?;
    std::fs::write(destination.join("model.graph.json"), serde_json::to_vec(&manifest)?)?;
    std::fs::write(destination.join("model.weights"), bytes)?;
    Ok(())
}

impl Bundle {
    pub(super) fn model(&mut self, batch: usize, cpu: bool) -> TractResult<TypedModel> {
        let graphs = if cpu { &self.manifest.cpu_graphs } else { &self.manifest.graphs };
        let a = graphs.iter().find(|g| g.batch == batch).context("missing batch graph")?;
        let mut model = TypedModel::default();
        let mut mapped = HashMap::<usize, TVec<OutletId>>::new();
        let mut parameter = |index: usize| -> TractResult<Arc<Tensor>> {
            let slot = self.tensors.get_mut(index).context("invalid tensor reference")?;
            if slot.is_none() {
                *slot = Some(tensor(&self.manifest.parameters[index], self.bytes)?);
            }
            Ok(slot.as_ref().unwrap().clone())
        };
        for n in &a.nodes {
            let mut inputs = TVec::new();
            for (id, slot) in &n.inputs {
                inputs.push(
                    *mapped.get(id).and_then(|o| o.get(*slot)).context("invalid graph edge")?,
                );
            }
            let outputs = match &n.op {
                Operator::Source => {
                    ensure!(n.dt == "I32");
                    tvec!(model.add_source(&n.name, DatumType::I32.fact(n.shape.clone()))?)
                }
                Operator::Const(index) => tvec!(model.add_const(&n.name, parameter(*index)?)?),
                Operator::Norm { axis, epsilon, scale, bias, shape } => model.wire_node(
                    &n.name,
                    FusedLayerNorm::new(
                        *axis,
                        parameter(*epsilon)?,
                        parameter(*scale)?,
                        parameter(*bias)?,
                        shape,
                    )?,
                    &inputs,
                )?,
                Operator::FusedConv { dims: d, channels_last, kernel, bias } => {
                    let dimensions = ConvDimensions {
                        batch: d[0],
                        input_channels: d[1],
                        input_length: d[2],
                        output_channels: d[3],
                        kernel_length: d[4],
                        output_length: d[5],
                        channels_last: *channels_last,
                    };
                    // Select and pack for this host; never serialize CPU-specific kernel state.
                    model.wire_node(
                        &n.name,
                        DirectFusedConvMax1D::new(
                            dimensions,
                            parameter(*kernel)?,
                            parameter(*bias)?,
                            false,
                        )?,
                        &inputs,
                    )?
                }
                Operator::Cast => model.wire_node(&n.name, Cast { to: DatumType::I64 }, &inputs)?,
                Operator::Gather(axis) => {
                    model.wire_node(&n.name, Gather { axis: *axis, output_type: None }, &inputs)?
                }
                Operator::Reshape(axis, from, to) => model.wire_node(
                    &n.name,
                    AxisOp::Reshape(
                        *axis,
                        from.iter().map(|d| d.to_dim()).collect(),
                        to.iter().map(|d| d.to_dim()).collect(),
                    ),
                    &inputs,
                )?,
                Operator::Remove(axis) => model.wire_node(&n.name, AxisOp::Rm(*axis), &inputs)?,
                Operator::Add(axis) => model.wire_node(&n.name, AxisOp::Add(*axis), &inputs)?,
                Operator::Conv {
                    kernel,
                    before,
                    after,
                    dilations,
                    strides,
                    input_channels,
                    output_channels,
                    group,
                } => model.wire_node(
                    &n.name,
                    Conv {
                        pool_spec: PoolSpec {
                            data_format: DataFormat::NHWC,
                            kernel_shape: kernel.clone().into(),
                            padding: PaddingSpec::Explicit(
                                before.clone().into(),
                                after.clone().into(),
                            ),
                            dilations: dilations.clone().map(Into::into),
                            strides: strides.clone().map(Into::into),
                            input_channels: *input_channels,
                            output_channels: *output_channels,
                        },
                        kernel_fmt: KernelFormat::OIHW,
                        group: *group,
                        q_params: None,
                    },
                    &inputs,
                )?,
                Operator::Gelu(fast_impl) => model.wire_node(
                    &n.name,
                    ElementWiseOp(Box::new(GeluApproximate { fast_impl: *fast_impl }), None),
                    &inputs,
                )?,
                Operator::Einsum(axes) => {
                    model.wire_node(&n.name, EinSum::new(axes.parse()?, DatumType::F32), &inputs)?
                }
                Operator::Binary(name) => {
                    let op: Box<dyn tract_core::ops::binary::BinMiniOp> = match name.as_str() {
                        "Add" => Box::new(math::Add),
                        "Max" => Box::new(math::Max),
                        "Sub" => Box::new(math::Sub),
                        "Mul" => Box::new(math::Mul),
                        _ => bail!("unsupported binary"),
                    };
                    model.wire_node(&n.name, TypedBinOp(op, None), &inputs)?
                }
                Operator::Unary(name) => {
                    let op: Box<dyn tract_core::ops::element_wise::ElementWiseMiniOp> =
                        match name.as_str() {
                            "Exp" => Box::new(math::Exp {}),
                            "Recip" => Box::new(math::Recip {}),
                            "Square" => Box::new(math::Square {}),
                            "Rsqrt" => Box::new(math::Rsqrt {}),
                            _ => bail!("unsupported unary"),
                        };
                    model.wire_node(&n.name, ElementWiseOp(op, None), &inputs)?
                }
                Operator::Reduce(axes, reducer) => model.wire_node(
                    &n.name,
                    Reduce {
                        axes: axes.clone().into(),
                        reducer: match reducer.as_str() {
                            "Max" => Reducer::Max,
                            "Sum" => Reducer::Sum,
                            "MeanOfSquares" => Reducer::MeanOfSquares,
                            _ => bail!("unsupported reduction"),
                        },
                    },
                    &inputs,
                )?,
            };
            ensure!(outputs.len() == 1);
            let fact = model.outlet_fact(outputs[0])?;
            ensure!(
                shape(fact)? == n.shape && format!("{:?}", fact.datum_type) == n.dt,
                "artifact fact mismatch at {}",
                n.name
            );
            ensure!(mapped.insert(n.id, outputs).is_none(), "duplicate node id");
        }
        let map = |os: Vec<(usize, usize)>| -> TractResult<Vec<OutletId>> {
            os.into_iter()
                .map(|(id, slot)| {
                    mapped
                        .get(&id)
                        .and_then(|o| o.get(slot))
                        .copied()
                        .context("invalid boundary outlet")
                })
                .collect()
        };
        model.set_input_outlets(&map(a.inputs.clone())?)?;
        model.select_output_outlets(&map(a.outputs.clone())?)?;
        ensure!(
            model.input_outlets()?.len() == 1 && model.output_outlets()?.len() == 1,
            "invalid model boundaries"
        );
        ensure!(
            shape(model.input_fact(0)?)? == [batch, crate::FEATURE_SIZE]
                && model.input_fact(0)?.datum_type == DatumType::I32,
            "invalid model input"
        );
        ensure!(
            shape(model.output_fact(0)?)? == [batch, crate::NUM_LABELS]
                && model.output_fact(0)?.datum_type == DatumType::F32,
            "invalid model output"
        );
        Ok(model)
    }
}

#[cfg(test)]
mod tests {
    use tract_core::runtime::{DefaultRuntime, RunOptions};
    use tract_core::tract_linalg::multithread::Executor;

    use super::*;
    use crate::{BATCH_CLASSES, FEATURE_SIZE, NUM_LABELS};

    #[test]
    fn rejects_incompatible_and_truncated_artifacts() -> TractResult<()> {
        let graph = include_bytes!("../models/model.graph.json");
        let weights = include_bytes!("../models/model.weights");
        ensure!(Bundle::read(graph, &weights[..weights.len() - 1]).is_err());
        let mut manifest: Manifest = serde_json::from_slice(graph)?;
        manifest.version += 1;
        ensure!(Bundle::read(&serde_json::to_vec(&manifest)?, weights).is_err());
        manifest.version = 2;
        manifest.parameters[0].offset = usize::MAX;
        ensure!(Bundle::read(&serde_json::to_vec(&manifest)?, weights).is_err());
        Ok(())
    }

    #[test]
    fn shares_weights_and_rejects_invalid_graph_edges() -> TractResult<()> {
        let mut bundle = Bundle::embedded()?;
        let first = bundle.model(8, true)?;
        let second = bundle.model(16, true)?;
        let constants = |model: &TypedModel| {
            model.nodes.iter().filter_map(|n| n.outputs[0].fact.konst.clone()).collect::<Vec<_>>()
        };
        let left = constants(&first);
        let right = constants(&second);
        ensure!(
            left.iter().any(|a| right.iter().any(|b| Arc::ptr_eq(a, b))),
            "weights were copied per batch"
        );
        let g = bundle.manifest.cpu_graphs.iter_mut().find(|g| g.batch == 8).unwrap();
        g.nodes[1].inputs[0].0 = usize::MAX;
        ensure!(bundle.model(8, true).is_err());
        Ok(())
    }

    #[test]
    fn cpu_artifact_matches_source_for_every_batch() -> TractResult<()> {
        let mut bundle = Bundle::embedded()?;
        for batch in BATCH_CLASSES {
            let mut source = crate::load_nnef_model(batch)?;
            crate::prepare_cpu_graph(&mut source, batch)?;
            let loaded = bundle.model(batch, true)?;
            let prepare = |model: TypedModel| -> TractResult<_> {
                let options =
                    RunOptions { executor: Some(Executor::SingleThread), ..RunOptions::default() };
                DefaultRuntime.prepare_with_options(model, &options)
            };
            let a = prepare(source)?;
            let b = prepare(loaded)?;
            let mut a = a.spawn()?;
            let mut b = b.spawn()?;
            // Full and padded partial input, with varied tokens across every row.
            for real in [1, batch] {
                let mut input = vec![256; batch * FEATURE_SIZE];
                for (i, value) in input[..real * FEATURE_SIZE].iter_mut().enumerate() {
                    *value = ((i * 17 + i / 97) % 257) as i32;
                }
                let x = crate::run_plan(a.as_mut(), &input, batch)?;
                let y = crate::run_plan(b.as_mut(), &input, batch)?;
                ensure!(x.len() == batch * NUM_LABELS);
                ensure!(
                    x.iter().zip(&y).all(|(x, y)| x.to_bits() == y.to_bits()),
                    "CPU artifact score mismatch for batch {batch}"
                );
            }
        }
        Ok(())
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn metal_artifact_matches_source_for_every_batch() -> TractResult<()> {
        use tract_core::transform::ModelTransform as _;
        let mut bundle = Bundle::embedded()?;
        for batch in BATCH_CLASSES {
            let prepare = |mut model: TypedModel| -> TractResult<_> {
                crate::prepare_gpu_graph(&mut model, batch)?;
                tract_metal::MetalTransform { gemm_impl: None }.transform(&mut model)?;
                let options = RunOptions { skip_order_opt_ram: true, ..RunOptions::default() };
                let plan = TypedSimplePlan::build(model.into_optimized()?, &options)?;
                crate::with_memory_arena(plan)
            };
            let a = Arc::new(prepare(crate::load_nnef_model(batch)?)?);
            let b = Arc::new(prepare(bundle.model(batch, false)?)?);
            let mut a = TypedSimpleState::new(&a)?;
            let mut b = TypedSimpleState::new(&b)?;
            for real in [1, batch] {
                let mut input = vec![256; batch * FEATURE_SIZE];
                for (i, value) in input[..real * FEATURE_SIZE].iter_mut().enumerate() {
                    *value = ((i * 17 + i / 97) % 257) as i32;
                }
                let tensor = Tensor::from_shape(&[batch, FEATURE_SIZE], &input)?.into_tvalue();
                let x = crate::decode_output(a.run(tvec!(tensor.clone()))?, batch)?;
                let y = crate::decode_output(b.run(tvec!(tensor))?, batch)?;
                ensure!(
                    x.iter().zip(&y).all(|(x, y)| x.to_bits() == y.to_bits()),
                    "Metal artifact score mismatch for batch {batch}"
                );
            }
        }
        Ok(())
    }
}
