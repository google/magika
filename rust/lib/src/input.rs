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

use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

use anyhow::Result;

use crate::cached_input::CachedInput;
use crate::config::ModelConfig;
use crate::{ContentType, DebugInfo, FileType, Options, RegularFileType, Rules, RulesDebugInfo};

/// Features to identify a file using AI.
pub struct Features {
    pub(crate) features: Vec<i32>,
    pub(crate) use_resolvers: bool,
    pub(crate) use_custom_rules: Option<Rules>,
}

/// Abstraction over file content.
pub trait Input {
    /// Returns the size of the input.
    fn length(&self) -> Result<u64>;

    /// Reads from the input at the given offset to fill the buffer.
    fn read_at(&mut self, buffer: &mut [u8], offset: u64) -> Result<()>;
}

const _: () = const {
    // We assume in the rest of the file, that u64 holds any usize.
    assert!(std::mem::size_of::<usize>() <= std::mem::size_of::<u64>());
};

impl Input for &[u8] {
    fn length(&self) -> Result<u64> {
        Ok(self.len() as u64)
    }

    fn read_at(&mut self, buffer: &mut [u8], offset: u64) -> Result<()> {
        let offset = offset.try_into().unwrap();
        buffer.copy_from_slice(&self[offset..][..buffer.len()]);
        Ok(())
    }
}

impl Input for std::fs::File {
    fn length(&self) -> Result<u64> {
        Ok(self.metadata()?.len())
    }

    fn read_at(&mut self, buffer: &mut [u8], offset: u64) -> Result<()> {
        self.seek(SeekFrom::Start(offset))?;
        Ok(self.read_exact(buffer)?)
    }
}

impl<T: Input> Input for &mut T {
    fn length(&self) -> Result<u64> {
        <T as Input>::length(self)
    }

    fn read_at(&mut self, buffer: &mut [u8], offset: u64) -> Result<()> {
        <T as Input>::read_at(self, buffer, offset)
    }
}

/// Result of features extraction.
pub enum FeaturesOr<T> {
    /// Features extracted for identification using AI.
    Features(Features),

    /// File or content identified without AI.
    Ruled(T),
}

impl FeaturesOr<FileType> {
    /// Extracts the features from a file path.
    ///
    /// Returns the file type directly if the file cannot be identified using AI.
    pub fn extract_file(file: impl AsRef<Path>, options: &Options) -> Result<Self> {
        let file = file.as_ref();
        let metadata = if options.follow_symlink {
            std::fs::metadata(file)?
        } else {
            std::fs::symlink_metadata(file)?
        };
        if metadata.is_dir() {
            Ok(FeaturesOr::Ruled(FileType::Directory))
        } else if metadata.is_symlink() {
            Ok(FeaturesOr::Ruled(FileType::Symlink))
        } else if !metadata.is_file() {
            Ok(FeaturesOr::Ruled(FileType::Unsupported))
        } else {
            Ok(FeaturesOr::extract_content(std::fs::File::open(file)?, options)?.into())
        }
    }
}

impl From<FeaturesOr<RegularFileType>> for FeaturesOr<FileType> {
    fn from(value: FeaturesOr<RegularFileType>) -> Self {
        match value {
            FeaturesOr::Features(x) => FeaturesOr::Features(x),
            FeaturesOr::Ruled(x) => FeaturesOr::Ruled(x.into()),
        }
    }
}

impl FeaturesOr<RegularFileType> {
    /// Extracts the features from file content.
    ///
    /// Returns the file type directly if the content cannot be identified using AI.
    pub fn extract_content(file: impl Input, options: &Options) -> Result<Self> {
        let config = &crate::model::CONFIG;
        let mut file = CachedInput::new(file)?;
        if file.length() == 0 {
            let content_types = vec![ContentType::Empty];
            let debug_info = options.debug_mode.then(|| Box::new(DebugInfo::Heuristic));
            return Ok(FeaturesOr::Ruled(RegularFileType { content_types, debug_info }));
        }
        if let Some(custom_rules) = &options.use_custom_rules {
            let content_types = custom_rules.identify(&mut file)?;
            if !content_types.is_empty() {
                let debug_info =
                    options.debug_mode.then(|| Box::new(DebugInfo::CustomRules(RulesDebugInfo {})));
                return Ok(FeaturesOr::Ruled(RegularFileType { content_types, debug_info }));
            }
        }
        if options.use_resolvers {
            let content_types = crate::resolvers::identify(&mut file)?;
            if !content_types.is_empty() {
                let debug_info = options.debug_mode.then(|| Box::new(DebugInfo::Resolvers));
                return Ok(FeaturesOr::Ruled(RegularFileType { content_types, debug_info }));
            }
        }
        if options.use_model {
            let features = extract_features(config, &mut file)?;
            if features[config.min_file_size_for_dl - 1] != config.padding_token {
                return Ok(FeaturesOr::Features(Features {
                    features,
                    use_resolvers: options.use_resolvers,
                    use_custom_rules: options.use_custom_rules.clone(),
                }));
            }
        }
        let prefix_len = file.ensure_prefix(config.block_size)?;
        let prefix = file.prefix(prefix_len);
        let content_type = match std::str::from_utf8(prefix) {
            // The file is not UTF-8.
            Err(_) if file.length() == prefix.len() as u64 => ContentType::Unknown,
            // The first block doesn't disprove that the file may be UTF-8.
            Err(e) if e.error_len().is_none() => ContentType::Txt,
            // The first block disproves that the file is UTF-8.
            Err(_) => ContentType::Unknown,
            // The file is UTF-8 or its first block doesn't disprove that it can't be.
            Ok(_) => ContentType::Txt,
        };
        let content_types = vec![content_type];
        let debug_info = options.debug_mode.then(|| Box::new(DebugInfo::Heuristic));
        Ok(FeaturesOr::Ruled(RegularFileType { content_types, debug_info }))
    }
}

fn extract_features(config: &ModelConfig, file: &mut CachedInput<impl Input>) -> Result<Vec<i32>> {
    debug_assert!(config.beg_size < config.block_size);
    debug_assert!(config.end_size < config.block_size);
    let prefix_len = file.ensure_prefix(config.block_size)?;
    let suffix_len = file.ensure_suffix(config.block_size)?;
    debug_assert_eq!(prefix_len, suffix_len);
    let beg = strip_prefix(file.prefix(prefix_len));
    let end = strip_suffix(file.suffix(suffix_len));
    let mut features = vec![config.padding_token; config.features_size()];
    let split_features = config.split_features(&mut features);
    copy_features(split_features.beg, beg, 0);
    copy_features(split_features.end, end, 1);
    Ok(features)
}

fn copy_features(dst: &mut [i32], src: &[u8], align: usize) {
    let len = std::cmp::min(dst.len(), src.len());
    let dst_len = dst.len(); // borrowing issue: cannot inline below
    let dst = &mut dst[(dst_len - len) * align..][..len];
    let src = &src[(src.len() - len) * align..][..len];
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
        struct Args {
            beg_size: usize,
            mid_size: usize,
            end_size: usize,
            block_size: usize,
            padding_token: i32,
            use_inputs_at_offsets: bool,
        }
        #[derive(Debug, Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Metadata {
            #[allow(dead_code)] // debugging only
            core_content_size: usize,
            #[allow(dead_code)] // debugging only
            left_ws_num: usize,
            #[allow(dead_code)] // debugging only
            right_ws_num: usize,
        }
        #[derive(Debug, Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Features {
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
            args: Args,
            #[allow(dead_code)] // debugging only
            metadata: Metadata,
            content_base64: String,
            features: Features,
        }
        const PATH: &str = "../../tests_data/reference/features_extraction_examples.json.gz";
        let mut tests = String::new();
        GzDecoder::new(File::open(PATH).unwrap()).read_to_string(&mut tests).unwrap();
        let tests: Vec<Test> = serde_json::from_str(&tests).unwrap();
        for test in tests {
            assert_eq!(test.args.mid_size, 0, "unsupported mid_size");
            assert!(!test.args.use_inputs_at_offsets, "unsupported use_inputs_at_offsets");
            assert!(test.features.mid.is_empty(), "unsupported mid");
            assert!(test.features.offset_0x8000_0x8007.is_empty(), "unsupported offset");
            assert!(test.features.offset_0x8800_0x8807.is_empty(), "unsupported offset");
            assert!(test.features.offset_0x9000_0x9007.is_empty(), "unsupported offset");
            assert!(test.features.offset_0x9800_0x9807.is_empty(), "unsupported offset");
            let config = ModelConfig {
                beg_size: test.args.beg_size,
                end_size: test.args.end_size,
                padding_token: test.args.padding_token,
                block_size: test.args.block_size,
                ..crate::model::CONFIG
            };
            let mut expected = Vec::new();
            expected.extend_from_slice(&test.features.beg);
            expected.extend_from_slice(&test.features.end);
            let content = BASE64.decode(test.content_base64.as_bytes()).unwrap();
            let mut file = CachedInput::new(&content[..]).unwrap();
            let actual = extract_features(&config, &mut file);
            let actual: Vec<_> = actual.unwrap().into_iter().map(|x| x as usize).collect();
            assert_eq!(actual, expected, "{test:?}");
        }
    }
}
