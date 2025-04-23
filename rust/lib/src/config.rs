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

use std::borrow::Cow;

use crate::ContentType;

#[derive(Debug)]
pub(crate) struct ModelConfig {
    pub(crate) beg_size: usize,
    pub(crate) end_size: usize,
    pub(crate) min_file_size_for_dl: usize,
    pub(crate) padding_token: i32,
    pub(crate) block_size: usize,
    pub(crate) thresholds: Cow<'static, [f32; ContentType::SIZE]>,
    pub(crate) overwrite_map: Cow<'static, [ContentType; ContentType::SIZE]>,
}

pub(crate) struct SplitFeatures<'a> {
    pub(crate) beg: &'a mut [i32],
    pub(crate) end: &'a mut [i32],
}

impl ModelConfig {
    pub(crate) fn features_size(&self) -> usize {
        self.beg_size + self.end_size
    }

    pub(crate) fn split_features<'a>(&self, features: &'a mut [i32]) -> SplitFeatures<'a> {
        let (beg, features) = features.split_at_mut(self.beg_size);
        let (end, features) = features.split_at_mut(self.end_size);
        debug_assert!(features.is_empty());
        SplitFeatures { beg, end }
    }
}
