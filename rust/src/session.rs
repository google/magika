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

use std::sync::Mutex;

use ndarray::Array2;
use onnxruntime::session::Session;

use crate::config::FEATURE_SIZE;
use crate::{MagikaBuilder, MagikaConfig, MagikaFeatures, MagikaInput, MagikaOutput, MagikaResult};

/// A Magika session to identify files.
#[derive(Debug)]
pub struct MagikaSession {
    pub(crate) session: Mutex<Session<'static>>,
    pub(crate) config: MagikaConfig,
}

impl MagikaSession {
    /// Initializes a new Magika session builder with default values.
    pub fn build() -> MagikaBuilder {
        MagikaBuilder::new()
    }

    /// Extracts the features from the file content for inference.
    ///
    /// This function can be parallelized. It doesn't take a lock.
    pub async fn extract(&self, file: impl MagikaInput) -> MagikaResult<MagikaFeatures> {
        self.config.extract_features(file).await
    }

    /// Identifies a single file from its features.
    pub fn identify(&self, features: &MagikaFeatures) -> MagikaResult<MagikaOutput> {
        let results = self.identify_batch(std::slice::from_ref(features))?;
        let [result] = results.try_into().ok().unwrap();
        Ok(result)
    }

    /// Identifies multiple files in parallel from their features.
    pub fn identify_batch(&self, features: &[MagikaFeatures]) -> MagikaResult<Vec<MagikaOutput>> {
        if features.len() == 0 {
            return Ok(Vec::new());
        }
        let input = Array2::from_shape_vec(
            [features.len(), 3 * FEATURE_SIZE],
            features.iter().map(|x| &x.0).flatten().cloned().collect(),
        )?;
        let mut session = self.session.lock()?;
        let input = vec![input];
        let mut output = session.run::<f32, f32, _>(input)?;
        assert_eq!(output.len(), 1);
        Ok(self.config.convert_output(output.pop().unwrap()))
    }
}
