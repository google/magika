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
        Ok(Features(extract_features_sync(file)?))
    }

    /// Extracts the features from a file (asynchronously).
    pub async fn extract_async(file: impl AsyncInput) -> Result<Self> {
        Ok(Features(extract_features_async(file).await?))
    }
}

pub(crate) const FEATURE_SIZE: usize = 512;
const FEATURE_PADDING: f32 = 256f32;
const BUFFER_SIZE: usize = 2 * 4096;

fn extract_features_sync(file: impl SyncInputApi) -> Result<Vec<f32>> {
    crate::future::exec(extract_features_async(file))
}

async fn extract_features_async(mut file: impl AsyncInputApi) -> Result<Vec<f32>> {
    let file_len = file.length().await?;
    if file_len < 2 * BUFFER_SIZE + FEATURE_SIZE {
        let mut content = vec![0; file_len];
        file.read_at(&mut content, 0).await?;
        let content = strip_prefix(strip_suffix(&content));
        extract_features(content, content, content)
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
