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

use std::fmt::Write;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{bail, ensure, Result};
use clap::Parser;
use colored::{Color, Colorize};
use ort::GraphOptimizationLevel;
use tokio::fs::File;

/// Determines the content type of files with deep-learning.
#[derive(Parser)]
#[command(version, arg_required_else_help = true)]
struct Flags {
    /// List of paths to the files to analyze.
    path: Vec<PathBuf>,

    /// Forces usage of colors.
    ///
    /// Colors are automatically enabled if the terminal supports them.
    #[arg(long)]
    colors: bool,

    /// Disables usage of colors.
    ///
    /// Colors are automatically disabled if the terminal doesn't support them.
    #[arg(long, conflicts_with = "colors")]
    no_colors: bool,

    /// Format string (use --help for details).
    ///
    /// The following placeholders are supported:
    ///
    ///   %c  A short code describing the content type
    ///   %d  A short description of the content type
    ///   %D  A long desccription of the content type
    ///   %g  The group of the content type
    ///   %m  The magic of the content type
    ///   %M  The MIME type of the content type
    ///   %e  The file extensions of the content type
    ///   %s  The score of the content type for the file
    ///   %%  A literal %
    #[arg(long, default_value = "%D (%g)", verbatim_doc_comment)]
    format: String,

    /// Number of files to identify in a single inference.
    #[arg(long, default_value = "1")]
    batch_size: usize,

    /// Number of tasks for batch parallelism.
    #[arg(long, default_value = "1")]
    num_tasks: usize,

    /// Number of threads for graph parallelism (ONNX Runtime configuration).
    ///
    /// This has no effect if --parallel-execution is false or unset.
    #[arg(long)]
    inter_threads: Option<usize>,

    /// Number of threads for node parallelism (ONNX Runtime configuration).
    #[arg(long)]
    intra_threads: Option<usize>,

    /// Graph optimization level, from 0 to 3 (ONNX Runtime configuration).
    #[arg(long)]
    optimization_level: Option<usize>,

    /// Whether to enable parallel execution (ONNX Runtime configuration).
    #[arg(long)]
    parallel_execution: Option<bool>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let flags = Arc::new(Flags::parse());
    ensure!(0 < flags.batch_size, "--batch-size cannot be zero");
    ensure!(0 < flags.num_tasks, "--num-tasks cannot be zero");
    if flags.colors {
        colored::control::set_override(true);
    }
    if flags.no_colors {
        colored::control::set_override(false);
    }
    let (result_sender, mut result_receiver) =
        tokio::sync::mpsc::channel::<Result<Response>>(flags.num_tasks * flags.batch_size);
    let (batch_sender, batch_receiver) = async_channel::bounded::<Batch>(flags.num_tasks);
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
    for _ in 0..flags.num_tasks {
        tokio::spawn({
            let flags = flags.clone();
            let magika = magika.clone();
            let batch_receiver = batch_receiver.clone();
            let result_sender = result_sender.clone();
            async move {
                if let Err(e) = infer_batch(&flags, &magika, &batch_receiver, &result_sender).await
                {
                    result_sender.send(Err(e)).await.unwrap();
                }
            }
        });
    }
    let mut results = vec![None; flags.path.len()];
    drop(result_sender);
    while let Some(response) = result_receiver.recv().await {
        let Response { index, result, color } = response?;
        results[index] = Some((result, color));
    }
    for (path, result) in flags.path.iter().zip(results.into_iter()) {
        let path = path.display();
        let (result, color) = result.unwrap();
        let mut output = format!("{path}: {result}").bold();
        if let Some(color) = color {
            output = output.color(color);
        }
        println!("{output}");
    }
    Ok(())
}

async fn extract_features(
    flags: &Flags, batch_sender: &async_channel::Sender<Batch>,
    result_sender: &tokio::sync::mpsc::Sender<Result<Response>>,
) -> Result<()> {
    let mut indices = Vec::new();
    let mut features = Vec::new();
    for (index, path) in flags.path.iter().enumerate() {
        let mut result = None;
        match tokio::fs::symlink_metadata(path).await {
            Ok(metadata) => {
                if metadata.is_dir() {
                    result = Some("directory".to_string());
                } else if metadata.is_symlink() {
                    result = Some("symbolic link".to_string());
                } else if !metadata.is_file() {
                    result = Some("unknown".to_string());
                } else if metadata.len() == 0 {
                    result = Some("empty".to_string());
                }
            }
            Err(error) => result = Some(format!("{error}")),
        }
        if let Some(result) = result {
            result_sender.send(Ok(Response { index, result, color: None })).await?;
            continue;
        }
        let file = File::open(path).await?;
        indices.push(index);
        features.push(magika::Features::extract_async(file).await?);
        if features.len() == flags.batch_size {
            batch_sender.send(Batch { indices, features }).await?;
            indices = Vec::new();
            features = Vec::new();
        }
    }
    if !indices.is_empty() {
        batch_sender.send(Batch { indices, features }).await?;
    }
    Ok(())
}

fn build_session(flags: &Flags) -> Result<magika::Session> {
    let mut magika = magika::Session::builder();
    if let Some(inter_threads) = flags.inter_threads {
        magika = magika.with_inter_threads(inter_threads);
    }
    if let Some(intra_threads) = flags.intra_threads {
        magika = magika.with_intra_threads(intra_threads);
    }
    if let Some(opt_level) = flags.optimization_level {
        let opt_level = match opt_level {
            0 => GraphOptimizationLevel::Disable,
            1 => GraphOptimizationLevel::Level1,
            2 => GraphOptimizationLevel::Level2,
            3 => GraphOptimizationLevel::Level3,
            _ => bail!("--optimization-level must be 0, 1, 2, or 3"),
        };
        magika = magika.with_optimization_level(opt_level);
    }
    if let Some(parallel_execution) = flags.parallel_execution {
        magika = magika.with_parallel_execution(parallel_execution);
    }
    Ok(magika.build()?)
}

async fn infer_batch(
    flags: &Flags, magika: &magika::Session, receiver: &async_channel::Receiver<Batch>,
    sender: &tokio::sync::mpsc::Sender<Result<Response>>,
) -> Result<()> {
    while let Ok(Batch { indices, features }) = receiver.recv().await {
        let batch = magika.identify_many_async(&features).await?;
        assert_eq!(batch.len(), indices.len());
        for (&index, output) in indices.iter().zip(batch.into_iter()) {
            let (result, color) = format(flags, output);
            sender.send(Ok(Response { index, result, color })).await?;
        }
    }
    Ok(())
}

fn format(flags: &Flags, output: magika::Output) -> (String, Option<Color>) {
    let mut result = String::new();
    let mut format = flags.format.chars();
    let label = output.label();
    loop {
        match format.next() {
            Some('%') => match format.next() {
                Some('c') => write!(&mut result, "{}", label.code()).unwrap(),
                Some('d') => write!(&mut result, "{}", label.short_desc().join(" | ")).unwrap(),
                Some('D') => write!(&mut result, "{}", label.long_desc().join(" | ")).unwrap(),
                Some('g') => write!(&mut result, "{}", label.group().join(" | ")).unwrap(),
                Some('m') => write!(&mut result, "{}", label.magic().join(" | ")).unwrap(),
                Some('M') => write!(&mut result, "{}", label.mime().join(" | ")).unwrap(),
                Some('e') => write!(&mut result, "{}", label.extension().join(" | ")).unwrap(),
                Some('s') => write!(&mut result, "{:.2}", output.score()).unwrap(),
                Some(c) => result.push(c),
                None => break,
            },
            Some(c) => result.push(c),
            None => break,
        }
    }
    (result, label.group().iter().find_map(|x| group_color(x)))
}

fn group_color(group: &str) -> Option<Color> {
    Some(match group {
        "document" => Color::BrightMagenta,
        "executable" => Color::BrightGreen,
        "archive" => Color::BrightRed,
        "audio" => Color::Yellow,
        "image" => Color::Yellow,
        "video" => Color::Yellow,
        "code" => Color::BrightBlue,
        _ => return None,
    })
}

struct Batch {
    indices: Vec<usize>,
    features: Vec<magika::Features>,
}

struct Response {
    index: usize,
    result: String,
    color: Option<Color>,
}
