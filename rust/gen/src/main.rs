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
        serde_json::from_reader(File::open("data/config.json")?)?;
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
        let exts = config.extensions;
        let mime = unwrap(config.mime_type, "mime_type", &short)?;
        let group = unwrap(config.group, "group", &short)?;
        let magic = unwrap(config.magic, "magic", &short)?;
        let long = unwrap(config.description, "description", &short)?;
        let text = config.tags.iter().any(|x| *x == "text");
        let label = Label { short, long, magic, group, mime, exts, text };
        labels.entry(target).or_default().push(label);
    }
    let labels = labels;
    let mut output = File::create("../lib/src/label.rs")?;
    let header = std::fs::read_to_string(file!())?;
    let header = header.split("\n\n").next().context("main.rs does not contain an empty line")?;
    writeln!(output, "{header}\n")?;
    writeln!(output, "use crate::output::Metadata;\n")?;
    writeln!(output, "// DO NOT EDIT, see link below for more information:")?;
    writeln!(output, "// https://github.com/google/magika/tree/main/rust/gen\n")?;
    writeln!(output, "/// Content type of a file.")?;
    writeln!(output, "#[derive(Debug, Copy, Clone, PartialEq, Eq)]\n#[repr(u32)]")?;
    writeln!(output, "pub enum Label {{")?;
    for (target, labels) in &labels {
        let doc = merge_ref(labels, |x| &x.long).join(" | ");
        writeln!(output, "    /// {doc}")?;
        writeln!(output, "    {},", capitalize(target))?;
    }
    writeln!(output, "}}\n")?;
    writeln!(output, "pub(crate) const MAX_LABEL: u32 = {};\n", labels.len() - 1)?;
    writeln!(output, "pub(crate) const METADATA: [Metadata; {}] = [", labels.len())?;
    for (target, labels) in &labels {
        writeln!(output, "    Metadata {{")?;
        writeln!(output, "        code: {target:?},")?;
        writeln!(output, "        short_desc: &{:?},", merge_ref(labels, |x| &x.short))?;
        writeln!(output, "        long_desc: &{:?},", merge_ref(labels, |x| &x.long))?;
        writeln!(output, "        magic: &{:?},", merge_ref(labels, |x| &x.magic))?;
        writeln!(output, "        group: &{:?},", merge_ref(labels, |x| &x.group))?;
        writeln!(output, "        mime: &{:?},", merge_ref(labels, |x| &x.mime))?;
        writeln!(output, "        extension: &{:?},", merge(labels, |x| &x.exts))?;
        writeln!(output, "        is_text: {:?},", labels.iter().all(|x| x.text))?;
        writeln!(output, "    }},")?;
    }
    writeln!(output, "];")?;
    Ok(())
}

#[derive(Deserialize)]
struct Config {
    extensions: Vec<String>,
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
    exts: Vec<String>,
    text: bool,
}

fn unwrap<T>(x: Option<T>, f: &str, n: &str) -> Result<T> {
    x.ok_or_else(|| anyhow!("missing {f} for {n:?}"))
}

fn merge_ref<T: Ord + AsRef<str>>(labels: &[Label], field: impl Fn(&Label) -> &T) -> Vec<String> {
    merge(labels, |x| std::slice::from_ref(field(x)))
}

fn merge<T: Ord + AsRef<str>>(labels: &[Label], field: impl Fn(&Label) -> &[T]) -> Vec<String> {
    let labels: BTreeSet<&T> = labels.iter().flat_map(field).collect();
    labels.into_iter().map(|x| x.as_ref().to_string()).collect()
}

fn capitalize(xs: &str) -> String {
    let mut xs = xs.as_bytes().to_vec();
    xs[0] = xs[0].to_ascii_uppercase();
    String::from_utf8(xs).unwrap()
}
