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
use std::pin::Pin;
use std::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};

use futures::Future;
use ndarray::IxDyn;
use onnxruntime::tensor::OrtOwnedTensor;
use serde::Deserialize;

use crate::input::{MagikaAsyncInputApi, MagikaSyncInputApi};
use crate::{MagikaAsyncInput, MagikaFeatures, MagikaOutput, MagikaResult, MagikaSyncInput};

/// Magika configuration.
#[derive(Debug, Deserialize)]
pub struct MagikaConfig {
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
    /// Parses a config file from a model directory.
    pub fn new(model_dir: impl AsRef<Path>) -> MagikaResult<Self> {
        MagikaConfig::parse(model_dir.as_ref().join("model_config.json"))
    }

    pub(crate) fn parse(path: impl AsRef<Path>) -> MagikaResult<Self> {
        Ok(serde_json::from_reader(File::open(path)?)?)
    }

    pub(crate) fn target_label(&self, index: usize) -> &str {
        &self
            .train_dataset_info
            .target_labels_info
            .target_labels_space[index]
    }

    /// Extracts the features from a file (synchronously).
    pub fn extract_features_sync(
        &self,
        file: impl MagikaSyncInput,
    ) -> MagikaResult<MagikaFeatures> {
        Ok(MagikaFeatures(extract_features_sync(file)?))
    }

    /// Extracts the features from a file (asynchronously).
    pub async fn extract_features_async(
        &self,
        file: impl MagikaAsyncInput,
    ) -> MagikaResult<MagikaFeatures> {
        Ok(MagikaFeatures(extract_features_async(file).await?))
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
const BUFFER_SIZE: usize = 2 * 4096;

fn extract_features_sync(file: impl MagikaSyncInputApi) -> MagikaResult<Vec<f32>> {
    let mut future = extract_features_async(file);
    let future = unsafe { Pin::new_unchecked(&mut future) };
    let waker = panic_waker();
    let mut context = Context::from_waker(&waker);
    match future.poll(&mut context) {
        Poll::Ready(x) => x,
        Poll::Pending => unreachable!(),
    }
}

async fn extract_features_async(mut file: impl MagikaAsyncInputApi) -> MagikaResult<Vec<f32>> {
    let file_len = file.length().await?;
    if file_len < 2 * BUFFER_SIZE + FEATURE_SIZE {
        let mut content = vec![0; file_len];
        file.read_at(&mut content, 0).await?;
        let content = strip_prefix(strip_suffix(&content));
        extract_features(&content, &content, &content)
    } else {
        let mut beg = [0; BUFFER_SIZE];
        file.read_at(&mut beg, 0).await?;
        let beg = strip_prefix(&beg);
        let mut end = [0; BUFFER_SIZE];
        file.read_at(&mut end, file_len - BUFFER_SIZE).await?;
        let end = strip_suffix(&end);
        let trimmed_beg = BUFFER_SIZE - beg.len();
        let trimmed_end = BUFFER_SIZE - end.len();
        let mid_offset = trimmed_beg + (file_len - trimmed_beg - trimmed_end - FEATURE_SIZE) / 2;
        let mut mid = [0; BUFFER_SIZE];
        file.read_at(&mut mid, mid_offset).await?;
        extract_features(&beg, &mid, &end)
    }
}

fn extract_features(beg: &[u8], mid: &[u8], end: &[u8]) -> MagikaResult<Vec<f32>> {
    let mut features = vec![FEATURE_PADDING; 3 * FEATURE_SIZE];
    copy_features(&mut features[..FEATURE_SIZE], beg, 0);
    copy_features(&mut features[FEATURE_SIZE..2 * FEATURE_SIZE], mid, 1);
    copy_features(&mut features[2 * FEATURE_SIZE..], end, 2);
    Ok(features)
}

fn copy_features(dst: &mut [f32], src: &[u8], align: usize) {
    let len = std::cmp::min(dst.len(), src.len());
    let dst_len = dst.len(); // borrowing issue: cannot inline below
    let dst = &mut dst[(dst_len - len) * align / 2..][..len];
    let src = &src[(src.len() - len) * align / 2..][..len];
    for (dst, src) in dst.iter_mut().zip(src.iter()) {
        *dst = *src as f32;
    }
}

fn strip_prefix(mut xs: &[u8]) -> &[u8] {
    while let Some(x) = xs.first() {
        if !x.is_ascii_whitespace() {
            break;
        }
        xs = &xs[1..];
    }
    xs
}

fn strip_suffix(mut xs: &[u8]) -> &[u8] {
    while let Some(x) = xs.last() {
        if !x.is_ascii_whitespace() {
            break;
        }
        xs = &xs[..xs.len() - 1];
    }
    xs
}

fn panic_waker() -> Waker {
    const PANIC_WAKER: RawWakerVTable = RawWakerVTable::new(clone, wake, wake, drop);
    fn clone(p: *const ()) -> RawWaker {
        RawWaker::new(p, &PANIC_WAKER)
    }
    fn wake(_: *const ()) {
        unreachable!()
    }
    fn drop(_: *const ()) {}
    let raw = clone(std::ptr::null());
    unsafe { Waker::from_raw(raw) }
}
