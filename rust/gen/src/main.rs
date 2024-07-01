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

use std::collections::BTreeMap;
use std::fs::File;
use std::io::Write;

use anyhow::{anyhow, Context, Result};
use serde::Deserialize;

fn main() -> Result<()> {
    let configs: BTreeMap<String, Config> =
        serde_json::from_reader(File::open("data/config.json")?)?;
    let mut labels = BTreeMap::<String, Option<Label>>::new();
    for (key, config) in configs {
        if !config.in_scope_for_output_content_type {
            continue;
        }
        if ["directory", "empty", "symlink"].contains(&key.as_str()) {
            continue;
        }
        let target = config.target_label.unwrap_or_else(|| key.clone());
        let is_target = target == key;
        let entry = labels.entry(target).or_default();
        if !is_target {
            continue;
        }
        assert!(entry.is_none());
        let exts = config.extensions;
        let mime = unwrap(config.mime_type, "mime_type", &key)?;
        let group = unwrap(config.group, "group", &key)?;
        let magic = unwrap(config.magic, "magic", &key)?;
        let desc = unwrap(config.description, "description", &key)?;
        let text = config.tags.iter().any(|x| *x == "text");
        *entry = Some(Label { desc, magic, group, mime, exts, text });
    }
    let labels = labels
        .into_iter()
        .map(|(target, label)| {
            let label = label.with_context(|| format!("missing label for {target}"))?;
            Ok((target, label))
        })
        .collect::<Result<BTreeMap<_, _>>>()?;
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
    for (target, Label { desc, .. }) in &labels {
        writeln!(output, "    /// {desc}")?;
        writeln!(output, "    {},", capitalize(target))?;
    }
    writeln!(output, "}}\n")?;
    writeln!(output, "pub(crate) const MAX_LABEL: u32 = {};\n", labels.len() - 1)?;
    writeln!(output, "pub(crate) const METADATA: [Metadata; {}] = [", labels.len())?;
    for (target, label) in &labels {
        let Label { desc, magic, group, mime, exts, text } = label;
        writeln!(output, "    Metadata {{")?;
        writeln!(output, "        code: {target:?},")?;
        writeln!(output, "        desc: {desc:?},")?;
        writeln!(output, "        magic: {magic:?},")?;
        writeln!(output, "        group: {group:?},")?;
        writeln!(output, "        mime: {mime:?},")?;
        writeln!(output, "        extension: &{exts:?},")?;
        writeln!(output, "        is_text: {text:?},")?;
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
    desc: String,
    magic: String,
    group: String,
    mime: String,
    exts: Vec<String>,
    text: bool,
}

fn unwrap<T>(x: Option<T>, f: &str, n: &str) -> Result<T> {
    x.ok_or_else(|| anyhow!("missing {f} for {n:?}"))
}

fn capitalize(xs: &str) -> String {
    let mut xs = xs.as_bytes().to_vec();
    xs[0] = xs[0].to_ascii_uppercase();
    String::from_utf8(xs).unwrap()
}
