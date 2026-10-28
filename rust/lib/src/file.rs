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

use std::borrow::Borrow;

use anyhow::{Result, ensure};
use ndarray::ArrayViewD;

use crate::{ContentType, Features, MODEL_LABELS, MODEL_TRAINING_LABELS, Options};

/// File types.
///
/// The word file is used in the Linux sense where everything is a file. This could be equivalently
/// understood as a path.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum FileType {
    /// The file is a directory.
    Directory,

    /// The file is a symbolic link.
    Symlink,

    /// The file is a regular file.
    Regular(RegularFileType),

    #[doc(hidden)]
    Unsupported,
}

/// Content types for a regular file.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct RegularFileType {
    /// The content types of the file.
    ///
    /// This list is a non-empty set (there is at least one element and all elements are
    /// distinct) and its first element is the main content type.
    pub content_types: Vec<ContentType>,

    /// Debugging information about the identification.
    ///
    /// Only present when debugging is enabled. Note that debugging is not covered by SemVer.
    pub debug_info: Option<Box<DebugInfo>>,
}

/// Debugging information about the identification.
///
/// Note that debugging is not covered by SemVer.
#[derive(Debug, Clone)]
pub enum DebugInfo {
    /// The custom rules triggered.
    CustomRules(RulesDebugInfo),

    /// The resolvers triggered.
    Resolvers,

    /// The rules did not trigger and the model was skipped.
    ///
    /// This happens when the file has little content.
    Heuristic,

    /// The model triggered.
    Model(ModelDebugInfo),
}

/// Debugging information from the rules.
#[derive(Debug, Clone)]
pub struct RulesDebugInfo {
    // TODO: Expose debugging information from magika-rules.
}

/// Debugging information from the model.
#[derive(Debug, Clone)]
pub struct ModelDebugInfo {
    /// The map from content type to model confidence for that type.
    ///
    /// This list has the same length as [`MODEL_LABELS`] and is in one-to-one correspondance.
    pub output: Vec<f32>,

    /// Reason why the inferred content type was overwritten.
    pub overwrite_reason: Option<OverwriteReason>,
}

/// Reason to overwrite an inferred content type.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum OverwriteReason {
    /// The inference score is too low for the inferred content type.
    LowConfidence,

    /// The inferred content type is not canonical.
    OverwriteMap,

    /// The rules vetoed the inferred content type.
    RulesVeto,

    /// The resolvers vetoed the inferred content type.
    ResolversVeto,
}

impl From<RegularFileType> for FileType {
    fn from(value: RegularFileType) -> Self {
        FileType::Regular(value)
    }
}

impl FileType {
    /// Returns the main content type for regular files.
    pub fn content_type(&self) -> Option<ContentType> {
        match self {
            FileType::Directory => None,
            FileType::Symlink => None,
            FileType::Regular(x) => Some(x.content_type()),
            FileType::Unsupported => None,
        }
    }

    /// Returns the content types of the file.
    ///
    /// This is empty for non-regular files and non-empty for regular files.
    pub fn content_types(&self) -> &[ContentType] {
        match self {
            FileType::Directory => &[],
            FileType::Symlink => &[],
            FileType::Regular(x) => x.content_types(),
            FileType::Unsupported => &[],
        }
    }

    /// Returns the file type information of the main content type.
    pub fn info(&self) -> &'static TypeInfo {
        match self {
            FileType::Directory => &crate::content::DIRECTORY,
            FileType::Symlink => &crate::content::SYMLINK,
            FileType::Regular(x) => x.info(),
            FileType::Unsupported => &crate::content::UNSUPPORTED,
        }
    }

    /// Returns the debug information for regular files.
    pub fn debug_info(&self) -> Option<&DebugInfo> {
        match self {
            FileType::Regular(x) => x.debug_info(),
            _ => None,
        }
    }
}

impl RegularFileType {
    /// Returns the main content type.
    pub fn content_type(&self) -> ContentType {
        self.content_types[0]
    }

    /// Returns the content types of the file.
    pub fn content_types(&self) -> &[ContentType] {
        &self.content_types
    }

    /// Returns the file type information of the main content type.
    pub fn info(&self) -> &'static TypeInfo {
        self.content_type().info()
    }

    /// Returns the debug information.
    pub fn debug_info(&self) -> Option<&DebugInfo> {
        self.debug_info.as_deref()
    }
}

impl DebugInfo {
    /// Returns the model debug information.
    pub fn model(&self) -> Option<&ModelDebugInfo> {
        match self {
            DebugInfo::Model(x) => Some(x),
            _ => None,
        }
    }
}

impl ModelDebugInfo {
    /// Returns the index of the best model guess.
    pub fn best_guess(&self) -> usize {
        best_index(&self.output)
    }
}

/// File type information.
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct TypeInfo {
    /// The unique label identifying this file type.
    pub label: &'static str,

    /// The MIME type of the file type.
    pub mime_type: &'static str,

    /// The group of the file type.
    pub group: &'static str,

    /// The description of the file type.
    pub description: &'static str,

    /// Possible extensions for the file type.
    pub extensions: &'static [&'static str],

    /// Whether the file type is text.
    pub is_text: bool,

    /// Whether this file type is supported by the model.
    #[cfg_attr(feature = "serde", serde(skip_serializing))]
    pub model_support: bool,

    /// Whether this file type is supported by the rules.
    #[cfg_attr(feature = "serde", serde(skip_serializing))]
    pub rules_support: bool,
}

impl TypeInfo {
    /// Returns the list of possible outputs ordered by label.
    pub fn possible_output() -> Vec<&'static Self> {
        TypeInfo::ALL.into_iter().filter(|x| x.model_support || x.rules_support).collect()
    }

    /// Returns the list of model outputs ordered by label.
    pub fn model_output() -> Vec<&'static Self> {
        TypeInfo::ALL.into_iter().filter(|x| x.model_support).collect()
    }
}

impl RegularFileType {
    pub(crate) fn convert(
        options: &Options, features: &[impl Borrow<Features>], tensor: ArrayViewD<f32>,
    ) -> Result<Vec<RegularFileType>> {
        let mut results = Vec::new();
        for (feature, view) in features.iter().zip(tensor.view().axis_iter(ndarray::Axis(0))) {
            let scores = view.to_slice().unwrap();
            let best = best_index(scores);
            let score = scores[best];
            let overwrite = MODEL_LABELS[best];
            let ml_is_confident = options.prediction_mode.is_confident(score, best);
            ensure!(
                options.use_custom_rules == feature.borrow().use_custom_rules,
                "use_custom_rules mismatch between extract and identify"
            );
            let rules_veto =
                options.use_custom_rules.as_ref().is_some_and(|x| x.vetos().contains(&overwrite));
            ensure!(
                options.use_resolvers == feature.borrow().use_resolvers,
                "use_resolvers mismatch between extract and identify"
            );
            let resolvers_veto =
                options.use_resolvers && crate::resolvers::VETOS.contains(&overwrite);
            let fallback =
                if overwrite.info().is_text { ContentType::Txt } else { ContentType::Unknown };
            let (content_type, mut overwrite_reason) = if !ml_is_confident {
                (fallback, Some(OverwriteReason::LowConfidence))
            } else if rules_veto {
                (fallback, Some(OverwriteReason::RulesVeto))
            } else if resolvers_veto {
                (fallback, Some(OverwriteReason::ResolversVeto))
            } else {
                (overwrite, Some(OverwriteReason::OverwriteMap))
            };
            if content_type == MODEL_TRAINING_LABELS[best] {
                overwrite_reason = None;
            }
            let debug_info = options.debug_mode.then(|| {
                let output = scores.to_vec();
                Box::new(DebugInfo::Model(ModelDebugInfo { output, overwrite_reason }))
            });
            let content_types = vec![content_type];
            results.push(RegularFileType { content_types, debug_info });
        }
        Ok(results)
    }
}

fn best_index(scores: &[f32]) -> usize {
    let mut best = 0;
    for (i, &x) in scores.iter().enumerate() {
        if scores[best].max(x) == x {
            best = i;
        }
    }
    best
}
