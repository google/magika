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

use std::fs::File;
use std::path::Path;

use ndarray::IxDyn;
use onnxruntime::tensor::OrtOwnedTensor;
use serde::Deserialize;

use crate::{MagikaFeatures, MagikaInput, MagikaOutput, MagikaResult};

#[derive(Debug, Deserialize)]
pub(crate) struct MagikaConfig {
    train_dataset_info: TrainDatasetInfo,
}

#[derive(Debug, Deserialize)]
struct TrainDatasetInfo {
    target_labels_info: TargetLabelsInfo,
}

#[derive(Debug, Deserialize)]
struct TargetLabelsInfo {
    target_labels_space: Vec<String>,
}

impl MagikaConfig {
    pub(crate) fn parse(path: impl AsRef<Path>) -> MagikaResult<Self> {
        Ok(serde_json::from_reader(File::open(path)?)?)
    }

    pub(crate) fn target_label(&self, index: usize) -> &str {
        &self
            .train_dataset_info
            .target_labels_info
            .target_labels_space[index]
    }

    pub(crate) async fn extract_features(
        &self,
        file: impl MagikaInput,
    ) -> MagikaResult<MagikaFeatures> {
        Ok(MagikaFeatures(extract_features(file).await?))
    }

    pub(crate) fn convert_output(&self, tensor: OrtOwnedTensor<f32, IxDyn>) -> Vec<MagikaOutput> {
        let mut results = Vec::new();
        for view in tensor.axis_iter(ndarray::Axis(0)) {
            let scores = view.to_slice().unwrap();
            let mut best = 0;
            for (i, &x) in scores.iter().enumerate() {
                if scores[best].max(x) == x {
                    best = i;
                }
            }
            let label = self.target_label(best).to_string();
            let score = scores[best];
            results.push(MagikaOutput { label, score });
        }
        results
    }
}

// TODO: Read those constants from the config file.
pub(crate) const FEATURE_SIZE: usize = 512;
const FEATURE_PADDING: f32 = 256f32;

async fn extract_features(mut file: impl MagikaInput) -> MagikaResult<Vec<f32>> {
    let mut features = vec![FEATURE_PADDING; 3 * FEATURE_SIZE];
    const BUFFER_SIZE: usize = 2 * 4096;
    let mut buffer = [0; BUFFER_SIZE];
    let file_len = file.length().await?;
    // We truncate the buffer to the file length because we use exact reads.
    let buffer = &mut buffer[..std::cmp::min(BUFFER_SIZE, file_len)];
    // Deal with the beginning of the file.
    file.read_at(buffer, 0).await?;
    let beg_trim = copy_features(buffer.iter(), features[..FEATURE_SIZE].iter_mut());
    // Deal with the end of the file.
    file.read_at(buffer, file_len - buffer.len()).await?;
    let end_trim = copy_features(
        buffer.iter().rev(),
        features[2 * FEATURE_SIZE..].iter_mut().rev(),
    );
    // Deal with the middle of the file.
    if file_len < beg_trim + end_trim {
        // The file is made of whitespace only. We don't even compute middle features.
        return Ok(features);
    }
    let mut file_mid = beg_trim + (file_len - beg_trim - end_trim) / 2;
    file_mid = file_mid.saturating_sub(FEATURE_SIZE / 2);
    if let Some(extra) = (file_mid + buffer.len()).checked_sub(file_len) {
        // The file is too short. We just use the same data as the end of the file, but read it in
        // file order instead of reverse order.
        file_mid -= extra;
    }
    file.read_at(buffer, file_mid).await?;
    // We don't use copy_features because we don't want to ignore whitespace.
    let mid_features = &mut features[FEATURE_SIZE..2 * FEATURE_SIZE];
    for (x, y) in buffer.iter().zip(mid_features.iter_mut()) {
        *y = *x as f32;
    }
    Ok(features)
}

/// Copies from `xs` to `ys` ignoring leading whitespace in `xs`.
///
/// Also converts from bytes to floats and returns how many bytes where ignored.
fn copy_features<'a>(
    xs: impl Iterator<Item = &'a u8>,
    ys: impl Iterator<Item = &'a mut f32>,
) -> usize {
    let mut ignored = 0;
    for (x, y) in xs
        .skip_while(|x| {
            let r = x.is_ascii_whitespace();
            ignored += r as usize;
            r
        })
        .zip(ys)
    {
        *y = *x as f32;
    }
    ignored
}
