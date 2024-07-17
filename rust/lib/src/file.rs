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

    /// The file is a regular file and was identified with deep-learning.
    Inferred(InferredType),

    /// The file is a regular file and was identified without deep-learning.
    Ruled(RuledType),
}

impl From<InferredType> for FileType {
    fn from(value: InferredType) -> Self {
        FileType::Inferred(value)
    }
}

impl From<RuledType> for FileType {
    fn from(value: RuledType) -> Self {
        FileType::Ruled(value)
    }
}

impl From<ContentType> for FileType {
    fn from(value: ContentType) -> Self {
        FileType::Ruled(value.into())
    }
}

/// Content type identified with deep-learning.
#[derive(Debug, Clone)]
pub struct InferredType {
    /// The inferred content type.
    pub content_type: ContentType,

    /// The inference score between 0 and 1.
    pub score: f32,
}

impl InferredType {
    /// Overrules an inferred content type.
    pub fn overrule_with(self, content_type: ContentType) -> RuledType {
        RuledType { content_type, overruled: Some(self) }
    }
}

/// Content type identified without deep-learning.
#[derive(Debug, Clone)]
pub struct RuledType {
    /// The ruled content type.
    pub content_type: ContentType,

    /// The overruled content type identified with deep-learning, if any.
    pub overruled: Option<InferredType>,
}

impl From<ContentType> for RuledType {
    fn from(content_type: ContentType) -> Self {
        RuledType { content_type, overruled: None }
    }
}

impl FileType {
    /// Returns the content type for regular files.
    pub fn content_type(&self) -> Option<ContentType> {
        match self {
            FileType::Directory => None,
            FileType::Symlink => None,
            FileType::Inferred(x) => Some(x.content_type),
            FileType::Ruled(x) => Some(x.content_type),
        }
    }

    /// Returns the file type information.
    pub fn info(&self) -> &'static TypeInfo {
        match self {
            FileType::Directory => &crate::content::DIRECTORY,
            FileType::Symlink => &crate::content::SYMLINK,
            FileType::Inferred(x) => x.content_type.info(),
            FileType::Ruled(x) => x.content_type.info(),
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
            FileType::Ruled(RuledType { overruled: None, .. }) => 1.0,
            FileType::Ruled(RuledType { overruled: Some(x), .. }) => x.score,
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
            // SAFETY: Labels are u32 smaller than NUM_LABELS.
            let label = unsafe { std::mem::transmute::<u32, Label>(best as u32) };
            let inferred = InferredType { content_type: label.content_type(), score: scores[best] };
            let config = &crate::model::CONFIG;
            let mut overwrite = config.overwrite_map[inferred.content_type as usize];
            if inferred.score < config.thresholds[overwrite as usize] {
                overwrite =
                    if overwrite.info().is_text { ContentType::Txt } else { ContentType::Unknown };
            }
            let file_type = if overwrite == inferred.content_type {
                inferred.into()
            } else {
                inferred.overrule_with(overwrite).into()
            };
            results.push(file_type);
        }
        results
    }
}
