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
use std::io::{ErrorKind, Read, Write as _};
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::Arc;

use anyhow::{ensure, Result};
use clap::{Args, Parser, ValueEnum};
use colored::ColoredString;
use magika::{
    self, Backend, ContentType, Features, FeaturesOrRuled, FileType, InferredType, OverwriteReason,
    Runtime, TypeInfo,
};
use serde::Serialize;

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
    experimental: Experimental,
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
struct Experimental {
    /// Selects the backend for inference.
    #[arg(hide = true, long, value_enum, default_value_t)]
    backend: BackendChoice,

    /// Reports the selected inference backend and exits.
    #[arg(hide = true, long)]
    backend_info: bool,

    /// Number of files to identify in a single inference.
    #[arg(hide = true, long, default_value = "8")]
    batch_size: usize,

    /// Number of resident inference threads.
    ///
    /// Inference on a GPU is bound by the device rather than by the host, so a handful of threads
    /// keep it busy and more only contend for it. Inference on a CPU is bound by the host, so every
    /// thread is one more core doing the work. This defaults accordingly: four on a GPU, all
    /// available logical CPUs on x86_64 Linux, and one fewer on other CPU targets.
    #[arg(hide = true, long)]
    threads: Option<usize>,

    /// Number of resident threads reading files and extracting features.
    ///
    /// Reading costs far less than inference, so this defaults to one per inference thread, which
    /// is already more than a run makes use of.
    #[arg(hide = true, long, default_value = "1")]
    readers: usize,
}

#[derive(Clone, Copy, Debug, Default, ValueEnum)]
enum BackendChoice {
    #[default]
    Auto,
    Cpu,
    Gpu,
}

/// Per-stage busy and waiting time.
#[cfg(feature = "_trace")]
#[derive(Default, Clone)]
struct Trace {
    stages: Arc<std::sync::Mutex<HashMap<String, Stage>>>,
}

#[cfg(feature = "_trace")]
struct Stage {
    busy_ns: u64,
    wait_ns: u64,
}

#[cfg(feature = "_trace")]
impl Trace {
    fn insert(&self, stage: Stage) {
        let name = std::thread::current().name().unwrap().to_string();
        assert!(self.stages.lock().unwrap().insert(name, stage).is_none());
    }

    fn report(&self, readers: usize, threads: usize) {
        let mut stages = self.stages.lock().unwrap();
        let mut report = Vec::new();
        report.push(("walk".to_string(), stages.remove("magika-walk").unwrap()));
        for i in 0..readers {
            report
                .push((format!("read[{i}]"), stages.remove(&format!("magika-read-{i}")).unwrap()));
        }
        report.push(("batch".to_string(), stages.remove("magika-batch").unwrap()));
        for i in 0..threads {
            report.push((
                format!("infer[{i}]"),
                stages.remove(&format!("magika-infer-{i}")).unwrap(),
            ));
        }
        assert!(stages.is_empty());
        eprintln!("trace  stage           busy      waiting   busy%");
        for (name, stage) in report {
            let busy = stage.busy_ns;
            let wait = stage.wait_ns;
            let total = busy + wait;
            let share = if total == 0 { 0.0 } else { busy as f64 / total as f64 * 100.0 };
            eprintln!(
                "trace  {:<14} {:>7.3}s  {:>7.3}s  {:>5.1}%",
                name,
                busy as f64 / 1e9,
                wait as f64 / 1e9,
                share
            );
        }
    }
}

#[cfg(feature = "_trace")]
impl Stage {
    fn start() -> (std::time::Instant, cpu_time::ThreadTime) {
        (std::time::Instant::now(), cpu_time::ThreadTime::now())
    }

    fn finalize(wall_thread: (std::time::Instant, cpu_time::ThreadTime)) -> Stage {
        let (wall, thread) = wall_thread;
        let total_ns = wall.elapsed().as_nanos() as u64;
        let busy_ns = thread.elapsed().as_nanos() as u64;
        let wait_ns = total_ns.saturating_sub(busy_ns);
        Stage { busy_ns, wait_ns }
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
        Backend::Cpu => {
            // The x86_64 Linux inference graph benefits from both SMT siblings, and traversal is
            // too small to justify reserving a logical CPU. Keep the original portable/macOS
            // policy, whose graph and host scheduling have different scaling behavior.
            #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
            {
                available
            }
            #[cfg(not(all(target_os = "linux", target_arch = "x86_64")))]
            {
                available.saturating_sub(1).max(1)
            }
        }
    }
}

fn main() -> Result<()> {
    let flags = Flags::parse();
    ensure!(flags.experimental.batch_size != 0, "--batch-size cannot be zero");
    let batch_size = flags.experimental.batch_size;
    ensure!(flags.experimental.threads != Some(0), "--threads cannot be zero");
    ensure!(flags.experimental.readers != 0, "--readers cannot be zero");
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
    let mut builder = Runtime::builder();
    builder = builder.with_max_batch(batch_size);
    builder = match flags.experimental.backend {
        BackendChoice::Auto => builder,
        BackendChoice::Cpu => builder.with_backend(Backend::Cpu),
        BackendChoice::Gpu => builder.with_backend(Backend::Gpu),
    };
    let runtime = Arc::new(builder.build()?);
    if flags.experimental.backend_info {
        let info = runtime.backend_info();
        let backend = match info.backend() {
            Backend::Cpu => "cpu",
            Backend::Gpu => "gpu",
        };
        println!("{backend} ({})", info.implementation());
        return Ok(());
    }
    let threads = match flags.experimental.threads {
        Some(threads) => threads,
        None => default_inference_threads(runtime.backend_info().backend()),
    };
    let readers = flags.experimental.readers;
    let (work_sender, work_receiver) = crossbeam_channel::bounded::<OrderPath>(readers);
    let (read_sender, read_receiver) =
        std::sync::mpsc::sync_channel::<ReadItem>(threads * batch_size);
    let (batch_sender, batch_receiver) = crossbeam_channel::bounded::<InferenceBatch>(threads);
    let (result_sender, result_receiver) =
        std::sync::mpsc::sync_channel::<Result<Response>>(threads * batch_size);
    let reorder_next = Arc::new(AtomicUsize::new(0));
    #[cfg(feature = "_trace")]
    let trace = Trace::default();
    let mut join_handles = Vec::new();
    join_handles.push(std::thread::Builder::new().name("magika-walk".to_string()).spawn({
        let flags = flags.clone();
        let result_sender = result_sender.clone();
        let reorder_next = reorder_next.clone();
        #[cfg(feature = "_trace")]
        let trace = trace.clone();
        move || {
            #[cfg(feature = "_trace")]
            let start = Stage::start();
            if let Err(e) = walk_paths(
                &flags,
                &work_sender,
                &result_sender,
                &reorder_next,
                4 * threads * batch_size,
            ) {
                let _ = result_sender.send(Err(e));
            }
            #[cfg(feature = "_trace")]
            trace.insert(Stage::finalize(start));
        }
    })?);
    for index in 0..readers {
        join_handles.push(std::thread::Builder::new().name(format!("magika-read-{index}")).spawn(
            {
                let work_receiver = work_receiver.clone();
                let read_sender = read_sender.clone();
                #[cfg(feature = "_trace")]
                let trace = trace.clone();
                move || {
                    #[cfg(feature = "_trace")]
                    let start = Stage::start();
                    read_files(&work_receiver, &read_sender);
                    #[cfg(feature = "_trace")]
                    trace.insert(Stage::finalize(start));
                }
            },
        )?)
    }
    drop(work_receiver);
    drop(read_sender);
    join_handles.push(std::thread::Builder::new().name("magika-batch".to_string()).spawn({
        let batch_sender = batch_sender.clone();
        let result_sender = result_sender.clone();
        #[cfg(feature = "_trace")]
        let trace = trace.clone();
        move || {
            #[cfg(feature = "_trace")]
            let start = Stage::start();
            if let Err(error) =
                batch_files(batch_size, &read_receiver, &batch_sender, &result_sender)
            {
                let _ = result_sender.send(Err(error));
            }
            #[cfg(feature = "_trace")]
            trace.insert(Stage::finalize(start));
        }
    })?);
    drop(batch_sender);
    for index in 0..threads {
        join_handles.push(
            std::thread::Builder::new().name(format!("magika-infer-{index}")).spawn({
                let batch_receiver = batch_receiver.clone();
                let result_sender = result_sender.clone();
                let runtime = runtime.clone();
                #[cfg(feature = "_trace")]
                let trace = trace.clone();
                move || {
                    #[cfg(feature = "_trace")]
                    let start = Stage::start();
                    if let Err(error) = infer_batches(&runtime, &batch_receiver, &result_sender) {
                        let _ = result_sender.send(Err(error));
                    }
                    #[cfg(feature = "_trace")]
                    trace.insert(Stage::finalize(start));
                }
            })?,
        );
    }
    drop(batch_receiver);
    drop(result_sender);
    let print_result = match print(&flags, result_receiver, reorder_next) {
        Err(e)
            if e.root_cause()
                .downcast_ref::<std::io::Error>()
                .is_some_and(|x| x.kind() == std::io::ErrorKind::BrokenPipe) =>
        {
            Ok(())
        }
        x => x,
    };
    for handle in join_handles {
        ensure!(handle.join().is_ok());
    }
    #[cfg(feature = "_trace")]
    trace.report(readers, threads);
    print_result
}

fn print(
    flags: &Flags, result_receiver: std::sync::mpsc::Receiver<Result<Response>>,
    reorder_next: Arc<AtomicUsize>,
) -> Result<()> {
    let mut stdout = std::io::stdout().lock();
    if flags.format.json {
        write!(stdout, "[")?;
    }
    let mut reorder = Reorder::new(reorder_next);
    let mut errors = false;
    while let Ok(response) = result_receiver.recv() {
        reorder.push(response?);
        while let Some(response) = reorder.pop() {
            errors |= response.result.is_err();
            if flags.format.json {
                if reorder.next.load(Relaxed) != 1 {
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
        if reorder.next.load(Relaxed) != 0 {
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
fn walk_paths(
    flags: &Flags, work_sender: &crossbeam_channel::Sender<OrderPath>,
    result_sender: &std::sync::mpsc::SyncSender<Result<Response>>, reorder_next: &AtomicUsize,
    max_dist: usize,
) -> Result<()> {
    let mut flags_paths: Vec<(PathBuf, Option<std::fs::FileType>)> =
        flags.path.iter().rev().map(|path| (path.clone(), None)).collect();
    let mut order = 0;
    while let Some((path, file_type)) = flags_paths.pop() {
        let processed = process_path(flags, &mut flags_paths, &path, file_type);
        if matches!(processed, Ok(ProcessPath::Recursive)) {
            continue;
        }
        // Make sure a specific non-recursive path does not get stranded for too long in the
        // pipeline. This bounds the reorder buffer without starving the pipeline.
        while reorder_next.load(Relaxed) + max_dist < order {
            std::hint::spin_loop();
        }
        let pending = OrderPath { order, path };
        match processed {
            Ok(ProcessPath::Content) => work_sender.send(pending)?,
            Ok(ProcessPath::Ruled(file_type)) => {
                result_sender.send(Ok(Response::new(pending, Ok(file_type))))?
            }
            Err(error) => result_sender.send(Ok(Response::new(pending, Err(error))))?,
            Ok(ProcessPath::Recursive) => unreachable!(),
        }
        order += 1;
    }
    Ok(())
}

/// Reads files and extracts their features.
///
/// Extraction reads two small blocks per file, which the asynchronous file API turns into a
/// handful of round trips through the blocking pool each time. Reading straight from a plain file
/// on a dedicated thread costs a system call per block instead, and there is nothing else for the
/// thread to interleave anyway.
fn read_files(
    work_receiver: &crossbeam_channel::Receiver<OrderPath>,
    sender: &std::sync::mpsc::SyncSender<ReadItem>,
) {
    while let Ok(pending) = work_receiver.recv() {
        let extracted = extract_path(&pending.path);
        if sender.send(ReadItem { pending, extracted }).is_err() {
            break;
        }
    }
}

/// Accumulates every reader's output into one global inference batch stream.
fn batch_files(
    batch_size: usize, receiver: &std::sync::mpsc::Receiver<ReadItem>,
    batch_sender: &crossbeam_channel::Sender<InferenceBatch>,
    result_sender: &std::sync::mpsc::SyncSender<Result<Response>>,
) -> Result<()> {
    let mut batcher = Batcher::new(batch_size);
    while let Ok(ReadItem { pending, extracted }) = receiver.recv() {
        match extracted {
            Ok(FeaturesOrRuled::Features(features)) => {
                if let Some(batch) = batcher.push(pending, features) {
                    batch_sender.send(batch)?;
                }
            }
            Ok(FeaturesOrRuled::Ruled(content_type)) => {
                let result = Ok(FileType::Ruled(content_type));
                result_sender.send(Ok(Response::new(pending, result)))?;
            }
            Err(error) => {
                result_sender.send(Ok(Response::new(pending, Err(error))))?;
            }
        }
    }
    if let Some(batch) = batcher.finish() {
        batch_sender.send(batch)?;
    }
    Ok(())
}

/// Reads a file and extracts its features.
fn extract_path(path: &Path) -> Result<FeaturesOrRuled> {
    if path.to_str() == Some("-") {
        let mut stdin = Vec::new();
        std::io::stdin().read_to_end(&mut stdin)?;
        return FeaturesOrRuled::extract(&stdin[..]);
    }
    FeaturesOrRuled::extract(std::fs::File::open(path)?)
}

enum ProcessPath {
    Recursive,
    Content,
    Ruled(FileType),
}

struct OrderPath {
    order: usize,
    path: PathBuf,
}

struct ReadItem {
    pending: OrderPath,
    extracted: Result<FeaturesOrRuled>,
}

fn process_path(
    flags: &Flags, paths: &mut Vec<(PathBuf, Option<std::fs::FileType>)>, path: &Path,
    known: Option<std::fs::FileType>,
) -> Result<ProcessPath> {
    if path.to_str() == Some("-") {
        return Ok(ProcessPath::Content);
    }
    // `read_dir` already reported the type of every entry it produced, so reading its metadata
    // again would be one extra system call per file on the one task that feeds every reader. Only
    // a symlink still needs a lookup, and only when it is being followed.
    let metadata = match known {
        Some(known) if flags.no_dereference || !known.is_symlink() => known,
        _ => {
            if flags.no_dereference {
                std::fs::symlink_metadata(path)?.file_type()
            } else {
                std::fs::metadata(path)?.file_type()
            }
        }
    };
    if metadata.is_dir() {
        return Ok(if flags.recursive {
            let mut dir_paths = Vec::new();
            for entry in std::fs::read_dir(path)? {
                let entry = entry?;
                dir_paths.push((entry.path(), entry.file_type().ok()));
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

fn infer_batches(
    runtime: &Runtime, receiver: &crossbeam_channel::Receiver<InferenceBatch>,
    sender: &std::sync::mpsc::SyncSender<Result<Response>>,
) -> Result<()> {
    // Create a session only when a thread receives its first batch. A short run never reaches most
    // threads, so spawning their private execution state up front would be pure startup overhead.
    let mut session = None;
    while let Ok(InferenceBatch { pending, features }) = receiver.recv() {
        let magika = match &mut session {
            Some(session) => session,
            slot => slot.insert(runtime.session()?),
        };
        let batch = magika.identify_features_batch(&features)?;
        debug_assert_eq!(batch.len(), pending.len());
        for (pending, output) in pending.into_iter().zip(batch) {
            let result = Ok(output);
            sender.send(Ok(Response::new(pending, result)))?;
        }
    }
    Ok(())
}

#[derive(Debug)]
struct Reorder {
    next: Arc<AtomicUsize>,
    todo: HashMap<usize, Response>,
}

impl Reorder {
    fn new(next: Arc<AtomicUsize>) -> Self {
        Reorder { next, todo: HashMap::new() }
    }

    fn is_empty(&self) -> bool {
        self.todo.is_empty()
    }

    fn push(&mut self, response: Response) {
        debug_assert!(self.next.load(Relaxed) <= response.order);
        let prev = self.todo.insert(response.order, response);
        debug_assert!(prev.is_none());
    }

    fn pop(&mut self) -> Option<Response> {
        let result = self.todo.remove(&self.next.load(Relaxed))?;
        self.next.fetch_add(1, Relaxed);
        Some(result)
    }
}

struct InferenceBatch {
    pending: Vec<OrderPath>,
    features: Vec<Features>,
}

struct Batcher {
    batch_size: usize,
    pending: Vec<OrderPath>,
    features: Vec<Features>,
}

impl Batcher {
    fn new(batch_size: usize) -> Self {
        Self {
            batch_size,
            pending: Vec::with_capacity(batch_size),
            features: Vec::with_capacity(batch_size),
        }
    }

    fn push(&mut self, pending: OrderPath, features: Features) -> Option<InferenceBatch> {
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
    result: Result<FileType>,
}

impl Response {
    fn new(pending: OrderPath, result: Result<FileType>) -> Self {
        Self { order: pending.order, path: pending.path, result }
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

impl From<anyhow::Error> for JsonError {
    fn from(value: anyhow::Error) -> Self {
        match value.root_cause().downcast_ref::<std::io::Error>() {
            Some(x) => match x.kind() {
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
