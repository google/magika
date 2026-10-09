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

use std::collections::{HashMap, HashSet};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::mpsc::{Receiver, SyncSender};
use std::thread::JoinHandle;

use anyhow::{Result, ensure};

use super::Engine;
use crate::{Backend, Features, FeaturesOrRuled, FileType, Options};

/// Inference threads it takes to keep a GPU queued.
///
/// The device is the bottleneck there, and it is already busy well before the host runs out of
/// cores, so further threads only queue behind each other. Measured on an M5 Max, throughput is
/// flat from four threads to sixteen.
const GPU_INFERENCE_THREADS: usize = 4;

/// How a [`Pipeline`] walks and identifies paths.
///
/// Start from the default and set fields: `let mut config = PipelineOptions::default();` then
/// `config.recursive = true;`.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct PipelineOptions {
    /// Identifies the files within directories instead of the directories themselves.
    pub recursive: bool,
    /// Reads the path `-` from the standard input.
    pub stdin: bool,
    /// Number of files to identify in a single inference, between 1 and 64.
    pub batch_size: usize,
    /// Number of inference threads, between 1 and 256, or `None` to size them for the backend.
    ///
    /// Inference on a GPU is bound by the device rather than by the host, so a handful of threads
    /// keep it busy and more only contend for it. Inference on a CPU is bound by the host, so every
    /// thread is one more core doing the work. This defaults accordingly: four on a GPU, all
    /// available logical CPUs on x86_64 Linux, and one fewer on other CPU targets.
    pub threads: Option<usize>,
    /// Number of threads reading files and extracting features, between 1 and 256.
    ///
    /// Reading costs far less than inference, so one is already more than a run makes use of.
    pub readers: usize,
}

impl Default for PipelineOptions {
    fn default() -> Self {
        PipelineOptions { recursive: false, stdin: false, batch_size: 8, threads: None, readers: 1 }
    }
}

/// The error of a directory entered again during a recursive walk.
#[derive(Debug, thiserror::Error)]
#[error("Directory cycle")]
pub struct DirectoryCycle;

/// Identifies many paths in parallel, and returns their results in order.
///
/// A walk thread traverses the paths, reader threads extract features, a batch thread groups them,
/// and inference threads identify the batches with the [`Engine`]. While the engine prepares a
/// GPU, only two thirds of the CPU inference threads start, leaving the host to the preparation;
/// the others start only if the GPU cannot be used.
pub struct Pipeline {
    engine: Arc<Engine>,
    options: Options,
    config: PipelineOptions,
}

impl Pipeline {
    /// Creates a pipeline identifying with `options`.
    pub fn new(engine: Arc<Engine>, options: Options, config: PipelineOptions) -> Result<Self> {
        ensure!((1..=64).contains(&config.batch_size), "the batch size must be between 1 and 64");
        ensure!(
            config.threads.is_none_or(|x| (1..=256).contains(&x)),
            "the number of threads must be between 1 and 256"
        );
        ensure!(
            (1..=256).contains(&config.readers),
            "the number of readers must be between 1 and 256"
        );
        Ok(Pipeline { engine, options, config })
    }

    /// Identifies paths, yielding each one with its result in the order of a sequential walk.
    ///
    /// An item is an error only when the pipeline itself fails, after which it yields nothing. A
    /// file that cannot be identified yields its error with its path. Dropping the iterator stops
    /// the pipeline.
    pub fn identify_paths(&self, paths: Vec<PathBuf>) -> Result<Identified> {
        let Pipeline { engine, options, config } = self;
        let batch_size = config.batch_size;
        // Queues are sized before knowing which backend identifies the files, so for the busiest.
        let threads = config.threads.unwrap_or_else(|| {
            default_inference_threads(Backend::Cpu).max(default_inference_threads(Backend::Gpu))
        });
        let (work_sender, work_receiver) = crossbeam_channel::bounded::<Pending>(config.readers);
        let (read_sender, read_receiver) =
            std::sync::mpsc::sync_channel::<ReadItem>(threads * batch_size);
        let (batch_sender, batch_receiver) = crossbeam_channel::bounded::<Vec<BatchItem>>(threads);
        let (result_sender, result_receiver) =
            std::sync::mpsc::sync_channel::<Result<Response>>(threads * batch_size);
        #[cfg(feature = "_trace")]
        let trace = Trace::default();
        let mut handles = Vec::new();
        let mut spawn = |name: String, f: Box<dyn FnOnce() + Send>| -> Result<()> {
            #[cfg(feature = "_trace")]
            let f = trace.stage(f);
            handles.push(std::thread::Builder::new().name(name).spawn(f)?);
            Ok(())
        };
        spawn("magika-walk".to_string(), {
            let (config, options) = (config.clone(), options.clone());
            let result_sender = result_sender.clone();
            Box::new(move || {
                if let Err(e) = walk_paths(paths, &config, &options, &work_sender, &result_sender) {
                    let _ = result_sender.send(Err(e));
                }
            })
        })?;
        for index in 0..config.readers {
            let (config, options) = (config.clone(), options.clone());
            let (work_receiver, read_sender) = (work_receiver.clone(), read_sender.clone());
            spawn(
                format!("magika-read-{index}"),
                Box::new(move || read_files(&work_receiver, &read_sender, &config, &options)),
            )?;
        }
        drop((work_receiver, read_sender));
        spawn("magika-batch".to_string(), {
            let (batch_sender, result_sender) = (batch_sender.clone(), result_sender.clone());
            Box::new(move || {
                if let Err(e) =
                    batch_files(batch_size, &read_receiver, &batch_sender, &result_sender)
                {
                    let _ = result_sender.send(Err(e));
                }
            })
        })?;
        drop(batch_sender);
        spawn("magika-model".to_string(), {
            let (engine, options, config) = (engine.clone(), options.clone(), config.clone());
            #[cfg(feature = "_trace")]
            let trace = trace.clone();
            Box::new(move || {
                let workers =
                    config.threads.unwrap_or_else(|| default_inference_threads(Backend::Cpu));
                // The CPU competes with preparing the GPU, which a long run waits for, so the
                // workers leave it a third of the host until it is ready. A GPU is as fast with as
                // many workers.
                let starting = match (engine.has_gpu(), config.threads) {
                    (false, _) | (true, Some(_)) => workers,
                    (true, None) => {
                        (workers * 2 / 3).max(default_inference_threads(Backend::Gpu)).min(workers)
                    }
                };
                let (engine, options) = (&*engine, &options);
                let (batch_receiver, result_sender) = (&batch_receiver, &result_sender);
                #[cfg(feature = "_trace")]
                let trace = &trace;
                std::thread::scope(|scope| {
                    // Each starting worker holds `done` until it finishes, which is when the run
                    // ends.
                    let spawn = |worker: usize, done: Option<crossbeam_channel::Sender<()>>| {
                        let work = move || {
                            let _done = done;
                            if let Err(e) =
                                infer_batches(engine, options, batch_receiver, result_sender)
                            {
                                let _ = result_sender.send(Err(e));
                            }
                        };
                        #[cfg(feature = "_trace")]
                        let work = trace.stage(work);
                        let spawned = std::thread::Builder::new()
                            .name(format!("magika-infer-{worker}"))
                            .spawn_scoped(scope, work);
                        if let Err(e) = spawned {
                            let _ = result_sender.send(Err(e.into()));
                        }
                    };
                    let (done, finished) = crossbeam_channel::bounded(0);
                    for worker in 0..starting {
                        spawn(worker, Some(done.clone()));
                    }
                    drop(done);
                    if engine.gpu_failed(&finished) {
                        for worker in starting..workers {
                            spawn(worker, None);
                        }
                    }
                });
            })
        })?;
        Ok(Identified {
            results: Some(result_receiver),
            reorder: Reorder::default(),
            handles,
            #[cfg(feature = "_trace")]
            trace: (trace, config.readers),
        })
    }
}

/// The results of [`Pipeline::identify_paths`], in order.
pub struct Identified {
    results: Option<Receiver<Result<Response>>>,
    reorder: Reorder,
    handles: Vec<JoinHandle<()>>,
    #[cfg(feature = "_trace")]
    trace: (Trace, usize),
}

impl Iterator for Identified {
    type Item = Result<(PathBuf, Result<FileType>)>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(response) = self.reorder.pop() {
                return Some(Ok((response.path, response.result)));
            }
            match self.results.as_ref()?.recv() {
                Ok(Ok(response)) => self.reorder.push(response),
                Ok(Err(error)) => {
                    self.results = None;
                    return Some(Err(error));
                }
                // Every stage is done. A stage that panicked lost the results it held, which
                // must not pass for a shorter run.
                Err(_) => {
                    self.results = None;
                    let panicked = self.handles.drain(..).any(|handle| handle.join().is_err());
                    if panicked || !self.reorder.is_empty() {
                        return Some(Err(anyhow::anyhow!("a pipeline thread panicked")));
                    }
                    return None;
                }
            }
        }
    }
}

impl Drop for Identified {
    fn drop(&mut self) {
        // Every stage stops once the stage after it is gone.
        self.results = None;
        for handle in self.handles.drain(..) {
            let _ = handle.join();
        }
        #[cfg(feature = "_trace")]
        self.trace.0.report(self.trace.1);
    }
}

/// Returns how many inference threads it takes to keep a backend busy.
///
/// `available_parallelism` is the portable answer on every target magika ships to, and it reports
/// what this process may use rather than what the machine is built from, so a container's CPU quota
/// and a restricted affinity mask both count.
fn default_inference_threads(backend: Backend) -> usize {
    let available = std::thread::available_parallelism().map_or(1, |x| x.get()).min(256);
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

/// Walks the requested paths and hands regular files to the read threads.
///
/// This task only traverses and stats. Reading file content is left to [`read_files`] so that it
/// happens on several threads at once instead of serializing behind traversal.
fn walk_paths(
    paths: Vec<PathBuf>, config: &PipelineOptions, options: &Options,
    work_sender: &crossbeam_channel::Sender<Pending>, result_sender: &SyncSender<Result<Response>>,
) -> Result<()> {
    let mut traversal = Traversal::new(paths);
    let mut order = 0;
    while let Some((path, file_type)) = traversal.pop() {
        let processed = process_path(config, options, &mut traversal, &path, file_type);
        if matches!(processed, Ok(ProcessPath::Recursive)) {
            continue;
        }
        let pending = Pending { order, path };
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
    work_receiver: &crossbeam_channel::Receiver<Pending>, sender: &SyncSender<ReadItem>,
    config: &PipelineOptions, options: &Options,
) {
    while let Ok(pending) = work_receiver.recv() {
        let extracted = extract_path(&pending.path, config, options);
        if sender.send(ReadItem { pending, extracted }).is_err() {
            break;
        }
    }
}

/// Accumulates every reader's output into one global inference batch stream.
fn batch_files(
    batch_size: usize, receiver: &Receiver<ReadItem>,
    batch_sender: &crossbeam_channel::Sender<Vec<BatchItem>>,
    result_sender: &SyncSender<Result<Response>>,
) -> Result<()> {
    let mut batch = Vec::with_capacity(batch_size);
    while let Ok(ReadItem { pending, extracted }) = receiver.recv() {
        match extracted {
            Ok(FeaturesOrRuled::Features(features)) => {
                batch.push(BatchItem { pending, features });
                if batch.len() == batch_size {
                    let full = std::mem::replace(&mut batch, Vec::with_capacity(batch_size));
                    batch_sender.send(full)?;
                }
            }
            Ok(FeaturesOrRuled::Ruled(file_type)) => {
                result_sender.send(Ok(Response::new(pending, Ok(file_type))))?;
            }
            Err(error) => {
                result_sender.send(Ok(Response::new(pending, Err(error))))?;
            }
        }
    }
    if !batch.is_empty() {
        batch_sender.send(batch)?;
    }
    Ok(())
}

/// Reads a file and extracts its features, unless rules identify it.
fn extract_path(
    path: &Path, config: &PipelineOptions, options: &Options,
) -> Result<FeaturesOrRuled> {
    if config.stdin && path.to_str() == Some("-") {
        let mut stdin = Vec::new();
        std::io::stdin().read_to_end(&mut stdin)?;
        return FeaturesOrRuled::extract_content(&stdin[..], options);
    }
    FeaturesOrRuled::extract_content(std::fs::File::open(path)?, options)
}

fn infer_batches(
    engine: &Engine, options: &Options, receiver: &crossbeam_channel::Receiver<Vec<BatchItem>>,
    sender: &SyncSender<Result<Response>>,
) -> Result<()> {
    // Sessions are spawned on a backend's first batch: a short run never reaches most threads.
    let mut session = engine.session();
    while let Ok(batch) = receiver.recv() {
        let features = batch.iter().map(|x| &x.features).collect::<Vec<_>>();
        let results = session.identify_features_batch(&features, options)?;
        debug_assert_eq!(results.len(), batch.len());
        for (item, output) in batch.into_iter().zip(results) {
            sender.send(Ok(Response::new(item.pending, Ok(output))))?;
        }
    }
    Ok(())
}

enum ProcessPath {
    Recursive,
    Content,
    Ruled(FileType),
}

struct Pending {
    order: usize,
    path: PathBuf,
}

struct ReadItem {
    pending: Pending,
    extracted: Result<FeaturesOrRuled>,
}

struct BatchItem {
    pending: Pending,
    features: Features,
}

/// A path still to process, or the point where traversal leaves a directory.
enum WalkEntry {
    Path(PathBuf, Option<std::fs::FileType>),
    LeaveDirectory(PathBuf),
}

struct Traversal {
    pending: Vec<WalkEntry>,
    ancestors: HashSet<PathBuf>,
}

impl Traversal {
    fn new(paths: Vec<PathBuf>) -> Self {
        let pending = paths.into_iter().rev().map(|path| WalkEntry::Path(path, None)).collect();
        Traversal { pending, ancestors: HashSet::new() }
    }

    fn push(&mut self, path: &Path) -> Result<()> {
        let canonical = std::fs::canonicalize(path)?;
        ensure!(self.ancestors.insert(canonical.clone()), DirectoryCycle);
        self.pending.push(WalkEntry::LeaveDirectory(canonical));
        Ok(())
    }

    fn pop(&mut self) -> Option<(PathBuf, Option<std::fs::FileType>)> {
        while let Some(entry) = self.pending.pop() {
            match entry {
                WalkEntry::Path(path, kind) => return Some((path, kind)),
                WalkEntry::LeaveDirectory(path) => drop(self.ancestors.remove(&path)),
            }
        }
        None
    }
}

fn process_path(
    config: &PipelineOptions, options: &Options, traversal: &mut Traversal, path: &Path,
    known: Option<std::fs::FileType>,
) -> Result<ProcessPath> {
    if config.stdin && path.to_str() == Some("-") {
        return Ok(ProcessPath::Content);
    }
    // `read_dir` already reported the type of every entry it produced, so reading its metadata
    // again would be one extra system call per file on the one task that feeds every reader. Only
    // a symlink still needs a lookup, and only when it is being followed.
    let metadata = match known {
        Some(known) if !options.follow_symlink || !known.is_symlink() => known,
        _ => {
            if options.follow_symlink {
                std::fs::metadata(path)?.file_type()
            } else {
                std::fs::symlink_metadata(path)?.file_type()
            }
        }
    };
    if metadata.is_dir() {
        return Ok(if config.recursive {
            traversal.push(path)?;
            let mut dir_paths = Vec::new();
            for entry in std::fs::read_dir(path)? {
                let entry = entry?;
                dir_paths.push((entry.path(), entry.file_type().ok()));
            }
            dir_paths.sort_by(|a, b| a.0.cmp(&b.0));
            while let Some((path, kind)) = dir_paths.pop() {
                traversal.pending.push(WalkEntry::Path(path, kind));
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
        return Ok(ProcessPath::Ruled(FileType::Unsupported));
    }
    Ok(ProcessPath::Content)
}

#[derive(Default)]
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

struct Response {
    order: usize,
    path: PathBuf,
    result: Result<FileType>,
}

impl Response {
    fn new(pending: Pending, result: Result<FileType>) -> Self {
        Self { order: pending.order, path: pending.path, result }
    }
}

/// Per-stage busy and waiting time, reported on the standard error when the run ends.
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
    /// Wraps a stage's thread body to record its busy and waiting time under the thread's name.
    fn stage<F: FnOnce() + Send>(&self, f: F) -> impl FnOnce() + Send + use<F> {
        let trace = self.clone();
        move || {
            let (wall, thread) = (std::time::Instant::now(), cpu_time::ThreadTime::now());
            f();
            let total_ns = wall.elapsed().as_nanos() as u64;
            let busy_ns = thread.elapsed().as_nanos() as u64;
            let stage = Stage { busy_ns, wait_ns: total_ns.saturating_sub(busy_ns) };
            let name = std::thread::current().name().unwrap().to_string();
            assert!(trace.stages.lock().unwrap().insert(name, stage).is_none());
        }
    }

    fn report(&self, readers: usize) {
        let mut stages = self.stages.lock().unwrap();
        let mut report = Vec::new();
        report.push(("walk".to_string(), stages.remove("magika-walk")));
        for i in 0..readers {
            report.push((format!("read[{i}]"), stages.remove(&format!("magika-read-{i}"))));
        }
        report.push(("batch".to_string(), stages.remove("magika-batch")));
        // The backend decides how many workers run once the model is loaded.
        for i in 0.. {
            let Some(stage) = stages.remove(&format!("magika-infer-{i}")) else { break };
            report.push((format!("infer[{i}]"), Some(stage)));
        }
        eprintln!("trace  stage           busy      waiting   busy%");
        for (name, stage) in report {
            let Some(Stage { busy_ns: busy, wait_ns: wait }) = stage else { continue };
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Builder, Runtime};

    fn pipeline(recursive: bool) -> Pipeline {
        let engine = Arc::new(Engine::new(Builder::default().with_backend(Backend::Cpu)).unwrap());
        let config = PipelineOptions { recursive, ..PipelineOptions::default() };
        Pipeline::new(engine, Options::default(), config).unwrap()
    }

    fn identify(pipeline: &Pipeline, paths: &[&Path]) -> Vec<(PathBuf, Result<FileType>)> {
        let paths = paths.iter().map(|x| x.to_path_buf()).collect();
        pipeline.identify_paths(paths).unwrap().map(Result::unwrap).collect()
    }

    fn label(result: &Result<FileType>) -> &'static str {
        result.as_ref().unwrap().info().label
    }

    #[test]
    fn identifies_like_a_session_in_walk_order() {
        let basic = Path::new("../../tests_data/basic");
        let results = identify(&pipeline(true), &[basic]);
        assert!(results.len() > 100, "{}", results.len());
        fn count(path: &Path) -> usize {
            match path.is_dir() {
                true => std::fs::read_dir(path).unwrap().map(|x| count(&x.unwrap().path())).sum(),
                false => 1,
            }
        }
        assert_eq!(results.len(), count(basic));
        let runtime = Runtime::builder().with_backend(Backend::Cpu).build().unwrap();
        let mut session = runtime.session().unwrap();
        let paths: Vec<_> = results.iter().map(|(path, _)| path.clone()).collect();
        let mut sorted = paths.clone();
        sorted.sort();
        assert_eq!(paths, sorted, "a sequential walk visits sorted paths");
        for (path, result) in &results {
            let reference = session.identify_file(path).unwrap();
            assert_eq!(label(result), reference.info().label, "{}", path.display());
        }
    }

    #[test]
    fn errors_and_special_paths_do_not_stop_the_run() {
        let dir = std::env::temp_dir().join(format!("magika-pipeline-{}", std::process::id()));
        std::fs::create_dir_all(dir.join("inner")).unwrap();
        std::fs::write(dir.join("inner/file.txt"), "hello world\n".repeat(10)).unwrap();
        #[cfg(unix)]
        std::os::unix::fs::symlink(&dir, dir.join("inner/cycle")).unwrap();
        let missing = dir.join("missing");
        let results = identify(&pipeline(false), &[&dir, &missing, &dir.join("inner/file.txt")]);
        assert!(matches!(results[0].1, Ok(FileType::Directory)));
        let error = results[1].1.as_ref().unwrap_err();
        assert!(error.downcast_ref::<std::io::Error>().is_some(), "{error}");
        assert_eq!(label(&results[2].1), "txt");
        #[cfg(unix)]
        {
            let results = identify(&pipeline(true), &[&dir]);
            let cycle = results.iter().find(|(path, _)| path.ends_with("cycle")).unwrap();
            assert!(cycle.1.as_ref().unwrap_err().downcast_ref::<DirectoryCycle>().is_some());
            let fifo = dir.join("fifo");
            assert!(std::process::Command::new("mkfifo").arg(&fifo).status().unwrap().success());
            let results = identify(&pipeline(false), &[&fifo]);
            assert!(matches!(results[0].1, Ok(FileType::Unsupported)));
        }
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn dropping_the_results_stops_the_run() {
        let pipeline = pipeline(true);
        let mut results = pipeline.identify_paths(vec!["../../tests_data".into()]).unwrap();
        assert!(results.next().is_some());
        drop(results);
    }

    #[test]
    fn invalid_options_are_rejected() {
        let engine = Arc::new(Engine::new(Builder::default().with_backend(Backend::Cpu)).unwrap());
        let config = PipelineOptions { batch_size: 0, ..PipelineOptions::default() };
        assert!(Pipeline::new(engine, Options::default(), config).is_err());
    }
}
