// Copyright 2026 Google LLC
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

use anyhow::Result;

use crate::Input;

pub(crate) enum CachedInput<I: Input> {
    Partial { input: I, length: u64, prefix: Box<[u8]>, suffix: Box<[u8]> },
    Full { content: Box<[u8]> },
}

impl<I: Input> CachedInput<I> {
    pub(crate) fn new(input: I) -> Result<Self> {
        let length = input.length()?;
        Ok(Self::Partial { input, length, prefix: Box::new([]), suffix: Box::new([]) })
    }

    pub(crate) fn length(&self) -> u64 {
        match self {
            CachedInput::Partial { length, .. } => *length,
            CachedInput::Full { content } => content.len() as u64,
        }
    }

    /// Makes sure the prefix of the file of the given length is cached.
    ///
    /// Returns the length of the cached prefix.
    pub(crate) fn ensure_prefix(&mut self, len: usize) -> Result<usize> {
        let len = (len as u64).min(self.length()) as usize;
        let CachedInput::Partial { input, length, prefix, suffix } = self else {
            return Ok(len);
        };
        if len <= prefix.len() {
            return Ok(len);
        }
        if *length <= len.saturating_add(suffix.len()) as u64 {
            self.make_full()?;
            return Ok(len);
        }
        let mut new_prefix = Vec::with_capacity(len);
        new_prefix.extend_from_slice(prefix);
        new_prefix.resize(len, 0);
        input.read_at(&mut new_prefix[prefix.len()..], prefix.len() as u64)?;
        *prefix = new_prefix.into_boxed_slice();
        Ok(len)
    }

    /// Returns the prefix of the file of the given length.
    ///
    /// The prefix must have been cached for that length.
    pub(crate) fn prefix(&self, len: usize) -> &[u8] {
        let content = match self {
            CachedInput::Partial { prefix, .. } => prefix,
            CachedInput::Full { content } => content,
        };
        &content[..len]
    }

    /// Makes sure the suffix of the file of the given length is cached.
    ///
    /// Returns the length of the cached suffix.
    pub(crate) fn ensure_suffix(&mut self, len: usize) -> Result<usize> {
        let len = (len as u64).min(self.length()) as usize;
        let CachedInput::Partial { input, length, prefix, suffix } = self else {
            return Ok(len);
        };
        if len <= suffix.len() {
            return Ok(len);
        }
        if *length <= prefix.len().saturating_add(len) as u64 {
            self.make_full()?;
            return Ok(len);
        }
        let read_len = len - suffix.len();
        let mut new_suffix = Vec::with_capacity(len);
        new_suffix.resize(read_len, 0);
        input.read_at(&mut new_suffix, *length - len as u64)?;
        new_suffix.extend_from_slice(suffix);
        *suffix = new_suffix.into_boxed_slice();
        Ok(len)
    }

    /// Returns the suffix of the file of the given length.
    ///
    /// The suffix must have been cached for that length.
    pub(crate) fn suffix(&self, len: usize) -> &[u8] {
        let content = match self {
            CachedInput::Partial { suffix, .. } => suffix,
            CachedInput::Full { content } => content,
        };
        &content[content.len() - len..]
    }

    fn make_full(&mut self) -> Result<()> {
        let CachedInput::Partial { input, length, prefix, suffix } = self else { unreachable!() };
        let length = *length as usize;
        let mut content = Vec::with_capacity(length);
        content.extend_from_slice(prefix);
        content.resize(length - suffix.len(), 0);
        input.read_at(&mut content[prefix.len()..], prefix.len() as u64)?;
        content.extend_from_slice(suffix);
        *self = CachedInput::Full { content: content.into_boxed_slice() };
        Ok(())
    }
}
