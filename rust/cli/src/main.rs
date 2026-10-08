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
use std::fmt::Write;
use std::io::{ErrorKind, Write as _};
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{ensure, Result};
use clap::{Args, Parser, ValueEnum};
use colored::ColoredString;
use magika::pipeline::{DirectoryCycle, Engine, Identified, Pipeline, PipelineOptions};
use magika::{
    self, Backend, ContentType, FileType, InferredType, OverwriteReason, Rules, Runtime, TypeInfo,
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

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, ValueEnum)]
enum RulesMode {
    /// Identifies files with the model only, after any custom rules.
    Off,
    /// Identifies files with rules first, and with the model when no rule decides.
    #[default]
    Enforce,
    /// Identifies files with rules only, as unknown when no rule decides. The model is not loaded.
    Only,
}

#[derive(Args)]
struct Experimental {
    /// Identifies files with format rules.
    ///
    /// A rule decides from the first 4 KiB and the size of a file, and only when every matching
    /// rule agrees.
    #[arg(hide = true, long, value_enum, default_value_t)]
    rules: RulesMode,

    /// Identifies files with the rules of this file before the built-in rules (can be repeated).
    ///
    /// Rules use the YARA subset and metadata of the built-in rules, and apply whatever --rules is.
    #[arg(hide = true, long, value_name = "PATH")]
    rules_file: Vec<PathBuf>,

    /// Checks the rules of --rules-file, prints the content types they identify, and exits.
    #[arg(hide = true, long, requires = "rules_file")]
    rules_check: bool,

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

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, ValueEnum)]
enum BackendChoice {
    #[default]
    Auto,
    Cpu,
    Gpu,
}

fn main() -> Result<()> {
    let flags = Flags::parse();
    let batch_size = flags.experimental.batch_size;
    ensure!((1..=64).contains(&batch_size), "--batch-size must be between 1 and 64");
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
    builder = match flags.experimental.rules {
        RulesMode::Off => builder.with_rules(false),
        RulesMode::Enforce => builder,
        RulesMode::Only => builder.with_model(false),
    };
    if !flags.experimental.rules_file.is_empty() {
        let rules = Rules::from_files(&flags.experimental.rules_file)?;
        if flags.experimental.rules_check {
            for content_type in rules.content_types() {
                println!("{}", content_type.info().label);
            }
            return Ok(());
        }
        builder = builder.with_custom_rules(rules);
    }
    builder = builder.with_follow_symlink(!flags.no_dereference);
    builder = match flags.experimental.backend {
        BackendChoice::Auto => builder,
        BackendChoice::Cpu => builder.with_backend(Backend::Cpu),
        BackendChoice::Gpu => builder.with_backend(Backend::Gpu),
    };
    if flags.experimental.backend_info {
        let info = builder.build()?.backend_info();
        let backend = match info.backend() {
            Backend::Cpu => "cpu",
            Backend::Gpu => "gpu",
        };
        println!("{backend} ({})", info.implementation());
        return Ok(());
    }
    let options = builder.options().clone();
    let threads = flags.experimental.threads;
    ensure!(threads.is_none_or(|x| (1..=256).contains(&x)), "--threads must be between 1 and 256");
    let readers = flags.experimental.readers;
    ensure!((1..=256).contains(&readers), "--readers must be between 1 and 256");
    let mut config = PipelineOptions::default();
    config.recursive = flags.recursive;
    config.stdin = true;
    config.batch_size = batch_size;
    config.threads = threads;
    config.readers = readers;
    let pipeline = Pipeline::new(Arc::new(Engine::new(builder)?), options, config)?;
    let mut errors = false;
    let result = match print(&flags, &mut errors, pipeline.identify_paths(flags.path.clone())?) {
        Err(e)
            if e.root_cause()
                .downcast_ref::<std::io::Error>()
                .is_some_and(|x| x.kind() == std::io::ErrorKind::BrokenPipe) =>
        {
            Ok(())
        }
        x => x,
    };
    result?;
    if errors {
        std::process::exit(1);
    }
    Ok(())
}

fn print(flags: &Flags, errors: &mut bool, identified: Identified) -> Result<()> {
    let mut stdout = std::io::stdout().lock();
    if flags.format.json {
        write!(stdout, "[")?;
    }
    let mut count = 0;
    for item in identified {
        let (path, result) = item?;
        let response = Response { path, result };
        *errors |= response.result.is_err();
        if flags.format.json {
            if count != 0 {
                write!(stdout, ",")?;
            }
            for line in serde_json::to_string_pretty(&response.json()?)?.lines() {
                write!(stdout, "\n  {line}")?;
            }
        } else {
            writeln!(stdout, "{}", response.format(flags)?)?;
        }
        count += 1;
    }
    if flags.format.json {
        if count != 0 {
            writeln!(stdout)?;
        }
        writeln!(stdout, "]")?;
    }
    Ok(())
}

/// A path and its identification.
#[derive(Debug)]
struct Response {
    path: PathBuf,
    result: Result<FileType>,
}

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
enum JsonError {
    Unknown,
    FileDoesNotExist,
    PermissionError,
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
        if let Some(x) = value.root_cause().downcast_ref::<std::io::Error>() {
            match x.kind() {
                ErrorKind::NotFound => return JsonError::FileDoesNotExist,
                ErrorKind::PermissionDenied => return JsonError::PermissionError,
                _ => (),
            }
        }
        match value.downcast_ref::<DirectoryCycle>() {
            Some(DirectoryCycle) => JsonError::DirectoryCycle,
            None => JsonError::Unknown,
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
        let path = self.path.to_string_lossy();
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
