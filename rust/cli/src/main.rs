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
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{bail, ensure, Result};
use clap::{Args, Parser};
use colored::{ColoredString, Colorize};
use magika::{ContentType, Features, FeaturesOrRuled, FileType, RuledType, Session, TypeInfo};
use ort::session::builder::GraphOptimizationLevel;
use serde::Serialize;
use tokio::fs::File;
use tokio::io::AsyncReadExt;

/// Determines the content type of files with deep-learning.
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
    /// Number of files to identify in a single inference.
    #[arg(hide = true, long, default_value = "1")]
    batch_size: usize,

    /// Number of tasks for batch parallelism (defaults to the number of CPUs).
    #[arg(hide = true, long)]
    num_tasks: Option<usize>,

    /// Number of threads for graph parallelism (ONNX Runtime configuration).
    ///
    /// This has no effect if --parallel-execution is false or unset.
    #[arg(hide = true, long)]
    inter_threads: Option<usize>,

    /// Number of threads for node parallelism (ONNX Runtime configuration).
    #[arg(hide = true, long)]
    intra_threads: Option<usize>,

    /// Graph optimization level, from 0 to 3 (ONNX Runtime configuration).
    #[arg(hide = true, long)]
    optimization_level: Option<usize>,

    /// Whether to enable parallel execution (ONNX Runtime configuration).
    #[arg(hide = true, long)]
    parallel_execution: Option<bool>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let flags = Arc::new(Flags::parse());
    ensure!(0 < flags.experimental.batch_size, "--batch-size cannot be zero");
    let num_tasks = flags.experimental.num_tasks.unwrap_or_else(num_cpus::get);
    ensure!(0 < num_tasks, "--num-tasks cannot be zero");
    ensure!(
        flags.path.iter().filter(|x| x.to_str() == Some("-")).count() <= 1,
        "only one path can be the standard input"
    );
    if flags.colors.enable {
        colored::control::set_override(true);
    }
    if flags.colors.disable {
        colored::control::set_override(false);
    }
    let (result_sender, mut result_receiver) =
        tokio::sync::mpsc::channel::<Result<Response>>(num_tasks * flags.experimental.batch_size);
    let (batch_sender, batch_receiver) = async_channel::bounded::<Batch>(num_tasks);
    tokio::spawn({
        let flags = flags.clone();
        let result_sender = result_sender.clone();
        async move {
            if let Err(e) = extract_features(&flags, &batch_sender, &result_sender).await {
                result_sender.send(Err(e)).await.unwrap();
            }
        }
    });
    let magika = Arc::new(build_session(&flags)?);
    for _ in 0..num_tasks {
        tokio::spawn({
            let magika = magika.clone();
            let batch_receiver = batch_receiver.clone();
            let result_sender = result_sender.clone();
            async move {
                if let Err(e) = infer_batch(&magika, &batch_receiver, &result_sender).await {
                    result_sender.send(Err(e)).await.unwrap();
                }
            }
        });
    }
    drop(result_sender);
    if flags.format.json {
        print!("[");
    }
    let mut reorder = Reorder::default();
    let mut errors = false;
    while let Some(response) = result_receiver.recv().await {
        reorder.push(response?);
        while let Some(response) = reorder.pop() {
            errors |= response.result.is_err();
            if flags.format.json {
                if reorder.next != 1 {
                    print!(",");
                }
                for line in serde_json::to_string_pretty(&response.json()?)?.lines() {
                    print!("\n  {line}");
                }
            } else {
                println!("{}", response.format(&flags)?);
            }
        }
    }
    debug_assert!(reorder.is_empty());
    if flags.format.json {
        if reorder.next != 0 {
            println!();
        }
        println!("]");
    }
    if errors {
        std::process::exit(1);
    }
    Ok(())
}

async fn extract_features(
    flags: &Flags, batch_sender: &async_channel::Sender<Batch>,
    result_sender: &tokio::sync::mpsc::Sender<Result<Response>>,
) -> Result<()> {
    let mut paths = Vec::new();
    let mut features = Vec::new();
    let mut flags_paths = flags.path.clone();
    flags_paths.reverse();
    let mut order = 0;
    while let Some(path) = flags_paths.pop() {
        let mut result = None;
        match process_path(flags, &mut flags_paths, &path).await {
            Err(x) => result = Some(Err(x)),
            Ok(ProcessPath::Recursive) => continue,
            Ok(ProcessPath::Ruled(x)) => result = Some(Ok(x)),
            Ok(ProcessPath::Features(x)) => features.push(x),
        };
        match result {
            Some(result) => result_sender.send(Ok(Response { order, path, result })).await?,
            None => paths.push((order, path)),
        }
        order += 1;
        if features.len() == flags.experimental.batch_size {
            batch_sender.send(Batch { paths, features }).await?;
            paths = Vec::new();
            features = Vec::new();
        }
    }
    if !paths.is_empty() {
        batch_sender.send(Batch { paths, features }).await?;
    }
    Ok(())
}

enum ProcessPath {
    Recursive,
    Features(Features),
    Ruled(FileType),
}

impl From<FeaturesOrRuled> for ProcessPath {
    fn from(value: FeaturesOrRuled) -> Self {
        match value {
            FeaturesOrRuled::Features(x) => ProcessPath::Features(x),
            FeaturesOrRuled::Ruled(x) => ProcessPath::Ruled(x.into()),
        }
    }
}

async fn process_path(
    flags: &Flags, paths: &mut Vec<PathBuf>, path: &Path,
) -> magika::Result<ProcessPath> {
    if path.to_str() == Some("-") {
        let mut stdin = Vec::new();
        tokio::io::stdin().read_to_end(&mut stdin).await?;
        return Ok(FeaturesOrRuled::extract_sync(&stdin[..])?.into());
    }
    let metadata = if flags.no_dereference {
        tokio::fs::symlink_metadata(&path).await?
    } else {
        tokio::fs::metadata(&path).await?
    };
    if metadata.is_dir() {
        return Ok(if flags.recursive {
            let mut entries = tokio::fs::read_dir(&path).await?;
            let mut dir_paths = Vec::new();
            while let Some(entry) = entries.next_entry().await? {
                dir_paths.push(entry.path());
            }
            dir_paths.sort();
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
    let file = File::open(&path).await?;
    Ok(FeaturesOrRuled::extract_async(file).await?.into())
}

fn build_session(flags: &Flags) -> Result<Session> {
    ort::init().with_telemetry(false).commit()?;
    let mut magika = Session::builder();
    if let Some(inter_threads) = flags.experimental.inter_threads {
        magika = magika.with_inter_threads(inter_threads);
    }
    // Apparently, SetIntraOpNumThreads must be called on MacOS, otherwise we get the following
    // error: intra op thread pool must have at least one thread for RunAsync.
    let intra_threads_default = cfg!(target_os = "macos").then_some(4);
    if let Some(intra_threads) = flags.experimental.intra_threads.or(intra_threads_default) {
        magika = magika.with_intra_threads(intra_threads);
    }
    if let Some(opt_level) = flags.experimental.optimization_level {
        let opt_level = match opt_level {
            0 => GraphOptimizationLevel::Disable,
            1 => GraphOptimizationLevel::Level1,
            2 => GraphOptimizationLevel::Level2,
            3 => GraphOptimizationLevel::Level3,
            _ => bail!("--optimization-level must be 0, 1, 2, or 3"),
        };
        magika = magika.with_optimization_level(opt_level);
    }
    if let Some(parallel_execution) = flags.experimental.parallel_execution {
        magika = magika.with_parallel_execution(parallel_execution);
    }
    Ok(magika.build()?)
}

async fn infer_batch(
    magika: &Session, receiver: &async_channel::Receiver<Batch>,
    sender: &tokio::sync::mpsc::Sender<Result<Response>>,
) -> Result<()> {
    while let Ok(Batch { paths, features }) = receiver.recv().await {
        let batch = magika.identify_features_batch_async(&features).await?;
        assert_eq!(batch.len(), paths.len());
        for ((order, path), output) in paths.into_iter().zip(batch.into_iter()) {
            let result = Ok(output);
            sender.send(Ok(Response { order, path, result })).await?;
        }
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

struct Batch {
    paths: Vec<(usize, PathBuf)>,
    features: Vec<Features>,
}

#[derive(Debug)]
struct Response {
    order: usize,
    path: PathBuf,
    result: Result<FileType, magika::Error>,
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
                        if let Ok(FileType::Ruled(RuledType { overruled: Some(x), .. })) =
                            &self.result
                        {
                            write!(
                                &mut result,
                                " [Low-confidence model best-guess: {} ({}), score={:.3}]",
                                x.content_type.info().description,
                                x.content_type.info().group,
                                x.score,
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
                    FileType::Inferred(x) => x.content_type.info(),
                    FileType::Ruled(RuledType { overruled: Some(x), .. }) => x.content_type.info(),
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

    fn description(&self) -> Cow<str> {
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
        match &self.result {
            Err(_) => result.bold().red(),
            Ok(x) => match x.info().group {
                "document" => result.bold().magenta(),
                "executable" => result.bold().green(),
                "archive" => result.bold().red(),
                "audio" => result.yellow(),
                "image" => result.yellow(),
                "video" => result.yellow(),
                "code" => result.bold().blue(),
                _ => result.bold(),
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
