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
#[cfg(feature = "tokio")]
use std::io::SeekFrom;
use std::os::unix::fs::FileExt as _;

#[cfg(feature = "tokio")]
use tokio::io::{AsyncReadExt as _, AsyncSeekExt as _};

use crate::Result;

/// Processed file content, ready for inference.
pub struct Features(pub(crate) Vec<f32>);

/// Synchronous abstraction over file content.
pub trait SyncInput: SyncInputApi {}

/// Asynchronous abstraction over file content.
pub trait AsyncInput: AsyncInputApi {}

pub trait SyncInputApi {
    /// Returns the size of the input.
    fn length(&self) -> Result<usize>;

    /// Reads from the input at the given offset to fill the buffer.
    fn read_at(&mut self, buffer: &mut [u8], offset: usize) -> Result<()>;
}

pub trait AsyncInputApi {
    /// Returns the size of the input.
    fn length(&self) -> impl Future<Output = Result<usize>>;

    /// Reads from the input at the given offset to fill the buffer.
    fn read_at(&mut self, buffer: &mut [u8], offset: usize) -> impl Future<Output = Result<()>>;
}

impl SyncInput for &[u8] {}
impl SyncInputApi for &[u8] {
    fn length(&self) -> Result<usize> {
        Ok(self.len())
    }

    fn read_at(&mut self, buffer: &mut [u8], offset: usize) -> Result<()> {
        buffer.copy_from_slice(&self[offset..][..buffer.len()]);
        Ok(())
    }
}

impl SyncInput for std::fs::File {}
impl SyncInputApi for std::fs::File {
    fn length(&self) -> Result<usize> {
        Ok(self.metadata()?.len() as usize)
    }

    fn read_at(&mut self, buffer: &mut [u8], offset: usize) -> Result<()> {
        Ok(self.read_exact_at(buffer, offset as u64)?)
    }
}

impl<T: SyncInputApi> SyncInput for &mut T {}
impl<T: SyncInputApi> SyncInputApi for &mut T {
    fn length(&self) -> Result<usize> {
        <T as SyncInputApi>::length(self)
    }

    fn read_at(&mut self, buffer: &mut [u8], offset: usize) -> Result<()> {
        <T as SyncInputApi>::read_at(self, buffer, offset)
    }
}

impl<T: SyncInputApi> AsyncInputApi for T {
    fn length(&self) -> impl Future<Output = Result<usize>> {
        std::future::ready(self.length())
    }

    fn read_at(&mut self, buffer: &mut [u8], offset: usize) -> impl Future<Output = Result<()>> {
        std::future::ready(self.read_at(buffer, offset))
    }
}

#[cfg(feature = "tokio")]
impl AsyncInput for tokio::fs::File {}
#[cfg(feature = "tokio")]
impl AsyncInputApi for tokio::fs::File {
    async fn length(&self) -> Result<usize> {
        Ok(self.metadata().await?.len() as usize)
    }

    async fn read_at(&mut self, buffer: &mut [u8], offset: usize) -> Result<()> {
        self.seek(SeekFrom::Start(offset as u64)).await?;
        self.read_exact(buffer).await?;
        Ok(())
    }
}

impl Features {
    /// Extracts the features from a file (synchronously).
    pub fn extract_sync(file: impl SyncInput) -> Result<Self> {
        Ok(Features(extract_features_sync(BUFFER_SIZE, file)?))
    }

    /// Extracts the features from a file (asynchronously).
    pub async fn extract_async(file: impl AsyncInput) -> Result<Self> {
        Ok(Features(extract_features_async(BUFFER_SIZE, file).await?))
    }
}

pub(crate) const FEATURE_SIZE: usize = 512;
const FEATURE_PADDING: f32 = 256f32;
const BUFFER_SIZE: usize = 8192;

fn extract_features_sync(buffer_size: usize, file: impl SyncInputApi) -> Result<Vec<f32>> {
    crate::future::exec(extract_features_async(buffer_size, file))
}

async fn extract_features_async(
    buffer_size: usize, mut file: impl AsyncInputApi,
) -> Result<Vec<f32>> {
    let file_len = file.length().await?;
    if file_len < 2 * buffer_size + FEATURE_SIZE {
        let mut content = vec![0; file_len];
        file.read_at(&mut content, 0).await?;
        let content = strip_prefix(strip_suffix(&content));
        extract_features(content, content, content)
    } else {
        let mut beg = vec![0; buffer_size];
        file.read_at(&mut beg, 0).await?;
        let beg = strip_prefix(&beg);
        let mut end = vec![0; buffer_size];
        file.read_at(&mut end, file_len - buffer_size).await?;
        let end = strip_suffix(&end);
        let trimmed_beg = buffer_size - beg.len();
        let trimmed_end = buffer_size - end.len();
        let mid_offset = trimmed_beg + (file_len - trimmed_beg - trimmed_end - FEATURE_SIZE) / 2;
        let mut mid = vec![0; FEATURE_SIZE];
        file.read_at(&mut mid, mid_offset).await?;
        extract_features(beg, &mid, end)
    }
}

fn extract_features(beg: &[u8], mid: &[u8], end: &[u8]) -> Result<Vec<f32>> {
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

fn strip_prefix(xs: &[u8]) -> &[u8] {
    strip(xs, |xs| xs.split_first())
}

fn strip_suffix(xs: &[u8]) -> &[u8] {
    strip(xs, |xs| xs.split_last())
}

fn strip(mut xs: &[u8], mut split: impl FnMut(&[u8]) -> Option<(&u8, &[u8])>) -> &[u8] {
    while let Some((&x, ys)) = split(xs) {
        if !is_whitespace(x) {
            break;
        }
        xs = ys;
    }
    xs
}

fn is_whitespace(x: u8) -> bool {
    x.is_ascii_whitespace() || x == 0x0b
}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::Read;

    use data_encoding::BASE64;
    use flate2::read::GzDecoder;
    use serde::Deserialize;

    use super::*;

    #[test]
    fn features_extraction_reference() {
        // We deny unknown fields to be sure we don't pass the tests by accident when the JSON
        // format is modified. Fields that are not used are simply marked as dead-code.
        #[derive(Debug, Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Info {
            beg_size: usize,
            mid_size: usize,
            end_size: usize,
            block_size: usize,
            padding_token: usize,
            #[allow(dead_code)] // debugging only
            core_content_size: usize,
            #[allow(dead_code)] // debugging only
            left_ws_num: usize,
            #[allow(dead_code)] // debugging only
            right_ws_num: usize,
        }
        #[derive(Debug, Deserialize)]
        #[serde(deny_unknown_fields)]
        struct FeaturesV1 {
            beg: Vec<usize>,
            mid: Vec<usize>,
            end: Vec<usize>,
        }
        #[derive(Debug, Deserialize)]
        struct FeaturesV2 {}
        #[derive(Debug, Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Test {
            test_info: Info,
            content: String,
            features_v1: FeaturesV1,
            #[allow(dead_code)] // we only implement v1 at this time
            features_v2: FeaturesV2,
        }
        const PATH: &str = "../../tests_data/features_extraction/reference.json.gz";
        let mut tests = String::new();
        GzDecoder::new(File::open(PATH).unwrap()).read_to_string(&mut tests).unwrap();
        let tests: Vec<Test> = serde_json::from_str(&tests).unwrap();
        for test in tests {
            assert_eq!(test.test_info.beg_size, FEATURE_SIZE);
            assert_eq!(test.test_info.mid_size, FEATURE_SIZE);
            assert_eq!(test.test_info.end_size, FEATURE_SIZE);
            assert_eq!(test.test_info.padding_token, FEATURE_PADDING as usize);
            let block_size = test.test_info.block_size;
            let mut expected = Vec::new();
            expected.extend_from_slice(&test.features_v1.beg);
            expected.extend_from_slice(&test.features_v1.mid);
            expected.extend_from_slice(&test.features_v1.end);
            let content = BASE64.decode(test.content.as_bytes()).unwrap();
            let actual = extract_features_sync(block_size, content.as_slice()).unwrap();
            let actual: Vec<_> = actual.into_iter().map(|x| x as usize).collect();
            assert_eq!(actual, expected);
        }
    }
}
