// Copyright 2026 Google LLC
// SPDX-License-Identifier: Apache-2.0

//! Read-only native rule images on Unix; serialized packs remain available for diagnostics.

use std::fs::File;
use std::os::fd::AsRawFd;

use anyhow::{ensure, Result};

pub(super) const MAGIC: &[u8; 9] = b"MAGIKAMM\x04";
pub(super) const ALIGNMENT: usize = 4096;

pub(super) fn enabled() -> bool {
    std::env::var_os("MAGIKA_RULES_MMAP").is_none_or(|x| x != "0")
}

pub(super) struct Mapping {
    pointer: *mut libc::c_void,
    length: usize,
}
impl Mapping {
    pub(super) fn new(file: &File) -> Result<Self> {
        let length = usize::try_from(file.metadata()?.len())?;
        ensure!(
            length > 0 && length <= super::cache::MAX_PAYLOAD_SIZE + 8 * 1024 * 1024,
            "invalid mapped pack size"
        );
        let pointer = unsafe {
            libc::mmap(
                std::ptr::null_mut(),
                length,
                libc::PROT_READ,
                libc::MAP_PRIVATE,
                file.as_raw_fd(),
                0,
            )
        };
        ensure!(
            pointer != libc::MAP_FAILED,
            "map rules image: {}",
            std::io::Error::last_os_error()
        );
        Ok(Self { pointer, length })
    }
    pub(super) fn bytes(&self) -> &[u8] {
        // The producer atomically replaces files; it never truncates mapped inodes.
        unsafe { std::slice::from_raw_parts(self.pointer.cast(), self.length) }
    }
}
impl Drop for Mapping {
    fn drop(&mut self) {
        unsafe {
            libc::munmap(self.pointer, self.length);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::compiler::Streams;
    use crate::rules::native::Api;
    use crate::rules::RuleSet;

    #[test]
    #[ignore = "requires a native Vectorscan compiler library"]
    fn mapped_image_is_relocatable_readonly_and_lives_with_worker() {
        let source = r#"rule fixture {
            meta: label = "png" enabled = true class = "full" fp_rate = 0 fn_rate = 0
            strings: $a = "MAGIKA_MMAP_TEST!" condition: $a at 0
        }"#;
        let rules = RuleSet::from_source(source).unwrap();
        let program = crate::rules::compiler::compile(source).unwrap();
        let original = rules.database.as_ref().unwrap();
        let images = original.image().unwrap();
        assert!(images.facts.is_empty());
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("image");
        std::fs::write(&path, &images.prefix).unwrap();
        let first = Mapping::new(&File::open(&path).unwrap()).unwrap();
        let second = Mapping::new(&File::open(&path).unwrap()).unwrap();
        assert_ne!(first.bytes().as_ptr(), second.bytes().as_ptr());
        let api = Api::load().unwrap();
        let streams = || Streams {
            prefix: (0..images.prefix.len(), program.prefix.outputs.clone()),
            facts: (0..0, Vec::new()),
        };
        let first = api.map_image(first, streams()).unwrap();
        let second = api.map_image(second, streams()).unwrap();
        let mut worker = first.worker().unwrap();
        drop(first);
        std::fs::remove_file(path).unwrap();
        for input in [b"MAGIKA_MMAP_TEST!".as_slice(), b"unrecognized content".as_slice()] {
            let size = input.len() as u64;
            let synthetic = crate::rules::preprocess::prepare(&crate::rules::preprocess::Blocks {
                prefix: input,
                size,
                tail: None,
            });
            let expected = original.worker().unwrap().scan(Some(&synthetic), input, size);
            assert_eq!(worker.scan(Some(&synthetic), input, size), expected);
            assert_eq!(second.worker().unwrap().scan(Some(&synthetic), input, size), expected);
        }
        // Mapped images are borrowed from their mapping and must never reach the native free.
        let freed = crate::rules::native::FREED.get();
        drop(worker);
        drop(second);
        assert_eq!(crate::rules::native::FREED.get(), freed, "a mapped image was freed");
        drop(rules);
        assert_eq!(crate::rules::native::FREED.get(), freed + 1, "the compiled original leaked");
    }

    #[test]
    #[ignore = "requires a native Vectorscan compiler library"]
    fn mapped_bounds_and_versions_are_checked_before_native_access() {
        let api = Api::load().unwrap();
        for (field, value) in [(0, 0), (4, 0), (8, u32::MAX), (36, u32::MAX)] {
            let mut bytes = vec![0; 104];
            bytes[..4].copy_from_slice(&0xdbdbdbdb_u32.to_ne_bytes());
            bytes[36..40].copy_from_slice(&64_u32.to_ne_bytes());
            bytes[field..field + 4].copy_from_slice(&value.to_ne_bytes());
            let mut file = tempfile::tempfile().unwrap();
            std::io::Write::write_all(&mut file, &bytes).unwrap();
            let mapped = Mapping::new(&file).unwrap();
            let streams = Streams {
                prefix: (0..bytes.len(), vec![Some(crate::ContentType::Png)]),
                facts: (0..0, Vec::new()),
            };
            assert!(api.map_image(mapped, streams).is_err());
        }
        let file = tempfile::tempfile().unwrap();
        assert!(Mapping::new(&file).is_err());
    }
}
