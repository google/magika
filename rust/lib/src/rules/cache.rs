// Copyright 2026 Google LLC
// SPDX-License-Identifier: Apache-2.0

//! Optional, disposable native database cache. No cache access occurs during scans.

use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use anyhow::{ensure, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::native::Api;

const MAGIC: &[u8; 9] = b"MAGIKAHS\x02";
pub(super) const MAX_DATABASE_SIZE: usize = 64 * 1024 * 1024;
const MAX_MANIFEST_SIZE: usize = 4 * 1024 * 1024;

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    key: String,
    labels: Vec<Option<String>>,
    engine: String,
    checksum: String,
}

fn key(source: &str) -> String {
    let mut hash = Sha256::new();
    hash.update((super::PREFIX_LIMIT as u64).to_le_bytes());
    // Source and compiler identity are available without constructing an AST or program.
    // Parser/regex versions are pinned; bump this identity when changing those pins.
    for part in [
        source.as_bytes(),
        b"yara-x-parser=1.20.0;regex-syntax=0.8.11",
        include_bytes!("compiler.rs"),
        include_bytes!("metadata.rs"),
        include_bytes!("native.rs"),
        include_bytes!("cache.rs"),
        env!("CARGO_PKG_VERSION").as_bytes(),
        crate::MODEL_NAME.as_bytes(),
    ] {
        hash.update((part.len() as u64).to_le_bytes());
        hash.update(part);
    }
    format!("{:x}", hash.finalize())
}

// Cover the output mapping and engine identity as well as the native bytes. This detects
// corruption; packs/cache directories are trusted configuration, not authenticated input.
fn checksum(manifest: &Manifest, payload: &[u8]) -> Result<String> {
    let mut hash = Sha256::new();
    hash.update(serde_json::to_vec(&(&manifest.key, &manifest.engine, &manifest.labels))?);
    hash.update(payload);
    Ok(format!("{:x}", hash.finalize()))
}

pub(super) fn default_directory() -> Option<PathBuf> {
    if let Some(path) = std::env::var_os("MAGIKA_RULES_CACHE") {
        return (!path.is_empty()).then(|| PathBuf::from(path));
    }
    let base = if cfg!(target_os = "windows") {
        std::env::var_os("LOCALAPPDATA").map(PathBuf::from)
    } else if cfg!(target_os = "macos") {
        std::env::var_os("HOME").map(|x| PathBuf::from(x).join("Library/Caches"))
    } else {
        std::env::var_os("XDG_CACHE_HOME")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|x| PathBuf::from(x).join(".cache")))
    };
    base.map(|x| x.join("magika/rules"))
}

fn prepare_directory(path: &Path) -> Result<()> {
    let mut builder = std::fs::DirBuilder::new();
    builder.recursive(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        builder.mode(0o700);
    }
    builder.create(path)?;
    let metadata = std::fs::symlink_metadata(path)?;
    ensure!(metadata.is_dir(), "cache path must be a directory, not a symlink");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        ensure!(
            metadata.permissions().mode() & 0o022 == 0,
            "cache directory must not be writable by other users"
        );
        validate_ancestors(path)?;
    }
    Ok(())
}

#[cfg(unix)]
fn validate_ancestors(path: &Path) -> Result<()> {
    use std::os::unix::fs::MetadataExt;
    let uid = unsafe { libc::geteuid() };
    let path = std::fs::canonicalize(path)?;
    for ancestor in path.ancestors() {
        let metadata = std::fs::metadata(ancestor)?;
        ensure!(metadata.uid() == uid || metadata.uid() == 0, "untrusted cache directory owner");
        // Root/user-owned sticky temporary directories protect owned child entries.
        ensure!(
            metadata.mode() & 0o022 == 0 || metadata.mode() & 0o1000 != 0,
            "cache ancestor is replaceable by other users"
        );
    }
    Ok(())
}

fn validate_open_file(file: &File) -> Result<()> {
    let metadata = file.metadata()?;
    ensure!(metadata.is_file(), "cache entry must be a regular file");
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        let uid = unsafe { libc::geteuid() };
        ensure!(metadata.uid() == uid || metadata.uid() == 0, "untrusted cache file owner");
        ensure!(metadata.mode() & 0o022 == 0, "cache file is writable by other users");
    }
    Ok(())
}

fn open_cache_file(path: &Path, writable: bool) -> Result<File> {
    let mut options = OpenOptions::new();
    options.read(true).write(writable).create(writable).truncate(false);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        let parent = path.parent().filter(|p| !p.as_os_str().is_empty()).unwrap_or(Path::new("."));
        validate_ancestors(parent)?;
        // Validate the opened inode, reject leaf symlinks atomically, and never block on a FIFO.
        options.custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK).mode(0o600);
    }
    let file = options.open(path)?;
    validate_open_file(&file)?;
    Ok(file)
}

fn lock_with_deadline(file: &File) -> Result<()> {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(1);
    loop {
        match file.try_lock() {
            Ok(()) => return Ok(()),
            Err(std::fs::TryLockError::WouldBlock) => {
                ensure!(std::time::Instant::now() < deadline, "rules cache lock timed out");
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
            Err(std::fs::TryLockError::Error(error)) => return Err(error.into()),
        }
    }
}

fn read(path: &Path, expected_key: &str) -> Result<super::RuleSet> {
    let mut file = open_cache_file(path, false)?;
    let mut magic = [0; 9];
    file.read_exact(&mut magic)?;
    ensure!(&magic == MAGIC, "invalid rules cache version");
    let mut length = [0; 4];
    file.read_exact(&mut length)?;
    let length = u32::from_le_bytes(length) as usize;
    ensure!(length <= MAX_MANIFEST_SIZE, "invalid cache manifest size");
    let mut bytes = vec![0; length];
    file.read_exact(&mut bytes)?;
    let manifest: Manifest = serde_json::from_slice(&bytes)?;
    ensure!(manifest.key == expected_key, "cache source/compiler identity mismatch");
    ensure!(manifest.labels.len() <= 100_000, "invalid output mapping size");
    let mut payload = Vec::new();
    file.take(MAX_DATABASE_SIZE as u64 + 1).read_to_end(&mut payload)?;
    ensure!(payload.len() <= MAX_DATABASE_SIZE, "invalid native payload size");
    ensure!(checksum(&manifest, &payload)? == manifest.checksum, "cache checksum mismatch");
    let database = if payload.is_empty() {
        ensure!(manifest.labels.is_empty() && manifest.engine == "empty", "invalid empty pack");
        None
    } else {
        let outputs = manifest
            .labels
            .iter()
            .map(|label| {
                label
                    .as_deref()
                    .map(|label| {
                        crate::ContentType::from_label(label)
                            .ok_or_else(|| anyhow::anyhow!("invalid cached label: {label}"))
                    })
                    .transpose()
            })
            .collect::<Result<Vec<_>>>()?;
        ensure!(outputs.iter().any(Option::is_some), "native pack needs a terminal label");
        let api = Api::load()?;
        ensure!(manifest.engine == api.identity(), "cache engine/CPU identity mismatch");
        Some(api.deserialize(&payload, outputs)?)
    };
    Ok(super::RuleSet { database, loaded_from_cache: true })
}

fn write_payload(
    path: &Path, key: String, engine: String, labels: Vec<Option<String>>, payload: &[u8],
    replace: bool,
) -> Result<()> {
    let mut manifest = Manifest { key, engine, labels, checksum: String::new() };
    manifest.checksum = checksum(&manifest, payload)?;
    let manifest = serde_json::to_vec(&manifest)?;
    ensure!(manifest.len() <= MAX_MANIFEST_SIZE, "cache manifest exceeds limit");
    let parent =
        path.parent().filter(|x| !x.as_os_str().is_empty()).unwrap_or_else(|| Path::new("."));
    let mut temporary = tempfile::NamedTempFile::new_in(parent)?;
    temporary.write_all(MAGIC)?;
    temporary.write_all(&(manifest.len() as u32).to_le_bytes())?;
    temporary.write_all(&manifest)?;
    #[cfg(test)]
    if std::env::var_os("MAGIKA_TEST_ABORT_CACHE_WRITE").is_some() {
        std::process::exit(86);
    }
    temporary.write_all(payload)?;
    temporary.as_file().sync_all()?;
    if replace {
        temporary.persist(path)?;
    } else {
        temporary.persist_noclobber(path)?;
    }
    Ok(())
}

fn compile(source: &str, output: Option<(&Path, String, bool)>) -> Result<super::RuleSet> {
    let program = super::compiler::compile(source)?;
    let labels = program.outputs.iter().map(|x| x.map(|x| x.info().label.to_owned())).collect();
    let (database, engine) = if program.outputs.iter().all(Option::is_none) {
        (None, "empty".to_owned())
    } else {
        let api = Api::load()?;
        let engine = api.identity();
        (Some(api.compile(program)?), engine)
    };
    if let Some((path, key, replace)) = output {
        let write = || {
            let payload =
                database.as_ref().map(|db| db.serialize()).transpose()?.unwrap_or_default();
            write_payload(path, key, engine, labels, &payload, replace)
        };
        if replace {
            let _ = write();
        } else {
            write()?;
        }
    }
    Ok(super::RuleSet { database, loaded_from_cache: false })
}

pub(super) fn export(source: &str, path: &Path) -> Result<()> {
    ensure!(!path.exists(), "compiled pack already exists: {}", path.display());
    compile(source, Some((path, key(source), false)))?;
    Ok(())
}

pub(super) fn load(
    source: &str, directory: Option<&Path>, shipped: Option<&Path>,
) -> Result<super::RuleSet> {
    ensure!(source.len() <= 4 * 1024 * 1024, "YARA source exceeds 4 MiB");
    let key = key(source);
    if let Some(path) = shipped {
        if let Ok(rules) = read(path, &key) {
            return Ok(rules);
        }
    }
    let Some(directory) = directory.filter(|path| prepare_directory(path).is_ok()) else {
        return compile(source, None);
    };
    // One native target per source/compiler entry. A different engine or CPU is rejected by
    // the manifest and replaces this disposable cache entry; shipped packs remain untouched.
    let path = directory.join(format!("{key}.hsdb"));
    if let Ok(rules) = read(&path, &key) {
        return Ok(rules);
    }
    // Keep lock files in place: unlinking one could create two independently locked inodes.
    let lock = open_cache_file(&directory.join(format!("{key}.lock")), true).and_then(|file| {
        lock_with_deadline(&file)?;
        Ok(file)
    });
    let Ok(_lock) = lock else {
        return compile(source, None);
    };
    if let Ok(rules) = read(&path, &key) {
        return Ok(rules);
    }
    compile(source, Some((&path, key, true)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ContentType, RuleSet};

    const SOURCE: &str = r#"rule a { meta: label = "png" enabled = true class = "full" fp_rate = 0 fn_rate = 0
        strings: $a = "ABCD" condition: $a at 0 and original_size >= 4 }"#;

    #[test]
    fn cache_lock_contention_falls_back_without_waiting_forever() {
        const CHILD_DIRECTORY: &str = "MAGIKA_TEST_CONTENDED_CACHE";
        if let Some(directory) = std::env::var_os(CHILD_DIRECTORY) {
            let directory = PathBuf::from(directory);
            let rules = load("", Some(&directory), None).unwrap();
            assert!(!rules.loaded_from_cache());
            assert_eq!(rules.identify(b"ABCD", 4), None);
            std::fs::write(directory.join("completed"), b"compiled without cache").unwrap();
            return;
        }
        let temp = tempfile::tempdir().unwrap();
        let lock = open_cache_file(&temp.path().join(format!("{}.lock", key(""))), true).unwrap();
        lock.lock().unwrap();
        let mut child = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "rules::cache::tests::cache_lock_contention_falls_back_without_waiting_forever",
            ])
            .env(CHILD_DIRECTORY, temp.path())
            .stdout(std::process::Stdio::null())
            .spawn()
            .unwrap();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        loop {
            if let Some(status) = child.try_wait().unwrap() {
                assert!(status.success());
                break;
            }
            if std::time::Instant::now() >= deadline {
                let _ = child.kill();
                let _ = child.wait();
                panic!("contended cache lock blocked initialization");
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        assert_eq!(
            std::fs::read(temp.path().join("completed")).unwrap(),
            b"compiled without cache"
        );
        assert!(!temp.path().join(format!("{}.hsdb", key(""))).exists());
    }

    #[test]
    #[cfg(unix)]
    fn cache_read_rejects_symlinks_and_other_writable_files() {
        use std::os::unix::fs::{symlink, PermissionsExt};
        let temp = tempfile::tempdir().unwrap();
        let pack = temp.path().join("valid.hsdb");
        export("", &pack).unwrap();
        assert!(read(&pack, &key("")).is_ok());
        let link = temp.path().join("linked.hsdb");
        symlink(&pack, &link).unwrap();
        assert!(read(&link, &key("")).is_err(), "cache followed a symlink");
        std::fs::set_permissions(&pack, std::fs::Permissions::from_mode(0o666)).unwrap();
        assert!(read(&pack, &key("")).is_err(), "cache trusted an other-writable file");
    }

    #[test]
    #[cfg(unix)]
    fn cache_directory_rejects_replaceable_ancestor() {
        use std::os::unix::fs::PermissionsExt;
        let temp = tempfile::tempdir().unwrap();
        let parent = temp.path().join("shared");
        let leaf = parent.join("cache");
        prepare_directory(&leaf).unwrap();
        std::fs::set_permissions(&parent, std::fs::Permissions::from_mode(0o777)).unwrap();
        assert!(prepare_directory(&leaf).is_err(), "cache trusts an attacker-replaceable ancestor");
    }

    #[test]
    #[cfg(unix)]
    fn cache_locks_reject_symlinks_without_touching_the_target() {
        let temp = tempfile::tempdir().unwrap();
        let target = temp.path().join("keep");
        std::fs::write(&target, b"unchanged").unwrap();
        let link = temp.path().join("entry.lock");
        std::os::unix::fs::symlink(&target, &link).unwrap();
        assert!(open_cache_file(&link, true).is_err());
        assert_eq!(std::fs::read(&target).unwrap(), b"unchanged");
    }

    #[test]
    #[cfg(unix)]
    fn cache_special_files_never_block() {
        const CHILD_PATH: &str = "MAGIKA_TEST_CACHE_FIFO";
        if let Some(path) = std::env::var_os(CHILD_PATH) {
            let path = PathBuf::from(path);
            assert!(open_cache_file(&path, false).is_err());
            assert!(open_cache_file(&path, true).is_err());
            std::fs::write(path.with_extension("checked"), b"checked both open modes").unwrap();
            return;
        }
        let temp = tempfile::tempdir().unwrap();
        let fifo = temp.path().join("entry.fifo");
        assert!(std::process::Command::new("mkfifo").arg(&fifo).status().unwrap().success());
        let mut child = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "rules::cache::tests::cache_special_files_never_block"])
            .env(CHILD_PATH, &fifo)
            .stdout(std::process::Stdio::null())
            .spawn()
            .unwrap();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        loop {
            if let Some(status) = child.try_wait().unwrap() {
                assert!(status.success());
                break;
            }
            if std::time::Instant::now() >= deadline {
                let _ = child.kill();
                let _ = child.wait();
                panic!("cache opening blocked on a FIFO");
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        // A misspelled test filter must not turn a zero-test child into success.
        assert_eq!(
            std::fs::read(fifo.with_extension("checked")).unwrap(),
            b"checked both open modes"
        );
    }

    #[test]
    fn cache_identity_covers_source_and_manifest() {
        assert_ne!(key(SOURCE), key(&format!("{SOURCE}\n// edit")));
        let mut manifest = Manifest {
            key: key(SOURCE),
            engine: "engine-5:cpu-1".into(),
            labels: vec![None, Some("png".into())],
            checksum: String::new(),
        };
        let original = checksum(&manifest, b"native bytes").unwrap();
        manifest.engine = "engine-6:cpu-2".into();
        assert_ne!(checksum(&manifest, b"native bytes").unwrap(), original);
        manifest.engine = "engine-5:cpu-1".into();
        manifest.labels[1] = Some("gif".into());
        assert_ne!(checksum(&manifest, b"native bytes").unwrap(), original);
    }

    #[test]
    #[ignore = "requires a native Vectorscan compiler library"]
    fn cache_roundtrip_edit_corruption_and_unwritable_directory() {
        let temp = tempfile::tempdir().unwrap();
        let source = temp.path().join("rules.yar");
        let cache = temp.path().join("cache");
        std::fs::write(&source, SOURCE).unwrap();
        let load = || RuleSet::from_file_with_cache(&source, Some(&cache)).unwrap();
        let first = load();
        assert!(!first.loaded_from_cache());
        assert_eq!(first.identify(b"ABCD", 4), Some(ContentType::Png));
        let calls = super::super::compiler::COMPILATIONS.get();
        let second = load();
        assert!(second.loaded_from_cache());
        assert_eq!(
            super::super::compiler::COMPILATIONS.get(),
            calls,
            "cache hit must skip parsing/lowering"
        );
        assert_eq!(second.identify(b"ABCD", 4), Some(ContentType::Png));
        assert_eq!(second.identify(b"ABCE", 4), None);
        let pack = std::fs::read_dir(&cache)
            .unwrap()
            .map(|x| x.unwrap().path())
            .find(|x| x.extension().is_some_and(|x| x == "hsdb"))
            .unwrap();
        let valid = std::fs::read(&pack).unwrap();
        for damaged in [b"partial write".to_vec(), {
            let mut bytes = valid.clone();
            *bytes.last_mut().unwrap() ^= 1;
            bytes
        }] {
            std::fs::write(&pack, damaged).unwrap();
            let repaired = load();
            assert!(!repaired.loaded_from_cache());
            assert_eq!(repaired.identify(b"ABCD", 4), Some(ContentType::Png));
            assert!(load().loaded_from_cache());
        }
        // A valid outer checksum cannot make an invalid native database acceptable.
        let length = u32::from_le_bytes(valid[9..13].try_into().unwrap()) as usize;
        let mut manifest: Manifest = serde_json::from_slice(&valid[13..13 + length]).unwrap();
        let mut payload = valid[13 + length..].to_vec();
        payload[0] ^= 1; // Invalid native magic; deserializer must reject it before use.
        manifest.checksum = checksum(&manifest, &payload).unwrap();
        let header = serde_json::to_vec(&manifest).unwrap();
        let mut bytes = MAGIC.to_vec();
        bytes.extend_from_slice(&(header.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&header);
        bytes.extend_from_slice(&payload);
        std::fs::write(&pack, bytes).unwrap();
        let rebuilt = load();
        assert!(!rebuilt.loaded_from_cache());
        assert_eq!(rebuilt.identify(b"ABCD", 4), Some(ContentType::Png));
        assert!(load().loaded_from_cache());
        for damage in ["label", "engine", "unknown-label"] {
            let mut manifest: Manifest = serde_json::from_slice(&valid[13..13 + length]).unwrap();
            let payload = &valid[13 + length..];
            if damage == "engine" {
                manifest.engine = "different-engine-or-cpu".into();
                manifest.checksum = checksum(&manifest, payload).unwrap();
            } else {
                *manifest.labels.iter_mut().find(|x| x.is_some()).unwrap() =
                    Some(if damage == "label" { "gif" } else { "not-a-label" }.into());
                if damage == "unknown-label" {
                    manifest.checksum = checksum(&manifest, payload).unwrap();
                }
            }
            let header = serde_json::to_vec(&manifest).unwrap();
            let mut bytes = MAGIC.to_vec();
            bytes.extend_from_slice(&(header.len() as u32).to_le_bytes());
            bytes.extend_from_slice(&header);
            bytes.extend_from_slice(payload);
            std::fs::write(&pack, bytes).unwrap();
            let repaired = load();
            assert!(!repaired.loaded_from_cache(), "{damage}");
            assert_eq!(repaired.identify(b"ABCD", 4), Some(ContentType::Png));
        }
        std::fs::write(&source, SOURCE.replace("\"png\"", "\"gif\"")).unwrap();
        let edited = load();
        assert!(!edited.loaded_from_cache());
        assert_eq!(edited.identify(b"ABCD", 4), Some(ContentType::Gif));
        std::fs::write(&source, SOURCE.replace("enabled = true", "enabled = false")).unwrap();
        assert_eq!(load().identify(b"ABCD", 4), None);
        std::fs::write(&source, format!("include \"other.yar\"\n{SOURCE}")).unwrap();
        assert!(RuleSet::from_file_with_cache(&source, Some(&cache)).is_err());
        std::fs::write(&source, SOURCE).unwrap();
        // A regular file cannot be a cache directory, even when tests run as an administrator.
        let no_directory = temp.path().join("not-a-directory");
        std::fs::write(&no_directory, b"keep").unwrap();
        let uncached = RuleSet::from_file_with_cache(&source, Some(&no_directory)).unwrap();
        assert!(!uncached.loaded_from_cache());
        assert_eq!(uncached.identify(b"ABCD", 4), Some(ContentType::Png));
        assert_eq!(std::fs::read(&no_directory).unwrap(), b"keep");
    }

    #[test]
    #[ignore = "requires a native Vectorscan compiler library"]
    fn private_variant_enablement_is_cached_as_configuration() {
        let temp = tempfile::tempdir().unwrap();
        let source = temp.path().join("variants.yar");
        let cache = temp.path().join("cache");
        let text = r#"
            private rule variant { meta: enabled = false strings: $a = "ABCD" condition: $a at 0 }
            private rule common { strings: $a = "EFGH" condition: $a at 0 }
            rule format { meta: label = "png" enabled = true class = "full" fp_rate = 0 fn_rate = 0 condition: variant or common }
        "#;
        std::fs::write(&source, text).unwrap();
        let load = || RuleSet::from_file_with_cache(&source, Some(&cache)).unwrap();
        let disabled = load();
        assert_eq!(disabled.identify(b"ABCD", 4), None);
        assert_eq!(disabled.identify(b"EFGH", 4), Some(ContentType::Png));
        assert!(load().loaded_from_cache());
        std::fs::write(&source, text.replace("enabled = false", "enabled = true")).unwrap();
        let enabled = load();
        assert!(!enabled.loaded_from_cache());
        assert_eq!(enabled.identify(b"ABCD", 4), Some(ContentType::Png));
        let public = text
            .replace("private rule variant", "rule variant")
            .replace("meta: enabled = false", "meta: label = \"png\" enabled = false");
        assert_eq!(RuleSet::from_source(&public).unwrap().identify(b"ABCD", 4), None);
        assert_eq!(
            RuleSet::from_source(&public.replace(
                "enabled = false",
                "enabled = true class = \"full\" fp_rate = 0 fn_rate = 0"
            ))
            .unwrap()
            .identify(b"ABCD", 4),
            Some(ContentType::Png)
        );
        let unsupported = text.replace("condition: $a at 0 }", "condition: filesize > 0 }");
        // The first helper is disabled, but the ordinary second helper is still required.
        assert!(super::super::compiler::compile(&unsupported).is_err());
        let unsupported_variant =
            text.replacen("condition: $a at 0 }", "condition: filesize > 0 }", 1);
        assert!(super::super::compiler::compile(&unsupported_variant).is_ok());
        assert!(super::super::compiler::compile(
            &unsupported_variant.replace("enabled = false", "enabled = true")
        )
        .is_err());
    }

    #[test]
    fn empty_export_is_portable_and_never_clobbers() {
        let temp = tempfile::tempdir().unwrap();
        let source = temp.path().join("empty.yar");
        let pack = source.with_extension("hsdb");
        let disabled = SOURCE.replace("enabled = true", "enabled = false");
        std::fs::write(&source, &disabled).unwrap();
        RuleSet::compile_file(&source, &pack).unwrap();
        let bytes = std::fs::read(&pack).unwrap();
        let calls = super::super::compiler::COMPILATIONS.get();
        let loaded = RuleSet::from_file_with_cache(&source, None).unwrap();
        assert_eq!(super::super::compiler::COMPILATIONS.get(), calls);
        assert!(loaded.loaded_from_cache());
        assert_eq!(loaded.identify(b"ABCD", 4), None);
        assert!(RuleSet::compile_file(&source, &pack).is_err());
        assert_eq!(std::fs::read(&pack).unwrap(), bytes);
        assert!(RuleSet::compile_file(&source, &source).is_err());
        assert_eq!(std::fs::read_to_string(&source).unwrap(), disabled);
    }

    #[test]
    #[ignore = "requires a native Vectorscan compiler library"]
    fn shipped_pack_reuse_and_source_edit_fallback() {
        let temp = tempfile::tempdir().unwrap();
        let source = temp.path().join("rules.yar");
        let pack = source.with_extension("hsdb");
        let cache = temp.path().join("cache");
        std::fs::write(&source, SOURCE).unwrap();
        RuleSet::compile_file(&source, &pack).unwrap();
        let shipped = std::fs::read(&pack).unwrap();
        let calls = super::super::compiler::COMPILATIONS.get();
        let first = RuleSet::from_file_with_cache(&source, Some(&cache)).unwrap();
        assert!(first.loaded_from_cache());
        assert_eq!(
            super::super::compiler::COMPILATIONS.get(),
            calls,
            "paired pack must skip parsing/lowering"
        );
        assert!(!cache.exists(), "a compatible shipped pack must not need a writable cache");
        assert_eq!(first.identify(b"ABCD", 4), Some(ContentType::Png));
        assert_eq!(first.identify(b"ABCE", 4), None);
        std::fs::write(&source, SOURCE.replace("\"png\"", "\"gif\"")).unwrap();
        let edited = RuleSet::from_file_with_cache(&source, Some(&cache)).unwrap();
        assert!(!edited.loaded_from_cache());
        assert_eq!(edited.identify(b"ABCD", 4), Some(ContentType::Gif));
        assert!(RuleSet::from_file_with_cache(&source, Some(&cache)).unwrap().loaded_from_cache());
        assert_eq!(
            std::fs::read(&pack).unwrap(),
            shipped,
            "fallback must not modify shipped artifacts"
        );
        std::fs::write(&source, SOURCE).unwrap();
        std::fs::write(&pack, b"incompatible pack").unwrap();
        let rebuilt = RuleSet::from_file_with_cache(&source, Some(&cache)).unwrap();
        assert!(!rebuilt.loaded_from_cache());
        assert_eq!(rebuilt.identify(b"ABCD", 4), Some(ContentType::Png));
    }

    // Invoked in separate test processes below, so lock and cache behavior cannot be hidden by
    // shared in-process state. Running the full suite without probe variables does no work here.
    #[test]
    #[ignore = "subprocess helper for native cache tests"]
    fn process_probe() {
        let Some(source) = std::env::var_os("MAGIKA_TEST_CACHE_SOURCE") else { return };
        let directory = std::env::var_os("MAGIKA_TEST_CACHE_DIRECTORY").unwrap();
        let report = std::env::var_os("MAGIKA_TEST_CACHE_REPORT").unwrap();
        let rules = RuleSet::from_file_with_cache(source, Some(Path::new(&directory))).unwrap();
        assert_eq!(rules.identify(b"ABCD", 4), Some(ContentType::Png));
        std::fs::write(report, if rules.loaded_from_cache() { "hit" } else { "compiled" }).unwrap();
    }

    #[test]
    #[ignore = "requires a native Vectorscan compiler library"]
    fn concurrent_processes_compile_once_and_recover_after_interrupted_write() {
        let temp = tempfile::tempdir().unwrap();
        let source = temp.path().join("rules.yar");
        let cache = temp.path().join("cache");
        std::fs::write(&source, SOURCE).unwrap();
        let start = |index: usize, abort: bool| {
            let mut command = std::process::Command::new(std::env::current_exe().unwrap());
            command
                .args(["--exact", "rules::cache::tests::process_probe", "--ignored"])
                .env("MAGIKA_TEST_CACHE_SOURCE", &source)
                .env("MAGIKA_TEST_CACHE_DIRECTORY", &cache)
                .env("MAGIKA_TEST_CACHE_REPORT", temp.path().join(format!("report-{index}")))
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::piped());
            if abort {
                command.env("MAGIKA_TEST_ABORT_CACHE_WRITE", "1");
            }
            command.spawn().unwrap()
        };
        assert_eq!(start(99, true).wait().unwrap().code(), Some(86));
        assert!(!std::fs::read_dir(&cache).unwrap().any(|x| x
            .unwrap()
            .path()
            .extension()
            .is_some_and(|x| x == "hsdb")));
        let children: Vec<_> = (0..4).map(|i| start(i, false)).collect();
        for child in children {
            let output = child.wait_with_output().unwrap();
            assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
        }
        let reports: Vec<_> = (0..4)
            .map(|i| std::fs::read_to_string(temp.path().join(format!("report-{i}"))).unwrap())
            .collect();
        assert_eq!(reports.iter().filter(|x| *x == "compiled").count(), 1);
        assert_eq!(reports.iter().filter(|x| *x == "hit").count(), 3);
        assert!(start(4, false).wait().unwrap().success());
        assert_eq!(std::fs::read_to_string(temp.path().join("report-4")).unwrap(), "hit");
    }
}
