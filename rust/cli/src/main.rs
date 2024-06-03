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
use ort::GraphOptimizationLevel;
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
    #[arg(long, default_value = "%D (confidence: %s)", verbatim_doc_comment)]
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
    ensure!(
        flags.path.iter().filter(|x| x.to_str() == Some("-")).count() <= 1,
        "only one path can be the standard input"
    );
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
        let Response { index, result } = response?;
        results[index] = Some(result);
    }
    for (path, result) in flags.path.iter().zip(results.into_iter()) {
        let path = path.display();
        let result = result.unwrap();
        println!("{path}: {result}");
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
        let features_or_output: magika::FeaturesOrOutput = if path.to_str() == Some("-") {
            let mut stdin = Vec::new();
            tokio::io::stdin().read_to_end(&mut stdin).await?;
            magika::FeaturesOrOutput::extract_sync(&stdin[..])?
        } else {
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
                result_sender.send(Ok(Response { index, result })).await?;
                continue;
            }
            magika::FeaturesOrOutput::extract_async(File::open(path).await?).await?
        };
        match features_or_output {
            magika::FeaturesOrOutput::Output(output) => {
                let result = format(flags, output);
                result_sender.send(Ok(Response { index, result })).await?;
                continue;
            }
            magika::FeaturesOrOutput::Features(x) => features.push(x),
        }
        indices.push(index);
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
            let result = format(flags, output);
            sender.send(Ok(Response { index, result })).await?;
        }
    }
    Ok(())
}

fn format(flags: &Flags, output: magika::Output) -> String {
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
    result
}

struct Batch {
    indices: Vec<usize>,
    features: Vec<magika::Features>,
}

struct Response {
    index: usize,
    result: String,
}
