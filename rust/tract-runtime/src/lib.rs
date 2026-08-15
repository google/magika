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

//! Fixed-shape tract runtime used by Magika's synchronous inference threads.

mod direct_conv;
mod gpu_conv;
mod layer_norm;

use std::sync::Arc;

#[cfg(all(not(target_os = "macos"), not(feature = "cuda")))]
use anyhow::bail;
use anyhow::{Context as _, Result, ensure};
use tract_core::prelude::{
    Framework as _, IntoTValue as _, IntoTensor as _, TValue, TVec, Tensor, ToDim as _, TypedModel,
    TypedSimplePlan, tvec,
};
use tract_core::runtime::{DefaultRuntime, RunOptions, Runnable, Runtime as _, State};
use tract_core::tract_linalg::multithread::Executor;

#[cfg(any(target_os = "macos", feature = "cuda"))]
use tract_core::transform::ModelTransform as _;
#[cfg(target_os = "macos")]
use tract_metal::MetalTransform;

/// Fixed batch shapes prepared by every runtime.
pub const BATCH_CLASSES: [usize; 6] = [1, 4, 8, 16, 32, 64];

const FEATURE_SIZE: usize = 2048;
const PADDING_TOKEN: i32 = 256;
const DIRECT_FUSED_MIN_BATCH: usize = 8;
/// Embedded release model bytes used by the benchmark's parity and size gates.
#[doc(hidden)]
pub const EMBEDDED_NNEF_MODEL: &[u8] = include_bytes!("../models/model.nnef.tgz");

/// User-facing runtime preference.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum BackendRequest {
    /// Prefer an available GPU and fall back to CPU.
    #[default]
    Auto,
    /// Require CPU inference.
    Cpu,
    /// Require an available compiled GPU backend.
    Gpu,
}

/// Resolved device class.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Backend {
    /// CPU inference.
    Cpu,
    /// GPU inference.
    Gpu,
}

/// Resolved runtime information suitable for diagnostics.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BackendInfo {
    backend: Backend,
    implementation: &'static str,
}

impl BackendInfo {
    /// Returns whether inference runs on CPU or GPU.
    pub fn backend(self) -> Backend {
        self.backend
    }

    /// Returns the selected tract implementation name.
    pub fn implementation(self) -> &'static str {
        self.implementation
    }
}

/// Shared prepared plans from which thread-private sessions are spawned.
pub struct Runtime {
    info: BackendInfo,
    plans: Vec<PreparedPlan>,
}

struct PreparedPlan {
    batch: usize,
    runnable: Arc<dyn Runnable>,
}

impl Runtime {
    /// Prepares all fixed batch plans for the requested device class.
    pub fn new(request: BackendRequest) -> Result<Self> {
        match request {
            BackendRequest::Cpu => Self::prepare_cpu(),
            BackendRequest::Gpu => Self::prepare_gpu(),
            BackendRequest::Auto => Self::prepare_gpu().or_else(|_| Self::prepare_cpu()),
        }
    }

    /// Returns the resolved backend.
    pub fn backend_info(&self) -> BackendInfo {
        self.info
    }

    /// Spawns private mutable execution state for one inference thread.
    pub fn session(&self) -> Result<Session> {
        let plans = self
            .plans
            .iter()
            .map(|plan| {
                Ok(SessionPlan {
                    batch: plan.batch,
                    state: plan
                        .runnable
                        .spawn()
                        .with_context(|| format!("spawning batch-{} state", plan.batch))?,
                })
            })
            .collect::<Result<_>>()?;
        let mut session = Session { info: self.info, plans };
        session.warm_all()?;
        Ok(session)
    }

    fn prepare_cpu() -> Result<Self> {
        static CPU: DefaultRuntime = DefaultRuntime;
        let options =
            RunOptions { executor: Some(Executor::SingleThread), ..RunOptions::default() };
        let mut plans = Vec::with_capacity(BATCH_CLASSES.len());
        for batch in BATCH_CLASSES {
            let mut model = load_model(batch)?;
            let fused_layer_norm = layer_norm::fuse_magika_layer_norm(&mut model)?;
            ensure!(
                fused_layer_norm > 0,
                "required CPU LayerNorm fusion did not match batch {batch}"
            );
            if batch >= DIRECT_FUSED_MIN_BATCH {
                let fused_conv = direct_conv::fuse_magika_conv_max(&mut model, batch)?;
                ensure!(fused_conv > 0, "required CPU Conv1D fusion did not match batch {batch}");
            }
            let runnable = CPU
                .prepare_with_options(model, &options)
                .with_context(|| format!("preparing CPU batch-{batch} plan"))?;
            plans.push(PreparedPlan { batch, runnable: Arc::from(runnable) });
        }
        Ok(Self { info: BackendInfo { backend: Backend::Cpu, implementation: "tract-cpu" }, plans })
    }

    #[cfg(target_os = "macos")]
    fn prepare_gpu() -> Result<Self> {
        let mut plans = Vec::with_capacity(BATCH_CLASSES.len());
        for batch in BATCH_CLASSES {
            let mut model = load_model(batch)?;
            prepare_gpu_graph(&mut model, batch)?;
            let gemm_impl = None;
            MetalTransform { gemm_impl }
                .transform(&mut model)
                .with_context(|| format!("lowering Metal batch-{batch} plan"))?;
            let model = model.into_optimized().context("optimizing the Metal model")?;
            let options = RunOptions { skip_order_opt_ram: true, ..RunOptions::default() };
            let runnable = TypedSimplePlan::build(model, &options)
                .with_context(|| format!("preparing Metal batch-{batch} plan"))?;
            let runnable: Arc<dyn Runnable> = Arc::new(Arc::new(runnable));
            plans.push(PreparedPlan { batch, runnable });
        }
        Ok(Self {
            info: BackendInfo { backend: Backend::Gpu, implementation: "tract-metal" },
            plans,
        })
    }

    #[cfg(all(not(target_os = "macos"), feature = "cuda"))]
    fn prepare_gpu() -> Result<Self> {
        let mut plans = Vec::with_capacity(BATCH_CLASSES.len());
        for batch in BATCH_CLASSES {
            let mut model = load_model(batch)?;
            prepare_gpu_graph(&mut model, batch)?;
            tract_cuda::CudaTransform
                .transform(&mut model)
                .with_context(|| format!("lowering CUDA batch-{batch} plan"))?;
            let model = model.into_optimized().context("optimizing the CUDA model")?;
            let options = RunOptions { skip_order_opt_ram: true, ..RunOptions::default() };
            let runnable = TypedSimplePlan::build(model, &options)
                .with_context(|| format!("preparing CUDA batch-{batch} plan"))?;
            let runnable: Arc<dyn Runnable> = Arc::new(Arc::new(runnable));
            plans.push(PreparedPlan { batch, runnable });
        }
        Ok(Self {
            info: BackendInfo { backend: Backend::Gpu, implementation: "tract-cuda" },
            plans,
        })
    }

    #[cfg(all(not(target_os = "macos"), not(feature = "cuda")))]
    fn prepare_gpu() -> Result<Self> {
        bail!("this build does not include a GPU backend")
    }
}

#[cfg(any(target_os = "macos", feature = "cuda"))]
fn prepare_gpu_graph(model: &mut TypedModel, batch: usize) -> Result<()> {
    let fused_layer_norm = layer_norm::fuse_magika_layer_norm_for_gpu(model)?;
    ensure!(
        fused_layer_norm == 2,
        "required both GPU LayerNorm fusions for batch {batch}, matched {fused_layer_norm}"
    );
    let lowered = gpu_conv::lower_magika_conv_to_matmul(model)?;
    ensure!(lowered > 0, "required GPU Conv1D lowering did not match batch {batch}");
    Ok(())
}

fn load_model(batch: usize) -> Result<TypedModel> {
    let model = tract_nnef::nnef()
        .model_for_read(&mut std::io::Cursor::new(EMBEDDED_NNEF_MODEL))
        .context("loading the embedded NNEF model")?;
    let mut model = if let Some(symbol) = model.symbols.get("N") {
        let symbols = std::collections::HashMap::from([(symbol, batch.to_dim())]);
        model.set_symbols(&symbols).context("binding the NNEF batch symbol")?
    } else {
        ensure!(model.input_fact(0)?.shape[0] == batch.to_dim(), "fixed NNEF batch mismatch");
        model
    };
    model = model.into_decluttered().context("decluttering before Magika graph fusion")?;
    Ok(model)
}

/// Thread-private inference state.
pub struct Session {
    info: BackendInfo,
    plans: Vec<SessionPlan>,
}

struct SessionPlan {
    batch: usize,
    state: Box<dyn State>,
}

impl Session {
    /// Returns the resolved backend.
    pub fn backend_info(&self) -> BackendInfo {
        self.info
    }

    /// Runs one accumulated request, routing it across resident fixed plans.
    pub fn run(&mut self, input: &[i32], batch: usize) -> Result<Vec<f32>> {
        ensure!(batch > 0, "inference batch cannot be empty");
        ensure!(input.len() == batch * FEATURE_SIZE, "invalid feature tensor length");
        let route = route_classes(batch);
        let mut input_offset = 0;
        let mut outputs = Vec::new();
        for class in route {
            let input_len = class * FEATURE_SIZE;
            let chunk = &input[input_offset..input_offset + input_len];
            input_offset += input_len;
            let plan = self
                .plans
                .iter_mut()
                .find(|plan| plan.batch == class)
                .with_context(|| format!("batch-{class} plan is not resident"))?;
            outputs.extend(run_plan(plan.state.as_mut(), chunk, class)?);
        }
        ensure!(input_offset == input.len());
        Ok(outputs)
    }

    fn warm_all(&mut self) -> Result<()> {
        for plan in &mut self.plans {
            let input = vec![PADDING_TOKEN; plan.batch * FEATURE_SIZE];
            run_plan(plan.state.as_mut(), &input, plan.batch)
                .with_context(|| format!("warming batch-{} plan", plan.batch))?;
        }
        Ok(())
    }
}

fn run_plan(state: &mut dyn State, input: &[i32], batch: usize) -> Result<Vec<f32>> {
    let input = Tensor::from_shape(&[batch, FEATURE_SIZE], input)?.into_tvalue();
    let mut output: TVec<TValue> = state.run(tvec!(input))?;
    let output = output.remove(0).into_tensor();
    ensure!(output.rank() > 0 && output.shape()[0] == batch, "invalid model output shape");
    Ok(output.to_plain_array_view::<f32>()?.iter().copied().collect())
}

fn route_classes(mut batch: usize) -> Vec<usize> {
    let mut route = Vec::new();
    while batch > 0 {
        let class = BATCH_CLASSES.iter().rev().copied().find(|class| *class <= batch).unwrap();
        route.push(class);
        batch -= class;
    }
    route
}

/// Low-level graph rewrites used by the benchmark and conversion release gate.
#[doc(hidden)]
pub mod bench {
    pub use crate::direct_conv::fuse_magika_conv_max;
    pub use crate::gpu_conv::lower_magika_conv_to_matmul;
    pub use crate::layer_norm::{fuse_magika_layer_norm, fuse_magika_layer_norm_for_gpu};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn routes_requests_over_fixed_classes() {
        assert_eq!(route_classes(1), [1]);
        assert_eq!(route_classes(10), [8, 1, 1]);
        assert_eq!(route_classes(100), [64, 32, 4]);
    }
}
