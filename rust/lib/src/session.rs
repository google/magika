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

use std::path::Path;

use anyhow::Result;
use ndarray::ArrayView2;

use crate::{BackendInfo, Features, FeaturesOrRuled, FileType, Input, Runtime};

/// A Magika session to identify files.
pub struct Session {
    pub(crate) inner: magika_tract_runtime::Session,
}

impl std::fmt::Debug for Session {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.debug_struct("Session").field("backend", &self.backend_info()).finish()
    }
}

impl Session {
    /// Creates a default session.
    pub fn new() -> Result<Self> {
        Runtime::new()?.session()
    }

    /// Returns the resolved CPU or GPU implementation.
    pub fn backend_info(&self) -> BackendInfo {
        self.inner.backend_info().into()
    }

    /// Identifies a single file.
    pub fn identify_file(&mut self, file: impl AsRef<Path>) -> Result<FileType> {
        let file = file.as_ref();
        let metadata = std::fs::symlink_metadata(file)?;
        if metadata.is_dir() {
            Ok(FileType::Directory)
        } else if metadata.is_symlink() {
            Ok(FileType::Symlink)
        } else {
            self.identify_content(std::fs::File::open(file)?)
        }
    }

    /// Identifies a single file from its content.
    pub fn identify_content(&mut self, file: impl Input) -> Result<FileType> {
        match FeaturesOrRuled::extract(file)? {
            FeaturesOrRuled::Ruled(content_type) => Ok(FileType::Ruled(content_type)),
            FeaturesOrRuled::Features(features) => self.identify_features(&features),
        }
    }

    /// Identifies a single file from its features.
    pub fn identify_features(&mut self, features: &Features) -> Result<FileType> {
        let results = self.identify_features_batch(std::slice::from_ref(features))?;
        let [result] = results.try_into().ok().unwrap();
        Ok(result)
    }

    /// Identifies multiple files in parallel from their features.
    pub fn identify_features_batch(&mut self, features: &[Features]) -> Result<Vec<FileType>> {
        if features.is_empty() {
            return Ok(Vec::new());
        }
        let input: Vec<_> =
            features.iter().flat_map(|features| features.0.iter().copied()).collect();
        let output = self.inner.run(&input, features.len())?;
        let output = ArrayView2::from_shape((features.len(), crate::model::NUM_LABELS), &output)?;
        Ok(FileType::convert(output.into_dyn()))
    }
}
