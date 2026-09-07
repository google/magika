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

//! Determines file content types using AI.
//!
//! # Examples
//!
//! ```rust
//! # fn main() -> anyhow::Result<()> {
//! // This example identifies one file at a time, so prepare a single-file plan.
//! let runtime = magika::Runtime::builder().with_max_batch(1).build()?;
//! let mut magika = runtime.session()?;
//!
//! // Files can be identified from their path.
//! assert_eq!(magika.identify_file("src/lib.rs")?.info().label, "rust");
//!
//! // Contents can also be identified directly from memory.
//! let result = magika.identify_content(&b"#!/bin/sh\necho hello"[..])?;
//! assert_eq!(result.info().label, "shell");
//! # Ok(())
//! # }
//! ```

pub use crate::backend::{Backend, BackendInfo};
pub use crate::builder::Builder;
pub use crate::content::{ContentType, MODEL_MAJOR_VERSION, MODEL_NAME};
pub use crate::file::{FileType, InferredType, OverwriteReason, TypeInfo};
pub use crate::input::{Features, FeaturesOrRuled, Input};
#[cfg(feature = "yara-rules")]
pub use crate::rules::RuleSet;
pub use crate::rules::{RulesMode, DEFAULT_RULES};
pub use crate::runtime::Runtime;
pub use crate::session::Session;

mod backend;
mod builder;
mod config;
mod content;
mod file;
mod input;
mod model;

// A regenerated model configuration must match the runtime's embedded model contract.
const _: () = {
    assert!(model::CONFIG.beg_size + model::CONFIG.end_size == magika_tract_runtime::FEATURE_SIZE);
    assert!(model::NUM_LABELS == magika_tract_runtime::NUM_LABELS);
};
mod rules;
mod runtime;
mod session;

#[cfg(test)]
mod tests {
    #[cfg(any(target_os = "macos", feature = "cuda"))]
    mod gpu;

    use std::fs::File;
    use std::io::Read;

    use data_encoding::BASE64;
    use flate2::read::GzDecoder;
    use serde::Deserialize;

    use super::*;

    #[derive(Debug, Deserialize, PartialEq, Eq)]
    #[serde(rename_all = "snake_case")]
    enum ReferencePredictionMode {
        HighConfidence,
        MediumConfidence,
        BestGuess,
    }

    #[derive(Debug, Deserialize, PartialEq, Eq)]
    #[serde(rename_all = "snake_case")]
    enum ReferenceOverwriteReason {
        None,
        LowConfidence,
        OverwriteMap,
    }

    #[test]
    fn reference_prediction_modes_reject_unknown_spellings() {
        for value in ["high-confidence", "high_confidnce", "", "unknown"] {
            let json = serde_json::to_string(value).unwrap();
            assert!(
                serde_json::from_str::<ReferencePredictionMode>(&json).is_err(),
                "accepted {value:?}"
            );
        }
    }

    #[test]
    fn reference_prediction_modes_decode_supported_vocabulary() {
        for (value, expected) in [
            ("high_confidence", ReferencePredictionMode::HighConfidence),
            ("medium_confidence", ReferencePredictionMode::MediumConfidence),
            ("best_guess", ReferencePredictionMode::BestGuess),
        ] {
            let json = serde_json::to_string(value).unwrap();
            assert_eq!(serde_json::from_str::<ReferencePredictionMode>(&json).unwrap(), expected);
        }
    }

    #[test]
    fn reference_overwrite_reasons_decode_strict_vocabulary() {
        for (value, expected) in [
            ("none", ReferenceOverwriteReason::None),
            ("low_confidence", ReferenceOverwriteReason::LowConfidence),
            ("overwrite_map", ReferenceOverwriteReason::OverwriteMap),
        ] {
            let json = serde_json::to_string(value).unwrap();
            assert_eq!(serde_json::from_str::<ReferenceOverwriteReason>(&json).unwrap(), expected);
        }
        for value in ["low-confidence", "overwrite-map", "", "unknown"] {
            let json = serde_json::to_string(value).unwrap();
            assert!(
                serde_json::from_str::<ReferenceOverwriteReason>(&json).is_err(),
                "accepted {value:?}"
            );
        }
    }

    #[derive(Debug, Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Prediction {
        dl: String,
        output: String,
        score: f32,
        overwrite_reason: ReferenceOverwriteReason,
    }

    fn assert_float(actual: f32, expected: f32, debug: &str) {
        // CPU reduction order differs from the ONNX reference. Compare an
        // absolute error instead of decimal truncation; labels remain exact.
        // Across all 116 CPU reference cases the largest observed delta is
        // 0.001402 on an eight-byte padded binary input. Keep that explicit
        // bound separate from exact label and overwrite-reason checks.
        const MAX_ABSOLUTE_ERROR: f32 = 0.002;
        assert!(
            (actual - expected).abs() <= MAX_ABSOLUTE_ERROR,
            "{debug}: actual {actual}, expected {expected}"
        );
    }

    fn assert_prediction(actual: FileType, expected: Prediction, debug: &str) {
        let actual = match actual {
            FileType::Inferred(x) => x,
            FileType::Ruled(content_type) => {
                assert_eq!(content_type.info().label, expected.output, "{debug}");
                assert_eq!(1.0, expected.score, "{debug}");
                assert_eq!(ReferenceOverwriteReason::None, expected.overwrite_reason, "{debug}");
                assert_eq!("undefined", expected.dl, "{debug}");
                return;
            }
            _ => unreachable!(),
        };
        assert_eq!(actual.content_type().info().label, expected.output, "{debug}");
        assert_float(actual.score, expected.score, debug);
        let overwrite_reason = match actual.content_type {
            None => ReferenceOverwriteReason::None,
            Some((_, OverwriteReason::LowConfidence)) => ReferenceOverwriteReason::LowConfidence,
            Some((_, OverwriteReason::OverwriteMap)) => ReferenceOverwriteReason::OverwriteMap,
        };
        assert_eq!(overwrite_reason, expected.overwrite_reason);
        assert_eq!(actual.inferred_type.info().label, expected.dl, "{debug}");
    }

    #[test]
    fn identify_by_path_reference() {
        #[derive(Debug, Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Test {
            prediction_mode: ReferencePredictionMode,
            path: String,
            status: String,
            prediction: Option<Prediction>,
        }
        let path =
            format!("../../tests_data/reference/{MODEL_NAME}-inference_examples_by_path.json.gz");
        let mut tests = String::new();
        GzDecoder::new(File::open(path).unwrap()).read_to_string(&mut tests).unwrap();
        let tests: Vec<Test> = serde_json::from_str(&tests).unwrap();
        let runtime = Runtime::builder().with_backend(Backend::Cpu).build().unwrap();
        let mut session = runtime.session().unwrap();
        let mut checked = 0;
        for test in tests {
            match test.prediction_mode {
                ReferencePredictionMode::HighConfidence => {}
                ReferencePredictionMode::MediumConfidence | ReferencePredictionMode::BestGuess => {
                    continue;
                }
            }
            checked += 1;
            assert_eq!(test.status, "ok"); // only scenario tested so far
            let expected = test.prediction.unwrap();
            let actual = session.identify_file(format!("../../{}", test.path)).unwrap();
            assert_prediction(actual, expected, &test.path);
        }
        assert!(checked > 0, "reference fixture filter must exercise predictions");
    }

    #[test]
    fn identify_by_content_reference() {
        #[derive(Debug, Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Test {
            prediction_mode: ReferencePredictionMode,
            content_base64: String,
            status: String,
            prediction: Option<Prediction>,
        }
        let path = format!(
            "../../tests_data/reference/{MODEL_NAME}-inference_examples_by_content.json.gz"
        );
        let mut tests = String::new();
        GzDecoder::new(File::open(path).unwrap()).read_to_string(&mut tests).unwrap();
        let tests: Vec<Test> = serde_json::from_str(&tests).unwrap();
        let runtime = Runtime::builder().with_backend(Backend::Cpu).build().unwrap();
        let mut session = runtime.session().unwrap();
        let mut checked = 0;
        for test in tests {
            match test.prediction_mode {
                ReferencePredictionMode::HighConfidence => {}
                ReferencePredictionMode::MediumConfidence | ReferencePredictionMode::BestGuess => {
                    continue;
                }
            }
            checked += 1;
            assert_eq!(test.status, "ok"); // only scenario tested so far
            let expected = test.prediction.unwrap();
            let content = BASE64.decode(test.content_base64.as_bytes()).unwrap();
            let actual = session.identify_content(content.as_slice()).unwrap();
            assert_prediction(actual, expected, &test.content_base64);
        }
        assert!(checked > 0, "reference fixture filter must exercise predictions");
    }

    #[test]
    fn mixed_batches_preserve_each_row_on_cpu_and_auto() {
        let mut paths: Vec<_> = std::fs::read_dir("../../tests_data/basic")
            .unwrap()
            .flat_map(|dir| std::fs::read_dir(dir.unwrap().path()).unwrap())
            .map(|entry| entry.unwrap().path())
            .filter(|path| path.is_file())
            .collect();
        paths.sort();
        let mut features: Vec<Features> = Vec::new();
        for path in paths {
            if let FeaturesOrRuled::Features(value) =
                FeaturesOrRuled::extract(File::open(path).unwrap()).unwrap()
            {
                if features.iter().all(|other| other.0 != value.0) {
                    features.push(value);
                }
            }
            if features.len() == 65 {
                break;
            }
        }
        assert_eq!(features.len(), 65, "batch-order coverage requires 65 distinct inputs");
        assert!(features.windows(2).all(|pair| pair[0].0 != pair[1].0));
        for backend in [Some(Backend::Cpu), None] {
            let builder = |maximum| {
                let mut builder = Runtime::builder().with_max_batch(maximum);
                if let Some(backend) = backend {
                    builder = builder.with_backend(backend);
                }
                builder
            };
            let reference = builder(1).build().unwrap();
            let mut single = reference.session().unwrap();
            let expected: Vec<_> =
                features.iter().map(|row| single.identify_features(row).unwrap()).collect();
            let labels: std::collections::HashSet<_> =
                expected.iter().map(|row| row.info().label).collect();
            assert!(labels.len() >= 8, "fixtures must distinguish reordered output labels");
            for maximum in [8, 64] {
                let runtime = builder(maximum).build().unwrap();
                let mut session = runtime.session().unwrap();
                for batch in [0, 1, 2, 3, 4, 5, 7, 8, 9, 15, 16, 17, 31, 32, 33, 63, 64, 65] {
                    let results = session.identify_features_batch(&features[..batch]).unwrap();
                    assert_eq!(results.len(), batch);
                    for (index, (actual, expected)) in results.iter().zip(&expected).enumerate() {
                        let context =
                            format!("{backend:?}, maximum {maximum}, batch {batch}, row {index}");
                        assert_eq!(actual.info().label, expected.info().label, "{context}");
                        assert!(
                            (actual.score() - expected.score()).abs() <= 1e-4,
                            "{context}: score {} != {}",
                            actual.score(),
                            expected.score()
                        );
                    }
                }
            }
        }
    }

    #[cfg(all(not(target_os = "macos"), not(feature = "cuda")))]
    #[test]
    fn forced_gpu_fails_when_no_gpu_backend_is_compiled() {
        assert!(Runtime::builder().with_backend(Backend::Gpu).build().is_err());
    }
}
