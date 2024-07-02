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
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{bail, ensure, Result};
use clap::{Args, CommandFactory, Parser};
use colored::{ColoredString, Colorize};
use data_encoding::HEXLOWER;
use ort::GraphOptimizationLevel;
use serde::Serialize;
use sha2::Digest;
use tokio::fs::File;
use tokio::io::AsyncReadExt;

/// Determines the content type of files with deep-learning.
#[derive(Parser)]
#[command(version, arg_required_else_help = true)]
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

    /// Generates a report for feedback.
    #[arg(long)]
    generate_report: bool,

    #[clap(flatten)]
    experimental: Experimental,
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
    ///   %c  A short code describing the content type
    ///   %d  The description of the content type
    ///   %g  The group of the content type
    ///   %m  The magic of the content type
    ///   %M  The MIME type of the content type
    ///   %e  The file extensions of the content type
    ///   %s  The score of the content type for the file
    ///   %S  The score of the content type for the file in percent
    ///   %%  A literal %
    #[arg(long = "format", verbatim_doc_comment)]
    custom: Option<String>,
}

#[derive(Args)]
struct Experimental {
    /// Number of files to identify in a single inference.
    #[arg(hide = true, long, default_value = "1")]
    batch_size: usize,

    /// Number of tasks for batch parallelism.
    #[arg(hide = true, long, default_value = "1")]
    num_tasks: usize,

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
    ensure!(0 < flags.experimental.num_tasks, "--num-tasks cannot be zero");
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
    let (result_sender, mut result_receiver) = tokio::sync::mpsc::channel::<Result<Response>>(
        flags.experimental.num_tasks * flags.experimental.batch_size,
    );
    let (batch_sender, batch_receiver) =
        async_channel::bounded::<Batch>(flags.experimental.num_tasks);
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
    for _ in 0..flags.experimental.num_tasks {
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
    let mut responses = (flags.generate_report || flags.format.json).then(Vec::new);
    let mut reorder = Reorder::default();
    while let Some(response) = result_receiver.recv().await {
        reorder.push(response?);
        while let Some(response) = reorder.pop() {
            if let Some(responses) = &mut responses {
                responses.push(response.clone());
            }
            if !flags.format.json {
                println!("{}", response.format(&flags)?);
            }
        }
    }
    debug_assert!(reorder.is_empty());
    let mut reports = Vec::new();
    if flags.generate_report {
        for response in responses.as_ref().unwrap() {
            let hash = sha256_hex(&response.path).await.ok();
            let features = serde_json::to_string(&JsonFeatures::new(&response.path).await?)?;
            let mut result = response.clone().json()?;
            result.path = "<REMOVED>".into();
            reports.push(JsonReportFile { hash, features, result });
        }
    }
    if flags.format.json {
        let json: Vec<Json> =
            responses.unwrap().into_iter().map(Response::json).collect::<Result<_>>()?;
        serde_json::to_writer_pretty(std::io::stdout(), &json)?;
        println!();
    }
    if flags.generate_report {
        let version = Flags::command().get_version().map(|x| x.to_string());
        let report = JsonReport { version, reports };
        eprintln!(
            "########################################\n\
             ###              REPORT              ###\n\
             ########################################"
        );
        serde_json::to_writer(std::io::stderr(), &report)?;
        eprintln!(
            "########################################\n\
             Please copy/paste the above as a description of your issue. \
             Open a GitHub issue or reach out at magika-dev@google.com.\n\
             Please include as many details as possible, e.g., what was \
             the expected content type.\n\
             IMPORTANT: do NOT submit private information or PII! The \
             extracted features include many bytes of the tested files!"
        );
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
        let features_or_output: magika::FeaturesOrOutput = if path.to_str() == Some("-") {
            let mut stdin = Vec::new();
            tokio::io::stdin().read_to_end(&mut stdin).await?;
            magika::FeaturesOrOutput::extract_sync(&stdin[..])?
        } else {
            let mut result = None;
            let metadata = if flags.no_dereference {
                tokio::fs::symlink_metadata(&path).await
            } else {
                tokio::fs::metadata(&path).await
            };
            match metadata {
                Ok(metadata) => {
                    if metadata.is_dir() {
                        if flags.recursive {
                            let mut entries = tokio::fs::read_dir(&path).await?;
                            let mut paths = Vec::new();
                            while let Some(entry) = entries.next_entry().await? {
                                paths.push(entry.path());
                            }
                            paths.sort();
                            while let Some(path) = paths.pop() {
                                flags_paths.push(path);
                            }
                            continue;
                        } else {
                            result = Some(CliOutput::Directory);
                        }
                    } else if metadata.is_symlink() {
                        result = Some(CliOutput::Symlink);
                    } else if !metadata.is_file() {
                        result = Some(CliOutput::Label(magika::Label::Unknown));
                    } else if metadata.len() == 0 {
                        result = Some(CliOutput::Empty);
                    }
                }
                Err(error) => result = Some(CliOutput::Error(format!("{error}"))),
            }
            let mut file = None;
            if result.is_none() {
                match File::open(&path).await {
                    Ok(x) => file = Some(x),
                    Err(e) => result = Some(CliOutput::Error(format!("{e}"))),
                }
            }
            if let Some(output) = result {
                result_sender.send(Ok(Response { order, path, is_dl: false, output })).await?;
                order += 1;
                continue;
            }
            magika::FeaturesOrOutput::extract_async(file.unwrap()).await?
        };
        match features_or_output {
            magika::FeaturesOrOutput::Output(output) => {
                let output = CliOutput::Output(output);
                result_sender.send(Ok(Response { order, path, is_dl: false, output })).await?;
                order += 1;
                continue;
            }
            magika::FeaturesOrOutput::Features(x) => features.push(x),
        }
        paths.push((order, path));
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

fn build_session(flags: &Flags) -> Result<magika::Session> {
    let mut magika = magika::Session::builder();
    if let Some(inter_threads) = flags.experimental.inter_threads {
        magika = magika.with_inter_threads(inter_threads);
    }
    if let Some(intra_threads) = flags.experimental.intra_threads {
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
    magika: &magika::Session, receiver: &async_channel::Receiver<Batch>,
    sender: &tokio::sync::mpsc::Sender<Result<Response>>,
) -> Result<()> {
    while let Ok(Batch { paths, features }) = receiver.recv().await {
        let batch = magika.identify_many_async(&features).await?;
        assert_eq!(batch.len(), paths.len());
        for ((order, path), output) in paths.into_iter().zip(batch.into_iter()) {
            let output = CliOutput::Output(output);
            sender.send(Ok(Response { order, path, is_dl: true, output })).await?;
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
    features: Vec<magika::Features>,
}

#[derive(Debug, Clone)]
struct Response {
    order: usize,
    path: PathBuf,
    is_dl: bool,
    output: CliOutput,
}

#[derive(Debug, Clone)]
enum CliOutput {
    Empty,
    Symlink,
    Directory,
    Error(String),
    Label(magika::Label),
    Output(magika::Output),
}

#[derive(Serialize)]
struct Json {
    path: PathBuf,
    dl: JsonResult,
    output: JsonResult,
}

#[derive(Default, Clone, Serialize)]
struct JsonResult {
    ct_label: Option<String>,
    score: Option<f32>,
    group: Option<String>,
    mime_type: Option<String>,
    magic: Option<String>,
    description: Option<String>,
}

#[derive(Serialize)]
struct JsonReport {
    version: Option<String>,
    reports: Vec<JsonReportFile>,
}

#[derive(Serialize)]
struct JsonReportFile {
    hash: Option<String>,
    features: String,
    result: Json,
}

#[derive(Serialize)]
struct JsonFeatures {
    beg: Vec<i32>,
    mid: Vec<i32>,
    end: Vec<i32>,
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
                    () if flags.modifiers.mime_type => "%M",
                    () if flags.modifiers.label => "%c",
                    () => "%d (%g)",
                });
                format.push_str(if flags.modifiers.output_score { " %S" } else { "" });
                format
            }
        };
        let mut format = format.chars();
        loop {
            match format.next() {
                Some('%') => match format.next() {
                    Some('p') => write!(&mut result, "{}", self.path.display())?,
                    Some('c') => write!(&mut result, "{}", self.code())?,
                    Some('d') => write!(&mut result, "{}", self.desc()?)?,
                    Some('g') => write!(&mut result, "{}", self.group())?,
                    Some('m') => write!(&mut result, "{}", self.magic())?,
                    Some('M') => write!(&mut result, "{}", self.mime())?,
                    Some('e') => write!(&mut result, "{}", join(self.extension()))?,
                    Some('s') => write!(&mut result, "{:.2}", self.score())?,
                    Some('S') => write!(&mut result, "{}%", (100. * self.score()).trunc())?,
                    Some(c) => result.push(c),
                    None => break,
                },
                Some(c) => result.push(c),
                None => break,
            }
        }
        Ok(self.color(result.into()))
    }

    fn json(self) -> Result<Json> {
        let output = JsonResult {
            ct_label: Some(self.code().to_string()),
            score: Some((self.score() * 1000.).trunc() / 1000.),
            group: Some(self.group().to_string()),
            mime_type: Some(self.mime().to_string()),
            magic: Some(self.magic().to_string()),
            description: Some(self.desc()?.to_string()),
        };
        let dl = if self.is_dl { output.clone() } else { JsonResult::default() };
        Ok(Json { path: self.path.to_path_buf(), dl, output })
    }

    fn code(&self) -> &str {
        match &self.output {
            CliOutput::Empty => "empty",
            CliOutput::Symlink => "symlink",
            CliOutput::Directory => "directory",
            CliOutput::Error(_) => "error",
            CliOutput::Label(x) => x.code(),
            CliOutput::Output(x) => x.label().code(),
        }
    }

    fn desc(&self) -> Result<Cow<str>> {
        Ok(match &self.output {
            CliOutput::Empty => "Empty file".into(),
            CliOutput::Symlink => {
                format!("Symbolic link to {}", std::fs::read_link(&self.path)?.display()).into()
            }
            CliOutput::Directory => "A directory".into(),
            CliOutput::Error(e) => e.into(),
            CliOutput::Label(x) => x.desc().into(),
            CliOutput::Output(x) => x.label().desc().into(),
        })
    }

    fn group(&self) -> &str {
        match &self.output {
            CliOutput::Empty => "inode",
            CliOutput::Symlink => "inode",
            CliOutput::Directory => "inode",
            CliOutput::Error(_) => "error",
            CliOutput::Label(x) => x.group(),
            CliOutput::Output(x) => x.label().group(),
        }
    }

    fn magic(&self) -> Cow<str> {
        match &self.output {
            CliOutput::Empty => self.code().into(),
            CliOutput::Symlink => format!("symlink link to {}", self.path.display()).into(),
            CliOutput::Directory => self.code().into(),
            CliOutput::Error(_) => self.code().into(),
            CliOutput::Label(x) => x.magic().into(),
            CliOutput::Output(x) => x.label().magic().into(),
        }
    }

    fn mime(&self) -> &str {
        match &self.output {
            CliOutput::Empty => "inode/x-empty",
            CliOutput::Symlink => "inode/symlink",
            CliOutput::Directory => "inode/directory",
            CliOutput::Error(_) => "error",
            CliOutput::Label(x) => x.mime(),
            CliOutput::Output(x) => x.label().mime(),
        }
    }

    fn extension(&self) -> &[&str] {
        match &self.output {
            CliOutput::Empty => &[],
            CliOutput::Symlink => &[],
            CliOutput::Directory => &[],
            CliOutput::Error(_) => &[],
            CliOutput::Label(x) => x.extension(),
            CliOutput::Output(x) => x.label().extension(),
        }
    }

    fn score(&self) -> f32 {
        match &self.output {
            CliOutput::Empty => 1.0,
            CliOutput::Symlink => 1.0,
            CliOutput::Directory => 1.0,
            CliOutput::Error(_) => 1.0,
            CliOutput::Label(_) => 1.0,
            CliOutput::Output(x) => x.score(),
        }
    }

    fn color(&self, result: ColoredString) -> ColoredString {
        let group = match &self.output {
            CliOutput::Error(_) => return result.bold().red(),
            CliOutput::Label(x) => x.group(),
            CliOutput::Output(x) => x.label().group(),
            _ => return result.bold(),
        };
        group_color(group, result)
    }
}

fn group_color(group: &str, result: ColoredString) -> ColoredString {
    match group {
        "document" => result.bold().magenta(),
        "executable" => result.bold().green(),
        "archive" => result.bold().red(),
        "audio" => result.yellow(),
        "image" => result.yellow(),
        "video" => result.yellow(),
        "code" => result.bold().blue(),
        _ => result.bold(),
    }
}

fn join<T: AsRef<str>>(xs: impl IntoIterator<Item = T>) -> String {
    let mut result = String::new();
    result.push('[');
    for (i, x) in xs.into_iter().enumerate() {
        if i != 0 {
            result.push('|');
        }
        result.push_str(x.as_ref());
    }
    result.push(']');
    result
}

async fn sha256_hex(path: &Path) -> Result<String> {
    let mut hash = sha2::Sha256::new();
    let mut buffer = [0; 4096];
    let mut file = File::open(path).await?;
    loop {
        let len = file.read(&mut buffer).await?;
        if len == 0 {
            break;
        }
        hash.update(&buffer[..len]);
    }
    Ok(HEXLOWER.encode(&hash.finalize()))
}

impl JsonFeatures {
    async fn new(path: &Path) -> Result<Self> {
        let features = magika::Features::extract_features_async(File::open(path).await?).await?;
        let n = features.len();
        assert_eq!(n % 3, 0);
        let beg = features[..n / 3].iter().map(|&x| x as i32).collect();
        let mid = features[n / 3..2 * n / 3].iter().map(|&x| x as i32).collect();
        let end = features[2 * n / 3..].iter().map(|&x| x as i32).collect();
        Ok(JsonFeatures { beg, mid, end })
    }
}
