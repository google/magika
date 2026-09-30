// Copyright 2026 Google LLC
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

/// Minimum confidence level for inference.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum PredictionMode {
    /// The model output is only used when highly confident.
    HighConfidence,
    /// The model output is used when reasonably confident.
    MediumConfidence,
    /// The model output is always used.
    BestGuess,
}

/// Configuration options for identification.
#[derive(Debug, Clone)]
pub struct Options {
    /// Identifies using rules (before inference).
    pub use_rules: bool,
    /// Identifies using inference (after rules).
    pub use_model: bool,
    /// Configures the minimum confidence level for inference.
    pub prediction_mode: PredictionMode,
}

impl Default for Options {
    fn default() -> Self {
        Self { use_rules: true, use_model: true, prediction_mode: PredictionMode::HighConfidence }
    }
}

impl PredictionMode {
    pub(crate) fn is_confident(self, score: f32, label: usize) -> bool {
        let config = &crate::model::CONFIG;
        match self {
            PredictionMode::HighConfidence => config.thresholds[label] <= score,
            PredictionMode::MediumConfidence => config.medium_confidence_threshold <= score,
            PredictionMode::BestGuess => true,
        }
    }
}
