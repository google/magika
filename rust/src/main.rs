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

use std::fs::File;
use std::path::PathBuf;

use anyhow::{ensure, Result};
use clap::{Parser, ValueEnum};
use magika::MagikaSession;
use onnxruntime::{GraphOptimizationLevel, LoggingLevel};

fn main() -> Result<()> {
    // TODO(release): Maybe print some warning or disclaimer about the tool readiness.
    let flags = Flags::parse();
    ensure!(
        !flags.path.is_empty(),
        "At least one path must be provided."
    );
    let mut magika = MagikaSession::build()
        .with_name("magika")
        .with_logging_level(flags.logging_level.into())
        .with_number_threads(1)
        .with_optimization_level(flags.optimization_level.into())
        .build(flags.model_dir)?;
    let files = flags
        .path
        .iter()
        .map(|x| Ok(File::open(x)?))
        .collect::<Result<Vec<_>>>()?;
    // TODO: Use identify_many once supported.
    let mut results = Vec::new();
    for file in &files {
        results.push(magika.identify(file)?);
    }
    for (path, result) in flags.path.iter().zip(results.iter()) {
        let path = path.display();
        let label = result.label();
        let score = result.score();
        println!("{path} is {label} with score {score}");
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
