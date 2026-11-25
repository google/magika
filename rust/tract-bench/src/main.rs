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

use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::{Context, Result, bail, ensure};
use magika_tract_runtime::{Backend, BackendRequest, Runtime, Session};

const FEATURE_SIZE: usize = 2048;
const NUM_LABELS: usize = 214;
const PADDING_TOKEN: usize = 256;

const USAGE: &str = "usage: magika-runtime-bench \
    [--backend auto|cpu|gpu] [--batch N] [--iterations N] [--threads N]";

#[derive(Debug)]
struct Options {
    backend: BackendRequest,
    batch: usize,
    iterations: usize,
    threads: Option<usize>,
}

impl Default for Options {
    fn default() -> Self {
        Self { backend: BackendRequest::Auto, batch: 8, iterations: 100, threads: None }
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
    let preparing = Instant::now();
    let runtime = Arc::new(Runtime::with_max_batch(options.backend, options.batch)?);
    let shared_prepare = preparing.elapsed();
    let info = runtime.backend_info();
    let threads = options.threads.unwrap_or_else(|| default_threads(info.backend()));
    bench(
        info.implementation(),
        options.batch,
        options.iterations,
        threads,
        shared_prepare,
        &input,
        || runtime.session(),
    )
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

fn bench(
    backend: &str, batch: usize, iterations: usize, threads: usize, shared_prepare: Duration,
    input: &[i32], make_session: impl Fn() -> Result<Session> + Sync,
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
                let prepared = (|| -> Result<Session> {
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
                    "auto" => BackendRequest::Auto,
                    "cpu" => BackendRequest::Cpu,
                    "gpu" => BackendRequest::Gpu,
                    backend => bail!("unknown backend {backend}"),
                }
            }
            "--batch" => options.batch = value()?.parse().context("invalid --batch")?,
            "--iterations" => {
                options.iterations = value()?.parse().context("invalid --iterations")?
            }
            "--threads" => options.threads = Some(value()?.parse().context("invalid --threads")?),
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
        assert_eq!(options.backend, BackendRequest::Gpu);
        assert_eq!(options.batch, 8);
        assert_eq!(options.threads, Some(4));
        assert_eq!(options.iterations, 100);
    }

    #[test]
    fn rejects_an_unknown_option() {
        assert!(parse_options(["--unknown"]).is_err());
    }
}
