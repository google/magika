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
//! let mut magika = magika::Session::new()?;
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

#![cfg_attr(feature = "_doc", feature(doc_cfg))]

pub use crate::backend::{Backend, BackendInfo};
pub use crate::builder::Builder;
pub use crate::content::{ContentType, MODEL_MAJOR_VERSION, MODEL_NAME};
pub use crate::file::{FileType, InferredType, OverwriteReason, TypeInfo};
pub use crate::input::{Features, FeaturesOrRuled, Input};
pub use crate::runtime::Runtime;
pub use crate::session::Session;

mod backend;
mod builder;
mod config;
mod content;
mod file;
mod input;
mod model;
mod runtime;
mod session;

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::Read;

    use data_encoding::BASE64;
    use flate2::read::GzDecoder;
    use serde::Deserialize;

    use super::*;

    #[cfg(any(target_os = "macos", feature = "cuda"))]
    mod gpu;

    #[derive(Debug, Deserialize, PartialEq, Eq)]
    #[serde(rename_all = "snake_case")]
    enum ReferenceOverwriteReason {
        None,
        LowConfidence,
        OverwriteMap,
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
        // CPU reduction order differs from the ONNX reference. Compare an absolute error instead of
        // decimal truncation; labels remain exact. Across all 116 CPU reference cases the largest
        // observed delta is 0.001402 on an eight-byte padded binary input. Keep that explicit bound
        // separate from exact label and overwrite-reason checks.
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

    #[derive(Debug, Deserialize, PartialEq, Eq)]
    #[serde(rename_all = "snake_case")]
    enum ReferencePredictionMode {
        HighConfidence,
        MediumConfidence,
        BestGuess,
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
            if test.prediction_mode != ReferencePredictionMode::HighConfidence {
                continue; // we only support high-confidence
            }
            assert_eq!(test.status, "ok"); // only scenario tested so far
            let expected = test.prediction.unwrap();
            let actual = session.identify_file(format!("../../{}", test.path)).unwrap();
            assert_prediction(actual, expected, &test.path);
            checked += 1;
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
            if test.prediction_mode != ReferencePredictionMode::HighConfidence {
                continue; // we only support high-confidence
            }
            assert_eq!(test.status, "ok"); // only scenario tested so far
            let expected = test.prediction.unwrap();
            let content = BASE64.decode(test.content_base64.as_bytes()).unwrap();
            let actual = session.identify_content(content.as_slice()).unwrap();
            assert_prediction(actual, expected, &test.content_base64);
            checked += 1;
        }
        assert!(checked > 0, "reference fixture filter must exercise predictions");
    }
}
