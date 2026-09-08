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

//! Isolated throughput benchmark for the inference runtime shipped by Magika.

#[cfg(feature = "ort-runtime")]
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::{Context, Result, bail, ensure};
use magika_tract_runtime::{Backend, BackendRequest, Runtime};
#[cfg(feature = "ort-runtime")]
use ndarray::Array2;

const FEATURE_SIZE: usize = 2048;
const NUM_LABELS: usize = 214;
const PADDING_TOKEN: usize = 256;

const USAGE: &str = "usage: magika-runtime-bench \
    [--backend auto|cpu|gpu|ort] [--batch N] [--iterations N] [--threads N] \
    [--onnx-model PATH] [--ort-intra-threads N] [--ort-inter-threads N]";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RequestedBackend {
    Auto,
    Cpu,
    Gpu,
    Ort,
}

#[derive(Debug)]
struct Options {
    backend: RequestedBackend,
    batch: usize,
    iterations: usize,
    threads: Option<usize>,
    onnx_model: Option<PathBuf>,
    ort_intra_threads: usize,
    ort_inter_threads: usize,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            backend: RequestedBackend::Auto,
            batch: 8,
            iterations: 100,
            threads: None,
            onnx_model: None,
            ort_intra_threads: 1,
            ort_inter_threads: 1,
        }
    }
}

trait InferenceSession {
    fn run(&mut self, input: &[i32], batch: usize) -> Result<Vec<f32>>;
}

impl InferenceSession for magika_tract_runtime::Session {
    fn run(&mut self, input: &[i32], batch: usize) -> Result<Vec<f32>> {
        self.run(input, batch)
    }
}

#[cfg(feature = "ort-runtime")]
struct OrtSession {
    session: ort::session::Session,
}

#[cfg(feature = "ort-runtime")]
impl OrtSession {
    fn load(model: Option<&Path>, intra_threads: usize, inter_threads: usize) -> Result<Self> {
        let mut builder = ort::session::Session::builder()?
            .with_intra_threads(intra_threads)
            .map_err(|error| anyhow::anyhow!(error.to_string()))?
            .with_inter_threads(inter_threads)
            .map_err(|error| anyhow::anyhow!(error.to_string()))?;
        let session = match model {
            Some(path) => builder
                .commit_from_file(path)
                .with_context(|| format!("loading ONNX model {}", path.display()))?,
            None => builder
                .commit_from_memory(include_bytes!(
                    "../../../assets/models/standard_v3_3/model.onnx"
                ))
                .context("loading the embedded ONNX model")?,
        };
        Ok(Self { session })
    }
}

#[cfg(feature = "ort-runtime")]
impl InferenceSession for OrtSession {
    fn run(&mut self, input: &[i32], batch: usize) -> Result<Vec<f32>> {
        let input = Array2::from_shape_vec([batch, FEATURE_SIZE], input.to_vec())?;
        let input = ort::value::Tensor::from_array(input)?;
        let mut output = self.session.run(ort::inputs!("bytes" => input))?;
        let output =
            output.remove("target_label").context("ONNX output target_label is missing")?;
        Ok(output.try_extract_array::<f32>()?.iter().copied().collect())
    }
}

fn main() -> Result<()> {
    let Some(options) = parse_options(std::env::args_os().skip(1))? else {
        println!("{USAGE}");
        return Ok(());
    };
    ensure!(options.batch > 0, "--batch must be greater than zero");
    ensure!(options.iterations > 0, "--iterations must be greater than zero");
    ensure!(options.threads != Some(0), "--threads must be greater than zero");

    let input_len = options.batch.checked_mul(FEATURE_SIZE).context("--batch is too large")?;
    let input =
        (0..input_len).map(|index| (index % (PADDING_TOKEN + 1)) as i32).collect::<Vec<_>>();
    match options.backend {
        RequestedBackend::Ort => run_ort(&options, &input),
        request => run_tract(&options, request, &input),
    }
}

fn run_tract(options: &Options, request: RequestedBackend, input: &[i32]) -> Result<()> {
    let request = match request {
        RequestedBackend::Auto => BackendRequest::Auto,
        RequestedBackend::Cpu => BackendRequest::Cpu,
        RequestedBackend::Gpu => BackendRequest::Gpu,
        RequestedBackend::Ort => unreachable!(),
    };
    let preparing = Instant::now();
    let runtime = Arc::new(Runtime::with_max_batch(request, options.batch)?);
    let shared_prepare = preparing.elapsed();
    let info = runtime.backend_info();
    let threads = options.threads.unwrap_or_else(|| default_threads(info.backend()));
    bench(
        info.implementation(),
        options.batch,
        options.iterations,
        threads,
        shared_prepare,
        input,
        || runtime.session(),
    )
}

#[cfg(feature = "ort-runtime")]
fn run_ort(options: &Options, input: &[i32]) -> Result<()> {
    ort::init().with_telemetry(false).commit();
    let threads = options.threads.unwrap_or_else(host_threads);
    bench("onnx-runtime", options.batch, options.iterations, threads, Duration::ZERO, input, || {
        OrtSession::load(
            options.onnx_model.as_deref(),
            options.ort_intra_threads,
            options.ort_inter_threads,
        )
    })
}

#[cfg(not(feature = "ort-runtime"))]
fn run_ort(_options: &Options, _input: &[i32]) -> Result<()> {
    bail!("this build does not include ONNX Runtime")
}

fn default_threads(backend: Backend) -> usize {
    match backend {
        Backend::Cpu => host_threads(),
        Backend::Gpu => 4,
    }
}

fn host_threads() -> usize {
    std::thread::available_parallelism().map(usize::from).unwrap_or(1).saturating_sub(1).max(1)
}

fn bench<S: InferenceSession>(
    backend: &str, batch: usize, iterations: usize, threads: usize, shared_prepare: Duration,
    input: &[i32], make_session: impl Fn() -> Result<S> + Sync,
) -> Result<()> {
    let (thread_prepare, elapsed) = std::thread::scope(|scope| -> Result<(Duration, Duration)> {
        let preparing = Instant::now();
        let (ready_sender, ready_receiver) = std::sync::mpsc::channel();
        let mut start_senders = Vec::with_capacity(threads);
        let mut handles = Vec::with_capacity(threads);
        for _ in 0..threads {
            let (start_sender, start_receiver) = std::sync::mpsc::sync_channel(0);
            start_senders.push(start_sender);
            let ready_sender = ready_sender.clone();
            let make_session = &make_session;
            handles.push(scope.spawn(move || -> Result<()> {
                let prepared = (|| -> Result<S> {
                    let mut session = make_session()?;
                    for _ in 0..10 {
                        validate_output(&session.run(input, batch)?, batch)?;
                    }
                    Ok(session)
                })();
                let ready = prepared.as_ref().map(|_| ()).map_err(|error| format!("{error:#}"));
                ready_sender.send(ready).ok();
                let mut session = prepared?;
                if start_receiver.recv().is_err() {
                    return Ok(());
                }
                for _ in 0..iterations {
                    validate_output(&session.run(input, batch)?, batch)?;
                }
                Ok(())
            }));
        }
        drop(ready_sender);
        let mut preparation_error = None;
        for _ in 0..threads {
            match ready_receiver.recv().context("inference thread stopped during preparation")? {
                Ok(()) => {}
                Err(error) => {
                    preparation_error.get_or_insert(error);
                }
            }
        }
        let thread_prepare = preparing.elapsed();
        if let Some(error) = preparation_error {
            drop(start_senders);
            bail!("{error}");
        }
        let started = Instant::now();
        for sender in start_senders {
            sender.send(()).context("inference thread stopped before the timed run")?;
        }
        for handle in handles {
            handle.join().map_err(|_| anyhow::anyhow!("inference thread panicked"))??;
        }
        Ok((thread_prepare, started.elapsed()))
    })?;
    let files = batch
        .checked_mul(iterations)
        .and_then(|files| files.checked_mul(threads))
        .context("benchmark workload is too large")?;
    let files_per_second = files as f64 / elapsed.as_secs_f64();
    println!(
        "backend\t{backend}\nworkload\tthreads={threads}\tbatch={batch}\titerations_per_thread={iterations}\ttotal_files={files}"
    );
    println!(
        "startup\tshared_prepare_us={}\tthread_prepare_and_warm_us={}",
        shared_prepare.as_micros(),
        thread_prepare.as_micros()
    );
    println!("result\twall_us={}\tfiles_per_second={files_per_second:.0}", elapsed.as_micros());
    Ok(())
}

fn validate_output(output: &[f32], batch: usize) -> Result<()> {
    let expected = batch.checked_mul(NUM_LABELS).context("benchmark batch is too large")?;
    ensure!(output.len() == expected, "backend returned an invalid output length");
    ensure!(output.iter().all(|score| score.is_finite()), "backend returned a non-finite score");
    Ok(())
}

fn parse_options(
    args: impl IntoIterator<Item = impl Into<std::ffi::OsString>>,
) -> Result<Option<Options>> {
    let mut options = Options::default();
    let mut args = args.into_iter().map(|arg| arg.into());
    while let Some(arg) = args.next() {
        let arg = arg.to_string_lossy();
        let mut value = || {
            args.next()
                .with_context(|| format!("missing value after {arg}"))
                .map(|value| value.to_string_lossy().into_owned())
        };
        match arg.as_ref() {
            "-h" | "--help" => return Ok(None),
            "--backend" => {
                options.backend = match value()?.as_str() {
                    "auto" => RequestedBackend::Auto,
                    "cpu" => RequestedBackend::Cpu,
                    "gpu" => RequestedBackend::Gpu,
                    "ort" => RequestedBackend::Ort,
                    backend => bail!("unknown backend {backend}"),
                }
            }
            "--batch" => options.batch = value()?.parse().context("invalid --batch")?,
            "--iterations" => {
                options.iterations = value()?.parse().context("invalid --iterations")?
            }
            "--threads" => options.threads = Some(value()?.parse().context("invalid --threads")?),
            "--onnx-model" => options.onnx_model = Some(PathBuf::from(value()?)),
            "--ort-intra-threads" => {
                options.ort_intra_threads =
                    value()?.parse().context("invalid --ort-intra-threads")?
            }
            "--ort-inter-threads" => {
                options.ort_inter_threads =
                    value()?.parse().context("invalid --ort-inter-threads")?
            }
            option => bail!("unknown option {option}\n{USAGE}"),
        }
    }
    Ok(Some(options))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_the_release_benchmark_shape() {
        let options = parse_options([
            "--backend",
            "gpu",
            "--batch",
            "8",
            "--threads",
            "4",
            "--iterations",
            "100",
        ])
        .unwrap()
        .unwrap();
        assert_eq!(options.backend, RequestedBackend::Gpu);
        assert_eq!(options.batch, 8);
        assert_eq!(options.threads, Some(4));
        assert_eq!(options.iterations, 100);
    }

    #[test]
    fn rejects_an_unknown_option() {
        assert!(parse_options(["--unknown"]).is_err());
    }
}
