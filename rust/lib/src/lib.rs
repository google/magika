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

//! Determines the type of a file with deep-learning.
//!
//! # Examples
//!
//! ```rust
//! # fn main() -> magika::Result<()> {
//! // A Magika session can be used multiple times across multiple threads.
//! let mut magika = magika::Session::new()?;
//!
//! // Files can be identified from their path.
//! assert_eq!(magika.identify_file_sync("src/lib.rs")?.info().label, "rust");
//!
//! // Contents can also be identified directly from memory.
//! let result = magika.identify_content_sync(&b"#!/bin/sh\necho hello"[..])?;
//! assert_eq!(result.info().label, "shell");
//! # Ok(())
//! # }
//! ```

#![cfg_attr(feature = "_doc", feature(doc_auto_cfg))]

pub use crate::builder::Builder;
pub use crate::content::{ContentType, MODEL_MAJOR_VERSION, MODEL_NAME};
pub use crate::error::{Error, Result};
pub use crate::file::{FileType, InferredType, OverwriteReason, TypeInfo};
pub use crate::input::{AsyncInput, Features, FeaturesOrRuled, SyncInput};
pub use crate::session::Session;

mod builder;
mod config;
mod content;
mod error;
mod file;
mod future;
mod input;
mod model;
mod session;

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::Read;

    use data_encoding::BASE64;
    use flate2::read::GzDecoder;
    use serde::Deserialize;

    use super::*;

    #[derive(Debug, Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Prediction {
        dl: String,
        output: String,
        score: f32,
        overwrite_reason: String,
    }

    fn assert_float(actual: f32, expected: f32, debug: &str) {
        const PRECISION: f32 = 10000.;
        let actual = (actual * PRECISION).trunc() / PRECISION;
        let expected = (expected * PRECISION).trunc() / PRECISION;
        assert_eq!(actual, expected, "{debug}");
    }

    fn assert_prediction(actual: FileType, expected: Prediction, debug: &str) {
        let actual = match actual {
            FileType::Inferred(x) => x,
            FileType::Ruled(content_type) => {
                assert_eq!(content_type.info().label, expected.output, "{debug}");
                assert_eq!(1.0, expected.score, "{debug}");
                assert_eq!("none", expected.overwrite_reason, "{debug}");
                assert_eq!("undefined", expected.dl, "{debug}");
                return;
            }
            _ => unreachable!(),
        };
        assert_eq!(actual.content_type().info().label, expected.output, "{debug}");
        assert_float(actual.score, expected.score, debug);
        let overwrite_reason = match actual.content_type {
            None => "none",
            Some((_, OverwriteReason::LowConfidence)) => "low-confidence",
            Some((_, OverwriteReason::OverwriteMap)) => "overwrite-map",
        };
        assert_eq!(overwrite_reason, expected.overwrite_reason);
        assert_eq!(actual.inferred_type.info().label, expected.dl, "{debug}");
    }

    #[test]
    fn identify_by_path_reference() {
        #[derive(Debug, Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Test {
            prediction_mode: String,
            path: String,
            status: String,
            prediction: Option<Prediction>,
        }
        let path =
            format!("../../tests_data/reference/{MODEL_NAME}-inference_examples_by_path.json.gz");
        let mut tests = String::new();
        GzDecoder::new(File::open(path).unwrap()).read_to_string(&mut tests).unwrap();
        let tests: Vec<Test> = serde_json::from_str(&tests).unwrap();
        let mut session = Session::new().unwrap();
        for test in tests {
            if test.prediction_mode != "high-confidence" {
                continue; // we only support high-confidence
            }
            assert_eq!(test.status, "ok"); // only scenario tested so far
            let expected = test.prediction.unwrap();
            let actual = session.identify_file_sync(format!("../../{}", test.path)).unwrap();
            assert_prediction(actual, expected, &test.path);
        }
    }

    #[test]
    fn identify_by_content_reference() {
        #[derive(Debug, Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Test {
            prediction_mode: String,
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
        let mut session = Session::new().unwrap();
        for test in tests {
            if test.prediction_mode != "high-confidence" {
                continue; // we only support high-confidence
            }
            assert_eq!(test.status, "ok"); // only scenario tested so far
            let expected = test.prediction.unwrap();
            let content = BASE64.decode(test.content_base64.as_bytes()).unwrap();
            let actual = session.identify_content_sync(content.as_slice()).unwrap();
            assert_prediction(actual, expected, &test.content_base64);
        }
    }
}
