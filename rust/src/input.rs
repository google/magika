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
use std::os::unix::fs::FileExt;

use sealed::sealed;

use crate::MagikaResult;

/// Abstraction over file content.
#[sealed]
pub trait MagikaInput: Copy {
    /// Returns the size of the input.
    fn len(self) -> MagikaResult<usize>;

    /// Reads from the input at the given offset to fill the buffer.
    fn read_at(self, buffer: &mut [u8], offset: usize) -> MagikaResult<()>;
}

#[sealed]
impl MagikaInput for &[u8] {
    fn len(self) -> MagikaResult<usize> {
        Ok(self.len())
    }

    fn read_at(self, buffer: &mut [u8], offset: usize) -> MagikaResult<()> {
        buffer.copy_from_slice(&self[offset..][..buffer.len()]);
        Ok(())
    }
}

#[sealed]
impl MagikaInput for &File {
    fn len(self) -> MagikaResult<usize> {
        Ok(self.metadata()?.len() as usize)
    }

    fn read_at(self, buffer: &mut [u8], offset: usize) -> MagikaResult<()> {
        Ok(self.read_exact_at(buffer, offset as u64)?)
    }
}
