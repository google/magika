// Copyright 2024 Google LLC
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

use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt::Write;
use std::io::{ErrorKind, Write as _};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::thread::JoinHandle;

use anyhow::{ensure, Context as _, Result};
use clap::{Args, Parser, ValueEnum};
use colored::ColoredString;
use magika::{
    self, Backend, BackendRequest, ContentType, Features, FeaturesOrRuled, FileType, InferredType,
    OverwriteReason, Runtime, Session, TypeInfo,
};
use serde::Serialize;
use tokio::io::AsyncReadExt;

/// Determines file content types using AI.
#[derive(Parser)]
#[command(name = "magika", version = Version, arg_required_else_help = true)]
struct Flags {
    /// List of paths to the files to analyze.
    ///
    /// Use a dash (-) to read from standard input (can only be used once).
    path: Vec<PathBuf>,

    /// Identifies files within directories instead of identifying the directory itself.
    #[arg(short, long)]
    recursive: bool,

    /// Identifies symbolic links as is instead of identifying their content by following them.
    #[arg(long)]
    no_dereference: bool,

    #[clap(flatten)]
    colors: Colors,

    #[clap(flatten)]
    modifiers: Modifiers,

    #[clap(flatten)]
    format: Format,

    #[clap(flatten)]
    compute: Compute,

    /// Reports the selected inference device and implementation.
    #[arg(short, long)]
    verbose: bool,
}

struct Version;
impl clap::builder::IntoResettable<clap::builder::Str> for Version {
    fn into_resettable(self) -> clap::builder::Resettable<clap::builder::Str> {
        let binary = clap::crate_version!();
        let model = magika::MODEL_NAME;
        clap::builder::Resettable::Value(format!("{binary} {model}").into())
    }
}

#[derive(Args)]
#[group(multiple = false)]
struct Colors {
    /// Prints with colors regardless of terminal support.
    #[arg(long = "colors")]
    enable: bool,

    /// Prints without colors regardless of terminal support.
    #[arg(long = "no-colors")]
    disable: bool,
}

#[derive(Args)]
#[group(conflicts_with = "format")]
struct Modifiers {
    /// Prints the prediction score in addition to the content type.
    #[arg(short = 's', long)]
    output_score: bool,

    /// Prints the MIME type instead of the content type description.
    #[arg(short = 'i', long)]
    mime_type: bool,

    /// Prints a simple label instead of the content type description.
    #[arg(short, long, conflicts_with = "mime_type")]
    label: bool,
}

#[derive(Args)]
#[group(id = "format", multiple = false)]
struct Format {
    /// Prints in JSON format.
    #[arg(long)]
    json: bool,

    /// Prints in JSONL format.
    #[arg(long)]
    jsonl: bool,

    /// Prints using a custom format (use --help for details).
    ///
    /// The following placeholders are supported:
    ///
    ///   %p  The file path
    ///   %l  The unique label identifying the content type
    ///   %d  The description of the content type
    ///   %g  The group of the content type
    ///   %m  The MIME type of the content type
    ///   %e  Possible file extensions for the content type
    ///   %s  The score of the content type for the file
    ///   %S  The score of the content type for the file in percent
    ///   %b  The model output if overruled (empty otherwise)
    ///   %%  A literal %
    #[arg(long = "format", verbatim_doc_comment)]
    custom: Option<String>,
}

#[derive(Args)]
struct Compute {
    /// Selects the inference device.
    #[arg(long, value_enum, default_value_t = BackendChoice::Auto)]
    backend: BackendChoice,

    /// Number of files to accumulate before dispatching inference.
    #[arg(long, default_value = "8")]
    batch_size: usize,

    /// Number of resident inference threads.
    ///
    /// Inference on a GPU is bound by the device rather than by the host, so a handful of threads
    /// keep it busy and more only contend for it. Inference on a CPU is bound by the host, so every
    /// thread is one more core doing the work. This defaults accordingly: four on a GPU, and one
    /// less than the available parallelism on a CPU.
    #[arg(long, alias = "num-tasks")]
    threads: Option<usize>,

    /// Prints per-stage busy and waiting time to standard error when the run finishes.
    #[arg(long)]
    trace_utilization: bool,

    /// Number of resident threads reading files and extracting features.
    ///
    /// Reading costs far less than inference, so this defaults to one per inference thread, which
    /// is already more than a run makes use of.
    #[arg(long)]
    readers: Option<usize>,
}

/// Per-stage busy and waiting time, empty unless --trace-utilization is set.
#[derive(Default)]
struct Trace {
    stages: Vec<Stage>,
}

struct Stage {
    name: String,
    busy_ns: std::sync::atomic::AtomicU64,
    wait_ns: std::sync::atomic::AtomicU64,
}

impl Trace {
    fn new(enabled: bool, names: impl IntoIterator<Item = String>) -> Self {
        let stages = if enabled {
            names
                .into_iter()
                .map(|name| Stage {
                    name,
                    busy_ns: std::sync::atomic::AtomicU64::new(0),
                    wait_ns: std::sync::atomic::AtomicU64::new(0),
                })
                .collect()
        } else {
            Vec::new()
        };
        Trace { stages }
    }

    fn enabled(&self) -> bool {
        !self.stages.is_empty()
    }

    fn start(&self) -> Option<std::time::Instant> {
        self.enabled().then(std::time::Instant::now)
    }

    fn add(&self, stage: usize, started: Option<std::time::Instant>, busy: bool) {
        let Some(started) = started else { return };
        let Some(stage) = self.stages.get(stage) else { return };
        let counter = if busy { &stage.busy_ns } else { &stage.wait_ns };
        counter
            .fetch_add(started.elapsed().as_nanos() as u64, std::sync::atomic::Ordering::Relaxed);
    }

    fn report(&self) {
        if !self.enabled() {
            return;
        }
        eprintln!("trace  stage           busy      waiting   busy%");
        for stage in &self.stages {
            let busy = stage.busy_ns.load(std::sync::atomic::Ordering::Relaxed);
            let wait = stage.wait_ns.load(std::sync::atomic::Ordering::Relaxed);
            let total = busy + wait;
            let share = if total == 0 { 0.0 } else { busy as f64 / total as f64 * 100.0 };
            eprintln!(
                "trace  {:<14} {:>7.3}s  {:>7.3}s  {:>5.1}%",
                stage.name,
                busy as f64 / 1e9,
                wait as f64 / 1e9,
                share
            );
        }
    }
}

#[derive(Clone, Copy, Debug, Default, ValueEnum)]
enum BackendChoice {
    #[default]
    Auto,
    Cpu,
    Gpu,
}

impl From<BackendChoice> for BackendRequest {
    fn from(backend: BackendChoice) -> Self {
        match backend {
            BackendChoice::Auto => BackendRequest::Auto,
            BackendChoice::Cpu => BackendRequest::Cpu,
            BackendChoice::Gpu => BackendRequest::Gpu,
        }
    }
}

/// Inference threads it takes to keep a GPU queued.
///
/// The device is the bottleneck there, and it is already busy well before the host runs out of
/// cores, so further threads only queue behind each other. Measured on an M5 Max, throughput is
/// flat from four threads to sixteen.
const GPU_INFERENCE_THREADS: usize = 4;

/// Returns how many inference threads it takes to keep the resolved backend busy.
///
/// `available_parallelism` is the portable answer on every target magika ships to, and it reports
/// what this process may use rather than what the machine is built from, so a container's CPU quota
/// and a restricted affinity mask both count.
fn default_inference_threads(backend: Backend) -> usize {
    let available = std::thread::available_parallelism().map_or(1, std::num::NonZeroUsize::get);
    match backend {
        // The device is the limit, not the host, so never ask the host for more than it takes to
        // keep the device queued, nor for more than it has.
        Backend::Gpu => available.min(GPU_INFERENCE_THREADS),
        // Every thread is one more core identifying files, so take the machine and leave one core.
        // The rest of the pipeline walks and prints on single threads of its own whatever the
        // machine size, and a run this long should not make the machine stop answering.
        Backend::Cpu => available.saturating_sub(1).max(1),
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    start().await
}

async fn start() -> Result<()> {
    let flags = Flags::parse();
    ensure!(0 < flags.compute.batch_size, "--batch-size cannot be zero");
    ensure!(flags.compute.threads != Some(0), "--threads cannot be zero");
    ensure!(flags.compute.readers != Some(0), "--readers cannot be zero");
    ensure!(
        flags.path.iter().filter(|x| x.to_str() == Some("-")).count() <= 1,
        "only one path can be the standard input"
    );
    let flags = Arc::new(flags);
    if flags.colors.enable {
        colored::control::set_override(true);
    }
    if flags.colors.disable {
        colored::control::set_override(false);
    }
    // The accumulator never emits more than one batch worth of files, so the larger resident
    // classes are unreachable and preparing them would only cost startup.
    let runtime = Arc::new(
        Session::builder()
            .with_backend(flags.compute.backend.into())
            .with_max_batch(flags.compute.batch_size)
            .build_runtime()?,
    );
    let threads = match flags.compute.threads {
        Some(threads) => threads,
        None => default_inference_threads(runtime.backend_info().backend()),
    };
    let readers = flags.compute.readers.unwrap_or(threads);
    if flags.verbose {
        let info = runtime.backend_info();
        let backend = match info.backend() {
            Backend::Cpu => "cpu",
            Backend::Gpu => "gpu",
        };
        eprintln!("backend: {backend} ({})", info.implementation());
    }
    let result_capacity = threads
        .checked_mul(flags.compute.batch_size)
        .context("--threads times --batch-size is too large")?;
    let (result_sender, result_receiver) =
        tokio::sync::mpsc::channel::<Result<Response>>(result_capacity);
    let (batch_sender, batch_receiver) = async_channel::bounded::<InferenceBatch>(threads);
    let trace = Arc::new(Trace::new(
        flags.compute.trace_utilization,
        std::iter::once("walk".to_string())
            .chain((0..readers).map(|i| format!("read[{i}]")))
            .chain(std::iter::once("batch".to_string()))
            .chain((0..threads).map(|i| format!("infer[{i}]"))),
    ));
    let reader_stage = 1;
    let accumulator_stage = reader_stage + readers;
    let infer_stage = accumulator_stage + 1;
    // Keep enough paths queued that no reader ever waits on traversal: one batch being filled and
    // one waiting behind it, for every reader.
    let work_capacity = readers
        .checked_mul(flags.compute.batch_size)
        .and_then(|x| x.checked_mul(2))
        .context("--readers times --batch-size is too large")?;
    // A permit follows each non-recursive path until its ordered response is printed. This bounds
    // the reorder buffer without starving the accumulator before it can fill a complete batch.
    let result_window = Arc::new(tokio::sync::Semaphore::new(work_capacity));
    let (work_sender, work_receiver) = async_channel::bounded::<Work>(work_capacity);
    let (read_sender, read_receiver) = async_channel::bounded::<ReadItem>(work_capacity);
    let walker = tokio::spawn({
        let flags = flags.clone();
        let result_sender = result_sender.clone();
        let result_window = result_window.clone();
        let trace = trace.clone();
        async move {
            if let Err(e) =
                walk_paths(&flags, &work_sender, &result_sender, result_window, &trace).await
            {
                let _ = result_sender.send(Err(e)).await;
            }
        }
    });
    let batch_size = flags.compute.batch_size;
    let accumulator = tokio::spawn({
        let batch_sender = batch_sender.clone();
        let result_sender = result_sender.clone();
        let trace = trace.clone();
        async move {
            if let Err(error) = accumulate_files(
                batch_size,
                &read_receiver,
                &batch_sender,
                &result_sender,
                &trace,
                accumulator_stage,
            )
            .await
            {
                let _ = result_sender.send(Err(error)).await;
            }
        }
    });
    let read_threads =
        spawn_read_threads(readers, &work_receiver, &read_sender, &trace, reader_stage)?;
    drop(work_receiver);
    drop(read_sender);
    drop(batch_sender);
    let inference_threads = spawn_inference_threads(
        threads,
        runtime,
        &batch_receiver,
        &result_sender,
        &trace,
        infer_stage,
    )?;
    drop(batch_receiver);
    drop(result_sender);
    let print_result = match print(&flags, result_receiver).await {
        Err(e)
            if e.root_cause()
                .downcast_ref::<std::io::Error>()
                .is_some_and(|x| x.kind() == std::io::ErrorKind::BrokenPipe) =>
        {
            Ok(())
        }
        x => x,
    };
    walker.await?;
    accumulator.await?;
    for thread in read_threads {
        thread.join().map_err(|_| anyhow::anyhow!("read thread panicked"))?;
    }
    for thread in inference_threads {
        thread.join().map_err(|_| anyhow::anyhow!("inference thread panicked"))?;
    }
    trace.report();
    print_result
}

async fn print(
    flags: &Flags, mut result_receiver: tokio::sync::mpsc::Receiver<Result<Response>>,
) -> Result<()> {
    let mut stdout = std::io::stdout().lock();
    if flags.format.json {
        write!(stdout, "[")?;
    }
    let mut reorder = Reorder::default();
    let mut errors = false;
    while let Some(response) = result_receiver.recv().await {
        reorder.push(response?);
        while let Some(response) = reorder.pop() {
            errors |= response.result.is_err();
            if flags.format.json {
                if reorder.next != 1 {
                    write!(stdout, ",")?;
                }
                for line in serde_json::to_string_pretty(&response.json()?)?.lines() {
                    write!(stdout, "\n  {line}")?;
                }
            } else {
                writeln!(stdout, "{}", response.format(flags)?)?;
            }
        }
    }
    debug_assert!(reorder.is_empty());
    if flags.format.json {
        if reorder.next != 0 {
            writeln!(stdout)?;
        }
        writeln!(stdout, "]")?;
    }
    if errors {
        std::process::exit(1);
    }
    Ok(())
}

/// Walks the requested paths and hands regular files to the read threads.
///
/// This task only traverses and stats. Reading file content is left to [`read_files`] so that it
/// happens on several threads at once instead of serializing behind traversal.
async fn walk_paths(
    flags: &Flags, work_sender: &async_channel::Sender<Work>,
    result_sender: &tokio::sync::mpsc::Sender<Result<Response>>,
    result_window: Arc<tokio::sync::Semaphore>, trace: &Trace,
) -> Result<()> {
    let mut flags_paths: Vec<(PathBuf, Option<std::fs::FileType>)> =
        flags.path.iter().rev().map(|path| (path.clone(), None)).collect();
    let mut order = 0;
    while let Some((path, file_type)) = flags_paths.pop() {
        let started = trace.start();
        let processed = process_path(flags, &mut flags_paths, &path, file_type).await;
        trace.add(0, started, true);
        let started = trace.start();
        match processed {
            Ok(ProcessPath::Recursive) => continue,
            processed => {
                let permit =
                    result_window.clone().acquire_owned().await.context("result window closed")?;
                let pending = Pending { order, path, permit };
                match processed {
                    Ok(ProcessPath::Content) => work_sender.send(Work::Content(pending)).await?,
                    // Standard input is already consumed and cannot be read again, so it travels with its
                    // features instead of its path.
                    Ok(ProcessPath::Features(features)) => {
                        work_sender.send(Work::Extracted { pending, features }).await?
                    }
                    Ok(ProcessPath::Ruled(file_type)) => {
                        result_sender.send(Ok(Response::new(pending, Ok(file_type)))).await?
                    }
                    Err(error) => {
                        result_sender.send(Ok(Response::new(pending, Err(error)))).await?
                    }
                    Ok(ProcessPath::Recursive) => unreachable!(),
                }
            }
        }
        trace.add(0, started, false);
        order += 1;
    }
    Ok(())
}

/// Spawns the threads that read files and extract features.
fn spawn_read_threads(
    readers: usize, receiver: &async_channel::Receiver<Work>,
    sender: &async_channel::Sender<ReadItem>, trace: &Arc<Trace>, first_stage: usize,
) -> Result<Vec<JoinHandle<()>>> {
    (0..readers)
        .map(|index| {
            let receiver = receiver.clone();
            let sender = sender.clone();
            let trace = trace.clone();
            let stage = first_stage + index;
            std::thread::Builder::new()
                .name(format!("magika-read-{index}"))
                .spawn(move || {
                    read_files(&receiver, &sender, &trace, stage);
                })
                .map_err(Into::into)
        })
        .collect()
}

/// Reads files and extracts their features.
///
/// Extraction reads two small blocks per file, which the asynchronous file API turns into a
/// handful of round trips through the blocking pool each time. Reading straight from a plain file
/// on a dedicated thread costs a system call per block instead, and there is nothing else for the
/// thread to interleave anyway.
fn read_files(
    work_receiver: &async_channel::Receiver<Work>, sender: &async_channel::Sender<ReadItem>,
    trace: &Trace, stage: usize,
) {
    loop {
        let waiting = trace.start();
        let Ok(work) = work_receiver.recv_blocking() else { break };
        trace.add(stage, waiting, false);

        let reading = trace.start();
        let (pending, extracted) = match work {
            Work::Content(pending) => {
                let extracted = extract_path(&pending.path);
                (pending, extracted)
            }
            Work::Extracted { pending, features } => {
                (pending, Ok(FeaturesOrRuled::Features(features)))
            }
        };
        trace.add(stage, reading, true);

        let handing = trace.start();
        if sender.send_blocking(ReadItem { pending, extracted }).is_err() {
            break;
        }
        trace.add(stage, handing, false);
    }
}

/// Accumulates every reader's output into one global inference batch stream.
async fn accumulate_files(
    batch_size: usize, receiver: &async_channel::Receiver<ReadItem>,
    batch_sender: &async_channel::Sender<InferenceBatch>,
    result_sender: &tokio::sync::mpsc::Sender<Result<Response>>, trace: &Trace, stage: usize,
) -> Result<()> {
    let mut accumulator = Accumulator::new(batch_size);
    loop {
        let waiting = trace.start();
        let Ok(ReadItem { pending, extracted }) = receiver.recv().await else {
            break;
        };
        trace.add(stage, waiting, false);

        let handling = trace.start();
        match extracted {
            Ok(FeaturesOrRuled::Features(features)) => {
                if let Some(batch) = accumulator.push(pending, features) {
                    batch_sender.send(batch).await?;
                }
            }
            Ok(FeaturesOrRuled::Ruled(content_type)) => {
                let result = Ok(FileType::Ruled(content_type));
                result_sender.send(Ok(Response::new(pending, result))).await?;
            }
            Err(error) => {
                result_sender.send(Ok(Response::new(pending, Err(error)))).await?;
            }
        }
        trace.add(stage, handling, true);
    }
    if let Some(batch) = accumulator.finish() {
        batch_sender.send(batch).await?;
    }
    Ok(())
}

/// Reads a file and extracts its features.
fn extract_path(path: &Path) -> magika::Result<FeaturesOrRuled> {
    FeaturesOrRuled::extract_sync(std::fs::File::open(path)?)
}

enum ProcessPath {
    Recursive,
    /// A regular file whose content still has to be read.
    Content,
    Features(Features),
    Ruled(FileType),
}

/// An item of work handed to a read thread.
enum Work {
    /// A regular file whose content still has to be read.
    Content(Pending),

    /// Standard input, whose features the walk already extracted.
    Extracted { pending: Pending, features: Features },
}

/// Feature extraction result produced by any reader and consumed by the single accumulator.
struct ReadItem {
    pending: Pending,
    extracted: magika::Result<FeaturesOrRuled>,
}

/// A path and its permit in the bounded, ordered result window.
struct Pending {
    order: usize,
    path: PathBuf,
    permit: tokio::sync::OwnedSemaphorePermit,
}

impl From<FeaturesOrRuled> for ProcessPath {
    fn from(value: FeaturesOrRuled) -> Self {
        match value {
            FeaturesOrRuled::Features(x) => ProcessPath::Features(x),
            FeaturesOrRuled::Ruled(x) => ProcessPath::Ruled(FileType::Ruled(x)),
        }
    }
}

async fn process_path(
    flags: &Flags, paths: &mut Vec<(PathBuf, Option<std::fs::FileType>)>, path: &Path,
    known: Option<std::fs::FileType>,
) -> magika::Result<ProcessPath> {
    if path.to_str() == Some("-") {
        let mut stdin = Vec::new();
        tokio::io::stdin().read_to_end(&mut stdin).await?;
        return Ok(FeaturesOrRuled::extract_sync(&stdin[..])?.into());
    }
    // `read_dir` already reported the type of every entry it produced, so reading its metadata
    // again would be one extra system call per file on the one task that feeds every reader. Only
    // a symlink still needs a lookup, and only when it is being followed.
    let metadata = match known {
        Some(known) if flags.no_dereference || !known.is_symlink() => known,
        _ => {
            if flags.no_dereference {
                tokio::fs::symlink_metadata(&path).await?.file_type()
            } else {
                tokio::fs::metadata(&path).await?.file_type()
            }
        }
    };
    if metadata.is_dir() {
        return Ok(if flags.recursive {
            let mut entries = tokio::fs::read_dir(&path).await?;
            let mut dir_paths = Vec::new();
            while let Some(entry) = entries.next_entry().await? {
                dir_paths.push((entry.path(), entry.file_type().await.ok()));
            }
            dir_paths.sort_by(|a, b| a.0.cmp(&b.0));
            while let Some(path) = dir_paths.pop() {
                paths.push(path);
            }
            ProcessPath::Recursive
        } else {
            ProcessPath::Ruled(FileType::Directory)
        });
    }
    if metadata.is_symlink() {
        return Ok(ProcessPath::Ruled(FileType::Symlink));
    }
    Ok(ProcessPath::Content)
}

fn spawn_inference_threads(
    threads: usize, runtime: Arc<Runtime>, receiver: &async_channel::Receiver<InferenceBatch>,
    sender: &tokio::sync::mpsc::Sender<Result<Response>>, trace: &Arc<Trace>, first_stage: usize,
) -> Result<Vec<JoinHandle<()>>> {
    (0..threads)
        .map(|index| {
            let receiver = receiver.clone();
            let sender = sender.clone();
            let runtime = runtime.clone();
            let trace = trace.clone();
            let stage = first_stage + index;
            std::thread::Builder::new()
                .name(format!("magika-inference-{index}"))
                .spawn(move || {
                    if let Err(error) = infer_batches(&runtime, &receiver, &sender, &trace, stage) {
                        let _ = sender.blocking_send(Err(error));
                    }
                })
                .map_err(Into::into)
        })
        .collect()
}

fn infer_batches(
    runtime: &Runtime, receiver: &async_channel::Receiver<InferenceBatch>,
    sender: &tokio::sync::mpsc::Sender<Result<Response>>, trace: &Trace, stage: usize,
) -> Result<()> {
    // The session is created on the first batch rather than up front. A run holding fewer files
    // than there are threads never reaches most of them, and a session that identifies nothing is
    // paid for all the same: it spawns state for every resident plan and warms each one.
    let mut session = None;
    loop {
        let waiting = trace.start();
        let Ok(InferenceBatch { pending, features }) = receiver.recv_blocking() else { break };
        trace.add(stage, waiting, false);

        let running = trace.start();
        let magika = match &mut session {
            Some(session) => session,
            slot => slot.insert(runtime.session()?),
        };
        let batch = magika.identify_features_batch_sync(&features)?;
        trace.add(stage, running, true);
        assert_eq!(batch.len(), pending.len());

        let sending = trace.start();
        for (pending, output) in pending.into_iter().zip(batch) {
            let result = Ok(output);
            sender.blocking_send(Ok(Response::new(pending, result)))?;
        }
        trace.add(stage, sending, false);
    }
    Ok(())
}

#[derive(Debug, Default)]
struct Reorder {
    next: usize,
    todo: HashMap<usize, Response>,
}

impl Reorder {
    fn is_empty(&self) -> bool {
        self.todo.is_empty()
    }

    fn push(&mut self, response: Response) {
        debug_assert!(self.next <= response.order);
        let prev = self.todo.insert(response.order, response);
        debug_assert!(prev.is_none());
    }

    fn pop(&mut self) -> Option<Response> {
        let result = self.todo.remove(&self.next)?;
        self.next += 1;
        Some(result)
    }
}

struct InferenceBatch {
    pending: Vec<Pending>,
    features: Vec<Features>,
}

struct Accumulator {
    batch_size: usize,
    pending: Vec<Pending>,
    features: Vec<Features>,
}

impl Accumulator {
    fn new(batch_size: usize) -> Self {
        Self {
            batch_size,
            pending: Vec::with_capacity(batch_size),
            features: Vec::with_capacity(batch_size),
        }
    }

    fn push(&mut self, pending: Pending, features: Features) -> Option<InferenceBatch> {
        self.pending.push(pending);
        self.features.push(features);
        (self.features.len() == self.batch_size).then(|| self.take())
    }

    fn finish(mut self) -> Option<InferenceBatch> {
        (!self.features.is_empty()).then(|| self.take())
    }

    fn take(&mut self) -> InferenceBatch {
        InferenceBatch {
            pending: std::mem::replace(&mut self.pending, Vec::with_capacity(self.batch_size)),
            features: std::mem::replace(&mut self.features, Vec::with_capacity(self.batch_size)),
        }
    }
}

#[derive(Debug)]
struct Response {
    order: usize,
    path: PathBuf,
    result: Result<FileType, magika::Error>,
    _permit: tokio::sync::OwnedSemaphorePermit,
}

impl Response {
    fn new(pending: Pending, result: Result<FileType, magika::Error>) -> Self {
        Self { order: pending.order, path: pending.path, result, _permit: pending.permit }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
enum JsonError {
    Unknown,
    FileDoesNotExist,
    PermissionError,
}

#[derive(Serialize)]
struct JsonResult<'a> {
    dl: &'a TypeInfo,
    output: &'a TypeInfo,
    score: f32,
}

impl From<magika::Error> for JsonError {
    fn from(value: magika::Error) -> Self {
        match value {
            magika::Error::IOError(x) => match x.kind() {
                ErrorKind::NotFound => JsonError::FileDoesNotExist,
                ErrorKind::PermissionDenied => JsonError::PermissionError,
                _ => JsonError::Unknown,
            },
            _ => JsonError::Unknown,
        }
    }
}

impl Response {
    fn format(self, flags: &Flags) -> Result<ColoredString> {
        let mut result = String::new();
        let format = match &flags.format.custom {
            Some(x) => x.clone(),
            None if flags.format.json => unreachable!(),
            None if flags.format.jsonl => {
                return Ok(serde_json::to_string(&self.json()?)?.into());
            }
            None => {
                let mut format = "%p: ".to_string();
                format.push_str(match () {
                    () if flags.modifiers.mime_type => "%m",
                    () if flags.modifiers.label => "%l",
                    () => "%d (%g)",
                });
                format.push_str("%b");
                format.push_str(if flags.modifiers.output_score { " %S" } else { "" });
                format
            }
        };
        let mut format = format.chars();
        loop {
            match format.next() {
                Some('%') => match format.next() {
                    Some('p') => write!(&mut result, "{}", self.path.display())?,
                    Some('l') => write!(&mut result, "{}", self.label())?,
                    Some('d') => write!(&mut result, "{}", self.description())?,
                    Some('g') => write!(&mut result, "{}", self.group())?,
                    Some('m') => write!(&mut result, "{}", self.mime_type())?,
                    Some('e') => write!(&mut result, "{}", join(self.extensions()))?,
                    Some('s') => write!(&mut result, "{:.2}", self.score())?,
                    Some('S') => write!(&mut result, "{}%", (100. * self.score()).trunc())?,
                    Some('b') => {
                        if let Ok(FileType::Inferred(InferredType {
                            content_type: Some((_, OverwriteReason::LowConfidence)),
                            inferred_type,
                            score,
                        })) = &self.result
                        {
                            write!(
                                &mut result,
                                " [Low-confidence model best-guess: {} ({}), score={:.3}]",
                                inferred_type.info().description,
                                inferred_type.info().group,
                                score,
                            )?;
                        }
                    }
                    Some(c) => result.push(c),
                    None => break,
                },
                Some(c) => result.push(c),
                None => break,
            }
        }
        Ok(self.color(result.into()))
    }

    fn json(self) -> Result<serde_json::Value> {
        let path = self.path.to_path_buf();
        let result = match self.result {
            Ok(x) => {
                let dl = match &x {
                    FileType::Inferred(x) => x.inferred_type.info(),
                    _ => ContentType::Undefined.info(),
                };
                let output = x.info();
                let score = (x.score() * 1000.).trunc() / 1000.;
                let value = serde_json::to_value(JsonResult { dl, output, score })?;
                serde_json::json!({ "status": "ok", "value": value })
            }
            Err(error) => serde_json::json!({ "status": JsonError::from(error) }),
        };
        Ok(serde_json::json!({ "path": path, "result": result }))
    }

    fn label(&self) -> &str {
        match &self.result {
            Err(_) => "error",
            Ok(x) => x.info().label,
        }
    }

    fn description(&self) -> Cow<'_, str> {
        match &self.result {
            Err(e) => e.to_string().into(),
            Ok(x) => x.info().description.into(),
        }
    }

    fn group(&self) -> &str {
        match &self.result {
            Err(_) => "error",
            Ok(x) => x.info().group,
        }
    }

    fn mime_type(&self) -> &str {
        match &self.result {
            Err(_) => "error",
            Ok(x) => x.info().mime_type,
        }
    }

    fn extensions(&self) -> &[&str] {
        match &self.result {
            Err(_) => &[],
            Ok(x) => x.info().extensions,
        }
    }

    fn score(&self) -> f32 {
        match &self.result {
            Err(_) => 1.0,
            Ok(x) => x.score(),
        }
    }

    fn color(&self, result: ColoredString) -> ColoredString {
        use colored::Colorize as _;
        // We only use true colors (except for errors). If the terminal doesn't support true colors,
        // the colored crate will automatically choose the closest one.
        match &self.result {
            Err(_) => result.bold().red(),
            Ok(x) => match x.info().group {
                // Tailwind Colors
                "application" => result.truecolor(0xf4, 0x3f, 0x5e), // Rose 500
                "archive" => result.truecolor(0xf5, 0x9e, 0x0b),     // Amber 500
                "audio" => result.truecolor(0x84, 0xcc, 0x16),       // Lime 500
                "code" => result.truecolor(0x8b, 0x5c, 0xf6),        // Violet 500
                "document" => result.truecolor(0x3b, 0x82, 0xf6),    // Blue 500
                "executable" => result.truecolor(0xec, 0x48, 0x99),  // Pink 500
                "image" => result.truecolor(0x06, 0xb6, 0xd4),       // Cyan 500
                "video" => result.truecolor(0x10, 0xb9, 0x81),       // Emerald 500
                _ => result.bold().truecolor(0xcc, 0xcc, 0xcc),
            },
        }
    }
}

fn join<T: AsRef<str>>(xs: impl IntoIterator<Item = T>) -> String {
    let mut result = String::new();
    result.push('[');
    for (i, x) in xs.into_iter().enumerate() {
        if i != 0 {
            result.push(',');
        }
        result.push_str(x.as_ref());
    }
    result.push(']');
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn model_features() -> Features {
        match FeaturesOrRuled::extract_sync(std::fs::File::open("../../README.md").unwrap())
            .unwrap()
        {
            FeaturesOrRuled::Features(features) => features,
            FeaturesOrRuled::Ruled(_) => panic!("README must exercise model inference"),
        }
    }

    #[tokio::test]
    async fn global_accumulator_combines_work_from_every_reader() {
        let features = model_features();
        let (batch_sender, batch_receiver) = async_channel::bounded::<InferenceBatch>(8);
        let (result_sender, _result_receiver) = tokio::sync::mpsc::channel(8);
        let trace = Trace::default();
        let (read_sender, read_receiver) = async_channel::bounded(8);
        let result_window = Arc::new(tokio::sync::Semaphore::new(8));

        for reader in 0..4 {
            let (work_sender, work_receiver) = async_channel::bounded(2);
            for item in 0..2 {
                let order = reader * 2 + item;
                let pending = Pending {
                    order,
                    path: PathBuf::from(format!("file-{order}")),
                    permit: result_window.clone().acquire_owned().await.unwrap(),
                };
                work_sender
                    .send_blocking(Work::Extracted { pending, features: features.clone() })
                    .unwrap();
            }
            drop(work_sender);
            read_files(&work_receiver, &read_sender, &trace, reader);
        }
        drop(read_sender);
        accumulate_files(8, &read_receiver, &batch_sender, &result_sender, &trace, 0)
            .await
            .unwrap();
        drop(batch_sender);

        let batches =
            std::iter::from_fn(|| batch_receiver.recv_blocking().ok()).collect::<Vec<_>>();
        assert_eq!(batches.len(), 1, "all readers must feed one accumulator");
        assert_eq!(batches[0].features.len(), 8);
    }

    #[tokio::test]
    async fn reorder_holds_capacity_until_the_response_is_printable() {
        let window = Arc::new(tokio::sync::Semaphore::new(2));
        let first = window.clone().acquire_owned().await.unwrap();
        let second = window.clone().acquire_owned().await.unwrap();
        let mut reorder = Reorder::default();

        reorder.push(Response {
            order: 1,
            path: PathBuf::from("second"),
            result: Ok(FileType::Directory),
            _permit: second,
        });
        assert!(window.clone().try_acquire_owned().is_err());

        reorder.push(Response {
            order: 0,
            path: PathBuf::from("first"),
            result: Ok(FileType::Directory),
            _permit: first,
        });
        drop(reorder.pop().unwrap());
        assert!(window.clone().try_acquire_owned().is_ok());
    }
}
