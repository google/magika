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

use crate::{BackendInfo, Features, FeaturesOrRuled, FileType, Input, Options, Runtime};

/// A Magika session to identify files.
pub struct Session {
    pub(crate) inner: magika_tract_runtime::Session,
    pub(crate) options: Options,
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

    /// Returns the session options.
    pub fn options(&self) -> &Options {
        &self.options
    }

    /// Returns the session options with mutable access.
    pub fn options_mut(&mut self) -> &mut Options {
        &mut self.options
    }

    /// Returns the resolved CPU or GPU implementation.
    pub fn backend_info(&self) -> BackendInfo {
        self.inner.backend_info().into()
    }

    /// Identifies a single file.
    pub fn identify_file(&mut self, file: impl AsRef<Path>) -> Result<FileType> {
        self.identify_features_or_ruled(FeaturesOrRuled::extract_file(file, &self.options)?)
    }

    /// Identifies a single file from its content.
    pub fn identify_content(&mut self, file: impl Input) -> Result<FileType> {
        self.identify_features_or_ruled(FeaturesOrRuled::extract_content(file, &self.options)?)
    }

    fn identify_features_or_ruled(
        &mut self, features_or_ruled: FeaturesOrRuled,
    ) -> Result<FileType> {
        match features_or_ruled {
            FeaturesOrRuled::Ruled(file_type) => Ok(file_type),
            FeaturesOrRuled::Features(features) => self.identify_features(&features),
        }
    }

    /// Identifies a single file from its features.
    pub fn identify_features(&mut self, features: &Features) -> Result<FileType> {
        let results = self.identify_features_batch([features])?;
        let [result] = results.try_into().ok().unwrap();
        Ok(result)
    }

    /// Identifies multiple files in parallel from their features.
    pub fn identify_features_batch<'a>(
        &mut self, features: impl IntoIterator<Item = &'a Features>,
    ) -> Result<Vec<FileType>> {
        let features = features.into_iter();
        let (lower, _) = features.size_hint();
        let mut input = Vec::with_capacity(lower * crate::model::CONFIG.features_size());
        let mut count = 0;
        for feature in features {
            count += 1;
            input.extend_from_slice(&feature.0);
        }
        if count == 0 {
            return Ok(Vec::new());
        }
        let output = self.inner.run(&input, count)?;
        let output = ArrayView2::from_shape((count, crate::model::LABELS.len()), &output)?;
        Ok(FileType::convert(self.options.prediction_mode, output.into_dyn()))
    }
}
