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

use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{ensure, Result};
use clap::{Parser, ValueEnum};
use futures::stream::FuturesUnordered;
use futures::StreamExt as _;
use magika::{MagikaConfig, MagikaError, MagikaFeatures, MagikaOutput, MagikaSession};
use onnxruntime::{GraphOptimizationLevel, LoggingLevel};
use tokio::fs::File;

#[tokio::main]
async fn main() -> Result<()> {
    // TODO(release): Maybe print some warning or disclaimer about the tool readiness.
    let flags = Arc::new(Flags::parse());
    ensure!(
        !flags.path.is_empty(),
        "At least one path must be provided."
    );
    let (result_sender, mut result_receiver) =
        tokio::sync::mpsc::channel::<Result<BatchResponse>>(flags.num_sessions);
    let (batch_sender, batch_receiver) = async_channel::bounded::<BatchRequest>(flags.num_sessions);
    let config = Arc::new(MagikaConfig::new(&flags.model_dir)?);
    tokio::spawn({
        let flags = flags.clone();
        let config = config.clone();
        let result_sender = result_sender.clone();
        async move {
            if let Err(e) = extract_features(&flags, &config, &batch_sender).await {
                result_sender.send(Err(e)).await.unwrap();
            }
        }
    });
    for _ in 0..flags.num_sessions {
        std::thread::spawn({
            let flags = flags.clone();
            let config = config.clone();
            let batch_receiver = batch_receiver.clone();
            let result_sender = result_sender.clone();
            move || {
                if let Err(e) = infer_batch(&flags, &config, &batch_receiver, &result_sender) {
                    result_sender.blocking_send(Err(e)).unwrap();
                }
            }
        });
    }
    // Update results.
    let mut results = vec![None; flags.path.len()];
    drop(result_sender);
    while let Some(batch) = result_receiver.recv().await {
        let batch = batch?;
        assert_eq!(batch.batch.len(), batch.mapping.len());
        for (result, index) in batch.batch.into_iter().zip(batch.mapping.into_iter()) {
            results[index] = Some(result);
        }
    }
    // Print results.
    for (path, result) in flags.path.iter().zip(results.into_iter()) {
        let result = result.unwrap();
        let path = path.display();
        let label = result.label();
        let score = result.score();
        println!("{path} is {label} with score {score}");
    }
    Ok(())
}

async fn extract_features(
    flags: &Flags,
    config: &MagikaConfig,
    sender: &async_channel::Sender<BatchRequest>,
) -> Result<()> {
    // Extract features concurrently.
    let mut features = FuturesUnordered::new();
    for (index, path) in flags.path.iter().enumerate() {
        features.push(async move {
            let file = File::open(path).await?;
            let features = config.extract_features_async(file).await?;
            Ok::<_, MagikaError>((index, features))
        });
    }
    // Send features by batch.
    loop {
        let mut batch = Vec::new();
        let mut mapping = Vec::new();
        while let Some(features) = features.next().await {
            let (index, features) = features?;
            batch.push(features);
            mapping.push(index);
            if batch.len() == flags.batch_size {
                break;
            }
        }
        let batch_size = mapping.len();
        sender.send(BatchRequest { batch, mapping }).await?;
        if flags.batch_size == 0 || batch_size < flags.batch_size {
            break Ok(());
        }
    }
}

fn infer_batch(
    flags: &Flags,
    config: &MagikaConfig,
    receiver: &async_channel::Receiver<BatchRequest>,
    sender: &tokio::sync::mpsc::Sender<Result<BatchResponse>>,
) -> Result<()> {
    let magika = MagikaSession::builder(config)
        .with_name("magika")
        .with_logging_level(flags.logging_level.into())
        .with_number_threads(flags.threads_per_session)
        .with_optimization_level(flags.optimization_level.into())
        .build(&flags.model_dir)?;
    // Infer by batch.
    while let Ok(BatchRequest { batch, mapping }) = receiver.recv_blocking() {
        let batch = magika.identify_batch(&batch)?;
        sender.blocking_send(Ok(BatchResponse { batch, mapping }))?;
    }
    Ok(())
}

/// Determines the content type of files with deep-learning.
#[derive(Parser)]
#[command(version)]
pub struct Flags {
    /// Directory containing the `model.onnx` file and configuration files.
    pub model_dir: PathBuf,

    /// List of paths to the files to analyze.
    pub path: Vec<PathBuf>,

    /// Number of files to identify in a single inference.
    #[arg(long, default_value = "1")]
    pub batch_size: usize,

    /// Number of threads per inference session (ONNX Runtime configuration).
    #[arg(long, default_value = "1")]
    pub threads_per_session: i16,

    /// Number of inference sessions to process batches (each session has a dedicated thread).
    #[arg(long, default_value = "1")]
    pub num_sessions: usize,

    /// Onnx environment logging level.
    #[arg(long, value_enum, default_value_t)]
    pub logging_level: OnnxLoggingLevel,

    /// Onnx session optimization level.
    #[arg(long, value_enum, default_value_t)]
    pub optimization_level: OnnxOptimizationLevel,
}

#[derive(Default, Copy, Clone, ValueEnum)]
pub enum OnnxLoggingLevel {
    Verbose,
    #[default]
    Info,
    Warning,
    Error,
    Fatal,
}

#[derive(Default, Copy, Clone, ValueEnum)]
pub enum OnnxOptimizationLevel {
    DisableAll,
    #[default]
    Basic,
    Extended,
    All,
}

impl From<OnnxLoggingLevel> for LoggingLevel {
    fn from(value: OnnxLoggingLevel) -> Self {
        match value {
            OnnxLoggingLevel::Verbose => LoggingLevel::Verbose,
            OnnxLoggingLevel::Info => LoggingLevel::Info,
            OnnxLoggingLevel::Warning => LoggingLevel::Warning,
            OnnxLoggingLevel::Error => LoggingLevel::Error,
            OnnxLoggingLevel::Fatal => LoggingLevel::Fatal,
        }
    }
}

impl From<OnnxOptimizationLevel> for GraphOptimizationLevel {
    fn from(value: OnnxOptimizationLevel) -> Self {
        match value {
            OnnxOptimizationLevel::DisableAll => GraphOptimizationLevel::DisableAll,
            OnnxOptimizationLevel::Basic => GraphOptimizationLevel::Basic,
            OnnxOptimizationLevel::Extended => GraphOptimizationLevel::Extended,
            OnnxOptimizationLevel::All => GraphOptimizationLevel::All,
        }
    }
}

struct BatchRequest {
    batch: Vec<MagikaFeatures>,
    mapping: Vec<usize>,
}

struct BatchResponse {
    batch: Vec<MagikaOutput>,
    mapping: Vec<usize>,
}
