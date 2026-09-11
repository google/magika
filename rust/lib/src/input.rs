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

use anyhow::Result;

use crate::config::ModelConfig;
use crate::{ContentType, RulesMode};

/// Features to identify a file using AI.
pub struct Features(pub(crate) Vec<i32>);

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
pub enum FeaturesOrRuled {
    /// Features extracted for identification using AI.
    Features(Features),

    /// Content identified with rules.
    Ruled(ContentType),
}

impl FeaturesOrRuled {
    /// Extracts the features from a file.
    ///
    /// Returns the content type directly if the file cannot be identified using AI.
    pub fn extract(file: impl Input) -> Result<Self> {
        Self::extract_with_rules(file, RulesMode::Off)
    }

    /// Extracts features or applies the promoted rule allowlist to original bytes.
    ///
    /// Abstention reuses the prefix and preserves ordinary feature extraction.
    pub fn extract_with_rules(file: impl Input, mode: RulesMode) -> Result<Self> {
        mode.check()?;
        Self::extract_with_matcher(file, crate::rules::uses_facts(mode), |prefix, size, tail| {
            crate::rules::identify(prefix, size, tail, mode)
        })
    }

    /// Applies a loaded custom ruleset to original bytes, then extracts ML features on abstention.
    #[cfg(feature = "yara-rules")]
    pub fn extract_with_ruleset(file: impl Input, rules: &crate::RuleSet) -> Result<Self> {
        Self::extract_with_matcher(file, rules.uses_facts(), |prefix, size, tail| {
            rules.identify(prefix, size, tail)
        })
    }

    /// The matcher receives the bounded prefix, the original size and, when `wants_tail`
    /// and the prefix opens an archive, the trailing window of the input read ahead of
    /// matching. A matcher that cannot use facts must not want it: the window costs the
    /// default pipeline a read it never needed.
    pub(crate) fn extract_with_matcher(
        file: impl Input, wants_tail: bool,
        identify: impl FnOnce(&[u8], u64, Option<&[u8]>) -> Option<ContentType>,
    ) -> Result<Self> {
        Self::extract_with_config(&crate::model::CONFIG, file, wants_tail, identify)
    }

    fn extract_with_config(
        config: &ModelConfig, mut file: impl Input, wants_tail: bool,
        identify: impl FnOnce(&[u8], u64, Option<&[u8]>) -> Option<ContentType>,
    ) -> Result<Self> {
        let file_len = file.length()?;
        if file_len == 0 {
            return Ok(FeaturesOrRuled::Ruled(ContentType::Empty));
        }
        let read_size = config.block_size.max(crate::rules::PREFIX_LIMIT);
        let mut first_block = vec![0; file_len.min(read_size as u64) as usize];
        file.read_at(&mut first_block, 0)?;
        let prefix = &first_block[..first_block.len().min(crate::rules::PREFIX_LIMIT)];
        // Only archives, and only for a matcher that scans facts, read ahead: their central
        // directory sits at the end. The window is read once and also serves the model's
        // end block, so no input reads its tail twice. Without the rules feature no matcher
        // scans facts, and every input keeps the model's own reads.
        let tail: Option<Vec<u8>> = match wants_tail {
            #[cfg(feature = "yara-rules")]
            true => crate::rules::preprocess::read_tail(&mut file, file_len, prefix)?,
            _ => None,
        };
        if let Some(content_type) = identify(prefix, file_len, tail.as_deref()) {
            return Ok(FeaturesOrRuled::Ruled(content_type));
        }
        // Rule lookahead must not change the model's whitespace trimming or tail window.
        first_block.truncate(config.block_size);
        let (first_block, features) =
            extract_features_with_prefix(config, file, file_len, first_block, tail)?;
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

#[cfg(test)]
fn extract_features(
    config: &ModelConfig, mut file: impl Input, file_len: u64,
) -> Result<(Vec<u8>, Vec<i32>)> {
    debug_assert!(config.beg_size < config.block_size);
    debug_assert!(config.end_size < config.block_size);
    let buffer_size = std::cmp::min(config.block_size as u64, file_len) as usize;
    let mut content_beg = vec![0; buffer_size];
    file.read_at(&mut content_beg, 0)?;
    extract_features_with_prefix(config, file, file_len, content_beg, None)
}

/// Extracts the features from the first block and the last `block_size` bytes, which come
/// from `held_tail` (a window ending at the input's end) when it is long enough, and are
/// read otherwise.
fn extract_features_with_prefix(
    config: &ModelConfig, mut file: impl Input, file_len: u64, content_beg: Vec<u8>,
    held_tail: Option<Vec<u8>>,
) -> Result<(Vec<u8>, Vec<i32>)> {
    let buffer_size = content_beg.len();
    let beg = strip_prefix(&content_beg);
    let mut tail = Vec::new();
    let end = if file_len == buffer_size as u64 {
        // The prefix already holds the complete file, including its tail.
        content_beg.as_slice()
    } else if let Some(held) = held_tail.as_deref().filter(|held| held.len() >= buffer_size) {
        &held[held.len() - buffer_size..]
    } else {
        tail.resize(buffer_size, 0);
        file.read_at(&mut tail, file_len - buffer_size as u64)?;
        tail.as_slice()
    };
    let end = strip_suffix(end);
    let mut features = vec![config.padding_token; config.features_size()];
    let split_features = config.split_features(&mut features);
    copy_features(split_features.beg, beg, 0);
    copy_features(split_features.end, end, 1);
    Ok((content_beg, features))
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

    struct CountingInput {
        bytes: Vec<u8>,
        reads: Vec<(u64, usize)>,
    }
    impl Input for CountingInput {
        fn length(&self) -> Result<u64> {
            Ok(self.bytes.len() as u64)
        }
        fn read_at(&mut self, buffer: &mut [u8], offset: u64) -> Result<()> {
            self.reads.push((offset, buffer.len()));
            buffer.copy_from_slice(&self.bytes[offset as usize..offset as usize + buffer.len()]);
            Ok(())
        }
    }

    #[test]
    fn a_complete_prefix_is_not_read_again_for_the_tail() {
        let block_size = crate::model::CONFIG.block_size;
        for len in [1, 7, 8, 9, 1023, 1024, 2048, block_size - 1, block_size, block_size + 1] {
            let mut input = CountingInput {
                bytes: (0..len).map(|i| (i % 256) as u8).collect(),
                reads: Vec::new(),
            };
            FeaturesOrRuled::extract_with_matcher(&mut input, false, |_, _, _| None).unwrap();
            let expected = if len <= block_size {
                vec![(0, len)]
            } else {
                vec![(0, block_size), (1, block_size)]
            };
            assert_eq!(input.reads, expected, "input length {len}");
        }
    }

    #[test]
    fn rule_prefix_is_independent_of_model_block_size() {
        for block_size in [2048, 4096, 8192] {
            let config = ModelConfig { block_size, ..crate::model::CONFIG };
            // Whitespace spanning the smaller block boundary makes accidental feature
            // extraction with the larger rule prefix observably wrong.
            let mut bytes = vec![b' '; 2000];
            bytes.extend((0..10000).map(|i| (i % 256) as u8));
            let expected =
                extract_features(&config, bytes.as_slice(), bytes.len() as u64).unwrap().1;
            let result = FeaturesOrRuled::extract_with_config(
                &config,
                bytes.as_slice(),
                false,
                |prefix, _, _| {
                    assert_eq!(prefix.len(), 4096, "model block size {block_size}");
                    assert!(prefix == &bytes[..4096]);
                    None
                },
            )
            .unwrap();
            let FeaturesOrRuled::Features(features) = result else { panic!("expected features") };
            assert_eq!(features.0, expected, "model block size {block_size}");
        }
    }

    #[test]
    fn rule_hit_skips_tail_and_miss_preserves_reference_features() {
        let mut input = CountingInput {
            bytes: (0..10000).map(|i| (i % 256) as u8).collect(),
            reads: Vec::new(),
        };
        let expected =
            extract_features(&crate::model::CONFIG, input.bytes.as_slice(), 10000).unwrap().1;
        let hit = FeaturesOrRuled::extract_with_matcher(&mut input, false, |prefix, size, _| {
            assert_eq!(size, 10000);
            assert_eq!(prefix.len(), 4096);
            assert_eq!(&prefix[..3], &[0, 1, 2]);
            Some(ContentType::Png)
        })
        .unwrap();
        assert!(matches!(hit, FeaturesOrRuled::Ruled(ContentType::Png)));
        assert_eq!(input.reads, [(0, 4096)]);
        input.reads.clear();
        let FeaturesOrRuled::Features(actual) =
            FeaturesOrRuled::extract_with_matcher(&mut input, false, |_, _, _| None).unwrap()
        else {
            panic!("expected features")
        };
        assert_eq!(actual.0, expected);
        assert_eq!(input.reads, [(0, 4096), (5904, 4096)]);
    }

    /// An input of `len` bytes whose first four are `head`, the rest a byte ramp.
    fn counting(head: &[u8], len: usize) -> CountingInput {
        let mut bytes: Vec<u8> = (0..len).map(|i| (i % 256) as u8).collect();
        bytes[..head.len()].copy_from_slice(head);
        CountingInput { bytes, reads: Vec::new() }
    }

    #[test]
    fn zip_inputs_keep_the_prefix_only_reads_unless_the_matcher_uses_facts() {
        let mut input = counting(b"PK\x03\x04", 100_000);
        let expected =
            extract_features(&crate::model::CONFIG, input.bytes.as_slice(), 100_000).unwrap().1;
        // The default pipeline, rules off: the model reads its own end block on abstention.
        let FeaturesOrRuled::Features(actual) = FeaturesOrRuled::extract(&mut input).unwrap()
        else {
            panic!("expected features")
        };
        assert_eq!(actual.0, expected);
        assert_eq!(input.reads, [(0, 4096), (95_904, 4096)]);
        input.reads.clear();
        // A matcher without facts rules: a hit costs the prefix read alone, a miss the same
        // two reads as any other input, and the matcher never sees a tail.
        let hit = FeaturesOrRuled::extract_with_matcher(&mut input, false, |prefix, size, tail| {
            assert_eq!((prefix.len(), size, tail), (4096, 100_000, None));
            Some(ContentType::Zip)
        })
        .unwrap();
        assert!(matches!(hit, FeaturesOrRuled::Ruled(ContentType::Zip)));
        assert_eq!(input.reads, [(0, 4096)]);
        input.reads.clear();
        let FeaturesOrRuled::Features(actual) =
            FeaturesOrRuled::extract_with_matcher(&mut input, false, |_, _, tail| {
                assert_eq!(tail, None);
                None
            })
            .unwrap()
        else {
            panic!("expected features")
        };
        assert_eq!(actual.0, expected);
        assert_eq!(input.reads, [(0, 4096), (95_904, 4096)]);
    }

    #[cfg(feature = "yara-rules")]
    #[test]
    fn zip_inputs_read_one_tail_window_for_a_facts_matcher_reused_for_the_features() {
        let mut input = counting(b"PK\x03\x04", 100_000);
        let bytes = input.bytes.clone();
        let hit = FeaturesOrRuled::extract_with_matcher(&mut input, true, |prefix, size, tail| {
            assert_eq!((prefix.len(), size), (4096, 100_000));
            assert_eq!(tail, Some(&bytes[83_616..]));
            Some(ContentType::Zip)
        })
        .unwrap();
        assert!(matches!(hit, FeaturesOrRuled::Ruled(ContentType::Zip)));
        assert_eq!(input.reads, [(0, 4096), (83_616, 16_384)]);
        // On a miss the held window feeds the model's end block, whether it is a suffix of
        // the input or holds it whole: the features are the reference ones, without a read.
        for (len, tail_read) in [
            (100_000, (83_616, 16_384)),
            (16_385, (1, 16_384)),
            (16_384, (0, 16_384)),
            (10_000, (0, 10_000)),
            (4097, (0, 4097)),
        ] {
            let mut input = counting(b"PK\x03\x04", len);
            let bytes = input.bytes.clone();
            let expected =
                extract_features(&crate::model::CONFIG, bytes.as_slice(), len as u64).unwrap().1;
            let FeaturesOrRuled::Features(actual) =
                FeaturesOrRuled::extract_with_matcher(&mut input, true, |_, _, tail| {
                    assert_eq!(tail, Some(&bytes[tail_read.0..]), "{len}");
                    None
                })
                .unwrap()
            else {
                panic!("expected features for {len}")
            };
            assert_eq!(actual.0, expected, "{len}");
            assert_eq!(input.reads, [(0, 4096), (tail_read.0 as u64, tail_read.1)], "{len}");
        }
        // A tiny archive is held by the prefix alone.
        let mut input = counting(b"PK\x05\x06", 4_000);
        FeaturesOrRuled::extract_with_matcher(&mut input, true, |prefix, _, tail| {
            assert_eq!((prefix.len(), tail), (4_000, None));
            None
        })
        .unwrap();
        assert_eq!(input.reads, [(0, 4_000)]);
    }

    #[test]
    fn other_inputs_keep_the_prefix_only_read_sequence() {
        // Even a matcher that uses facts reads no tail ahead of matching a non-archive.
        let mut input = counting(b"\x89PNG", 100_000);
        let expected =
            extract_features(&crate::model::CONFIG, input.bytes.as_slice(), 100_000).unwrap().1;
        let hit = FeaturesOrRuled::extract_with_matcher(&mut input, true, |_, _, tail| {
            assert_eq!(tail, None);
            Some(ContentType::Png)
        })
        .unwrap();
        assert!(matches!(hit, FeaturesOrRuled::Ruled(ContentType::Png)));
        assert_eq!(input.reads, [(0, 4096)]);
        input.reads.clear();
        let FeaturesOrRuled::Features(actual) =
            FeaturesOrRuled::extract_with_matcher(&mut input, true, |_, _, tail| {
                assert_eq!(tail, None);
                None
            })
            .unwrap()
        else {
            panic!("expected features")
        };
        assert_eq!(actual.0, expected);
        assert_eq!(input.reads, [(0, 4096), (95_904, 4096)]);
    }

    #[test]
    fn original_whitespace_empty_tiny_and_io_error_semantics() {
        FeaturesOrRuled::extract_with_matcher(&b"  \0header"[..], false, |prefix, _, _| {
            assert_eq!(prefix, b"  \0header");
            None
        })
        .unwrap();
        assert!(matches!(
            FeaturesOrRuled::extract_with_matcher(&b""[..], true, |_, _, _| panic!(
                "empty must not scan"
            ))
            .unwrap(),
            FeaturesOrRuled::Ruled(ContentType::Empty)
        ));
        for (bytes, expected) in
            [(&b"abc"[..], ContentType::Txt), (&b"\xff"[..], ContentType::Unknown)]
        {
            assert!(
                matches!(FeaturesOrRuled::extract(bytes).unwrap(), FeaturesOrRuled::Ruled(ct) if ct == expected)
            );
        }
        struct Unreadable;
        impl Input for Unreadable {
            fn length(&self) -> Result<u64> {
                Ok(4096)
            }
            fn read_at(&mut self, _: &mut [u8], _: u64) -> Result<()> {
                Err(std::io::Error::new(std::io::ErrorKind::UnexpectedEof, "changed input").into())
            }
        }
        let error = FeaturesOrRuled::extract_with_matcher(Unreadable, true, |_, _, _| {
            panic!("must not scan unread bytes")
        })
        .err()
        .unwrap();
        assert_eq!(
            error.downcast_ref::<std::io::Error>().unwrap().kind(),
            std::io::ErrorKind::UnexpectedEof
        );
    }

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
            let actual = extract_features(&config, content.as_slice(), content.len() as u64);
            let actual: Vec<_> = actual.unwrap().1.into_iter().map(|x| x as usize).collect();
            assert_eq!(actual, expected, "{test:?}");
        }
    }
}
