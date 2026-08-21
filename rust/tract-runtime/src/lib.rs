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
mod embedding;
#[cfg(any(target_os = "macos", feature = "cuda"))]
mod gpu_conv;
mod layer_norm;

use std::sync::Arc;

#[cfg(all(not(target_os = "macos"), not(feature = "cuda")))]
use anyhow::bail;
use anyhow::{Context as _, Result, ensure};
// Only the GPU preparers build a plan by hand; the CPU one goes through the runtime.
#[cfg(any(target_os = "macos", feature = "cuda"))]
use tract_core::prelude::TypedSimplePlan;
use tract_core::prelude::{
    Framework as _, IntoTValue as _, IntoTensor as _, TValue, TVec, Tensor, ToDim as _, TypedModel,
    tvec,
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
const NUM_LABELS: usize = 214;
const PADDING_TOKEN: i32 = 256;
const DIRECT_FUSED_MIN_BATCH: usize = 8;
const EXPECTED_CONVOLUTIONS: usize = 1;
const EXPECTED_EMBEDDINGS: usize = 1;
const EXPECTED_LAYER_NORMS: usize = 2;
/// Largest score difference tolerated between a GPU and the CPU on the same input.
///
/// They run different kernels and do not agree to the bit: the release gate measures about 1.4e-5
/// between them. This sits far above that and far below a different answer.
const GPU_AGREEMENT_EPSILON: f32 = 1e-3;
/// Embedded release model bytes used by the benchmark's parity and size gates.
#[doc(hidden)]
pub const EMBEDDED_NNEF_MODEL: &[u8] = include_bytes!("../models/model.nnef.tgz");
const EMBEDDED_GPU_PROBE: &[u8] = include_bytes!("../models/model.probe.f32le");

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
    /// Prepares every fixed batch plan for the requested device class.
    pub fn new(request: BackendRequest) -> Result<Self> {
        Self::with_classes(request, &BATCH_CLASSES, &BATCH_CLASSES)
    }

    /// Prepares plans for requests accumulated up to `max_batch` items.
    ///
    /// The x86_64 CPU graph keeps only the largest reachable class: smaller requests are padded
    /// into that optimized fused graph and their extra scores discarded. Other architectures keep
    /// the original set of reachable fixed plans and routing behavior.
    pub fn with_max_batch(request: BackendRequest, max_batch: usize) -> Result<Self> {
        ensure!(max_batch > 0, "the maximum batch cannot be zero");
        let classes: Vec<usize> =
            BATCH_CLASSES.iter().copied().filter(|class| *class <= max_batch).collect();
        ensure!(!classes.is_empty(), "no resident plan can serve a batch of {max_batch}");
        #[cfg(target_arch = "x86_64")]
        let cpu_classes = &classes[classes.len() - 1..];
        #[cfg(not(target_arch = "x86_64"))]
        let cpu_classes = classes.as_slice();
        Self::with_classes(request, cpu_classes, &classes)
    }

    fn with_classes(
        request: BackendRequest, cpu_classes: &[usize], gpu_classes: &[usize],
    ) -> Result<Self> {
        match request {
            BackendRequest::Cpu => Self::prepare_cpu(cpu_classes),
            BackendRequest::Gpu => {
                let gpu = Self::prepare_gpu(gpu_classes)?;
                ensure!(
                    gpu.passes_gpu_probe()?,
                    "the GPU does not identify files correctly on this machine"
                );
                Ok(gpu)
            }
            BackendRequest::Auto => match Self::prepare_gpu(gpu_classes) {
                Ok(gpu) if gpu_agreement_passes(gpu.passes_gpu_probe()) => Ok(gpu),
                _ => Self::prepare_cpu(cpu_classes),
            },
        }
    }

    /// Reports whether this runtime identifies a known input like the release CPU reference.
    ///
    /// A GPU backend that loads is not a GPU backend that computes. A driver can accept every
    /// kernel, report a device and return answers that are simply wrong, and nothing downstream
    /// can tell: a wrong content type is a content type, and the command line exits successfully
    /// having mislabelled everything. The converter records the batch-one CPU scores for this
    /// exact embedded model, and the release gate regenerates them. Comparing against those bytes
    /// keeps the check fail-closed without building a CPU plan on every GPU process startup.
    fn passes_gpu_probe(&self) -> Result<bool> {
        if self.info.backend == Backend::Cpu {
            return Ok(true);
        }
        let input: Vec<i32> =
            (0..FEATURE_SIZE).map(|index| (index % (PADDING_TOKEN as usize + 1)) as i32).collect();
        let candidate = self.session()?.run(&input, 1)?;
        Ok(scores_agree_with_bytes(EMBEDDED_GPU_PROBE, &candidate))
    }

    /// Returns the resolved backend.
    pub fn backend_info(&self) -> BackendInfo {
        self.info
    }

    /// Creates a session for one inference thread; private execution state is spawned on first use.
    pub fn session(&self) -> Result<Session> {
        let plans = self
            .plans
            .iter()
            .map(|plan| SessionPlan {
                batch: plan.batch,
                runnable: plan.runnable.clone(),
                state: None,
            })
            .collect();
        Ok(Session { info: self.info, plans })
    }

    fn prepare_cpu(classes: &[usize]) -> Result<Self> {
        static CPU: DefaultRuntime = DefaultRuntime;
        let options =
            RunOptions { executor: Some(Executor::SingleThread), ..RunOptions::default() };
        let mut plans = Vec::with_capacity(classes.len());
        for &batch in classes {
            let mut model = load_model(batch)?;
            let fused_layer_norm = layer_norm::fuse_magika_layer_norm(&mut model)?;
            ensure!(
                fused_layer_norm == EXPECTED_LAYER_NORMS,
                "required {EXPECTED_LAYER_NORMS} CPU LayerNorm fusions for batch {batch}, matched {fused_layer_norm}"
            );
            if batch >= DIRECT_FUSED_MIN_BATCH {
                let fused_conv = direct_conv::fuse_magika_conv_max(&mut model, batch)?;
                ensure!(
                    fused_conv == EXPECTED_CONVOLUTIONS,
                    "required {EXPECTED_CONVOLUTIONS} CPU Conv1D fusion for batch {batch}, matched {fused_conv}"
                );
            }
            let runnable = CPU
                .prepare_with_options(model, &options)
                .with_context(|| format!("preparing CPU batch-{batch} plan"))?;
            plans.push(PreparedPlan { batch, runnable: Arc::from(runnable) });
        }
        Ok(Self { info: BackendInfo { backend: Backend::Cpu, implementation: "tract-cpu" }, plans })
    }

    #[cfg(target_os = "macos")]
    fn prepare_gpu(classes: &[usize]) -> Result<Self> {
        let mut plans = Vec::with_capacity(classes.len());
        for &batch in classes {
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
            let runnable = with_memory_arena(runnable)
                .with_context(|| format!("sizing the Metal batch-{batch} memory arena"))?;
            let runnable: Arc<dyn Runnable> = Arc::new(Arc::new(runnable));
            plans.push(PreparedPlan { batch, runnable });
        }
        Ok(Self {
            info: BackendInfo { backend: Backend::Gpu, implementation: "tract-metal" },
            plans,
        })
    }

    #[cfg(all(not(target_os = "macos"), feature = "cuda"))]
    fn prepare_gpu(classes: &[usize]) -> Result<Self> {
        ensure!(unsafe { cudarc::nvrtc::sys::is_culib_present() });
        let mut plans = Vec::with_capacity(classes.len());
        for &batch in classes {
            let mut model = load_model(batch)?;
            prepare_gpu_graph(&mut model, batch)?;
            tract_cuda::CudaTransform
                .transform(&mut model)
                .with_context(|| format!("lowering CUDA batch-{batch} plan"))?;
            let model = model.into_optimized().context("optimizing the CUDA model")?;
            let options = RunOptions { skip_order_opt_ram: true, ..RunOptions::default() };
            let runnable = TypedSimplePlan::build(model, &options)
                .with_context(|| format!("preparing CUDA batch-{batch} plan"))?;
            let runnable = with_memory_arena(runnable)
                .with_context(|| format!("sizing the CUDA batch-{batch} memory arena"))?;
            let runnable: Arc<dyn Runnable> = Arc::new(Arc::new(runnable));
            plans.push(PreparedPlan { batch, runnable });
        }
        Ok(Self {
            info: BackendInfo { backend: Backend::Gpu, implementation: "tract-cuda" },
            plans,
        })
    }

    #[cfg(all(not(target_os = "macos"), not(feature = "cuda")))]
    fn prepare_gpu(_classes: &[usize]) -> Result<Self> {
        bail!("this build does not include a GPU backend")
    }
}

fn gpu_agreement_passes(agreement: Result<bool>) -> bool {
    matches!(agreement, Ok(true))
}

/// Gives a GPU plan the arena that lets it reuse device buffers between nodes.
///
/// Without it every intermediate allocates a fresh device buffer on every node of every inference,
/// which is a system call each time. tract installs this itself only when a caller passes memory
/// sizing hints, and building a plan directly bypasses that.
#[cfg(any(target_os = "macos", feature = "cuda"))]
fn with_memory_arena(runnable: TypedSimplePlan) -> Result<TypedSimplePlan> {
    // Every batch is bound to a concrete value before the plan is built, so the graph has no free
    // symbols left and the arena can be sized without hints.
    let hints = Default::default();
    let handler = tract_gpu::session_handler::DeviceSessionHandler::from_plan(&runnable, &hints)?;
    Ok(runnable.with_session_handler(handler))
}

#[cfg(any(target_os = "macos", feature = "cuda"))]
fn prepare_gpu_graph(model: &mut TypedModel, batch: usize) -> Result<()> {
    let fused_layer_norm = layer_norm::fuse_magika_layer_norm_for_gpu(model)?;
    ensure!(
        fused_layer_norm == EXPECTED_LAYER_NORMS,
        "required {EXPECTED_LAYER_NORMS} GPU LayerNorm fusions for batch {batch}, matched {fused_layer_norm}"
    );
    let lowered = gpu_conv::lower_magika_conv_to_matmul(model)?;
    ensure!(
        lowered == EXPECTED_CONVOLUTIONS,
        "required {EXPECTED_CONVOLUTIONS} GPU Conv1D lowering for batch {batch}, matched {lowered}"
    );
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
    // Folded here rather than in a backend preparer: it removes work from the graph itself, so it
    // is worth the same on a CPU, on Metal and on CUDA.
    let folded = embedding::fuse_magika_embedding(&mut model)?;
    ensure!(
        folded == EXPECTED_EMBEDDINGS,
        "required {EXPECTED_EMBEDDINGS} embedding folds for batch {batch}, matched {folded}"
    );
    Ok(model)
}

/// Thread-private inference state.
pub struct Session {
    info: BackendInfo,
    plans: Vec<SessionPlan>,
}

struct SessionPlan {
    batch: usize,
    runnable: Arc<dyn Runnable>,
    state: Option<Box<dyn State>>,
}

impl SessionPlan {
    /// Returns the execution state, spawning it the first time this class is reached.
    ///
    /// A session holds one plan per resident batch class, but a caller that accumulates full
    /// batches only ever routes to the largest. Spawning every class up front made a session cost
    /// the sum of all of them: the execution state is negligible until the plan runs once, and
    /// then it holds that plan's intermediate tensors for good. On this model the classes below
    /// eight are the expensive ones, because only eight and above carry the fused convolution and
    /// the rest still expand the input before multiplying.
    fn state(&mut self) -> Result<&mut dyn State> {
        let state = match &mut self.state {
            Some(state) => state,
            slot => slot.insert(
                self.runnable
                    .spawn()
                    .with_context(|| format!("spawning batch-{} state", self.batch))?,
            ),
        };
        Ok(state.as_mut())
    }
}

impl Session {
    /// Returns the resolved backend.
    pub fn backend_info(&self) -> BackendInfo {
        self.info
    }

    /// Runs one accumulated request, routing it across resident fixed plans.
    pub fn run(&mut self, input: &[i32], batch: usize) -> Result<Vec<f32>> {
        ensure!(batch > 0, "inference batch cannot be empty");
        ensure!(input.len() == expected_input_len(batch)?, "invalid feature tensor length");
        let mut input_offset = 0;
        let mut remaining = batch;
        let mut outputs = Vec::new();
        while remaining > 0 {
            // Use the largest plan that fits. If none does (the x86_64 runtime deliberately keeps
            // only its largest optimized class), pad the tail through the smallest resident plan.
            let index = self.plans.iter().rposition(|plan| plan.batch <= remaining).unwrap_or(0);
            let plan = &mut self.plans[index];
            let class = plan.batch;
            let served = remaining.min(class);
            let input_len = served * FEATURE_SIZE;
            let chunk = &input[input_offset..input_offset + input_len];
            input_offset += input_len;
            remaining -= served;
            if served == class {
                outputs.extend(run_plan(plan.state()?, chunk, class)?);
            } else {
                let mut padded = vec![PADDING_TOKEN; class * FEATURE_SIZE];
                padded[..input_len].copy_from_slice(chunk);
                let mut padded_output = run_plan(plan.state()?, &padded, class)?;
                padded_output.truncate(served * NUM_LABELS);
                outputs.extend(padded_output);
            }
        }
        ensure!(input_offset == input.len());
        Ok(outputs)
    }
}

fn run_plan(state: &mut dyn State, input: &[i32], batch: usize) -> Result<Vec<f32>> {
    let input = Tensor::from_shape(&[batch, FEATURE_SIZE], input)?.into_tvalue();
    decode_output(state.run(tvec!(input))?, batch)
}

fn expected_input_len(batch: usize) -> Result<usize> {
    batch.checked_mul(FEATURE_SIZE).context("inference batch is too large")
}

fn decode_output(mut output: TVec<TValue>, batch: usize) -> Result<Vec<f32>> {
    ensure!(output.len() == 1, "model returned {} outputs instead of one", output.len());
    let output = output.remove(0).into_tensor();
    ensure!(
        output.shape() == [batch, NUM_LABELS],
        "invalid model output shape {:?}",
        output.shape()
    );
    let scores = output.to_plain_array_view::<f32>()?.iter().copied().collect::<Vec<_>>();
    ensure!(scores.iter().all(|score| score.is_finite()), "model returned a non-finite score");
    Ok(scores)
}

/// Reports whether two backends produced the same scores for the same input.
#[cfg(test)]
fn scores_agree(reference: &[f32], candidate: &[f32]) -> bool {
    reference.len() == candidate.len()
        && reference
            .iter()
            .zip(candidate)
            .all(|(reference, candidate)| (reference - candidate).abs() <= GPU_AGREEMENT_EPSILON)
}

fn scores_agree_with_bytes(reference: &[u8], candidate: &[f32]) -> bool {
    reference.len() == std::mem::size_of_val(candidate)
        && reference.as_chunks::<4>().0.iter().zip(candidate).all(|(reference, candidate)| {
            let reference = f32::from_le_bytes(*reference);
            (reference - candidate).abs() <= GPU_AGREEMENT_EPSILON
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Mirrors the routing loop of [`Session::run`] without preparing any plan.
    fn route_classes(classes: &[usize], mut batch: usize) -> Vec<usize> {
        let mut route = Vec::new();
        while batch > 0 {
            let index = classes.iter().rposition(|class| *class <= batch).unwrap();
            route.push(classes[index]);
            batch -= classes[index];
        }
        route
    }

    #[test]
    fn scores_agree_within_the_gap_between_backends_but_not_beyond_it() {
        let reference = [0.5, 0.25, 0.25];
        assert!(scores_agree(&reference, &reference));
        // The release gate measures about 1.4e-5 between a GPU and the CPU, which has to pass.
        assert!(scores_agree(&reference, &[0.500014, 0.249987, 0.250001]));
        // A different answer has to fail, and so does a truncated one.
        assert!(!scores_agree(&reference, &[0.25, 0.5, 0.25]));
        assert!(!scores_agree(&reference, &[0.5, 0.25]));
    }

    #[test]
    fn embedded_gpu_probe_matches_the_release_cpu_model() -> Result<()> {
        let input = (0..FEATURE_SIZE)
            .map(|index| (index % (PADDING_TOKEN as usize + 1)) as i32)
            .collect::<Vec<_>>();
        let output = Runtime::prepare_cpu(&[1])?.session()?.run(&input, 1)?;
        assert!(scores_agree_with_bytes(EMBEDDED_GPU_PROBE, &output));
        assert!(!scores_agree_with_bytes(&EMBEDDED_GPU_PROBE[..4], &output));
        Ok(())
    }

    #[test]
    fn backend_selection_falls_back_when_the_gpu_probe_errors() {
        assert!(!gpu_agreement_passes(Err(anyhow::anyhow!("probe failed"))));
        assert!(!gpu_agreement_passes(Ok(false)));
        assert!(gpu_agreement_passes(Ok(true)));
    }

    #[test]
    fn routes_requests_over_fixed_classes() {
        let all = &BATCH_CLASSES;
        assert_eq!(route_classes(all, 1), [1]);
        assert_eq!(route_classes(all, 10), [8, 1, 1]);
        assert_eq!(route_classes(all, 100), [64, 32, 4]);
    }

    #[test]
    fn routes_requests_over_the_resident_classes_only() {
        // A session prepared for a maximum of eight must still serve a larger request rather than
        // reaching for a class it never prepared.
        let resident = &[1, 4, 8];
        assert_eq!(route_classes(resident, 20), [8, 8, 4]);
        assert_eq!(route_classes(resident, 64), [8; 8]);
        assert_eq!(route_classes(&[1], 3), [1, 1, 1]);
    }

    #[test]
    fn rejects_an_output_with_the_wrong_number_of_classes() {
        let output = Tensor::zero::<f32>(&[2, NUM_LABELS - 1]).unwrap().into_tvalue();
        assert!(decode_output(tvec!(output), 2).is_err());
    }

    #[test]
    fn rejects_more_than_one_model_output() {
        let output = Tensor::zero::<f32>(&[2, NUM_LABELS]).unwrap().into_tvalue();
        assert!(decode_output(tvec!(output.clone(), output), 2).is_err());
    }

    #[test]
    fn rejects_a_non_finite_model_output() {
        let mut output = vec![0.0_f32; NUM_LABELS];
        output[7] = f32::NAN;
        let output = Tensor::from_shape(&[1, NUM_LABELS], &output).unwrap().into_tvalue();
        assert!(decode_output(tvec!(output), 1).is_err());
    }

    #[test]
    fn rejects_an_input_length_that_overflows() {
        assert!(expected_input_len(usize::MAX).is_err());
    }

    #[test]
    fn release_cpu_graph_has_every_required_fusion() -> Result<()> {
        let mut model = load_model(DIRECT_FUSED_MIN_BATCH)?;
        assert_eq!(layer_norm::fuse_magika_layer_norm(&mut model)?, EXPECTED_LAYER_NORMS);
        assert_eq!(
            direct_conv::fuse_magika_conv_max(&mut model, DIRECT_FUSED_MIN_BATCH)?,
            EXPECTED_CONVOLUTIONS
        );
        Ok(())
    }

    /// A target without a vectorized packer runs the release model through the fallback packing
    /// path. Every machine this project can test on takes the vectorized one, so without this the
    /// fallback would only ever have been exercised on a small synthetic convolution, at the real
    /// dimensions of nothing.
    #[test]
    fn the_fallback_packing_path_scores_the_release_model_the_same() -> Result<()> {
        let batch = DIRECT_FUSED_MIN_BATCH;
        let prepare = |portable: bool| -> Result<Vec<f32>> {
            let mut model = load_model(batch)?;
            ensure!(layer_norm::fuse_magika_layer_norm(&mut model)? == EXPECTED_LAYER_NORMS);
            let fused = if portable {
                direct_conv::fuse_magika_conv_max_portable(&mut model, batch)?
            } else {
                direct_conv::fuse_magika_conv_max(&mut model, batch)?
            };
            ensure!(fused == EXPECTED_CONVOLUTIONS);
            static CPU: DefaultRuntime = DefaultRuntime;
            let options =
                RunOptions { executor: Some(Executor::SingleThread), ..RunOptions::default() };
            let runnable = CPU.prepare_with_options(model, &options)?;
            let input: Vec<i32> =
                (0..batch * FEATURE_SIZE).map(|index| (index % 257) as i32).collect();
            run_plan(runnable.spawn()?.as_mut(), &input, batch)
        };
        let scores = prepare(false)?;
        let fallback = prepare(true)?;
        ensure!(scores.len() == batch * NUM_LABELS);
        // The two paths walk the reduction in a different order, so they agree closely rather than
        // to the bit. This bound is far below the gap between two different content types.
        for (left, right) in scores.iter().zip(fallback.iter()) {
            ensure!((left - right).abs() <= 1e-5, "{left} != {right}");
        }
        Ok(())
    }
}
