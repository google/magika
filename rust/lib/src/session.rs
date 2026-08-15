// Copyright 2024 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     https://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::fmt;
use std::path::Path;

use ndarray::ArrayView2;

use crate::{
    AsyncInput, BackendInfo, Builder, Error, Features, FeaturesOrRuled, FileType, Result, SyncInput,
};

/// Thread-private Magika inference state.
pub struct Session {
    pub(crate) inner: magika_tract_runtime::Session,
}

impl fmt::Debug for Session {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_struct("Session").field("backend", &self.backend_info()).finish()
    }
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

    /// Returns the resolved CPU or GPU implementation.
    pub fn backend_info(&self) -> BackendInfo {
        self.inner.backend_info()
    }

    /// Identifies a single file (synchronously).
    pub fn identify_file_sync(&mut self, file: impl AsRef<Path>) -> Result<FileType> {
        let file = file.as_ref();
        let metadata = std::fs::symlink_metadata(file)?;
        if metadata.is_dir() {
            Ok(FileType::Directory)
        } else if metadata.is_symlink() {
            Ok(FileType::Symlink)
        } else {
            self.identify_content_sync(std::fs::File::open(file)?)
        }
    }

    /// Identifies a single file asynchronously.
    ///
    /// File access and feature extraction are asynchronous. Inference itself is synchronous; use a
    /// dedicated inference thread when calling this method from a latency-sensitive executor.
    pub async fn identify_file_async(&mut self, file: impl AsRef<Path>) -> Result<FileType> {
        let file = file.as_ref();
        let metadata = tokio::fs::symlink_metadata(file).await?;
        if metadata.is_dir() {
            Ok(FileType::Directory)
        } else if metadata.is_symlink() {
            Ok(FileType::Symlink)
        } else {
            self.identify_content_async(tokio::fs::File::open(file).await?).await
        }
    }

    /// Identifies a single file from its content (synchronously).
    pub fn identify_content_sync(&mut self, file: impl SyncInput) -> Result<FileType> {
        match FeaturesOrRuled::extract_sync(file)? {
            FeaturesOrRuled::Ruled(content_type) => Ok(FileType::Ruled(content_type)),
            FeaturesOrRuled::Features(features) => self.identify_features_sync(&features),
        }
    }

    /// Identifies a single file from its content asynchronously.
    ///
    /// File access and feature extraction are asynchronous. Inference itself is synchronous.
    pub async fn identify_content_async(&mut self, file: impl AsyncInput) -> Result<FileType> {
        match FeaturesOrRuled::extract_async(file).await? {
            FeaturesOrRuled::Ruled(content_type) => Ok(FileType::Ruled(content_type)),
            FeaturesOrRuled::Features(features) => self.identify_features_sync(&features),
        }
    }

    /// Identifies a single file from its features (synchronously).
    pub fn identify_features_sync(&mut self, features: &Features) -> Result<FileType> {
        let results = self.identify_features_batch_sync(std::slice::from_ref(features))?;
        let [result] = results.try_into().ok().unwrap();
        Ok(result)
    }

    /// Identifies a single file from its features.
    ///
    /// Tract inference is synchronous. This method remains for API compatibility and does not move
    /// computation onto an async executor.
    pub async fn identify_features_async(&mut self, features: &Features) -> Result<FileType> {
        self.identify_features_sync(features)
    }

    /// Identifies multiple files from their features (synchronously).
    pub fn identify_features_batch_sync(&mut self, features: &[Features]) -> Result<Vec<FileType>> {
        if features.is_empty() {
            return Ok(Vec::new());
        }
        let feature_size = crate::model::CONFIG.features_size();
        let input =
            features.iter().flat_map(|features| features.0.iter().copied()).collect::<Vec<_>>();
        debug_assert_eq!(input.len(), features.len() * feature_size);
        let output = self.inner.run(&input, features.len()).map_err(Error::inference)?;
        let output = ArrayView2::from_shape((features.len(), crate::model::NUM_LABELS), &output)?;
        Ok(FileType::convert(output.into_dyn()))
    }

    /// Identifies multiple files from their features.
    ///
    /// Tract inference is synchronous. Use dedicated inference threads for concurrent batches.
    pub async fn identify_features_batch_async(
        &mut self, features: &[Features],
    ) -> Result<Vec<FileType>> {
        self.identify_features_batch_sync(features)
    }
}
