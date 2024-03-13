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

/// The result of a file type identification.
#[derive(Clone)]
pub struct MagikaOutput {
    pub(crate) label: String,
    pub(crate) score: f32,
}

impl MagikaOutput {
    /// Returns the most probable label.
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Returns the score, between 0 and 1, of most probable label.
    pub fn score(&self) -> f32 {
        self.score
    }
}
