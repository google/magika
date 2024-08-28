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
use std::path::Path;

use anyhow::{Context, Result};
use serde::Deserialize;

fn main() -> Result<()> {
    let content_types: BTreeMap<String, ContentType> =
        serde_json::from_reader(File::open("../../assets/content_types_kb.min.json")?)?;
    let model_name = std::fs::read_link("model")?;
    let model_name = model_name
        .components()
        .last()
        .context("model link")?
        .as_os_str()
        .to_str()
        .context("model name")?;
    let model_config = serde_json::from_reader(File::open("model/config.min.json")?)?;
    let content_types = generate_content_types(content_types, model_name, &model_config)?;
    generate_model_config(&content_types, model_config)?;
    Ok(())
}

fn generate_content_types(
    mut content_types: BTreeMap<String, ContentType>, model_name: &str, model_config: &ModelConfig,
) -> Result<Vec<String>> {
    // We only want to generate content types that are already exposed or that are model labels.
    // This is a conservative approach to avoid exposing the whole knowledge base if it contains
    // experimental content types that won't ever be exposed in the future.
    let content_types_content = std::fs::read_to_string("content_types")?;
    let mut labels = content_types_content.lines().collect::<BTreeSet<_>>();
    labels.extend(model_config.target_labels_space.iter().map(|x| x.as_str()));
    let mut content_types_file = File::create("content_types")?;
    for label in &labels {
        writeln!(&mut content_types_file, "{label}")?;
    }
    content_types.retain(|x, _| labels.contains(x.as_str()));
    let mut output = create_generated_file("../lib/src/content.rs")?;
    writeln!(output, "use crate::file::TypeInfo;\n")?;
    writeln!(output, "/// Model name (only comparable with equality).")?;
    writeln!(output, "pub const MODEL_NAME: &str = {model_name:?};\n")?;
    struct Variant {
        label: String,
        doc: String,
    }
    let mut variants = Vec::new();
    for (label, info) in content_types {
        let ContentType { mime_type, group, description, extensions, is_text } = info.clone();
        let mime_type = mime_type.unwrap_or_else(|| {
            if is_text { "text/plain" } else { "application/octet-stream" }.to_string()
        });
        let group = group.unwrap_or_else(|| "unknown".to_string());
        let description = description.unwrap_or_else(|| label.clone());
        if !matches!(label.as_str(), "directory" | "symlink") {
            variants.push(Variant { label: label.clone(), doc: description.clone() });
        }
        writeln!(output, "pub(crate) static {}: TypeInfo = TypeInfo {{", const_name(&label))?;
        writeln!(output, "    label: {label:?},")?;
        writeln!(output, "    mime_type: {mime_type:?},")?;
        writeln!(output, "    group: {group:?},")?;
        writeln!(output, "    description: {description:?},")?;
        writeln!(output, "    extensions: &{extensions:?},")?;
        writeln!(output, "    is_text: {is_text:?},")?;
        writeln!(output, "}};\n")?;
    }
    writeln!(output, "/// Content types for regular files.")?;
    writeln!(output, "#[derive(Debug, Copy, Clone, PartialEq, Eq)]")?;
    writeln!(output, "#[non_exhaustive]")?;
    writeln!(output, "pub enum ContentType {{")?;
    for Variant { label, doc } in &variants {
        writeln!(output, "    /// {doc}")?;
        writeln!(output, "    {},", enum_name(label))?;
    }
    writeln!(output, "}}\n")?;
    writeln!(output, "impl ContentType {{")?;
    writeln!(output, "    pub(crate) const SIZE: usize = {};\n", variants.len())?;
    writeln!(output, "    /// Returns the content type information.")?;
    writeln!(output, "    pub fn info(self) -> &'static TypeInfo {{")?;
    writeln!(output, "        match self {{")?;
    for Variant { label, .. } in &variants {
        writeln!(
            output,
            "            ContentType::{} => &{},",
            enum_name(label),
            const_name(label),
        )?;
    }
    writeln!(output, "        }}")?;
    writeln!(output, "    }}")?;
    writeln!(output, "}}")?;
    Ok(variants.into_iter().map(|x| x.label).collect())
}

fn generate_model_config(content_types: &[String], model_config: ModelConfig) -> Result<()> {
    let ModelConfig {
        beg_size,
        mid_size,
        end_size,
        use_inputs_at_offsets,
        medium_confidence_threshold,
        min_file_size_for_dl,
        padding_token,
        block_size,
        target_labels_space,
        thresholds,
        overwrite_map,
    } = model_config;
    let mut output = create_generated_file("../lib/src/model.rs")?;
    writeln!(output, "use std::borrow::Cow;\n")?;
    writeln!(output, "use crate::config::ModelConfig;")?;
    writeln!(output, "use crate::ContentType;\n")?;
    writeln!(output, "pub(crate) const CONFIG: ModelConfig = ModelConfig {{")?;
    writeln!(output, "    beg_size: {beg_size},")?;
    writeln!(output, "    mid_size: {mid_size},")?;
    writeln!(output, "    end_size: {end_size},")?;
    writeln!(output, "    use_inputs_at_offsets: {use_inputs_at_offsets},")?;
    writeln!(output, "    min_file_size_for_dl: {min_file_size_for_dl},")?;
    writeln!(output, "    padding_token: {padding_token},")?;
    writeln!(output, "    block_size: {block_size},")?;
    writeln!(output, "    thresholds: Cow::Borrowed(&THRESHOLDS),")?;
    writeln!(output, "    overwrite_map: Cow::Borrowed(&OVERWRITE_MAP),")?;
    writeln!(output, "}};\n")?;
    let mut thresholds_array = vec![medium_confidence_threshold; content_types.len()];
    for (label, threshold) in thresholds {
        let pos = content_types.iter().position(|x| *x == label).unwrap();
        thresholds_array[pos] = threshold;
    }
    writeln!(output, "#[rustfmt::skip]")?;
    writeln!(output, "const THRESHOLDS: [f32; ContentType::SIZE] = {thresholds_array:?};")?;
    writeln!(output, "const OVERWRITE_MAP: [ContentType; ContentType::SIZE] = [")?;
    let mut overwrite_array = content_types.to_vec();
    for (src, dst) in overwrite_map {
        let pos = content_types.iter().position(|x| *x == src).unwrap();
        overwrite_array[pos] = dst;
    }
    for label in overwrite_array {
        writeln!(output, "    ContentType::{},", enum_name(&label))?;
    }
    writeln!(output, "];\n")?;
    writeln!(output, "#[derive(Debug, Copy, Clone, PartialEq, Eq)]\n#[repr(u32)]")?;
    writeln!(output, "#[allow(dead_code)] // only constructed through transmute")?;
    writeln!(output, "pub(crate) enum Label {{")?;
    for label in &target_labels_space {
        writeln!(output, "    {},", enum_name(label))?;
    }
    writeln!(output, "}}\n")?;
    writeln!(output, "pub(crate) const NUM_LABELS: usize = {};", target_labels_space.len())?;
    writeln!(output, "impl Label {{")?;
    writeln!(output, "    pub(crate) fn content_type(self) -> ContentType {{")?;
    writeln!(output, "        match self {{")?;
    for label in &target_labels_space {
        let name = enum_name(label);
        writeln!(output, "            Label::{name} => ContentType::{name},")?;
    }
    writeln!(output, "        }}")?;
    writeln!(output, "    }}")?;
    writeln!(output, "}}")?;
    Ok(())
}

fn create_generated_file(path: impl AsRef<Path>) -> Result<File> {
    let header = std::fs::read_to_string(file!())?;
    let header = header.split("\n\n").next().context("main.rs does not contain an empty line")?;
    let mut output = File::create(path)?;
    writeln!(output, "{header}\n")?;
    writeln!(output, "// DO NOT EDIT, see link below for more information:")?;
    writeln!(output, "// https://github.com/google/magika/tree/main/rust/gen\n")?;
    Ok(output)
}

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct ContentType {
    mime_type: Option<String>,
    group: Option<String>,
    description: Option<String>,
    extensions: Vec<String>,
    is_text: bool,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ModelConfig {
    beg_size: usize,
    mid_size: usize,
    end_size: usize,
    use_inputs_at_offsets: bool,
    medium_confidence_threshold: f32,
    min_file_size_for_dl: usize,
    padding_token: i32,
    block_size: usize,
    target_labels_space: Vec<String>,
    thresholds: BTreeMap<String, f32>,
    overwrite_map: BTreeMap<String, String>,
}

fn enum_name(xs: &str) -> String {
    assert!(xs.is_ascii());
    let mut xs = xs.as_bytes().to_vec();
    match xs[0] {
        b'A'..=b'Z' => (),
        b'a'..=b'z' => xs[0] = xs[0].to_ascii_uppercase(),
        _ => xs.insert(0, b'_'),
    }
    String::from_utf8(xs).unwrap()
}

fn const_name(xs: &str) -> String {
    assert!(xs.is_ascii());
    let mut xs = xs.as_bytes().to_ascii_uppercase();
    if !xs[0].is_ascii_uppercase() {
        xs.insert(0, b'_');
    }
    String::from_utf8(xs).unwrap()
}
