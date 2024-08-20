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
use std::io::{Read, Seek, SeekFrom};

use tokio::io::{AsyncReadExt as _, AsyncSeekExt as _};

use crate::config::ModelConfig;
use crate::future::exec;
use crate::{ContentType, Result};

/// Features to identify a file with deep-learning.
pub struct Features(pub(crate) Vec<i32>);

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
        self.seek(SeekFrom::Start(offset as u64))?;
        Ok(self.read_exact(buffer)?)
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

impl AsyncInput for tokio::fs::File {}
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

/// Result of features extraction.
pub enum FeaturesOrRuled {
    /// Features extracted for deep-learning.
    Features(Features),

    /// Content identified without deep-learning.
    Ruled(ContentType),
}

impl FeaturesOrRuled {
    /// Extracts the features from a file (synchronously).
    ///
    /// Returns the content type directly if the file is not suited for deep-learning.
    pub fn extract_sync(file: impl SyncInput) -> Result<Self> {
        exec(Self::extract(file))
    }

    /// Extracts the features from a file (asynchronously).
    ///
    /// Returns the content type directly if the file is not suited for deep-learning.
    pub async fn extract_async(file: impl AsyncInput) -> Result<Self> {
        Self::extract(file).await
    }

    pub(crate) async fn extract(file: impl AsyncInputApi) -> Result<Self> {
        let config = &crate::model::CONFIG;
        let file_len = file.length().await?;
        if file_len == 0 {
            return Ok(FeaturesOrRuled::Ruled(ContentType::Empty));
        }
        let (first_block, features) = extract_features_async(config, file, file_len).await?;
        if features[config.min_file_size_for_dl - 1] != config.padding_token {
            return Ok(FeaturesOrRuled::Features(Features(features)));
        }
        debug_assert!(first_block.len() <= config.block_size);
        let content_type = match std::str::from_utf8(&first_block) {
            Ok(_) => ContentType::Txt,
            Err(_) => ContentType::Unknown,
        };
        Ok(FeaturesOrRuled::Ruled(content_type))
    }
}

async fn extract_features_async(
    config: &ModelConfig, mut file: impl AsyncInputApi, file_len: usize,
) -> Result<(Vec<u8>, Vec<i32>)> {
    debug_assert!(config.beg_size < config.block_size);
    debug_assert!(config.mid_size < config.block_size);
    debug_assert!(config.end_size < config.block_size);
    let buffer_size = std::cmp::min(config.block_size, file_len);
    let mut content_beg = vec![0; buffer_size];
    file.read_at(&mut content_beg, 0).await?;
    let beg = strip_prefix(&content_beg);
    let mut end = vec![0; buffer_size];
    file.read_at(&mut end, file_len - buffer_size).await?;
    let end = strip_suffix(&end);
    let mid_len = std::cmp::min(config.mid_size, file_len);
    let mid_off = (file_len - mid_len) / 2;
    let mut mid = vec![0; mid_len];
    file.read_at(&mut mid, mid_off).await?;
    let mut features = vec![config.padding_token; config.features_size()];
    let split_features = config.split_features(&mut features);
    copy_features(split_features.beg, beg, 0);
    copy_features(split_features.mid, &mid, 1);
    copy_features(split_features.end, end, 2);
    for (offset, features) in split_features.off {
        let mut buffer = Vec::new();
        if offset + features.len() <= file_len {
            buffer = vec![0; features.len()];
            file.read_at(&mut buffer, offset).await?;
        }
        copy_features(features, &buffer, 0);
    }
    Ok((content_beg, features))
}

fn copy_features(dst: &mut [i32], src: &[u8], align: usize) {
    let len = std::cmp::min(dst.len(), src.len());
    let dst_len = dst.len(); // borrowing issue: cannot inline below
    let dst = &mut dst[(dst_len - len) * align / 2..][..len];
    let src = &src[(src.len() - len) * align / 2..][..len];
    for (dst, src) in dst.iter_mut().zip(src.iter()) {
        *dst = *src as i32;
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
    use std::borrow::Cow;
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
            padding_token: i32,
            #[allow(dead_code)] // debugging only
            core_content_size: usize,
            #[allow(dead_code)] // debugging only
            left_ws_num: usize,
            #[allow(dead_code)] // debugging only
            right_ws_num: usize,
        }
        #[derive(Debug, Deserialize)]
        #[serde(deny_unknown_fields)]
        #[allow(dead_code)] // we only implement v2
        struct FeaturesV1 {
            beg: Vec<usize>,
            mid: Vec<usize>,
            end: Vec<usize>,
        }
        #[derive(Debug, Deserialize)]
        #[serde(deny_unknown_fields)]
        struct FeaturesV2 {
            beg: Vec<usize>,
            mid: Vec<usize>,
            end: Vec<usize>,
            offset_0x8000_0x8007: Vec<usize>,
            offset_0x8800_0x8807: Vec<usize>,
            offset_0x9000_0x9007: Vec<usize>,
            offset_0x9800_0x9807: Vec<usize>,
        }
        #[derive(Debug, Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Test {
            test_info: Info,
            content: String,
            #[allow(dead_code)] // we only implement v2
            features_v1: FeaturesV1,
            features_v2: FeaturesV2,
        }
        const PATH: &str = "../../tests_data/features_extraction/reference.json.gz";
        let mut tests = String::new();
        GzDecoder::new(File::open(PATH).unwrap()).read_to_string(&mut tests).unwrap();
        let tests: Vec<Test> = serde_json::from_str(&tests).unwrap();
        for test in tests {
            let config = ModelConfig {
                beg_size: test.test_info.beg_size,
                mid_size: test.test_info.mid_size,
                end_size: test.test_info.end_size,
                use_inputs_at_offsets: true,
                min_file_size_for_dl: 16,
                padding_token: test.test_info.padding_token,
                block_size: test.test_info.block_size,
                thresholds: Cow::Borrowed(&[0.; ContentType::SIZE]),
                overwrite_map: Cow::Borrowed(&[ContentType::Unknown; ContentType::SIZE]),
            };
            let mut expected = Vec::new();
            expected.extend_from_slice(&test.features_v2.beg);
            expected.extend_from_slice(&test.features_v2.mid);
            expected.extend_from_slice(&test.features_v2.end);
            expected.extend_from_slice(&test.features_v2.offset_0x8000_0x8007);
            expected.extend_from_slice(&test.features_v2.offset_0x8800_0x8807);
            expected.extend_from_slice(&test.features_v2.offset_0x9000_0x9007);
            expected.extend_from_slice(&test.features_v2.offset_0x9800_0x9807);
            let content = BASE64.decode(test.content.as_bytes()).unwrap();
            let actual = extract_features_async(&config, content.as_slice(), content.len());
            let actual = exec(actual).unwrap().1;
            let actual: Vec<_> = actual.into_iter().map(|x| x as usize).collect();
            assert_eq!(actual, expected, "{test:?}");
        }
    }
}
