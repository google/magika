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

use std::future::Future;

use ndarray::Array2;

use crate::input::FEATURE_SIZE;
use crate::{AsyncInput, Builder, Features, FeaturesOrOutput, Output, Result, SyncInput};

/// A Magika session to identify files.
#[derive(Debug)]
pub struct Session {
    pub(crate) session: ort::Session,
}

impl Session {
    /// Creates a default session.
    pub fn new() -> Result<Self> {
        Session::builder().build()
    }

    /// Initializes a new Magika session builder with default values.
    pub fn builder() -> Builder {
        Builder::default()
    }

    /// Identifies a single file (synchronously).
    ///
    /// If the file is not suited for deep learning, a simple heuristic is used.
    pub fn identify_sync(&self, file: impl SyncInput) -> Result<Output> {
        match FeaturesOrOutput::extract_sync(file)? {
            FeaturesOrOutput::Output(output) => Ok(output),
            FeaturesOrOutput::Features(features) => self.identify_one_sync(&features),
        }
    }

    /// Identifies a single file (asynchronously).
    ///
    /// If the file is not suited for deep learning, a simple heuristic is used.
    pub async fn identify_async(&self, file: impl AsyncInput) -> Result<Output> {
        match FeaturesOrOutput::extract_async(file).await? {
            FeaturesOrOutput::Output(output) => Ok(output),
            FeaturesOrOutput::Features(features) => self.identify_one_async(&features).await,
        }
    }

    /// Identifies a single file from its features (synchronously).
    pub fn identify_one_sync(&self, features: &Features) -> Result<Output> {
        let results = self.identify_many_sync(std::slice::from_ref(features))?;
        let [result] = results.try_into().ok().unwrap();
        Ok(result)
    }

    /// Identifies a single file from its features (asynchronously).
    pub async fn identify_one_async(&self, features: &Features) -> Result<Output> {
        let results = self.identify_many_async(std::slice::from_ref(features)).await?;
        let [result] = results.try_into().ok().unwrap();
        Ok(result)
    }

    /// Identifies multiple files in parallel from their features (synchronously).
    pub fn identify_many_sync(&self, features: &[Features]) -> Result<Vec<Output>> {
        let run_async = |input| {
            Ok(std::future::ready(Ok(self.session.run(ort::inputs!("bytes" => input)?)?)))
        };
        crate::future::exec(identify_async(run_async, features))
    }

    /// Identifies multiple files in parallel from their features (asynchronously).
    pub async fn identify_many_async(&self, features: &[Features]) -> Result<Vec<Output>> {
        let run_async = |input| {
            Ok(async { Ok(self.session.run_async(ort::inputs!("bytes" => input)?)?.await?) })
        };
        identify_async(run_async, features).await
    }
}

async fn identify_async<'a, F, Fut>(run_async: F, features: &[Features]) -> Result<Vec<Output>>
where
    F: FnOnce(Array2<f32>) -> Result<Fut>,
    Fut: Future<Output = Result<ort::SessionOutputs<'a>>>,
{
    if features.is_empty() {
        return Ok(Vec::new());
    }
    let input = Array2::from_shape_vec(
        [features.len(), 3 * FEATURE_SIZE],
        features.iter().flat_map(|x| &x.0).cloned().collect(),
    )?;
    let mut output = run_async(input)?.await?;
    let output = output.remove("target_label").unwrap();
    let output = output.try_extract_tensor()?;
    Ok(Output::convert(output))
}
