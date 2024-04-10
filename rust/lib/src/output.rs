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

use std::fmt::Display;

use ndarray::ArrayViewD;

use crate::label::METADATA;
use crate::Label;

/// The result of a file type identification.
#[derive(Debug, Clone)]
pub struct Output {
    pub(crate) label: Label,
    pub(crate) score: f32,
}

impl Output {
    /// Returns the most probable label.
    pub fn label(&self) -> Label {
        self.label
    }

    /// Returns the score, between 0 and 1, of most probable label.
    pub fn score(&self) -> f32 {
        self.score
    }
}

impl Label {
    /// Returns the content type non-capitalized enum variant.
    pub fn code(self) -> &'static str {
        METADATA[self as usize].code
    }

    /// Returns a short description of the content type.
    pub fn short_desc(self) -> &'static str {
        METADATA[self as usize].short
    }

    /// Returns a long description of the content type.
    pub fn long_desc(self) -> &'static str {
        METADATA[self as usize].long
    }

    /// Returns the magic of the content type.
    pub fn magic(self) -> &'static str {
        METADATA[self as usize].magic
    }

    /// Returns the group of the content type.
    pub fn group(self) -> &'static str {
        METADATA[self as usize].group
    }

    /// Returns the MIME type of the content type.
    pub fn mime(self) -> &'static str {
        METADATA[self as usize].mime
    }

    /// Returns whether the content type is text.
    pub fn is_text(self) -> bool {
        METADATA[self as usize].text
    }
}

pub(crate) struct Metadata {
    pub(crate) code: &'static str,
    pub(crate) short: &'static str,
    pub(crate) long: &'static str,
    pub(crate) magic: &'static str,
    pub(crate) group: &'static str,
    pub(crate) mime: &'static str,
    pub(crate) text: bool,
}

impl Display for Label {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str(self.code())
    }
}

impl Output {
    pub(crate) fn convert(tensor: ArrayViewD<f32>) -> Vec<Self> {
        let mut results = Vec::new();
        for view in tensor.view().axis_iter(ndarray::Axis(0)) {
            let scores = view.to_slice().unwrap();
            let mut best = 0;
            for (i, &x) in scores.iter().enumerate() {
                if scores[best].max(x) == x {
                    best = i;
                }
            }
            assert!(best as u32 <= crate::label::MAX_LABEL);
            // SAFETY: Labels are consecutive u32 from 0 to MAX_LABEL.
            let label = unsafe { std::mem::transmute::<u32, Label>(best as u32) };
            let score = scores[best];
            results.push(Output { label, score });
        }
        results
    }
}
