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

use ndarray::ArrayViewD;

use crate::model::Label;
use crate::ContentType;

/// File types.
///
/// The word file is used in the Linux sense where everything is a file. This could be equivalently
/// understood as a path.
#[derive(Debug, Clone)]
pub enum FileType {
    /// The file is a directory.
    Directory,

    /// The file is a symbolic link.
    Symlink,

    /// The file is a regular file and was identified using AI.
    Inferred(InferredType),

    /// The file is a regular file and was identified using rules.
    Ruled(ContentType),
}

/// Content type identified using AI.
#[derive(Debug, Clone)]
pub struct InferredType {
    /// The content type.
    ///
    /// The inferred content type may be overwritten for a variety of reasons. Use
    /// [`Self::content_type()`] to access the final content type (after possible overwrite).
    pub content_type: Option<(ContentType, OverwriteReason)>,

    /// The inferred content type.
    pub inferred_type: ContentType,

    /// The inference score between 0 and 1.
    pub score: f32,
}

/// Reason to overwrite an inferred content type.
#[derive(Debug, Clone)]
pub enum OverwriteReason {
    /// The inference score is too low for the inferred content type.
    LowConfidence,

    /// The inferred content type is not canonical.
    OverwriteMap,
}

impl FileType {
    /// Returns the content type for regular files.
    pub fn content_type(&self) -> Option<ContentType> {
        match self {
            FileType::Directory => None,
            FileType::Symlink => None,
            FileType::Inferred(x) => Some(x.content_type()),
            FileType::Ruled(x) => Some(*x),
        }
    }

    /// Returns the file type information.
    pub fn info(&self) -> &'static TypeInfo {
        match self {
            FileType::Directory => &crate::content::DIRECTORY,
            FileType::Symlink => &crate::content::SYMLINK,
            FileType::Inferred(x) => x.content_type().info(),
            FileType::Ruled(x) => x.info(),
        }
    }

    /// Returns the score of the identification, between 0 and 1.
    ///
    /// If the model was run, this is the model score. Otherwise this is 1.
    pub fn score(&self) -> f32 {
        match self {
            FileType::Directory => 1.0,
            FileType::Symlink => 1.0,
            FileType::Inferred(x) => x.score,
            FileType::Ruled(_) => 1.0,
        }
    }
}

impl InferredType {
    /// Returns the content type.
    pub fn content_type(&self) -> ContentType {
        match self.content_type {
            Some((x, _)) => x,
            None => self.inferred_type,
        }
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
}

impl FileType {
    pub(crate) fn convert(tensor: ArrayViewD<f32>) -> Vec<FileType> {
        let mut results = Vec::new();
        for view in tensor.view().axis_iter(ndarray::Axis(0)) {
            let scores = view.to_slice().unwrap();
            let mut best = 0;
            for (i, &x) in scores.iter().enumerate() {
                if scores[best].max(x) == x {
                    best = i;
                }
            }
            assert!(best < crate::model::NUM_LABELS);
            let score = scores[best];
            // SAFETY: Labels are u32 smaller than NUM_LABELS.
            let label = unsafe { std::mem::transmute::<u32, Label>(best as u32) };
            let inferred_type = label.content_type();
            let config = &crate::model::CONFIG;
            let mut content_type = if score < config.thresholds[inferred_type as usize] {
                let is_text = inferred_type.info().is_text;
                Some((
                    if is_text { ContentType::Txt } else { ContentType::Unknown },
                    OverwriteReason::LowConfidence,
                ))
            } else {
                let overwrite = config.overwrite_map[inferred_type as usize];
                (overwrite != inferred_type).then_some((overwrite, OverwriteReason::OverwriteMap))
            };
            if content_type.as_ref().is_some_and(|(x, _)| *x == inferred_type) {
                content_type = None;
            }
            results.push(FileType::Inferred(InferredType { content_type, inferred_type, score }));
        }
        results
    }
}
