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

//! Compares Magika inference runtimes during the tract feasibility spike.

#[cfg(feature = "tract-runtime")]
mod direct_conv;
#[cfg(feature = "tract-runtime")]
mod plan_pool;

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use anyhow::{Context, Result, anyhow, bail, ensure};

#[cfg(feature = "ort-runtime")]
use ndarray::Array2;
#[cfg(feature = "tract-runtime")]
use plan_pool::PlanPoolBackend;
#[cfg(all(feature = "metal", target_os = "macos"))]
use tract_core::prelude::TypedSimplePlan;
#[cfg(feature = "tract-runtime")]
use tract_core::prelude::{
    Framework as _, IntoTValue as _, IntoTensor as _, TValue, TVec, Tensor, ToDim as _, TypedModel,
    TypedSimpleState, tvec,
};
#[cfg(feature = "tract-runtime")]
use tract_core::runtime::{DefaultRuntime, RunOptions, Runnable, State};
#[cfg(feature = "tract-runtime")]
use tract_core::tract_linalg::multithread::Executor;

#[cfg(all(feature = "metal", target_os = "macos"))]
use tract_core::transform::ModelTransform as _;

#[cfg(all(feature = "metal", target_os = "macos"))]
use tract_metal::{MetalGemmImplKind, MetalTransform};

const FEATURE_SIZE: usize = 2048;
const PADDING_TOKEN: i32 = 256;
#[cfg(feature = "tract-runtime")]
const MAGIKA_LAZY_IM2COL_MIN_KERNEL: &str = "5";

#[cfg(feature = "ort-runtime")]
const ONNX_MODEL: &[u8] = include_bytes!("../../../assets/models/standard_v3_3/model.onnx");
#[cfg(feature = "tract-runtime")]
const NNEF_MODEL: &[u8] = include_bytes!("../models/model.nnef.tgz");

trait Backend {
    fn name(&self) -> &'static str;
    fn run(&mut self, input: &[i32], batch: usize) -> Result<Vec<f32>>;
    fn selected_classes(&self, _batch: usize) -> Option<Vec<usize>> {
        None
    }
    fn plan_op_counts(&self) -> Option<std::collections::BTreeMap<String, usize>> {
        None
    }
    fn profile_plan(
        &mut self, _input: &[i32], _batch: usize,
    ) -> Result<Option<Vec<(String, String, Duration)>>> {
        Ok(None)
    }
}

#[cfg(feature = "ort-runtime")]
struct OrtBackend {
    session: ort::session::Session,
}

#[cfg(feature = "ort-runtime")]
impl OrtBackend {
    fn load() -> Result<Self> {
        let session = ort::session::Session::builder()?
            .commit_from_memory(ONNX_MODEL)
            .context("loading the embedded ONNX model")?;
        Ok(Self { session })
    }
}

#[cfg(feature = "ort-runtime")]
impl Backend for OrtBackend {
    fn name(&self) -> &'static str {
        "onnx-runtime"
    }

    fn run(&mut self, input: &[i32], batch: usize) -> Result<Vec<f32>> {
        let input = Array2::from_shape_vec([batch, FEATURE_SIZE], input.to_vec())?;
        let input = ort::value::Tensor::from_array(input)?;
        let mut output = self.session.run(ort::inputs!("bytes" => input))?;
        let output =
            output.remove("target_label").context("ONNX output target_label is missing")?;
        let output = output.try_extract_array::<f32>()?;
        Ok(output.iter().copied().collect())
    }
}

#[cfg(feature = "tract-runtime")]
struct TractBackend {
    name: &'static str,
    state: Box<dyn State>,
}

#[cfg(feature = "tract-runtime")]
impl TractBackend {
    fn load_cpu(
        threads: usize, fixed_batch: Option<usize>, nnef_model: Option<&Path>, direct_fused: bool,
    ) -> Result<Self> {
        let runnable = Self::prepare_cpu(threads, fixed_batch, nnef_model, direct_fused)?;
        Self::spawn("tract-cpu", runnable.as_ref())
    }

    fn prepare_cpu(
        threads: usize, fixed_batch: Option<usize>, nnef_model: Option<&Path>, direct_fused: bool,
    ) -> Result<std::sync::Arc<dyn Runnable>> {
        static CPU: DefaultRuntime = DefaultRuntime;
        let options =
            RunOptions { executor: Some(Executor::multithread(threads)), ..RunOptions::default() };
        Self::prepare_with_runtime_and_options(
            "tract-cpu",
            &CPU,
            Some(&options),
            fixed_batch,
            nnef_model,
            direct_fused,
        )
    }

    fn prepare_with_runtime_and_options(
        name: &'static str, runtime: &'static dyn tract_core::runtime::Runtime,
        options: Option<&RunOptions>, fixed_batch: Option<usize>, nnef_model: Option<&Path>,
        direct_fused: bool,
    ) -> Result<std::sync::Arc<dyn Runnable>> {
        let model = Self::load_model(fixed_batch, nnef_model, direct_fused)?;
        let runnable = match options {
            Some(options) => runtime.prepare_with_options(model, options),
            None => runtime.prepare(model),
        }
        .with_context(|| format!("preparing {name}"))?;
        Ok(std::sync::Arc::from(runnable))
    }

    fn spawn(name: &'static str, runnable: &dyn Runnable) -> Result<Self> {
        let state = runnable.spawn().with_context(|| format!("spawning {name} state"))?;
        Ok(Self { name, state })
    }

    fn load_model(
        fixed_batch: Option<usize>, nnef_model: Option<&Path>, direct_fused: bool,
    ) -> Result<TypedModel> {
        let mut model = match nnef_model {
            Some(path) => tract_nnef::nnef()
                .model_for_path(path)
                .with_context(|| format!("loading {}", path.display()))?,
            None => tract_nnef::nnef()
                .model_for_read(&mut std::io::Cursor::new(NNEF_MODEL))
                .context("loading the embedded NNEF model")?,
        };
        let Some(batch) = fixed_batch else {
            ensure!(!direct_fused, "--direct-fused-conv requires --fixed-batch");
            return Ok(model);
        };
        let Some(symbol) = model.symbols.get("N") else {
            ensure!(
                model.input_fact(0)?.shape[0] == batch.to_dim(),
                "fixed NNEF batch does not match --batch {batch}"
            );
            if direct_fused {
                model =
                    model.into_decluttered().context("decluttering before direct Conv1D fusion")?;
                if !direct_conv::fuse_magika_conv_max(&mut model, batch)? {
                    eprintln!("direct_fusion\tfallback\tbatch={batch}");
                }
            }
            return Ok(model);
        };
        let symbols = std::collections::HashMap::from([(symbol, batch.to_dim())]);
        let mut model = model.set_symbols(&symbols).context("binding the NNEF batch symbol")?;
        if direct_fused {
            model = model.into_decluttered().context("decluttering before direct Conv1D fusion")?;
            if !direct_conv::fuse_magika_conv_max(&mut model, batch)? {
                eprintln!("direct_fusion\tfallback\tbatch={batch}");
            }
        }
        Ok(model)
    }

    #[cfg(all(feature = "metal", target_os = "macos"))]
    fn load_metal(
        fixed_batch: Option<usize>, gemm: Option<&str>, nnef_model: Option<&Path>,
    ) -> Result<Self> {
        let gemm_impl = match gemm {
            None | Some("auto") => None,
            Some("mlx") => Some(MetalGemmImplKind::Mlx),
            Some("mfa") => Some(MetalGemmImplKind::Mfa),
            Some("ggml") => Some(MetalGemmImplKind::Ggml),
            Some(gemm) => bail!("unknown Metal GEMM implementation: {gemm}"),
        };
        let mut model = Self::load_model(fixed_batch, nnef_model, false)?;
        MetalTransform { gemm_impl }.transform(&mut model).context("transforming for Metal")?;
        let model = model.into_optimized().context("optimizing the Metal model")?;
        let options = RunOptions { skip_order_opt_ram: true, ..RunOptions::default() };
        let runnable = TypedSimplePlan::build(model, &options).context("preparing tract-metal")?;
        Self::spawn("tract-metal", &std::sync::Arc::new(runnable))
    }
}

#[cfg(feature = "tract-runtime")]
impl Backend for TractBackend {
    fn name(&self) -> &'static str {
        self.name
    }

    fn run(&mut self, input: &[i32], batch: usize) -> Result<Vec<f32>> {
        let input = Tensor::from_shape(&[batch, FEATURE_SIZE], input)?.into_tvalue();
        let mut output: TVec<TValue> = self.state.run(tvec!(input))?;
        let output = output.remove(0).into_tensor();
        Ok(output.to_plain_array_view::<f32>()?.iter().copied().collect())
    }

    fn plan_op_counts(&self) -> Option<std::collections::BTreeMap<String, usize>> {
        let model = self.state.runnable().typed_model()?;
        let mut counts = std::collections::BTreeMap::new();
        for node in &model.nodes {
            *counts.entry(node.op.name().to_string()).or_default() += 1;
        }
        Some(counts)
    }

    fn profile_plan(
        &mut self, input: &[i32], batch: usize,
    ) -> Result<Option<Vec<(String, String, Duration)>>> {
        if self.name != "tract-cpu" {
            return Ok(None);
        }
        let state = self
            .state
            .downcast_mut::<TypedSimpleState>()
            .context("tract CPU state is not a typed simple state")?;
        let input = Tensor::from_shape(&[batch, FEATURE_SIZE], input)?.into_tvalue();
        let mut samples = Vec::new();
        let _outputs =
            state.run_plan_with_eval(tvec!(input), |session, op_state, node, inputs| {
                let start = Instant::now();
                let result = tract_core::plan::eval(session, op_state, node, inputs);
                samples.push((node.name.clone(), node.op.name().to_string(), start.elapsed()));
                result
            })?;
        samples.sort_by_key(|(_, _, elapsed)| std::cmp::Reverse(*elapsed));
        Ok(Some(samples))
    }
}

#[derive(Clone, Copy, Debug, Default)]
enum PoolRouting {
    Ceil,
    #[default]
    Exact,
}

#[derive(Debug)]
struct Options {
    backend: Option<String>,
    batch: usize,
    batch_sweep: bool,
    compute_owners: Option<usize>,
    direct_fused_conv: bool,
    fixed_batch: bool,
    iterations: usize,
    metal_gemm: Option<String>,
    nnef_model: Option<PathBuf>,
    plan_pool: Option<Vec<usize>>,
    pool_routing: PoolRouting,
    plan_summary: bool,
    profile_plan: bool,
    reverse: bool,
    tract_threads: usize,
    verify_batches: bool,
    verify_only: bool,
}

fn main() -> Result<()> {
    let options = parse_options()?;
    configure_tract_codegen();
    ensure!(options.batch > 0, "--batch must be greater than zero");
    ensure!(options.iterations > 0, "--iterations must be greater than zero");
    ensure!(
        !(options.fixed_batch && (options.batch_sweep || options.verify_batches)),
        "--fixed-batch cannot be combined with a multi-shape sweep"
    );
    ensure!(
        !(options.fixed_batch && options.plan_pool.is_some()),
        "--fixed-batch and --plan-pool are alternative fixed-shape modes"
    );
    ensure!(
        !(options.compute_owners.is_some() && options.plan_pool.is_some()),
        "--compute-owners does not yet support a resident plan pool"
    );
    #[cfg(feature = "ort-runtime")]
    ort::init().with_telemetry(false).commit();

    if let Some(owners) = options.compute_owners {
        return bench_compute_owner_backends(&options, owners);
    }

    let mut backends: Vec<(Box<dyn Backend>, Duration)> = Vec::new();

    #[cfg(feature = "ort-runtime")]
    if wants_backend(&options, "ort") {
        load_backend(&mut backends, OrtBackend::load)?;
    }
    #[cfg(feature = "tract-runtime")]
    if wants_backend(&options, "cpu") {
        if let Some(classes) = options.plan_pool.as_deref() {
            load_backend(&mut backends, || {
                PlanPoolBackend::load_cpu(
                    classes,
                    options.tract_threads,
                    options.nnef_model.as_deref(),
                    options.direct_fused_conv,
                    options.pool_routing,
                )
            })?;
        } else {
            load_backend(&mut backends, || {
                TractBackend::load_cpu(
                    options.tract_threads,
                    options.fixed_batch.then_some(options.batch),
                    options.nnef_model.as_deref(),
                    options.direct_fused_conv,
                )
            })?;
        }
    }
    #[cfg(all(feature = "metal", target_os = "macos"))]
    if wants_backend(&options, "metal") {
        if let Some(classes) = options.plan_pool.as_deref() {
            load_backend(&mut backends, || {
                PlanPoolBackend::load_metal(
                    classes,
                    options.metal_gemm.as_deref(),
                    options.nnef_model.as_deref(),
                    options.pool_routing,
                )
            })?;
        } else {
            load_backend(&mut backends, || {
                TractBackend::load_metal(
                    options.fixed_batch.then_some(options.batch),
                    options.metal_gemm.as_deref(),
                    options.nnef_model.as_deref(),
                )
            })?;
        }
    }
    if options.plan_summary {
        print_plan_summaries(&backends);
    }
    if options.reverse {
        backends.reverse();
    }

    ensure!(!backends.is_empty(), "enable at least one runtime feature");
    if options.batch_sweep {
        print_header(&backends);
        print_runtime_options(&options);
        let batches: Vec<usize> = if options.plan_pool.is_some() {
            (1..=10).chain([16, 32, 64]).collect()
        } else {
            vec![1, 2, 4, 8, 16, 32]
        };
        for batch in batches {
            let corpus = load_corpus(batch)?;
            print_routes(&backends, batch);
            let (outputs, first_runs) = verify(&mut backends, &corpus, batch)?;
            println!("verification_batch\t{batch}");
            print_verification(&backends, &outputs);
            if !options.verify_only {
                for ((backend, cold), first) in backends.iter_mut().zip(first_runs) {
                    bench_backend(
                        backend.as_mut(),
                        *cold,
                        first,
                        &corpus,
                        batch,
                        options.iterations,
                    )?;
                }
            }
        }
        return Ok(());
    }
    if options.verify_batches {
        print_header(&backends);
        print_runtime_options(&options);
        for batch in [1, 2, 3, 8, 16] {
            let corpus = load_corpus(batch)?;
            let (outputs, _) = verify(&mut backends, &corpus, batch)?;
            println!("verification_batch\t{batch}");
            print_verification(&backends, &outputs);
        }
        return Ok(());
    }
    let corpus = load_corpus(options.batch)?;
    print_routes(&backends, options.batch);
    let (outputs, first_runs) = verify(&mut backends, &corpus, options.batch)?;
    if options.profile_plan {
        print_plan_profiles(&mut backends, &corpus, options.batch)?;
    }
    print_header(&backends);
    print_runtime_options(&options);
    println!(
        "workload\tbatch={}\titerations={}\ttotal_files={}",
        options.batch,
        options.iterations,
        options.batch * options.iterations
    );
    print_verification(&backends, &outputs);

    if !options.verify_only {
        for ((backend, cold), first) in backends.iter_mut().zip(first_runs) {
            bench_backend(
                backend.as_mut(),
                *cold,
                first,
                &corpus,
                options.batch,
                options.iterations,
            )?;
        }
    }

    Ok(())
}

fn load_backend<B: Backend + 'static>(
    backends: &mut Vec<(Box<dyn Backend>, Duration)>, load: impl FnOnce() -> Result<B>,
) -> Result<()> {
    let start = Instant::now();
    let backend = load()?;
    backends.push((Box::new(backend), start.elapsed()));
    Ok(())
}

fn bench_compute_owner_backends(options: &Options, owners: usize) -> Result<()> {
    ensure!(owners > 0, "--compute-owners must be greater than zero");
    ensure!(
        cfg!(feature = "ort-runtime") || cfg!(feature = "tract-runtime"),
        "enable at least one CPU runtime feature"
    );
    let input = load_corpus(options.batch)?;
    println!(
        "owner_columns\tbackend\towners\tbatch\titerations_per_owner\twall_us\tfiles_per_second"
    );

    let run_ort = || -> Result<()> {
        #[cfg(feature = "ort-runtime")]
        bench_compute_owners(
            "onnx-runtime",
            owners,
            options.batch,
            options.iterations,
            &input,
            OrtBackend::load,
        )?;
        Ok(())
    };
    let run_tract = || -> Result<()> {
        #[cfg(feature = "tract-runtime")]
        {
            let runnable = TractBackend::prepare_cpu(
                options.tract_threads,
                options.fixed_batch.then_some(options.batch),
                options.nnef_model.as_deref(),
                options.direct_fused_conv,
            )?;
            bench_compute_owners(
                "tract-cpu",
                owners,
                options.batch,
                options.iterations,
                &input,
                || TractBackend::spawn("tract-cpu", runnable.as_ref()),
            )?;
        }
        Ok(())
    };
    if options.reverse {
        run_tract()?;
        run_ort()?;
    } else {
        run_ort()?;
        run_tract()?;
    }
    Ok(())
}

fn bench_compute_owners<B: Backend>(
    name: &'static str, owners: usize, batch: usize, iterations: usize, input: &[i32],
    load: impl Fn() -> Result<B> + Sync,
) -> Result<()> {
    let elapsed = std::thread::scope(|scope| -> Result<Duration> {
        let (ready_sender, ready_receiver) = std::sync::mpsc::channel();
        let (done_sender, done_receiver) = std::sync::mpsc::channel();
        let mut job_senders = Vec::with_capacity(owners);
        let mut handles = Vec::with_capacity(owners);

        for owner in 0..owners {
            let (job_sender, job_receiver) = std::sync::mpsc::sync_channel::<()>(1);
            job_senders.push(job_sender);
            let ready_sender = ready_sender.clone();
            let done_sender = done_sender.clone();
            let load = &load;
            handles.push(scope.spawn(move || {
                let mut backend = match load() {
                    Ok(backend) => backend,
                    Err(error) => {
                        let _ = ready_sender.send(Err(format!("{error:#}")));
                        return;
                    }
                };
                for _ in 0..10 {
                    if let Err(error) = backend.run(input, batch) {
                        let _ = ready_sender.send(Err(format!("{error:#}")));
                        return;
                    }
                }
                if ready_sender.send(Ok(())).is_err() {
                    return;
                }
                while job_receiver.recv().is_ok() {
                    let result =
                        backend.run(input, batch).map(|_| ()).map_err(|e| format!("{e:#}"));
                    let failed = result.is_err();
                    if done_sender.send((owner, result)).is_err() || failed {
                        return;
                    }
                }
            }));
        }
        drop(ready_sender);
        drop(done_sender);

        let mut readiness_error = None;
        for _ in 0..owners {
            if let Err(error) =
                ready_receiver.recv().context("a compute owner exited before ready")?
            {
                readiness_error.get_or_insert(error);
            }
        }
        if let Some(error) = readiness_error {
            drop(job_senders);
            for handle in handles {
                handle.join().map_err(|_| anyhow!("a compute owner panicked"))?;
            }
            bail!("preparing a {name} compute owner: {error}");
        }

        let total_jobs = owners * iterations;
        let start = Instant::now();
        let mut dispatched = 0;
        for sender in job_senders.iter().take(total_jobs.min(owners)) {
            sender.send(()).context("starting a compute owner")?;
            dispatched += 1;
        }
        for _ in 0..total_jobs {
            let (owner, result) = done_receiver.recv().context("a compute owner exited early")?;
            result.map_err(|error| anyhow!(error))?;
            if dispatched < total_jobs {
                job_senders[owner].send(()).context("queueing accumulated compute")?;
                dispatched += 1;
            }
        }
        let elapsed = start.elapsed();
        drop(job_senders);
        for handle in handles {
            handle.join().map_err(|_| anyhow!("a compute owner panicked"))?;
        }
        Ok(elapsed)
    })?;

    let files = owners * iterations * batch;
    println!(
        "owner_result\t{name}\t{owners}\t{batch}\t{iterations}\t{}\t{:.3}",
        micros(elapsed),
        files as f64 / elapsed.as_secs_f64()
    );
    Ok(())
}

fn verify(
    backends: &mut [(Box<dyn Backend>, Duration)], input: &[i32], batch: usize,
) -> Result<(Vec<Vec<f32>>, Vec<Duration>)> {
    let mut outputs = Vec::with_capacity(backends.len());
    let mut first_runs = Vec::with_capacity(backends.len());
    for (backend, _) in backends.iter_mut() {
        let start = Instant::now();
        let output = backend.run(input, batch)?;
        first_runs.push(start.elapsed());
        ensure!(output.len() % batch == 0, "{} returned an invalid output shape", backend.name());
        outputs.push(output);
    }

    if let Some(reference) = outputs.first() {
        let labels = reference.len() / batch;
        for (backend_index, candidate) in outputs.iter().enumerate().skip(1) {
            ensure!(
                reference.len() == candidate.len(),
                "runtime output lengths differ: {} != {}",
                reference.len(),
                candidate.len()
            );
            let max_abs = reference
                .iter()
                .zip(candidate)
                .map(|(left, right)| (left - right).abs())
                .fold(0.0_f32, f32::max);
            for row in 0..batch {
                let range = row * labels..(row + 1) * labels;
                ensure!(
                    argmax(&reference[range.clone()]) == argmax(&candidate[range]),
                    "winning label differs for row {row}: {} vs {}",
                    backends[0].0.name(),
                    backends[backend_index].0.name()
                );
            }
            ensure!(
                max_abs <= 1.0e-4,
                "score difference exceeds tolerance for {}: {max_abs}",
                backends[backend_index].0.name()
            );
        }
    }
    Ok((outputs, first_runs))
}

fn bench_backend(
    backend: &mut dyn Backend, cold: Duration, first: Duration, input: &[i32], batch: usize,
    iterations: usize,
) -> Result<()> {
    for _ in 0..10 {
        let _ = std::hint::black_box(backend.run(input, batch)?);
    }
    let mut samples = Vec::with_capacity(iterations);
    for _ in 0..iterations {
        let start = Instant::now();
        let _ = std::hint::black_box(backend.run(input, batch)?);
        samples.push(start.elapsed());
    }
    samples.sort_unstable();
    let total: Duration = samples.iter().sum();
    let mean = total / iterations as u32;
    let p50 = samples[iterations / 2];
    let p95 = samples[(iterations * 95 / 100).min(iterations - 1)];
    let mean_per_item_us = total.as_secs_f64() * 1_000_000.0 / iterations as f64 / batch as f64;
    let items_per_second = batch as f64 * iterations as f64 / total.as_secs_f64();
    println!(
        "result\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{mean_per_item_us:.3}\t{items_per_second:.3}\t{}\t{}\t{}",
        batch,
        backend.name(),
        micros(cold),
        micros(first),
        micros(mean),
        micros(p50),
        micros(p95),
        iterations,
        batch * iterations,
        micros(total)
    );
    Ok(())
}

fn print_header(backends: &[(Box<dyn Backend>, Duration)]) {
    let executable_bytes = std::env::current_exe()
        .and_then(std::fs::metadata)
        .map(|metadata| metadata.len())
        .unwrap_or(0);
    println!("host\t{}\t{}", std::env::consts::OS, std::env::consts::ARCH);
    println!("executable_bytes\t{executable_bytes}");
    #[cfg(feature = "ort-runtime")]
    println!("model_bytes\tonnx\t{}", ONNX_MODEL.len());
    #[cfg(feature = "tract-runtime")]
    println!("model_bytes\tnnef\t{}", NNEF_MODEL.len());
    println!(
        "backends\t{}",
        backends.iter().map(|(backend, _)| backend.name()).collect::<Vec<_>>().join(",")
    );
    println!(
        "columns\tbatch\tbackend\tcold_us\tfirst_us\tmean_us\tp50_us\tp95_us\tmean_per_item_us\titems_per_second\titerations\ttotal_files\ttotal_us"
    );
}

#[cfg_attr(not(feature = "tract-runtime"), allow(unused_variables))]
fn print_runtime_options(options: &Options) {
    #[cfg(feature = "tract-runtime")]
    {
        println!("tract_cpu_threads\t{}", options.tract_threads);
        let batch_plan = match &options.plan_pool {
            Some(classes) => format!(
                "pool:{}",
                classes.iter().map(usize::to_string).collect::<Vec<_>>().join(",")
            ),
            None if options.fixed_batch => "fixed".to_string(),
            None => "symbolic".to_string(),
        };
        println!("tract_batch_plan\t{batch_plan}");
        if options.plan_pool.is_some() {
            println!("tract_pool_routing\t{:?}", options.pool_routing);
        }
        println!(
            "tract_lazy_im2col_min_kernel\t{}",
            std::env::var("TRACT_LAZY_IM2COL_MIN_KERNEL")
                .unwrap_or_else(|_| MAGIKA_LAZY_IM2COL_MIN_KERNEL.to_string())
        );
    }
    #[cfg(all(feature = "metal", target_os = "macos"))]
    println!("metal_gemm\t{}", options.metal_gemm.as_deref().unwrap_or("auto"));
    #[cfg(feature = "tract-runtime")]
    println!(
        "nnef_source\t{}",
        options
            .nnef_model
            .as_deref()
            .map_or("embedded", |path| path.to_str().unwrap_or("non-utf8"))
    );
}

fn print_verification(backends: &[(Box<dyn Backend>, Duration)], outputs: &[Vec<f32>]) {
    if outputs.len() < 2 {
        println!("verification\tskipped\tneed at least two compiled backends");
        return;
    }
    for index in 1..outputs.len() {
        let max_abs = outputs[0]
            .iter()
            .zip(&outputs[index])
            .map(|(left, right)| (left - right).abs())
            .fold(0.0_f32, f32::max);
        println!(
            "verification\t{}\t{}\tmax_abs={max_abs}",
            backends[0].0.name(),
            backends[index].0.name()
        );
    }
}

fn load_corpus(batch: usize) -> Result<Vec<i32>> {
    const FILES: &[&str] = &[
        "README.md",
        "rust/lib/src/lib.rs",
        "rust/cli/src/main.rs",
        "assets/magika-screenshot.png",
        "assets/models/standard_v3_3/model.onnx",
        "tests_data/mitra/elf/elf64.elf",
    ];
    let mut rows = Vec::with_capacity(batch * FEATURE_SIZE);
    for row in 0..batch {
        let path = Path::new(FILES[row % FILES.len()]);
        let content = std::fs::read(path).with_context(|| format!("reading {}", path.display()))?;
        rows.extend(extract_features(&content));
    }
    Ok(rows)
}

fn extract_features(content: &[u8]) -> [i32; FEATURE_SIZE] {
    let content = strip_ascii_whitespace(content);
    let mut features = [PADDING_TOKEN; FEATURE_SIZE];
    let beginning = &content[..content.len().min(FEATURE_SIZE / 2)];
    let end_start = content.len().saturating_sub(FEATURE_SIZE / 2);
    let end = &content[end_start..];
    for (destination, source) in features.iter_mut().zip(beginning) {
        *destination = i32::from(*source);
    }
    let destination_start = FEATURE_SIZE - end.len();
    for (destination, source) in features[destination_start..].iter_mut().zip(end) {
        *destination = i32::from(*source);
    }
    features
}

fn strip_ascii_whitespace(mut content: &[u8]) -> &[u8] {
    while content.first().is_some_and(|byte| byte.is_ascii_whitespace() || *byte == 0x0b) {
        content = &content[1..];
    }
    while content.last().is_some_and(|byte| byte.is_ascii_whitespace() || *byte == 0x0b) {
        content = &content[..content.len() - 1];
    }
    content
}

fn argmax(values: &[f32]) -> usize {
    values
        .iter()
        .enumerate()
        .max_by(|(_, left), (_, right)| left.total_cmp(right))
        .map(|(index, _)| index)
        .unwrap_or(0)
}

fn parse_options() -> Result<Options> {
    let mut options = Options {
        backend: None,
        batch: 1,
        batch_sweep: false,
        compute_owners: None,
        direct_fused_conv: false,
        fixed_batch: false,
        iterations: 100,
        metal_gemm: None,
        nnef_model: None,
        plan_pool: None,
        pool_routing: PoolRouting::default(),
        plan_summary: false,
        profile_plan: false,
        reverse: false,
        tract_threads: default_tract_threads(),
        verify_batches: false,
        verify_only: false,
    };
    let mut args = std::env::args().skip(1);
    while let Some(argument) = args.next() {
        match argument.as_str() {
            "--backend" => {
                let backend = args.next().context("--backend needs a value")?;
                ensure!(
                    matches!(backend.as_str(), "ort" | "cpu" | "metal"),
                    "--backend must be one of ort, cpu, or metal"
                );
                options.backend = Some(backend);
            }
            "--batch" => {
                options.batch = args.next().context("--batch needs a value")?.parse()?;
            }
            "--batch-sweep" => options.batch_sweep = true,
            "--compute-owners" => {
                options.compute_owners =
                    Some(args.next().context("--compute-owners needs a value")?.parse()?);
            }
            "--direct-fused-conv" => options.direct_fused_conv = true,
            "--fixed-batch" => options.fixed_batch = true,
            "--iterations" => {
                options.iterations = args.next().context("--iterations needs a value")?.parse()?;
            }
            "--metal-gemm" => {
                let gemm = args.next().context("--metal-gemm needs a value")?;
                ensure!(
                    matches!(gemm.as_str(), "auto" | "mlx" | "mfa" | "ggml"),
                    "--metal-gemm must be one of auto, mlx, mfa, or ggml"
                );
                options.metal_gemm = Some(gemm);
            }
            "--nnef-model" => {
                options.nnef_model =
                    Some(PathBuf::from(args.next().context("--nnef-model needs a path")?));
            }
            "--plan-pool" => {
                let value = args.next().context("--plan-pool needs comma-separated classes")?;
                let mut classes = value
                    .split(',')
                    .map(str::parse::<usize>)
                    .collect::<std::result::Result<Vec<_>, _>>()?;
                ensure!(!classes.is_empty(), "--plan-pool needs at least one class");
                ensure!(classes.iter().all(|class| *class > 0), "plan classes must be positive");
                classes.sort_unstable();
                classes.dedup();
                options.plan_pool = Some(classes);
            }
            "--pool-routing" => {
                options.pool_routing = match args.next().as_deref() {
                    Some("ceil") => PoolRouting::Ceil,
                    Some("exact") => PoolRouting::Exact,
                    Some(value) => bail!("unknown pool routing: {value}"),
                    None => bail!("--pool-routing needs ceil or exact"),
                };
            }
            "--plan-summary" => options.plan_summary = true,
            "--profile-plan" => options.profile_plan = true,
            "--tract-threads" => {
                options.tract_threads =
                    args.next().context("--tract-threads needs a value")?.parse()?;
                ensure!(options.tract_threads > 0, "--tract-threads must be greater than zero");
            }
            "--reverse" => options.reverse = true,
            "--verify-batches" => options.verify_batches = true,
            "--verify" => options.verify_only = true,
            "-h" | "--help" => {
                print_help();
                std::process::exit(0);
            }
            _ => bail!("unknown argument: {argument}"),
        }
    }
    #[cfg(not(target_os = "macos"))]
    {
        ensure!(
            options.backend.as_deref() != Some("metal"),
            "the Metal backend is available only on macOS"
        );
        ensure!(options.metal_gemm.is_none(), "--metal-gemm is available only on macOS");
    }
    #[cfg(all(target_os = "macos", not(feature = "metal")))]
    ensure!(
        options.backend.as_deref() != Some("metal"),
        "the Metal backend requires building with --features metal"
    );
    Ok(options)
}

fn print_help() {
    #[cfg(target_os = "macos")]
    const BACKENDS: &str = "ort|cpu|metal";
    #[cfg(not(target_os = "macos"))]
    const BACKENDS: &str = "ort|cpu";
    #[cfg(target_os = "macos")]
    const METAL: &str = " [--metal-gemm auto|mlx|mfa|ggml]";
    #[cfg(not(target_os = "macos"))]
    const METAL: &str = "";

    println!(
        "usage: magika-runtime-bench [--verify] [--reverse] \
         [--backend {BACKENDS}] [--batch N] \
         [--batch-sweep] [--compute-owners N] [--direct-fused-conv] \
         [--fixed-batch] [--iterations N]{METAL} [--nnef-model PATH] [--plan-summary] \
         [--plan-pool 1,8,16,32,64] [--pool-routing exact|ceil] [--profile-plan] \
         [--tract-threads N] [--verify-batches]"
    );
}

fn wants_backend(options: &Options, name: &str) -> bool {
    options.backend.as_deref().is_none_or(|selected| selected == name)
}

fn print_plan_summaries(backends: &[(Box<dyn Backend>, Duration)]) {
    for (backend, _) in backends {
        let Some(counts) = backend.plan_op_counts() else { continue };
        println!("plan_nodes\t{}\t{}", backend.name(), counts.values().sum::<usize>());
        for (op, count) in counts {
            println!("plan_op\t{}\t{op}\t{count}", backend.name());
        }
    }
}

fn print_routes(backends: &[(Box<dyn Backend>, Duration)], batch: usize) {
    for (backend, _) in backends {
        if let Some(classes) = backend.selected_classes(batch) {
            let classes = classes.iter().map(usize::to_string).collect::<Vec<_>>().join("+");
            println!("plan_route\t{}\trequest={batch}\tclasses={classes}", backend.name());
        }
    }
}

fn print_plan_profiles(
    backends: &mut [(Box<dyn Backend>, Duration)], input: &[i32], batch: usize,
) -> Result<()> {
    for (backend, _) in backends {
        let Some(samples) = backend.profile_plan(input, batch)? else { continue };
        let total = samples.iter().map(|(_, _, elapsed)| *elapsed).sum::<Duration>();
        println!("profile_total\t{}\t{}", backend.name(), micros(total));
        for (name, op, elapsed) in samples.into_iter().take(12) {
            let share = elapsed.as_secs_f64() / total.as_secs_f64() * 100.0;
            println!(
                "profile_node\t{}\t{}\t{share:.2}\t{op}\t{name}",
                backend.name(),
                micros(elapsed)
            );
        }
    }
    Ok(())
}

fn default_tract_threads() -> usize {
    std::thread::available_parallelism().map_or(1, |parallelism| parallelism.get().min(2))
}

#[cfg(feature = "tract-runtime")]
fn configure_tract_codegen() {
    if std::env::var_os("TRACT_LAZY_IM2COL_MIN_KERNEL").is_none() {
        // No worker threads exist yet, and tract reads this process-wide codegen knob only while
        // preparing a model. Users can still override it before launching the benchmark.
        unsafe {
            std::env::set_var("TRACT_LAZY_IM2COL_MIN_KERNEL", MAGIKA_LAZY_IM2COL_MIN_KERNEL);
        }
    }
}

#[cfg(not(feature = "tract-runtime"))]
fn configure_tract_codegen() {}

fn micros(duration: Duration) -> u128 {
    duration.as_micros()
}
