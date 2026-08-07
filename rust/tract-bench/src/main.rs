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

use std::path::Path;
use std::time::{Duration, Instant};

use anyhow::{Context, Result, bail, ensure};

#[cfg(feature = "ort-runtime")]
use ndarray::Array2;
#[cfg(feature = "tract-runtime")]
use tract_core::prelude::{
    Framework as _, IntoTValue as _, IntoTensor as _, TValue, TVec, Tensor, tvec,
};
#[cfg(feature = "tract-runtime")]
use tract_core::runtime::{DefaultRuntime, RunOptions, State};
#[cfg(feature = "tract-runtime")]
use tract_core::tract_linalg::multithread::Executor;

#[cfg(all(feature = "metal", target_vendor = "apple"))]
use tract_metal as _;

const FEATURE_SIZE: usize = 2048;
const PADDING_TOKEN: i32 = 256;

#[cfg(feature = "ort-runtime")]
const ONNX_MODEL: &[u8] = include_bytes!("../../../assets/models/standard_v3_3/model.onnx");
#[cfg(feature = "tract-runtime")]
const NNEF_MODEL: &[u8] = include_bytes!("../models/model.nnef.tgz");

trait Backend {
    fn name(&self) -> &'static str;
    fn run(&mut self, input: &[i32], batch: usize) -> Result<Vec<f32>>;
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
    fn load_cpu(threads: usize) -> Result<Self> {
        static CPU: DefaultRuntime = DefaultRuntime;
        let options =
            RunOptions { executor: Some(Executor::multithread(threads)), ..RunOptions::default() };
        Self::load_with_runtime_and_options("tract-cpu", &CPU, Some(&options))
    }

    #[cfg(all(feature = "metal", target_vendor = "apple"))]
    fn load_with_runtime(
        name: &'static str, runtime: &'static dyn tract_core::runtime::Runtime,
    ) -> Result<Self> {
        Self::load_with_runtime_and_options(name, runtime, None)
    }

    fn load_with_runtime_and_options(
        name: &'static str, runtime: &'static dyn tract_core::runtime::Runtime,
        options: Option<&RunOptions>,
    ) -> Result<Self> {
        let model = tract_nnef::nnef()
            .model_for_read(&mut std::io::Cursor::new(NNEF_MODEL))
            .context("loading the embedded NNEF model")?;
        let runnable = match options {
            Some(options) => runtime.prepare_with_options(model, options),
            None => runtime.prepare(model),
        }
        .with_context(|| format!("preparing {name}"))?;
        let state = runnable.spawn().with_context(|| format!("spawning {name} state"))?;
        Ok(Self { name, state })
    }

    #[cfg(all(feature = "metal", target_vendor = "apple"))]
    fn load_metal() -> Result<Self> {
        let runtime = tract_core::runtime::runtime_for_name("metal")?
            .context("the tract Metal runtime was not registered")?;
        Self::load_with_runtime("tract-metal", runtime)
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
}

#[derive(Debug)]
struct Options {
    batch: usize,
    iterations: usize,
    reverse: bool,
    tract_threads: usize,
    verify_only: bool,
}

fn main() -> Result<()> {
    let options = parse_options()?;
    ensure!(options.batch > 0, "--batch must be greater than zero");
    ensure!(options.iterations > 0, "--iterations must be greater than zero");
    #[cfg(feature = "tract-runtime")]
    ensure!(options.batch == 1, "the spike's NNEF model is specialized to --batch 1");

    #[cfg(feature = "ort-runtime")]
    ort::init().with_telemetry(false).commit();

    let corpus = load_corpus(options.batch)?;
    let mut backends: Vec<(Box<dyn Backend>, Duration)> = Vec::new();

    #[cfg(feature = "ort-runtime")]
    load_backend(&mut backends, OrtBackend::load)?;
    #[cfg(feature = "tract-runtime")]
    load_backend(&mut backends, || TractBackend::load_cpu(options.tract_threads))?;
    #[cfg(all(feature = "metal", target_vendor = "apple"))]
    load_backend(&mut backends, TractBackend::load_metal)?;
    if options.reverse {
        backends.reverse();
    }

    ensure!(!backends.is_empty(), "enable at least one runtime feature");
    let (outputs, first_runs) = verify(&mut backends, &corpus, options.batch)?;
    print_header(&backends);
    #[cfg(feature = "tract-runtime")]
    println!("tract_cpu_threads\t{}", options.tract_threads);
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
    println!(
        "result\t{}\t{}\t{}\t{}\t{}\t{}",
        backend.name(),
        micros(cold),
        micros(first),
        micros(mean),
        micros(p50),
        micros(p95)
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
    println!("columns\tbackend\tcold_us\tfirst_us\tmean_us\tp50_us\tp95_us");
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
        batch: 1,
        iterations: 100,
        reverse: false,
        tract_threads: default_tract_threads(),
        verify_only: false,
    };
    let mut args = std::env::args().skip(1);
    while let Some(argument) = args.next() {
        match argument.as_str() {
            "--batch" => {
                options.batch = args.next().context("--batch needs a value")?.parse()?;
            }
            "--iterations" => {
                options.iterations = args.next().context("--iterations needs a value")?.parse()?;
            }
            "--tract-threads" => {
                options.tract_threads =
                    args.next().context("--tract-threads needs a value")?.parse()?;
                ensure!(options.tract_threads > 0, "--tract-threads must be greater than zero");
            }
            "--reverse" => options.reverse = true,
            "--verify" => options.verify_only = true,
            "-h" | "--help" => {
                println!(
                    "usage: magika-runtime-bench [--verify] [--reverse] [--batch N] \
                     [--iterations N] [--tract-threads N]"
                );
                std::process::exit(0);
            }
            _ => bail!("unknown argument: {argument}"),
        }
    }
    Ok(options)
}

fn default_tract_threads() -> usize {
    std::thread::available_parallelism().map_or(1, |parallelism| parallelism.get().min(2))
}

fn micros(duration: Duration) -> u128 {
    duration.as_micros()
}
