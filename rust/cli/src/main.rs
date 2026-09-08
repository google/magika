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
use std::sync::Arc;

use anyhow::{ensure, Context, Result};
use clap::{Args, Parser, ValueEnum};
use colored::ColoredString;
use magika::{
    self, Backend, ContentType, Features, FeaturesOrRuled, FileType, InferredType, OverwriteReason,
    Runtime, TypeInfo,
};
use serde::Serialize;

mod progress;
use progress::Progress;

/// Determines file content types using AI.
#[derive(Parser)]
#[command(name = "magika", version = Version, arg_required_else_help = true)]
struct Flags {
    /// Enables the selected ruleset (requires the yara-rules feature).
    #[arg(long, value_enum, default_value = "off")]
    rules: Rules,
    /// Loads a YARA pack with per-rule enforcement metadata. Requires --rules=enforce.
    #[arg(long)]
    rules_file: Option<PathBuf>,
    /// Compiles a YARA file to a sibling .hsdb file and exits; refuses to overwrite.
    #[arg(long, conflicts_with_all = ["path", "rules_file", "write_default_rules"])]
    compile_rules: Option<PathBuf>,
    /// Writes the bundled YARA source to a new file and exits.
    #[arg(long, conflicts_with_all = ["path", "rules_file"])]
    write_default_rules: Option<PathBuf>,
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

#[derive(Clone, Copy, Debug, ValueEnum)]
enum Rules {
    Off,
    Enforce,
}
impl From<Rules> for magika::RulesMode {
    fn from(value: Rules) -> Self {
        match value {
            Rules::Off => Self::Off,
            Rules::Enforce => Self::Enforce,
        }
    }
}

#[derive(Clone)]
struct RuleInput {
    mode: magika::RulesMode,
    #[cfg(feature = "yara-rules")]
    pack: Option<magika::RuleSet>,
}
impl RuleInput {
    fn extract(&self, input: impl magika::Input) -> Result<FeaturesOrRuled> {
        #[cfg(feature = "yara-rules")]
        if self.mode == magika::RulesMode::Enforce {
            if let Some(pack) = &self.pack {
                return FeaturesOrRuled::extract_with_ruleset(input, pack);
            }
        }
        FeaturesOrRuled::extract_with_rules(input, self.mode)
    }
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

    /// Number of files to identify in a single inference (1 through 64).
    #[arg(hide = true, long, default_value = "8", value_parser = clap::builder::RangedU64ValueParser::<usize>::new().range(1..=64))]
    batch_size: usize,

    /// Number of resident inference threads (1 through 256).
    ///
    /// Inference on a GPU is bound by the device rather than by the host, so a handful of threads
    /// keep it busy and more only contend for it. Inference on a CPU is bound by the host, so every
    /// thread is one more core doing the work. This defaults accordingly: four on a GPU, all
    /// available logical CPUs on x86_64 Linux, and one fewer on other CPU targets.
    #[arg(hide = true, long, value_parser = clap::builder::RangedU64ValueParser::<usize>::new().range(1..=256))]
    threads: Option<usize>,

    /// Number of resident threads reading files and extracting features (1 through 256).
    ///
    /// Defaults to one reader, independently of the number of inference threads.
    #[arg(hide = true, long, default_value = "1", value_parser = clap::builder::RangedU64ValueParser::<usize>::new().range(1..=256))]
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

    fn report(&self, readers: usize) {
        let mut stages = self.stages.lock().unwrap();
        let mut report = Vec::new();
        report.push(("walk".to_string(), stages.remove("magika-walk").unwrap()));
        for i in 0..readers {
            report
                .push((format!("read[{i}]"), stages.remove(&format!("magika-read-{i}")).unwrap()));
        }
        report.push(("batch".to_string(), stages.remove("magika-batch").unwrap()));
        let threads = stages.keys().filter(|name| name.starts_with("magika-infer-")).count();
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
    let available =
        std::thread::available_parallelism().map_or(1, std::num::NonZeroUsize::get).min(256);
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

// Keep enough bounded lookahead to batch sparse ML misses at low worker counts.
fn reorder_window(threads: usize, batch_size: usize) -> usize {
    (4 * threads * batch_size).max(256)
}

fn main() -> Result<()> {
    let flags = Flags::parse();
    if let Some(path) = &flags.write_default_rules {
        std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)
            .with_context(|| format!("create rules source {}", path.display()))?
            .write_all(magika::DEFAULT_RULES.as_bytes())
            .with_context(|| format!("write rules source {}", path.display()))?;
        return Ok(());
    }
    if let Some(path) = &flags.compile_rules {
        #[cfg(feature = "yara-rules")]
        {
            let output = path.with_extension("hsdb");
            magika::RuleSet::compile_file(path, &output).with_context(|| {
                format!("compile rules {} to {}", path.display(), output.display())
            })?;
            return Ok(());
        }
        #[cfg(not(feature = "yara-rules"))]
        {
            let _ = path;
            anyhow::bail!("--compile-rules requires a build with the yara-rules Cargo feature");
        }
    }
    ensure!(
        flags.rules_file.is_none() || matches!(flags.rules, Rules::Enforce),
        "--rules-file requires --rules=enforce"
    );
    ensure!(
        cfg!(feature = "yara-rules") || matches!(flags.rules, Rules::Off),
        "--rules=enforce requires a build with the yara-rules Cargo feature"
    );
    let batch_size = flags.experimental.batch_size;
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
    // CLI inference receives features only; readers own the selected rule pack.
    let builder = Runtime::builder().with_max_batch(batch_size);
    let builder = match flags.experimental.backend {
        BackendChoice::Auto => builder,
        BackendChoice::Cpu => builder.with_backend(Backend::Cpu),
        BackendChoice::Gpu => builder.with_backend(Backend::Gpu),
    };
    if flags.experimental.backend_info {
        let runtime = builder.build()?;
        let info = runtime.backend_info();
        let backend = match info.backend() {
            Backend::Cpu => "cpu",
            Backend::Gpu => "gpu",
        };
        println!("{backend} ({})", info.implementation());
        return Ok(());
    }
    // Reserve bounded queues before backend selection completes. The inference coordinator
    // chooses the existing CPU/GPU worker policy once the background model load finishes.
    let threads = flags.experimental.threads.unwrap_or_else(|| {
        default_inference_threads(Backend::Cpu).max(default_inference_threads(Backend::Gpu))
    });
    let readers = flags.experimental.readers;
    let (work_sender, work_receiver) = crossbeam_channel::bounded::<OrderPath>(readers);
    let (read_sender, read_receiver) =
        std::sync::mpsc::sync_channel::<ReadItem>(threads * batch_size);
    let (batch_sender, batch_receiver) = crossbeam_channel::bounded::<InferenceBatch>(threads);
    let (result_sender, result_receiver) =
        std::sync::mpsc::sync_channel::<Result<Response>>(threads * batch_size);
    let reorder_next = Arc::new(Progress::default());
    #[cfg(feature = "_trace")]
    let trace = Trace::default();
    let mut join_handles = Vec::new();
    join_handles.push(std::thread::Builder::new().name("magika-model".to_string()).spawn({
        let batch_receiver = batch_receiver.clone();
        let result_sender = result_sender.clone();
        let requested_threads = flags.experimental.threads;
        #[cfg(feature = "_trace")]
        let trace = trace.clone();
        move || {
            let result = prepare_and_infer(
                move || builder.build(),
                requested_threads,
                &batch_receiver,
                &result_sender,
                #[cfg(feature = "_trace")]
                &trace,
            );
            if let Err(error) = result {
                let _ = result_sender.send(Err(error));
            }
        }
    })?);
    // Start backend preparation before loading an explicit pack, so the two can overlap.
    let rule_input = RuleInput {
        mode: flags.rules.into(),
        #[cfg(feature = "yara-rules")]
        pack: if matches!(flags.rules, Rules::Enforce) {
            let source = flags.rules_file.clone().or_else(|| {
                let exe = std::env::current_exe().ok()?;
                let source = exe.parent()?.join("rules/promoted.yar");
                source.is_file().then_some(source)
            });
            Some(match source {
                Some(path) => magika::RuleSet::from_file(&path)
                    .with_context(|| format!("failed to load rules from {}", path.display()))?,
                None => magika::RuleSet::bundled()?,
            })
        } else {
            None
        },
    };
    join_handles.push(std::thread::Builder::new().name("magika-walk".to_string()).spawn({
        let flags = flags.clone();
        let read_sender = read_sender.clone();
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
                &read_sender,
                &result_sender,
                &reorder_next,
                reorder_window(threads, batch_size),
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
                let rule_input = rule_input.clone();
                let work_receiver = work_receiver.clone();
                let read_sender = read_sender.clone();
                #[cfg(feature = "_trace")]
                let trace = trace.clone();
                move || {
                    #[cfg(feature = "_trace")]
                    let start = Stage::start();
                    read_files(&work_receiver, &read_sender, &rule_input);
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
    trace.report(readers);
    print_result
}

fn print(
    flags: &Flags, result_receiver: std::sync::mpsc::Receiver<Result<Response>>,
    reorder_next: Arc<Progress>,
) -> Result<()> {
    print_to(flags, result_receiver, reorder_next, &mut std::io::stdout().lock())
}

fn print_to(
    flags: &Flags, result_receiver: std::sync::mpsc::Receiver<Result<Response>>,
    reorder_next: Arc<Progress>, stdout: &mut impl std::io::Write,
) -> Result<()> {
    // Dropping Reorder wakes traversal on every return, including write failures.
    let mut reorder = Reorder::new(reorder_next);
    if flags.format.json {
        write!(stdout, "[")?;
    }
    let mut errors = false;
    let mut printed = 0;
    let mut failure = None;
    let mut write_failed = false;
    {
        let mut output_row = |response: Response| -> Result<()> {
            errors |= response.result.is_err();
            if flags.format.json {
                // Serialize before writing a comma: a bad row must not corrupt the array.
                let json = serde_json::to_string_pretty(&response.json()?)?;
                if printed != 0 {
                    write!(stdout, ",")?;
                }
                for line in json.lines() {
                    write!(stdout, "\n  {line}")?;
                }
            } else {
                writeln!(stdout, "{}", response.format(flags)?)?;
            }
            printed += 1;
            Ok(())
        };
        'responses: while let Ok(response) = result_receiver.recv() {
            match response {
                Ok(response) => reorder.push(response),
                Err(error) => {
                    failure = Some(error);
                    break;
                }
            }
            while let Some(response) = reorder.pop() {
                if let Err(error) = output_row(response) {
                    write_failed = error.root_cause().is::<std::io::Error>();
                    failure = Some(error);
                    break 'responses;
                }
            }
        }
        if failure.is_none() && !reorder.is_empty() {
            failure = Some(anyhow::anyhow!("classification stopped with missing ordered results"));
        }
        if failure.is_some() && !write_failed {
            // Keep completed results already received, even when a failed earlier row leaves a gap.
            // Do not wait for new work after a pipeline failure.
            for response in result_receiver.try_iter().flatten() {
                reorder.push(response);
            }
            let mut remaining: Vec<_> =
                reorder.todo.drain().map(|(_, response)| response).collect();
            remaining.sort_by_key(|response| response.order);
            for response in remaining {
                if let Err(error) = output_row(response) {
                    if error.root_cause().is::<std::io::Error>() {
                        write_failed = true;
                        break;
                    }
                }
            }
        }
    }
    let close_result = (|| -> Result<()> {
        if flags.format.json && !write_failed {
            if printed != 0 {
                writeln!(stdout)?;
            }
            writeln!(stdout, "]")?;
        }
        Ok(())
    })();
    // Return to main so worker joins and tracing still run. A later broken pipe must
    // not turn an input error that was already observed into a successful exit.
    if errors {
        anyhow::bail!("one or more input files failed");
    }
    match failure {
        Some(error) => Err(error),
        None => close_result,
    }
}

/// Walks the requested paths and hands regular files to the read threads.
///
/// This task only traverses and stats. Reading file content is left to [`read_files`] so that it
/// happens on several threads at once instead of serializing behind traversal.
fn walk_paths(
    flags: &Flags, work_sender: &crossbeam_channel::Sender<OrderPath>,
    read_sender: &std::sync::mpsc::SyncSender<ReadItem>,
    result_sender: &std::sync::mpsc::SyncSender<Result<Response>>, reorder_next: &Progress,
    max_dist: usize,
) -> Result<()> {
    let mut flags_paths = Traversal {
        pending: flags.path.iter().rev().map(|path| WalkEntry::Path(path.clone(), None)).collect(),
        ..Traversal::default()
    };
    let mut order = 0;
    let mut submitted = 0;
    while let Some((path, file_type)) = flags_paths.pop() {
        let processed = process_path(flags, &mut flags_paths, &path, file_type);
        if matches!(processed, Ok(ProcessPath::Recursive)) {
            continue;
        }
        // Make sure a specific non-recursive path does not get stranded for too long in the
        // pipeline. This bounds the reorder buffer without starving the pipeline.
        if !reorder_next.wait_with_notify(order, max_dist, |blocked_on| {
            read_sender.send(ReadItem::Flush { submitted, blocked_on }).is_ok()
        }) {
            return Ok(());
        }
        let pending = OrderPath { order, path };
        match processed {
            Ok(ProcessPath::Content) => {
                work_sender.send(pending)?;
                submitted += 1;
            }
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
    sender: &std::sync::mpsc::SyncSender<ReadItem>, rules: &RuleInput,
) {
    while let Ok(pending) = work_receiver.recv() {
        let extracted = extract_path(&pending.path, rules);
        if sender.send(ReadItem::File { pending, extracted }).is_err() {
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
    let mut completed = 0;
    let mut flush_after = None;
    while let Ok(item) = receiver.recv() {
        match item {
            ReadItem::Flush { submitted, blocked_on } => {
                flush_after = Some((submitted, blocked_on));
            }
            ReadItem::File { pending, extracted } => {
                completed += 1;
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
        }
        // The walker reports how many file reads it dispatched before its ordered-output
        // window filled. Wait for those in-flight reads, then release the partial batch.
        // Scheduling delays alone never split an otherwise full inference batch.
        if let Some((_, blocked_on)) = flush_after.filter(|(target, _)| completed >= *target) {
            // A full batch already in inference will release its own window. Flush only
            // when the result blocking output is still in this partial batch.
            if batcher.pending.iter().any(|pending| pending.order == blocked_on) {
                batch_sender.send(batcher.take())?;
            }
            flush_after = None;
        }
    }
    if let Some(batch) = batcher.finish() {
        batch_sender.send(batch)?;
    }
    Ok(())
}

/// Reads a file and extracts its features.
fn extract_path(path: &Path, rules: &RuleInput) -> Result<FeaturesOrRuled> {
    if path.to_str() == Some("-") {
        let mut stdin = Vec::new();
        std::io::stdin().read_to_end(&mut stdin)?;
        return rules.extract(&stdin[..]);
    }
    rules.extract(std::fs::File::open(path)?)
}

enum ProcessPath {
    Recursive,
    Content,
    Ruled(FileType),
}

enum WalkEntry {
    Path(PathBuf, Option<std::fs::FileType>),
    LeaveDirectory(PathBuf),
}

#[derive(Debug)]
enum TraversalError {
    DirectoryCycle,
}

impl std::fmt::Display for TraversalError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DirectoryCycle => formatter.write_str("directory cycle"),
        }
    }
}

impl std::error::Error for TraversalError {}

#[derive(Default)]
struct Traversal {
    pending: Vec<WalkEntry>,
    ancestors: std::collections::HashSet<PathBuf>,
}

impl Traversal {
    fn pop(&mut self) -> Option<(PathBuf, Option<std::fs::FileType>)> {
        while let Some(entry) = self.pending.pop() {
            match entry {
                WalkEntry::Path(path, kind) => return Some((path, kind)),
                WalkEntry::LeaveDirectory(path) => {
                    self.ancestors.remove(&path);
                }
            }
        }
        None
    }
}

struct OrderPath {
    order: usize,
    path: PathBuf,
}

enum ReadItem {
    File { pending: OrderPath, extracted: Result<FeaturesOrRuled> },
    Flush { submitted: usize, blocked_on: usize },
}

fn process_path(
    flags: &Flags, paths: &mut Traversal, path: &Path, known: Option<std::fs::FileType>,
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
            let canonical = std::fs::canonicalize(path)?;
            if paths.ancestors.contains(&canonical) {
                return Err(TraversalError::DirectoryCycle.into());
            }
            let mut dir_paths = Vec::new();
            for entry in std::fs::read_dir(path)? {
                let entry = entry?;
                dir_paths.push((entry.path(), entry.file_type().ok()));
            }
            dir_paths.sort_by(|a, b| a.0.cmp(&b.0));
            paths.ancestors.insert(canonical.clone());
            paths.pending.push(WalkEntry::LeaveDirectory(canonical));
            while let Some((path, kind)) = dir_paths.pop() {
                paths.pending.push(WalkEntry::Path(path, kind));
            }
            ProcessPath::Recursive
        } else {
            ProcessPath::Ruled(FileType::Directory)
        });
    }
    if metadata.is_symlink() {
        return Ok(ProcessPath::Ruled(FileType::Symlink));
    }
    if !metadata.is_file() {
        return Err(std::io::Error::new(
            ErrorKind::Unsupported,
            format!("unsupported non-regular file: {}", path.display()),
        )
        .into());
    }
    Ok(ProcessPath::Content)
}

/// Model preparation runs concurrently with readers; only inference waits for it.
fn prepare_and_infer(
    prepare: impl FnOnce() -> Result<Runtime> + Send + 'static, requested_threads: Option<usize>,
    batch_receiver: &crossbeam_channel::Receiver<InferenceBatch>,
    result_sender: &std::sync::mpsc::SyncSender<Result<Response>>,
    #[cfg(feature = "_trace")] trace: &Trace,
) -> Result<()> {
    // Loading starts immediately, but the loader must not keep result channels open. If all
    // inputs are ruled, the coordinator can finish without waiting for an unused model.
    // The detached loader owns its inputs; an ML request observes its result (or failure).
    let (runtime_tx, runtime_rx) = std::sync::mpsc::sync_channel(1);
    std::thread::Builder::new().name("magika-load".to_string()).spawn(move || {
        let runtime = prepare();
        let _ = runtime_tx.send(runtime);
    })?;
    let Ok(first) = batch_receiver.recv() else { return Ok(()) };
    let runtime =
        runtime_rx.recv().context("model loader stopped before returning a runtime")??;
    let threads = requested_threads
        .unwrap_or_else(|| default_inference_threads(runtime.backend_info().backend()));
    std::thread::scope(|scope| -> Result<()> {
        let mut workers = Vec::new();
        let mut first = Some(first);
        for index in 0..threads {
            let runtime = &runtime;
            let first = first.take();
            #[cfg(feature = "_trace")]
            let trace = trace.clone();
            workers.push(
                std::thread::Builder::new().name(format!("magika-infer-{index}")).spawn_scoped(
                    scope,
                    move || {
                        #[cfg(feature = "_trace")]
                        let start = Stage::start();
                        if let Err(error) =
                            infer_batches(runtime, first, batch_receiver, result_sender)
                        {
                            let _ = result_sender.send(Err(error));
                        }
                        #[cfg(feature = "_trace")]
                        trace.insert(Stage::finalize(start));
                    },
                )?,
            );
        }
        for worker in workers {
            ensure!(worker.join().is_ok(), "inference worker panicked");
        }
        Ok(())
    })
}

fn infer_batches(
    runtime: &Runtime, first: Option<InferenceBatch>,
    receiver: &crossbeam_channel::Receiver<InferenceBatch>,
    sender: &std::sync::mpsc::SyncSender<Result<Response>>,
) -> Result<()> {
    // Create a session only when a thread receives its first batch. A short run never reaches most
    // threads, so spawning their private execution state up front would be pure startup overhead.
    let mut session = None;
    #[cfg(feature = "_trace")]
    let mut batch_counts = std::collections::BTreeMap::<usize, usize>::new();
    for InferenceBatch { pending, features } in first.into_iter().chain(receiver.iter()) {
        #[cfg(feature = "_trace")]
        {
            *batch_counts.entry(features.len()).or_default() += 1;
        }
        let magika = match &mut session {
            Some(session) => session,
            slot => slot.insert(runtime.session()?),
        };
        let batch = magika.identify_features_batch(&features)?;
        dispatch_inference_results(pending, batch, sender)?;
    }
    #[cfg(feature = "_trace")]
    if !batch_counts.is_empty() {
        eprintln!("trace inference batch sizes: {batch_counts:?}");
    }
    Ok(())
}

fn dispatch_inference_results(
    pending: Vec<OrderPath>, batch: Vec<FileType>,
    sender: &std::sync::mpsc::SyncSender<Result<Response>>,
) -> Result<()> {
    ensure!(
        batch.len() == pending.len(),
        "inference returned {} rows for {} inputs",
        batch.len(),
        pending.len()
    );
    for (pending, output) in pending.into_iter().zip(batch) {
        sender.send(Ok(Response::new(pending, Ok(output))))?;
    }
    Ok(())
}

#[derive(Debug)]
struct Reorder {
    next: usize,
    progress: Arc<Progress>,
    todo: HashMap<usize, Response>,
}

impl Reorder {
    fn new(progress: Arc<Progress>) -> Self {
        Reorder { next: 0, progress, todo: HashMap::new() }
    }

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
        self.progress.advance(self.next);
        Some(result)
    }
}

impl Drop for Reorder {
    fn drop(&mut self) {
        self.progress.close();
    }
}

#[cfg(test)]
mod reorder_tests {
    use super::*;

    #[test]
    fn wrong_inference_row_count_errors_before_dispatch() {
        for returned in [0, 1, 3] {
            let pending =
                (0..2).map(|order| OrderPath { order, path: PathBuf::from("sample") }).collect();
            let batch = vec![FileType::Ruled(ContentType::Png); returned];
            let (sender, receiver) = std::sync::mpsc::sync_channel(4);
            assert!(dispatch_inference_results(pending, batch, &sender).is_err());
            assert!(receiver.try_recv().is_err(), "partial results escaped a malformed batch");
        }
    }

    fn output_response(order: usize) -> Response {
        Response::new(
            OrderPath { order, path: PathBuf::from(format!("{order}.png")) },
            Ok(FileType::Ruled(ContentType::Png)),
        )
    }

    #[test]
    fn pipeline_errors_close_json_and_preserve_buffered_results() {
        let flags = Flags::try_parse_from(["magika", "--json", "sample"]).unwrap();
        let (sender, receiver) = std::sync::mpsc::channel();
        sender.send(Ok(output_response(0))).unwrap();
        sender.send(Ok(output_response(2))).unwrap();
        sender.send(Err(anyhow::anyhow!("inference failed"))).unwrap();
        drop(sender);
        let mut output = Vec::new();
        let result = print_to(&flags, receiver, Arc::new(Progress::default()), &mut output);
        assert!(result.is_err());
        let rows: Vec<serde_json::Value> = serde_json::from_slice(&output).unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0]["path"], "0.png");
        assert_eq!(rows[1]["path"], "2.png");
    }

    #[test]
    fn a_broken_pipe_while_closing_json_does_not_mask_input_errors() {
        struct FailClosing;
        impl std::io::Write for FailClosing {
            fn write(&mut self, buffer: &[u8]) -> std::io::Result<usize> {
                if buffer.first() == Some(&b']') {
                    Err(std::io::Error::from(ErrorKind::BrokenPipe))
                } else {
                    Ok(buffer.len())
                }
            }
            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }
        let flags = Flags::try_parse_from(["magika", "--json", "sample"]).unwrap();
        let (sender, receiver) = std::sync::mpsc::channel();
        let mut response = output_response(0);
        response.result = Err(std::io::Error::from(ErrorKind::NotFound).into());
        sender.send(Ok(response)).unwrap();
        drop(sender);
        let error = print_to(&flags, receiver, Arc::new(Progress::default()), &mut FailClosing)
            .unwrap_err();
        assert!(error.to_string().contains("input files failed"), "{error}");
        assert!(!error.root_cause().is::<std::io::Error>());
    }

    #[test]
    fn a_later_broken_pipe_does_not_mask_a_pipeline_failure() {
        struct FailAfterOpening;
        impl std::io::Write for FailAfterOpening {
            fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
                if bytes == b"[" {
                    Ok(1)
                } else {
                    Err(ErrorKind::BrokenPipe.into())
                }
            }
            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }
        for buffered in [false, true] {
            let flags = Flags::try_parse_from(["magika", "--json", "sample"]).unwrap();
            let (sender, receiver) = std::sync::mpsc::channel();
            sender.send(Err(anyhow::anyhow!("model unavailable"))).unwrap();
            if buffered {
                sender.send(Ok(output_response(1))).unwrap();
            }
            drop(sender);
            let error =
                print_to(&flags, receiver, Arc::new(Progress::default()), &mut FailAfterOpening)
                    .unwrap_err();
            assert_eq!(error.to_string(), "model unavailable");
        }
    }

    #[test]
    fn pipeline_failure_before_any_output_still_writes_an_empty_json_array() {
        let flags = Flags::try_parse_from(["magika", "--json", "sample"]).unwrap();
        let (sender, receiver) = std::sync::mpsc::channel();
        sender.send(Err(anyhow::anyhow!("model unavailable"))).unwrap();
        drop(sender);
        let mut output = Vec::new();
        assert!(print_to(&flags, receiver, Arc::new(Progress::default()), &mut output).is_err());
        assert_eq!(
            serde_json::from_slice::<serde_json::Value>(&output).unwrap(),
            serde_json::json!([])
        );
    }

    #[test]
    #[cfg(unix)]
    fn non_utf8_json_path_is_a_fallible_error_not_a_panic() {
        use std::os::unix::ffi::OsStringExt;
        let mut response = output_response(0);
        response.path = std::ffi::OsString::from_vec(vec![b'f', 0xff]).into();
        assert!(response.json().is_err());
    }

    #[test]
    #[cfg(unix)]
    fn named_pipes_are_rejected_before_reader_dispatch() {
        let directory = std::env::temp_dir().join(format!(
            "magika-fifo-{}-{}",
            std::process::id(),
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
        ));
        std::fs::create_dir(&directory).unwrap();
        let path = directory.join("pipe");
        assert!(std::process::Command::new("mkfifo").arg(&path).status().unwrap().success());
        let flags = Flags::try_parse_from(["magika", "sample"]).unwrap();
        let mut pending = Traversal::default();
        let result = process_path(&flags, &mut pending, &path, None);
        let known = std::fs::metadata(&path).unwrap().file_type();
        let from_directory = process_path(&flags, &mut pending, &path, Some(known));
        let link = directory.join("link");
        std::os::unix::fs::symlink(&path, &link).unwrap();
        let followed = process_path(&flags, &mut pending, &link, None);
        let socket = directory.join("socket");
        let listener = std::os::unix::net::UnixListener::bind(&socket).unwrap();
        let socket_result = process_path(&flags, &mut pending, &socket, None);
        drop(listener);
        std::fs::remove_dir_all(directory).unwrap();
        assert!(result.is_err(), "FIFO dispatched to a blocking reader");
        assert!(from_directory.is_err(), "recursive FIFO dispatched to a blocking reader");
        assert!(followed.is_err(), "followed FIFO symlink dispatched to a blocking reader");
        assert!(socket_result.is_err(), "socket dispatched to a file reader");
    }

    #[test]
    fn experimental_resource_limits_reject_invalid_values_before_execution() {
        for (flag, limit) in [("--batch-size", 64), ("--threads", 256), ("--readers", 256)] {
            for value in [0, limit + 1, usize::MAX] {
                assert!(
                    Flags::try_parse_from(["magika", flag, &value.to_string(), "sample"]).is_err(),
                    "{flag} accepted {value}"
                );
            }
            for value in [1, limit] {
                assert!(
                    Flags::try_parse_from(["magika", flag, &value.to_string(), "sample"]).is_ok()
                );
            }
        }
    }

    #[test]
    fn sparse_misses_fill_batches_without_unbounded_lookahead() {
        fn run(window: usize) -> Vec<usize> {
            let progress = Arc::new(Progress::default());
            let (read_tx, read_rx) = std::sync::mpsc::sync_channel(16);
            let (batch_tx, batch_rx) = crossbeam_channel::bounded::<InferenceBatch>(2);
            let (result_tx, result_rx) = std::sync::mpsc::sync_channel(16);
            let walker_progress = progress.clone();
            let producer = std::thread::spawn(move || {
                for order in 0..1000 {
                    if !walker_progress.wait_with_notify(order, window, |blocked_on| {
                        read_tx.send(ReadItem::Flush { submitted: order, blocked_on }).is_ok()
                    }) {
                        break;
                    }
                    read_tx
                        .send(ReadItem::File {
                            pending: OrderPath { order, path: PathBuf::from("sample") },
                            extracted: if order % 20 == 0 {
                                FeaturesOrRuled::extract(&[65_u8; 128][..])
                            } else {
                                Ok(FeaturesOrRuled::Ruled(ContentType::Png))
                            },
                        })
                        .unwrap();
                }
            });
            let inference_tx = result_tx.clone();
            let inference = std::thread::spawn(move || {
                let mut sizes = Vec::new();
                while let Ok(batch) = batch_rx.recv() {
                    sizes.push(batch.features.len());
                    for pending in batch.pending {
                        inference_tx
                            .send(Ok(Response::new(pending, Ok(FileType::Ruled(ContentType::Txt)))))
                            .unwrap();
                    }
                }
                sizes
            });
            let batcher =
                std::thread::spawn(move || batch_files(8, &read_rx, &batch_tx, &result_tx));
            let mut reorder = Reorder::new(progress);
            let mut printed = 0;
            for _ in 0..1000 {
                reorder.push(
                    result_rx.recv_timeout(std::time::Duration::from_secs(2)).unwrap().unwrap(),
                );
                while let Some(response) = reorder.pop() {
                    assert_eq!(response.order, printed);
                    printed += 1;
                }
                assert!(reorder.todo.len() <= window);
            }
            assert_eq!(printed, 1000);
            producer.join().unwrap();
            batcher.join().unwrap().unwrap();
            inference.join().unwrap()
        }
        assert!(run(4 * 2 * 8).len() > 7);
        assert_eq!(run(reorder_window(2, 8)), [8, 8, 8, 8, 8, 8, 2]);
    }

    #[test]
    fn rule_only_pipeline_finishes_while_model_preparation_is_blocked() {
        use std::time::Duration;
        let (started_tx, started_rx) = std::sync::mpsc::channel();
        let (release_tx, release_rx) = std::sync::mpsc::channel();
        let (batch_tx, batch_rx) = crossbeam_channel::bounded(1);
        let (result_tx, result_rx) = std::sync::mpsc::sync_channel(1);
        let inference_tx = result_tx.clone();
        let model = std::thread::spawn(move || {
            prepare_and_infer(
                move || {
                    started_tx.send(()).unwrap();
                    release_rx.recv().unwrap();
                    anyhow::bail!("test model unavailable")
                },
                Some(1),
                &batch_rx,
                &inference_tx,
                #[cfg(feature = "_trace")]
                &Trace::default(),
            )
        });
        started_rx.recv_timeout(Duration::from_secs(1)).unwrap();
        let (read_tx, read_rx) = std::sync::mpsc::sync_channel(1);
        read_tx
            .send(ReadItem::File {
                pending: OrderPath { order: 0, path: PathBuf::from("hit") },
                extracted: Ok(FeaturesOrRuled::Ruled(ContentType::Png)),
            })
            .unwrap();
        drop(read_tx);
        batch_files(8, &read_rx, &batch_tx, &result_tx).unwrap();
        let result = result_rx.recv_timeout(Duration::from_secs(1));
        drop(batch_tx);
        drop(result_tx);
        let closed = result_rx.recv_timeout(Duration::from_secs(1));
        // Always release the blocked initializer, including on an assertion failure.
        release_tx.send(()).unwrap();
        assert!(model.join().unwrap().is_ok());
        assert!(matches!(closed, Err(std::sync::mpsc::RecvTimeoutError::Disconnected)));
        assert!(matches!(result.unwrap().unwrap().result, Ok(FileType::Ruled(ContentType::Png))));
    }

    #[test]
    fn ml_input_observes_model_preparation_failure() {
        let (batch_tx, batch_rx) = crossbeam_channel::bounded(1);
        let (result_tx, _result_rx) = std::sync::mpsc::sync_channel(1);
        let FeaturesOrRuled::Features(features) =
            FeaturesOrRuled::extract(&[65_u8; 128][..]).unwrap()
        else {
            panic!("expected ML features");
        };
        batch_tx
            .send(InferenceBatch {
                pending: vec![OrderPath { order: 0, path: PathBuf::from("miss") }],
                features: vec![features],
            })
            .unwrap();
        let error = prepare_and_infer(
            || anyhow::bail!("test model unavailable"),
            Some(1),
            &batch_rx,
            &result_tx,
            #[cfg(feature = "_trace")]
            &Trace::default(),
        )
        .unwrap_err();
        assert!(error.to_string().contains("test model unavailable"));
    }

    #[test]
    fn full_batch_backpressure_does_not_split_the_next_batch() {
        let (read_tx, read_rx) = std::sync::mpsc::sync_channel(32);
        let (batch_tx, batch_rx) = crossbeam_channel::bounded(3);
        let (result_tx, _result_rx) = std::sync::mpsc::sync_channel(1);
        for order in 0..16 {
            read_tx
                .send(ReadItem::File {
                    pending: OrderPath { order, path: PathBuf::from("miss") },
                    extracted: FeaturesOrRuled::extract(&[65_u8; 128][..]),
                })
                .unwrap();
            if order == 8 {
                // Ordered output is waiting for the already-dispatched first batch.
                read_tx.send(ReadItem::Flush { submitted: 9, blocked_on: 0 }).unwrap();
            }
        }
        drop(read_tx);
        batch_files(8, &read_rx, &batch_tx, &result_tx).unwrap();
        drop(batch_tx);
        assert_eq!(batch_rx.iter().map(|batch| batch.features.len()).collect::<Vec<_>>(), [8, 8]);
    }

    #[test]
    fn flush_waits_for_outstanding_readers_before_releasing_a_partial_batch() {
        let (read_tx, read_rx) = std::sync::mpsc::sync_channel(4);
        let (batch_tx, batch_rx) = crossbeam_channel::bounded(2);
        let (result_tx, _result_rx) = std::sync::mpsc::sync_channel(1);
        let worker = std::thread::spawn(move || batch_files(8, &read_rx, &batch_tx, &result_tx));
        // The traversal notification overtakes the second reader's result.
        read_tx.send(ReadItem::Flush { submitted: 2, blocked_on: 0 }).unwrap();
        for order in [1, 0] {
            read_tx
                .send(ReadItem::File {
                    pending: OrderPath { order, path: PathBuf::from("miss") },
                    extracted: FeaturesOrRuled::extract(&[65_u8; 128][..]),
                })
                .unwrap();
        }
        // Keep the input channel open: the explicit barrier must release output before EOF.
        let batch = batch_rx.recv_timeout(std::time::Duration::from_secs(1));
        drop(read_tx);
        worker.join().unwrap().unwrap();
        assert_eq!(batch.unwrap().features.len(), 2);
        assert!(batch_rx.try_recv().is_err());
    }

    #[test]
    fn ordered_results_advance_progress_and_drop_closes_it() {
        let progress = Arc::new(Progress::default());
        let mut reorder = Reorder::new(progress.clone());
        let response = |order| {
            Response::new(
                OrderPath { order, path: PathBuf::from("sample") },
                Ok(FileType::Directory),
            )
        };
        reorder.push(response(1));
        assert!(reorder.pop().is_none());
        reorder.push(response(0));
        assert_eq!(reorder.pop().unwrap().order, 0);
        assert!(progress.wait(5, 4));
        assert_eq!(reorder.pop().unwrap().order, 1);
        assert!(progress.wait(6, 4));
        assert!(reorder.is_empty());
        drop(reorder);
        assert!(!progress.wait(0, 4));
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
    UnsupportedFileType,
    DirectoryCycle,
}

#[derive(Serialize)]
struct JsonResult<'a> {
    dl: &'a TypeInfo,
    output: &'a TypeInfo,
    score: f32,
}

impl From<anyhow::Error> for JsonError {
    fn from(value: anyhow::Error) -> Self {
        if let Some(TraversalError::DirectoryCycle) = value.downcast_ref::<TraversalError>() {
            return JsonError::DirectoryCycle;
        }
        match value.root_cause().downcast_ref::<std::io::Error>() {
            Some(x) => match x.kind() {
                ErrorKind::NotFound => JsonError::FileDoesNotExist,
                ErrorKind::PermissionDenied => JsonError::PermissionError,
                ErrorKind::Unsupported => JsonError::UnsupportedFileType,
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
        let path = serde_json::to_value(&self.path)?;
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
