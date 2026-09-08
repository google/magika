// Copyright 2026 Google LLC
// SPDX-License-Identifier: Apache-2.0

//! Unix-only experiment. The original serialized loader remains the default.

use anyhow::{ensure, Result};
use std::fs::File;
use std::os::fd::AsRawFd;

pub(super) const MAGIC: &[u8; 9] = b"MAGIKAMM\x03";
pub(super) const ALIGNMENT: usize = 4096;

pub(super) fn enabled() -> bool {
    std::env::var_os("MAGIKA_RULES_MMAP_SPIKE").is_some_and(|x| x == "1")
}

pub(super) struct Mapping {
    pointer: *mut libc::c_void,
    length: usize,
}
impl Mapping {
    pub(super) fn new(file: &File) -> Result<Self> {
        let length = usize::try_from(file.metadata()?.len())?;
        ensure!(
            length > 0 && length <= super::cache::MAX_DATABASE_SIZE + 8 * 1024 * 1024,
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
    use crate::rules::{native::Api, RuleSet};

    #[test]
    #[ignore = "requires a native Vectorscan compiler library"]
    fn mmap_spike_image_is_relocatable_readonly_and_lives_with_worker() {
        let source = r#"rule fixture {
            meta: label = "png" enabled = true class = "full" fp_rate = 0 fn_rate = 0
            strings: $a = "MAGIKA_MMAP_TEST!" condition: $a at 0
        }"#;
        let rules = RuleSet::from_source(source).unwrap();
        let outputs = crate::rules::compiler::compile(source).unwrap().outputs;
        let original = rules.database.as_ref().unwrap();
        let bytes = original.image().unwrap();
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("image");
        std::fs::write(&path, &bytes).unwrap();
        let first = Mapping::new(&File::open(&path).unwrap()).unwrap();
        let second = Mapping::new(&File::open(&path).unwrap()).unwrap();
        assert_ne!(first.bytes().as_ptr(), second.bytes().as_ptr());
        let api = Api::load().unwrap();
        let first = api.map_image(first, 0, outputs.clone()).unwrap();
        let second = api.map_image(second, 0, outputs).unwrap();
        let mut worker = first.worker().unwrap();
        drop(first);
        std::fs::remove_file(path).unwrap();
        for input in [b"MAGIKA_MMAP_TEST!".as_slice(), b"unrecognized content".as_slice()] {
            let expected = original.worker().unwrap().scan(input, input.len() as u64);
            assert_eq!(worker.scan(input, input.len() as u64), expected);
            assert_eq!(second.worker().unwrap().scan(input, input.len() as u64), expected);
        }
    }

    #[test]
    #[ignore = "requires a native Vectorscan compiler library"]
    fn mmap_spike_bounds_and_versions_are_checked_before_native_access() {
        let api = Api::load().unwrap();
        for (field, value) in [(0, 0), (4, 0), (8, u32::MAX), (36, u32::MAX)] {
            let mut bytes = vec![0; 104];
            bytes[..4].copy_from_slice(&0xdbdbdbdb_u32.to_ne_bytes());
            bytes[36..40].copy_from_slice(&64_u32.to_ne_bytes());
            bytes[field..field + 4].copy_from_slice(&value.to_ne_bytes());
            let mut file = tempfile::tempfile().unwrap();
            std::io::Write::write_all(&mut file, &bytes).unwrap();
            let mapped = Mapping::new(&file).unwrap();
            assert!(api.map_image(mapped, 0, vec![Some(crate::ContentType::Png)]).is_err());
        }
        let file = tempfile::tempfile().unwrap();
        assert!(Mapping::new(&file).is_err());
    }
}
