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

use crate::MagikaResult;

/// Processed file content, ready for inference.
pub struct MagikaFeatures(pub(crate) Vec<f32>);

/// Synchronous abstraction over file content.
pub trait MagikaSyncInput: MagikaSyncInputApi {}

/// Asynchronous abstraction over file content.
pub trait MagikaAsyncInput: MagikaAsyncInputApi {}

pub trait MagikaSyncInputApi {
    /// Returns the size of the input.
    fn length(&self) -> MagikaResult<usize>;

    /// Reads from the input at the given offset to fill the buffer.
    fn read_at(&mut self, buffer: &mut [u8], offset: usize) -> MagikaResult<()>;
}

pub trait MagikaAsyncInputApi {
    /// Returns the size of the input.
    fn length(&self) -> impl Future<Output = MagikaResult<usize>>;

    /// Reads from the input at the given offset to fill the buffer.
    fn read_at(
        &mut self,
        buffer: &mut [u8],
        offset: usize,
    ) -> impl Future<Output = MagikaResult<()>>;
}

impl MagikaSyncInput for [u8] {}
impl MagikaSyncInputApi for [u8] {
    fn length(&self) -> MagikaResult<usize> {
        Ok(self.len())
    }

    fn read_at(&mut self, buffer: &mut [u8], offset: usize) -> MagikaResult<()> {
        buffer.copy_from_slice(&self[offset..][..buffer.len()]);
        Ok(())
    }
}

impl MagikaSyncInput for std::fs::File {}
impl MagikaSyncInputApi for std::fs::File {
    fn length(&self) -> MagikaResult<usize> {
        Ok(self.metadata()?.len() as usize)
    }

    fn read_at(&mut self, buffer: &mut [u8], offset: usize) -> MagikaResult<()> {
        Ok(self.read_exact_at(buffer, offset as u64)?)
    }
}

impl<T: MagikaSyncInputApi> MagikaAsyncInputApi for T {
    fn length(&self) -> impl Future<Output = MagikaResult<usize>> {
        std::future::ready(self.length())
    }

    fn read_at(
        &mut self,
        buffer: &mut [u8],
        offset: usize,
    ) -> impl Future<Output = MagikaResult<()>> {
        std::future::ready(self.read_at(buffer, offset))
    }
}

#[cfg(feature = "tokio")]
impl MagikaAsyncInput for tokio::fs::File {}
#[cfg(feature = "tokio")]
impl MagikaAsyncInputApi for tokio::fs::File {
    async fn length(&self) -> MagikaResult<usize> {
        Ok(self.metadata().await?.len() as usize)
    }

    async fn read_at(&mut self, buffer: &mut [u8], offset: usize) -> MagikaResult<()> {
        self.seek(SeekFrom::Start(offset as u64)).await?;
        self.read_exact(buffer).await?;
        Ok(())
    }
}
