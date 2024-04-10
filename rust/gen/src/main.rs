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

use std::collections::{BTreeMap, BTreeSet};
use std::fs::File;
use std::io::Write;

use anyhow::{anyhow, Context, Result};
use serde::Deserialize;

fn main() -> Result<()> {
    let configs: BTreeMap<String, Config> =
        serde_json::from_reader(File::open("gen/data/config.json")?)?;
    let mut labels = BTreeMap::<String, Vec<Label>>::new();
    for (short, config) in configs {
        if !config.in_scope_for_output_content_type {
            continue;
        }
        if ["directory", "empty", "symlink"].contains(&short.as_str()) {
            continue;
        }
        let target = match config.target_label {
            Some(x) => x,
            None => continue,
        };
        let mime = unwrap(config.mime_type, "mime_type", &short)?;
        let group = unwrap(config.group, "group", &short)?;
        let magic = unwrap(config.magic, "magic", &short)?;
        let long = unwrap(config.description, "description", &short)?;
        let text = config.tags.iter().any(|x| *x == "text");
        labels.entry(target).or_default().push(Label { short, long, magic, group, mime, text });
    }
    let labels = labels;
    let mut output = File::create("lib/src/label.rs")?;
    let header = std::fs::read_to_string(file!())?;
    let header = header.split("\n\n").next().context("main.rs does not contain an empty line")?;
    writeln!(output, "{header}\n")?;
    writeln!(output, "use crate::output::Metadata;\n")?;
    writeln!(output, "/// Content type of a file.")?;
    writeln!(output, "#[derive(Debug, Copy, Clone, PartialEq, Eq)]\n#[repr(u32)]")?;
    writeln!(output, "pub enum Label {{")?;
    for (target, labels) in &labels {
        let doc = merge(labels, |x| &x.long);
        writeln!(output, "    /// {doc}")?;
        writeln!(output, "    {},", capitalize(target))?;
    }
    writeln!(output, "}}\n")?;
    writeln!(output, "pub(crate) const MAX_LABEL: u32 = {};\n", labels.len() - 1)?;
    writeln!(output, "pub(crate) const METADATA: [Metadata; {}] = [", labels.len())?;
    for (target, labels) in &labels {
        writeln!(output, "    Metadata {{")?;
        writeln!(output, "        code: {target:?},")?;
        writeln!(output, "        short: {:?},", merge(labels, |x| &x.short))?;
        writeln!(output, "        long: {:?},", merge(labels, |x| &x.long))?;
        writeln!(output, "        magic: {:?},", merge(labels, |x| &x.magic))?;
        writeln!(output, "        group: {:?},", merge(labels, |x| &x.group))?;
        writeln!(output, "        mime: {:?},", merge(labels, |x| &x.mime))?;
        writeln!(output, "        text: {:?},", labels.iter().all(|x| x.text))?;
        writeln!(output, "    }},")?;
    }
    writeln!(output, "];")?;
    Ok(())
}

#[derive(Deserialize)]
struct Config {
    mime_type: Option<String>,
    group: Option<String>,
    magic: Option<String>,
    description: Option<String>,
    tags: Vec<String>,
    target_label: Option<String>,
    in_scope_for_output_content_type: bool,
}

#[derive(Debug)]
struct Label {
    short: String,
    long: String,
    magic: String,
    group: String,
    mime: String,
    text: bool,
}

fn unwrap(x: Option<String>, f: &str, n: &str) -> Result<String> {
    x.ok_or_else(|| anyhow!("missing {f} for {n:?}"))
}

fn merge(labels: &[Label], field: impl Fn(&Label) -> &str) -> String {
    let labels: BTreeSet<_> = labels.iter().map(field).collect();
    let labels: Vec<_> = labels.into_iter().collect();
    let mut result = labels[0].to_string();
    for label in &labels[1..] {
        result.push_str(" | ");
        result.push_str(label);
    }
    result
}

fn capitalize(xs: &str) -> String {
    let mut xs = xs.as_bytes().to_vec();
    xs[0] = xs[0].to_ascii_uppercase();
    String::from_utf8(xs).unwrap()
}
