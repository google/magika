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

use anyhow::{ensure, Context, Result};
use serde::Deserialize;

fn main() -> Result<()> {
    let content_types: BTreeMap<String, ContentType> =
        serde_json::from_reader(File::open("../../assets/content_types_kb.min.json")?)?;
    let active_model = std::fs::read_to_string("../../assets/active_model.txt")?;
    let model_name = active_model.trim();
    ensure!(!model_name.is_empty(), "assets/active_model.txt is empty");
    let model_path = format!("../../assets/models/{model_name}/config.min.json");
    let model_config = serde_json::from_reader(File::open(model_path)?)?;
    let content_types = filter_content_types(content_types, &model_config)?;
    generate_lib_content(&content_types, model_name, &model_config)?;
    generate_lib_model(model_config)?;
    generate_ffi_content(&content_types)?;
    generate_ffi_header()?;
    Ok(())
}

fn filter_content_types(
    mut content_types: BTreeMap<String, ContentType>, model_config: &ModelConfig,
) -> Result<BTreeMap<String, ContentType>> {
    // We only want to generate content types that are already exposed or that are active heuristic,
    // resolver, or model labels. This is a conservative approach to avoid exposing the whole
    // knowledge base if it contains experimental content types that won't ever be exposed in the
    // future.
    let content_types_content = std::fs::read_to_string("content_types")?;
    let model_labels =
        model_config.target_labels_space.iter().map(|x| x.as_str()).collect::<BTreeSet<_>>();
    let mut labels = content_types_content.lines().map(str::to_owned).collect::<BTreeSet<_>>();
    labels.extend(model_config.target_labels_space.iter().cloned());
    for (label, info) in &content_types {
        if info.in_heuristics || info.in_resolvers || info.in_ml_model {
            labels.insert(label.clone());
        }
    }
    check_content_types(&content_types, model_config, &model_labels, &labels)?;
    let mut content_types_file = File::create("content_types")?;
    for label in &labels {
        writeln!(&mut content_types_file, "{label}")?;
    }
    content_types.retain(|x, _| labels.contains(x.as_str()));
    Ok(content_types)
}

fn check_content_types(
    content_types: &BTreeMap<String, ContentType>, model_config: &ModelConfig,
    model_labels: &BTreeSet<&str>, labels: &BTreeSet<String>,
) -> Result<()> {
    for (label, info) in content_types {
        ensure!(
            !info.vetoed_by_resolvers || info.in_resolvers,
            "vetoed_by_resolvers requires in_resolvers for {label:?}"
        );
        let in_model = model_labels.contains(label.as_str())
            && !model_config.overwrite_map.contains_key(label.as_str());
        ensure!(info.in_ml_model == in_model, "inconsistent in_ml_model for {label:?}");
    }
    for label in labels {
        ensure!(content_types.contains_key(label), "unknown content type {label:?}");
    }
    Ok(())
}

fn generate_lib_content(
    content_types: &BTreeMap<String, ContentType>, model_name: &str, model_config: &ModelConfig,
) -> Result<()> {
    let mut output = create_generated_file("../lib/src/content.rs")?;
    writeln!(output, "use crate::file::TypeInfo;\n")?;
    writeln!(output, "/// Model name (only comparable with equality).")?;
    writeln!(output, "pub const MODEL_NAME: &str = {model_name:?};\n")?;
    writeln!(output, "/// Model major version.")?;
    writeln!(output, "pub const MODEL_MAJOR_VERSION: u32 = {};\n", model_config.version_major)?;
    struct Variant {
        label: String,
        doc: String,
    }
    let mut variants = Vec::new();
    for (label, info) in content_types {
        let mime_type = info.mime_type();
        let group = info.group();
        let description = info.description(label);
        let extensions = &info.extensions;
        let is_text = info.is_text;
        let in_heuristics = info.in_heuristics;
        let in_resolvers = info.in_resolvers;
        let in_ml_model = info.in_ml_model;
        let vetoed_by_resolvers = info.vetoed_by_resolvers;
        if !is_special_label(label) {
            variants.push(Variant { label: label.clone(), doc: description.clone() });
        }
        writeln!(output, "pub(crate) static {}: TypeInfo = TypeInfo {{", const_name(label))?;
        writeln!(output, "    label: {label:?},")?;
        writeln!(output, "    mime_type: {mime_type:?},")?;
        writeln!(output, "    group: {group:?},")?;
        writeln!(output, "    description: {description:?},")?;
        writeln!(output, "    extensions: &{extensions:?},")?;
        writeln!(output, "    is_text: {is_text:?},")?;
        writeln!(output, "    in_heuristics: {in_heuristics:?},")?;
        writeln!(output, "    in_resolvers: {in_resolvers:?},")?;
        writeln!(output, "    in_ml_model: {in_ml_model:?},")?;
        writeln!(output, "    vetoed_by_resolvers: {vetoed_by_resolvers:?},")?;
        writeln!(output, "}};\n")?;
    }
    writeln!(output, "impl TypeInfo {{")?;
    writeln!(output, "    /// All file type information ordered by label.")?;
    writeln!(output, "    pub const ALL: [&'static Self; {}] = [", content_types.len())?;
    for label in content_types.keys() {
        writeln!(output, "        &{},", const_name(label))?;
    }
    writeln!(output, "    ];")?;
    writeln!(output, "}}\n")?;
    writeln!(output, "/// Content types for regular files.")?;
    writeln!(output, "#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]")?;
    writeln!(output, "#[non_exhaustive]")?;
    writeln!(output, "pub enum ContentType {{")?;
    for Variant { label, doc } in &variants {
        writeln!(output, "    /// {doc}")?;
        writeln!(output, "    {},", enum_name(label))?;
    }
    writeln!(output, "}}\n")?;
    writeln!(output, "impl ContentType {{")?;
    writeln!(output, "    /// Returns a content type from its label.")?;
    writeln!(output, "    pub fn from_label(label: &str) -> Option<Self> {{")?;
    writeln!(output, "        Some(match label {{")?;
    for Variant { label, .. } in &variants {
        writeln!(output, "            {label:?} => Self::{},", enum_name(label))?;
    }
    writeln!(output, "            _ => return None,")?;
    writeln!(output, "        }})")?;
    writeln!(output, "    }}\n")?;
    writeln!(output, "    /// Returns the content type information.")?;
    writeln!(output, "    pub fn info(self) -> &'static TypeInfo {{")?;
    writeln!(output, "        match self {{")?;
    for Variant { label, .. } in variants {
        writeln!(
            output,
            "            ContentType::{} => &{},",
            enum_name(&label),
            const_name(&label),
        )?;
    }
    writeln!(output, "        }}")?;
    writeln!(output, "    }}")?;
    writeln!(output, "}}")?;
    Ok(())
}

fn generate_lib_model(model_config: ModelConfig) -> Result<()> {
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
        protection: _,
        aes_key_hex: _,
        version_major: _,
    } = model_config;
    let mut output = create_generated_file("../lib/src/model.rs")?;
    writeln!(output, "use crate::ContentType;")?;
    writeln!(output, "use crate::config::ModelConfig;\n")?;
    writeln!(output, "pub(crate) const CONFIG: ModelConfig = ModelConfig {{")?;
    writeln!(output, "    beg_size: {beg_size},")?;
    ensure!(mid_size == 0, "unsupported mid_size");
    writeln!(output, "    end_size: {end_size},")?;
    ensure!(!use_inputs_at_offsets, "unsupported use_inputs_at_offsets");
    writeln!(output, "    medium_confidence_threshold: {medium_confidence_threshold},")?;
    writeln!(output, "    min_file_size_for_dl: {min_file_size_for_dl},")?;
    writeln!(output, "    padding_token: {padding_token},")?;
    writeln!(output, "    block_size: {block_size},")?;
    writeln!(output, "}};\n")?;
    let num_labels = target_labels_space.len();
    let mut thresholds_array = vec![medium_confidence_threshold; num_labels];
    for (label, threshold) in thresholds {
        let pos = target_labels_space.iter().position(|x| *x == label).unwrap();
        thresholds_array[pos] = threshold;
    }
    writeln!(output, "#[rustfmt::skip]")?;
    writeln!(output, "pub(crate) const THRESHOLDS: [f32; {num_labels}] = {thresholds_array:?};")?;
    writeln!(output, "/// Content types output by the model (after overwrite).")?;
    writeln!(output, "pub const MODEL_LABELS: [ContentType; {num_labels}] = [")?;
    let mut overwrite_array = target_labels_space.to_vec();
    for (src, dst) in overwrite_map {
        let pos = target_labels_space.iter().position(|x| *x == src).unwrap();
        overwrite_array[pos] = dst;
    }
    for label in overwrite_array {
        writeln!(output, "    ContentType::{},", enum_name(&label))?;
    }
    writeln!(output, "];\n")?;
    writeln!(output, "/// Content types output by the model (before overwrite).")?;
    writeln!(output, "pub const MODEL_TRAINING_LABELS: [ContentType; {num_labels}] = [")?;
    for label in &target_labels_space {
        writeln!(output, "    ContentType::{},", enum_name(label))?;
    }
    writeln!(output, "];")?;
    Ok(())
}

fn generate_ffi_content(content_types: &BTreeMap<String, ContentType>) -> Result<()> {
    let mut output = create_generated_file("../ffi/src/content.rs")?;
    writeln!(output, "use std::ptr;\n")?;
    writeln!(output, "use crate::MagikaTypeInfo;\n")?;
    let mut variants = Vec::new();
    for (label, info) in content_types {
        let mime_type = info.mime_type();
        let group = info.group();
        let description = info.description(label);
        if !is_special_label(label) {
            variants.push(label.clone());
        }
        let mut extensions = "[".to_string();
        for extension in &info.extensions {
            extensions.push_str(&format!("c{extension:?}.as_ptr(), "));
        }
        extensions.push_str("ptr::null()].as_ptr()");
        let is_text = info.is_text;
        writeln!(
            output,
            "#[rustfmt::skip] pub(crate) static {}: MagikaTypeInfo = MagikaTypeInfo {{",
            const_name(label)
        )?;
        writeln!(output, "    label: c{label:?}.as_ptr(),")?;
        writeln!(output, "    mime_type: c{mime_type:?}.as_ptr(),")?;
        writeln!(output, "    group: c{group:?}.as_ptr(),")?;
        writeln!(output, "    description: c{description:?}.as_ptr(),")?;
        writeln!(output, "    extensions: {extensions},")?;
        writeln!(output, "    is_text: {is_text:?},")?;
        writeln!(output, "}};\n")?;
    }
    writeln!(
        output,
        "pub(crate) fn content_type_info(content_type: magika::ContentType) -> &'static MagikaTypeInfo {{"
    )?;
    writeln!(output, "    match content_type {{")?;
    for label in &variants {
        writeln!(
            output,
            "        magika::ContentType::{} => &{},",
            enum_name(label),
            const_name(label),
        )?;
    }
    writeln!(output, "        _ => unreachable!(),")?;
    writeln!(output, "    }}")?;
    writeln!(output, "}}")?;
    Ok(())
}

fn generate_ffi_header() -> Result<()> {
    let _ = cbindgen::generate("../ffi")?.write_to_file("../ffi/include/magika.h");
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
    in_heuristics: bool,
    in_resolvers: bool,
    in_ml_model: bool,
    vetoed_by_resolvers: bool,
}

impl ContentType {
    fn mime_type(&self) -> String {
        self.mime_type.clone().unwrap_or_else(|| {
            if self.is_text { "text/plain" } else { "application/octet-stream" }.to_string()
        })
    }

    fn group(&self) -> String {
        self.group.clone().unwrap_or_else(|| "unknown".to_string())
    }

    fn description(&self, label: &str) -> String {
        self.description.clone().unwrap_or_else(|| label.to_string())
    }
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
    #[allow(dead_code)]
    protection: String,
    #[allow(dead_code)]
    aes_key_hex: String,
    version_major: u32,
}

fn enum_name(label: &str) -> String {
    label_name(label, false)
}

fn const_name(label: &str) -> String {
    label_name(label, true)
}

fn label_name(label: &str, scream: bool) -> String {
    let mut name = String::new();
    for (i, word) in label.split('_').enumerate() {
        if i == 0 && !word.starts_with(|x: char| x.is_alphabetic()) || i != 0 && scream {
            name.push('_');
        }
        for (j, x) in word.chars().enumerate() {
            name.push(if j == 0 || scream { x.to_ascii_uppercase() } else { x });
        }
    }
    name
}

fn is_special_label(label: &str) -> bool {
    matches!(label, "directory" | "symlink" | "unsupported")
}
