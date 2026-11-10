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
use std::collections::HashMap;
use std::path::Path;

use anyhow::Result;

use crate::config::ModelConfig;
use crate::{ContentType, DebugInfo, FileType, Options, RegularFileType, Rules, RulesDebugInfo};

/// Features to identify a file using AI.
pub struct Features {
    pub(crate) features: Vec<i32>,
    pub(crate) use_resolvers: bool,
    pub(crate) use_custom_rules: Option<Rules>,
}

/// Files up to this size are read in a single call instead of splitting prefix and suffix reads.
const SMALL_FILE_MAX: usize = 64 * 1024;

/// File content to be identified.
pub struct Input<'a> {
    content: Content<'a>,
    path: Option<Cow<'a, Path>>,
}

enum Content<'a> {
    Partial { file: std::fs::File, length: u64, prefix: Box<[u8]>, suffix: Box<[u8]> },
    Full(Cow<'a, [u8]>),
}

const _: () = const {
    // We assume in the rest of the file, that u64 holds any usize.
    assert!(std::mem::size_of::<usize>() <= std::mem::size_of::<u64>());
};

impl<'a> Input<'a> {
    /// Creates an input from a byte slice.
    pub fn from_slice(content: &'a [u8]) -> Self {
        Self { content: Content::Full(Cow::Borrowed(content)), path: None }
    }

    pub(crate) fn with_path(mut self, path: impl Into<Cow<'a, Path>>) -> Self {
        self.path = Some(path.into());
        self
    }

    // TODO: Uncomment to use by the resolvers.
    // pub(crate) fn path(&self) -> Option<&Path> {
    //     self.path.as_deref()
    // }
}

impl Input<'static> {
    /// Creates an input from a boxed byte slice.
    pub fn from_box(content: Box<[u8]>) -> Self {
        Self::from_vec(content.into_vec())
    }

    /// Creates an input from a byte vector.
    pub fn from_vec(content: Vec<u8>) -> Self {
        Self { content: Content::Full(Cow::Owned(content)), path: None }
    }

    /// Creates an input from a file.
    pub fn from_file(file: std::fs::File) -> Result<Self> {
        let length = file.metadata()?.len();
        Ok(Self::from_file_with_length(file, length))
    }

    pub(crate) fn from_file_with_length(file: std::fs::File, length: u64) -> Self {
        let content = Content::Partial { file, length, prefix: Box::new([]), suffix: Box::new([]) };
        Self { content, path: None }
    }
}

impl<'a> From<&'a [u8]> for Input<'a> {
    fn from(value: &'a [u8]) -> Self {
        Self::from_slice(value)
    }
}

impl From<Box<[u8]>> for Input<'static> {
    fn from(value: Box<[u8]>) -> Self {
        Self::from_box(value)
    }
}

impl From<Vec<u8>> for Input<'static> {
    fn from(value: Vec<u8>) -> Self {
        Self::from_vec(value)
    }
}

impl TryFrom<std::fs::File> for Input<'static> {
    type Error = anyhow::Error;

    fn try_from(value: std::fs::File) -> Result<Self> {
        Self::from_file(value)
    }
}

impl Input<'_> {
    pub(crate) fn length(&self) -> u64 {
        match &self.content {
            Content::Partial { length, .. } => *length,
            Content::Full(content) => content.len() as u64,
        }
    }

    /// Makes sure the prefix of the file of the given length is cached.
    ///
    /// Returns the length of the cached prefix.
    pub(crate) fn ensure_prefix(&mut self, len: usize) -> Result<usize> {
        let len = (len as u64).min(self.length()) as usize;
        let Content::Partial { file, length, prefix, suffix } = &mut self.content else {
            return Ok(len);
        };
        if len <= prefix.len() {
            return Ok(len);
        }
        if *length <= SMALL_FILE_MAX.max(len.saturating_add(suffix.len())) as u64 {
            self.make_full()?;
            return Ok(len);
        }
        let mut new_prefix = Vec::with_capacity(len);
        new_prefix.extend_from_slice(prefix);
        new_prefix.resize(len, 0);
        read_at(file, &mut new_prefix[prefix.len()..], prefix.len() as u64)?;
        *prefix = new_prefix.into_boxed_slice();
        Ok(len)
    }

    /// Returns the prefix of the file of the given length.
    ///
    /// The prefix must have been cached for that length.
    pub(crate) fn prefix(&self, len: usize) -> &[u8] {
        let content: &[u8] = match &self.content {
            Content::Partial { prefix, .. } => prefix,
            Content::Full(content) => content,
        };
        &content[..len]
    }

    /// Makes sure the suffix of the file of the given length is cached.
    ///
    /// Returns the length of the cached suffix.
    pub(crate) fn ensure_suffix(&mut self, len: usize) -> Result<usize> {
        let len = (len as u64).min(self.length()) as usize;
        let Content::Partial { file, length, prefix, suffix } = &mut self.content else {
            return Ok(len);
        };
        if len <= suffix.len() {
            return Ok(len);
        }
        if *length <= SMALL_FILE_MAX.max(prefix.len().saturating_add(len)) as u64 {
            self.make_full()?;
            return Ok(len);
        }
        let read_len = len - suffix.len();
        let mut new_suffix = Vec::with_capacity(len);
        new_suffix.resize(read_len, 0);
        read_at(file, &mut new_suffix, *length - len as u64)?;
        new_suffix.extend_from_slice(suffix);
        *suffix = new_suffix.into_boxed_slice();
        Ok(len)
    }

    /// Returns the suffix of the file of the given length.
    ///
    /// The suffix must have been cached for that length.
    pub(crate) fn suffix(&self, len: usize) -> &[u8] {
        let content: &[u8] = match &self.content {
            Content::Partial { suffix, .. } => suffix,
            Content::Full(content) => content,
        };
        &content[content.len() - len..]
    }

    fn make_full(&mut self) -> Result<()> {
        let Content::Partial { file, length, prefix, suffix } = &mut self.content else {
            unreachable!()
        };
        let length = *length as usize;
        let mut content = Vec::with_capacity(length);
        content.extend_from_slice(prefix);
        content.resize(length - suffix.len(), 0);
        read_at(file, &mut content[prefix.len()..], prefix.len() as u64)?;
        content.extend_from_slice(suffix);
        self.content = Content::Full(Cow::Owned(content));
        Ok(())
    }
}

fn read_at(file: &mut std::fs::File, buffer: &mut [u8], offset: u64) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::FileExt;
        Ok(file.read_exact_at(buffer, offset)?)
    }
    #[cfg(windows)]
    {
        use std::io::Read;
        use std::os::windows::fs::FileExt;
        let n = file.seek_read(buffer, offset)?;
        Ok(file.read_exact(&mut buffer[n..])?)
    }
    #[cfg(not(any(unix, windows)))]
    {
        use std::io::{Read, Seek, SeekFrom};
        file.seek(SeekFrom::Start(offset))?;
        Ok(file.read_exact(buffer)?)
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
            let file = Input::from_file_with_length(std::fs::File::open(file)?, metadata.len())
                .with_path(file);
            Ok(FeaturesOr::extract_content(file, options)?.into())
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
    pub fn extract_content<'a>(file: impl Into<Input<'a>>, options: &Options) -> Result<Self> {
        let config = &crate::model::CONFIG;
        let mut file = file.into();
        if file.length() == 0 {
            let content_types = vec![ContentType::Empty];
            let metadata = HashMap::new();
            let debug_info = options.debug_mode.then(|| Box::new(DebugInfo::Heuristic));
            return Ok(FeaturesOr::Ruled(RegularFileType { content_types, metadata, debug_info }));
        }
        if let Some(custom_rules) = &options.use_custom_rules {
            let content_types = custom_rules.identify(&mut file)?;
            if !content_types.is_empty() {
                let metadata = HashMap::new();
                let debug_info =
                    options.debug_mode.then(|| Box::new(DebugInfo::CustomRules(RulesDebugInfo {})));
                return Ok(FeaturesOr::Ruled(RegularFileType {
                    content_types,
                    metadata,
                    debug_info,
                }));
            }
        }
        if options.use_resolvers {
            let content_types = crate::resolvers::identify(&mut file)?;
            if !content_types.is_empty() {
                let metadata = HashMap::new();
                let debug_info = options.debug_mode.then(|| Box::new(DebugInfo::Resolvers));
                return Ok(FeaturesOr::Ruled(RegularFileType {
                    content_types,
                    metadata,
                    debug_info,
                }));
            }
        }
        let prefix_len = file.ensure_prefix(config.block_size)?;
        if options.use_model
            && strip_prefix(file.prefix(prefix_len)).len() >= config.min_file_size_for_dl
        {
            let features = extract_features(config, &mut file)?;
            return Ok(FeaturesOr::Features(Features {
                features,
                use_resolvers: options.use_resolvers,
                use_custom_rules: options.use_custom_rules.clone(),
            }));
        }
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
        let metadata = HashMap::new();
        let debug_info = options.debug_mode.then(|| Box::new(DebugInfo::Heuristic));
        Ok(FeaturesOr::Ruled(RegularFileType { content_types, metadata, debug_info }))
    }
}

fn extract_features(config: &ModelConfig, file: &mut Input) -> Result<Vec<i32>> {
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
            let mut file = Input::from_slice(&content);
            let actual = extract_features(&config, &mut file);
            let actual: Vec<_> = actual.unwrap().into_iter().map(|x| x as usize).collect();
            assert_eq!(actual, expected, "{test:?}");
        }
    }
}
