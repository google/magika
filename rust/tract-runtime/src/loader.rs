// Copyright 2026 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Loads the embedded model without parsing NNEF.
//!
//! Parsing the NNEF archive takes about 14 ms, half to decompress it and half to parse it, and
//! happens before the runtime can identify anything. The embedded graph is instead the typed model
//! that parsing produces for each batch class, stored as JSON with its weights stored raw, so
//! loading it only wires nodes, in about 3 ms. It is derived from the NNEF archive: `rust/sync.sh`
//! writes both with `tract-bench`'s `convert-model`, and `cargo test` checks that they agree.

use std::sync::Arc;

use anyhow::{Context as _, Result, bail, ensure};
use serde::{Deserialize, Serialize};
use tract_core::internal::*;
use tract_core::ops::array::Gather;
use tract_core::ops::binary::{BinMiniOp, TypedBinOp};
use tract_core::ops::cast::Cast;
use tract_core::ops::change_axes::AxisOp;
use tract_core::ops::cnn::{Conv, KernelFormat, PaddingSpec, PoolSpec};
use tract_core::ops::einsum::EinSum;
use tract_core::ops::element_wise::{ElementWiseMiniOp, ElementWiseOp};
use tract_core::ops::math;
use tract_core::ops::nn::{DataFormat, GeluApproximate, Reduce, Reducer};

const GRAPH: &[u8] = include_bytes!("../models/model.graph.json");
const WEIGHTS: &[u8] = include_bytes!("../models/model.weights");

/// Returns the embedded model bound to `batch`.
pub(crate) fn load(batch: usize) -> Result<TypedModel> {
    let manifest: Manifest = serde_json::from_slice(GRAPH).context("reading the model graph")?;
    let graph = manifest.graphs.iter().find(|x| x.batch == batch).context("no graph for batch")?;
    graph.load(&manifest.weights, WEIGHTS).with_context(|| format!("loading batch {batch}"))
}

#[derive(Serialize, Deserialize)]
struct Manifest {
    graphs: Vec<Graph>,
    /// Where each constant lies in the weights, shared by every graph.
    weights: Vec<Weight>,
}

#[derive(Serialize, Deserialize, PartialEq)]
struct Weight {
    shape: Vec<usize>,
    /// Offset of the little-endian f32 values in the weights.
    offset: usize,
}

#[derive(Serialize, Deserialize)]
struct Graph {
    batch: usize,
    /// In evaluation order, so every input is loaded before the node reading it.
    nodes: Vec<Node>,
    input: usize,
    output: usize,
}

#[derive(Serialize, Deserialize)]
struct Node {
    name: String,
    /// Indices of the nodes whose single output this node reads, in order.
    inputs: Vec<usize>,
    op: Op,
}

/// The operators the model uses before Magika fuses its own, described by their parameters.
#[derive(Serialize, Deserialize)]
enum Op {
    Source { shape: Vec<usize> },
    Const(usize),
    CastToI64,
    Gather { axis: usize },
    Reshape { axis: usize, from: Vec<usize>, to: Vec<usize> },
    RemoveAxis(usize),
    AddAxis(usize),
    Conv(ConvSpec),
    EinSum(String),
    Binary(String),
    Unary(String),
    Gelu { fast: bool },
    Reduce { axes: Vec<usize>, reducer: String },
}

#[derive(Serialize, Deserialize)]
struct ConvSpec {
    kernel: Vec<usize>,
    before: Vec<usize>,
    after: Vec<usize>,
    dilations: Option<Vec<usize>>,
    strides: Option<Vec<usize>>,
    input_channels: usize,
    output_channels: usize,
    group: usize,
}

impl Graph {
    fn load(&self, weights: &[Weight], bytes: &[u8]) -> Result<TypedModel> {
        let mut model = TypedModel::default();
        let mut outlets = Vec::<OutletId>::with_capacity(self.nodes.len());
        for node in &self.nodes {
            let inputs = node
                .inputs
                .iter()
                .map(|x| outlets.get(*x).copied().context("input not loaded yet"))
                .collect::<Result<TVec<_>>>()?;
            let name = &node.name;
            let outputs = match &node.op {
                Op::Source { shape } => tvec!(model.add_source(name, i32::fact(shape))?),
                Op::Const(index) => {
                    let weight = weights.get(*index).context("no such weight")?;
                    tvec!(model.add_const(name, weight.load(bytes)?)?)
                }
                op => model.wire_node(name, op.build()?, &inputs)?,
            };
            ensure!(outputs.len() == 1, "{name} has {} outputs", outputs.len());
            outlets.push(outputs[0]);
        }
        let outlet = |index: usize| outlets.get(index).copied().context("no such node");
        model.set_input_outlets(&[outlet(self.input)?])?;
        model.select_output_outlets(&[outlet(self.output)?])?;
        Ok(model)
    }
}

impl Weight {
    fn load(&self, bytes: &[u8]) -> Result<Arc<Tensor>> {
        let len = self.shape.iter().try_fold(4usize, |x, y| x.checked_mul(*y));
        let end = len.and_then(|x| x.checked_add(self.offset)).context("weight too large")?;
        let data = bytes.get(self.offset..end).context("weight out of bounds")?;
        let values: Vec<f32> = data.as_chunks().0.iter().map(|x| f32::from_le_bytes(*x)).collect();
        Ok(Tensor::from_shape(&self.shape, &values)?.into_arc_tensor())
    }
}

impl Op {
    fn build(&self) -> Result<Box<dyn TypedOp>> {
        let dims = |xs: &[usize]| xs.iter().map(|x| x.to_dim()).collect();
        Ok(match self {
            Op::Source { .. } | Op::Const(_) => unreachable!("sources and constants are not wired"),
            Op::CastToI64 => Box::new(Cast { to: DatumType::I64 }),
            Op::Gather { axis } => Box::new(Gather { axis: *axis, output_type: None }),
            Op::Reshape { axis, from, to } => {
                Box::new(AxisOp::Reshape(*axis, dims(from), dims(to)))
            }
            Op::RemoveAxis(axis) => Box::new(AxisOp::Rm(*axis)),
            Op::AddAxis(axis) => Box::new(AxisOp::Add(*axis)),
            Op::Conv(spec) => Box::new(Conv {
                pool_spec: PoolSpec {
                    data_format: DataFormat::NHWC,
                    kernel_shape: spec.kernel.clone().into(),
                    padding: PaddingSpec::Explicit(
                        spec.before.clone().into(),
                        spec.after.clone().into(),
                    ),
                    dilations: spec.dilations.clone().map(Into::into),
                    strides: spec.strides.clone().map(Into::into),
                    input_channels: spec.input_channels,
                    output_channels: spec.output_channels,
                },
                kernel_fmt: KernelFormat::OIHW,
                group: spec.group,
                q_params: None,
            }),
            Op::EinSum(axes) => Box::new(EinSum::new(axes.parse()?, DatumType::F32)),
            Op::Binary(name) => {
                let op: Box<dyn BinMiniOp> = match name.as_str() {
                    "Add" => Box::new(math::Add),
                    "Sub" => Box::new(math::Sub),
                    "Mul" => Box::new(math::Mul),
                    "Max" => Box::new(math::Max),
                    _ => bail!("unsupported binary operator {name}"),
                };
                Box::new(TypedBinOp(op, None))
            }
            Op::Unary(name) => {
                let op: Box<dyn ElementWiseMiniOp> = match name.as_str() {
                    "Exp" => Box::new(math::Exp {}),
                    "Recip" => Box::new(math::Recip {}),
                    "Square" => Box::new(math::Square {}),
                    "Rsqrt" => Box::new(math::Rsqrt {}),
                    _ => bail!("unsupported unary operator {name}"),
                };
                Box::new(ElementWiseOp(op, None))
            }
            Op::Gelu { fast } => {
                Box::new(ElementWiseOp(Box::new(GeluApproximate { fast_impl: *fast }), None))
            }
            Op::Reduce { axes, reducer } => Box::new(Reduce {
                axes: axes.clone().into(),
                reducer: match reducer.as_str() {
                    "Max" => Reducer::Max,
                    "Sum" => Reducer::Sum,
                    "MeanOfSquares" => Reducer::MeanOfSquares,
                    _ => bail!("unsupported reducer {reducer}"),
                },
            }),
        })
    }
}

/// Exports the graph and weights that [`load`] embeds from the release NNEF archive.
///
/// `tract-bench`'s `convert-model` writes them next to the archive.
#[cfg(feature = "_export")]
pub(crate) fn export(nnef: &[u8]) -> Result<(Vec<u8>, Vec<u8>)> {
    let models = crate::BATCH_CLASSES.iter().map(|&x| Ok((x, crate::parse_nnef(nnef, x)?)));
    let (manifest, weights) = export::graph(&models.collect::<Result<Vec<_>>>()?)?;
    Ok((serde_json::to_vec(&manifest)?, weights))
}

#[cfg(any(test, feature = "_export"))]
mod export {
    use std::collections::HashMap;

    use super::*;

    /// Describes `models` as the embedded graph and its weights, sharing identical constants.
    pub(super) fn graph(models: &[(usize, TypedModel)]) -> Result<(Manifest, Vec<u8>)> {
        let mut manifest = Manifest { graphs: Vec::new(), weights: Vec::new() };
        let mut bytes = Vec::new();
        for (batch, model) in models {
            let order = model.eval_order()?;
            let index: HashMap<usize, usize> =
                order.iter().enumerate().map(|(i, id)| (*id, i)).collect();
            let mut nodes = Vec::new();
            for id in &order {
                let node = model.node(*id);
                ensure!(node.outputs.len() == 1, "{} has several outputs", node.name);
                let inputs = node
                    .inputs
                    .iter()
                    .map(|x| {
                        ensure!(x.slot == 0, "{} reads a secondary output", node.name);
                        Ok(index[&x.node])
                    })
                    .collect::<Result<_>>()?;
                let op = Op::export(node, &mut manifest.weights, &mut bytes)?;
                nodes.push(Node { name: node.name.clone(), inputs, op });
            }
            let [input] = model.input_outlets()?[..] else { bail!("several inputs") };
            let [output] = model.output_outlets()?[..] else { bail!("several outputs") };
            let (input, output) = (index[&input.node], index[&output.node]);
            manifest.graphs.push(Graph { batch: *batch, nodes, input, output });
        }
        Ok((manifest, bytes))
    }

    impl Op {
        fn export(node: &TypedNode, weights: &mut Vec<Weight>, bytes: &mut Vec<u8>) -> Result<Op> {
            let usizes = |xs: &[TDim]| xs.iter().map(|x| x.to_usize()).collect::<TractResult<_>>();
            let op = &node.op;
            Ok(if op.name() == "Source" {
                let shape = node.outputs[0].fact.shape.iter().map(|x| x.to_usize());
                Op::Source { shape: shape.collect::<TractResult<_>>()? }
            } else if op.name() == "Const" {
                let tensor = node.outputs[0].fact.konst.as_ref().context("constant value")?;
                Op::Const(Weight::intern(tensor, weights, bytes)?)
            } else if let Some(op) = op.downcast_ref::<Cast>() {
                ensure!(op.to == DatumType::I64, "unsupported cast to {:?}", op.to);
                Op::CastToI64
            } else if let Some(op) = op.downcast_ref::<Gather>() {
                ensure!(op.output_type.is_none(), "unsupported typed gather");
                Op::Gather { axis: op.axis }
            } else if let Some(op) = op.downcast_ref::<AxisOp>() {
                match op {
                    AxisOp::Reshape(axis, from, to) => {
                        Op::Reshape { axis: *axis, from: usizes(from)?, to: usizes(to)? }
                    }
                    AxisOp::Rm(axis) => Op::RemoveAxis(*axis),
                    AxisOp::Add(axis) => Op::AddAxis(*axis),
                    _ => bail!("unsupported axis operator {op:?}"),
                }
            } else if let Some(op) = op.downcast_ref::<Conv>() {
                let spec = &op.pool_spec;
                ensure!(
                    op.kernel_fmt == KernelFormat::OIHW
                        && op.q_params.is_none()
                        && spec.data_format == DataFormat::NHWC,
                    "unsupported convolution"
                );
                let PaddingSpec::Explicit(before, after) = &spec.padding else {
                    bail!("unsupported convolution padding")
                };
                Op::Conv(ConvSpec {
                    kernel: spec.kernel_shape.to_vec(),
                    before: before.to_vec(),
                    after: after.to_vec(),
                    dilations: spec.dilations.as_ref().map(|x| x.to_vec()),
                    strides: spec.strides.as_ref().map(|x| x.to_vec()),
                    input_channels: spec.input_channels,
                    output_channels: spec.output_channels,
                    group: op.group,
                })
            } else if let Some(op) = op.downcast_ref::<EinSum>() {
                ensure!(op.operating_dt == DatumType::F32 && op.q_params.is_none());
                Op::EinSum(op.axes.to_string())
            } else if let Some(op) = op.downcast_ref::<TypedBinOp>() {
                ensure!(op.1.is_none(), "unsupported binary output type");
                Op::Binary(op.0.name().to_string())
            } else if let Some(op) = op.downcast_ref::<ElementWiseOp>() {
                ensure!(op.1.is_none(), "unsupported unary output type");
                match op.0.downcast_ref::<GeluApproximate>() {
                    Some(gelu) => Op::Gelu { fast: gelu.fast_impl },
                    None => Op::Unary(op.0.name().to_string()),
                }
            } else if let Some(op) = op.downcast_ref::<Reduce>() {
                Op::Reduce { axes: op.axes.to_vec(), reducer: format!("{:?}", op.reducer) }
            } else {
                bail!("unsupported operator {} in {}", op.name(), node.name)
            })
        }
    }

    impl Weight {
        /// Returns the index of `tensor` in `weights`, appending it if no weight is identical.
        fn intern(
            tensor: &Tensor, weights: &mut Vec<Weight>, bytes: &mut Vec<u8>,
        ) -> Result<usize> {
            ensure!(tensor.datum_type() == DatumType::F32, "unsupported constant type");
            let mut data = Vec::new();
            for x in tensor.try_as_plain()?.as_slice::<f32>()? {
                data.extend_from_slice(&x.to_le_bytes());
            }
            let shape = tensor.shape().to_vec();
            let same = |x: &Weight| x.shape == shape && bytes[x.offset..][..data.len()] == data;
            if let Some(index) = weights.iter().position(same) {
                return Ok(index);
            }
            weights.push(Weight { shape, offset: bytes.len() });
            bytes.extend_from_slice(&data);
            Ok(weights.len() - 1)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn nnef_models() -> Result<Vec<(usize, TypedModel)>> {
        crate::BATCH_CLASSES.iter().map(|&x| Ok((x, crate::load_nnef_model(x)?))).collect()
    }

    #[test]
    fn embedded_graph_is_exported_from_the_nnef_model() -> Result<()> {
        let (manifest, bytes) = export::graph(&nnef_models()?)?;
        let stale = "is stale: run rust/sync.sh";
        ensure!(serde_json::to_vec(&manifest)? == GRAPH, "model.graph.json {stale}");
        ensure!(bytes == WEIGHTS, "model.weights {stale}");
        Ok(())
    }

    #[test]
    fn loaded_graphs_are_the_nnef_graphs() -> Result<()> {
        let loaded = crate::BATCH_CLASSES.iter().map(|&x| Ok((x, load(x)?)));
        let (loaded, bytes) = export::graph(&loaded.collect::<Result<Vec<_>>>()?)?;
        let (expected, expected_bytes) = export::graph(&nnef_models()?)?;
        ensure!(serde_json::to_vec(&loaded)? == serde_json::to_vec(&expected)?);
        ensure!(bytes == expected_bytes);
        Ok(())
    }
}
