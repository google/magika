// Copyright 2026 Google LLC
// SPDX-License-Identifier: Apache-2.0

use std::cell::RefCell;
use std::sync::{Arc, OnceLock};

use super::{native, RuleSet};
use crate::ContentType;

/// A completed scan's disposition. Partial or conflicting results never identify a format.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Decision {
    Match(ContentType),
    NoMatch,
    Conflict,
    InsufficientInput,
    EngineError,
}

impl Decision {
    pub(super) fn content_type(self) -> Option<ContentType> {
        match self {
            Self::Match(label) => Some(label),
            _ => None,
        }
    }
}

static BUNDLED: OnceLock<Result<RuleSet, String>> = OnceLock::new();
thread_local! {
    // One scratch allocation per participating thread. Changing packs replaces it; the Arc
    // keeps the corresponding database/library alive without leaking a borrowed scanner.
    static SCANNER: RefCell<Option<native::Worker>> = const { RefCell::new(None) };
}

pub(super) fn scan(database: &Arc<native::Database>, prefix: &[u8], size: u64) -> Decision {
    SCANNER.with(|cell| {
        let Ok(mut slot) = cell.try_borrow_mut() else { return Decision::EngineError };
        if slot.as_ref().is_none_or(|worker| !Arc::ptr_eq(&worker.database, database)) {
            match database.worker() {
                Ok(worker) => *slot = Some(worker),
                Err(_) => {
                    *slot = None;
                    return Decision::EngineError;
                }
            }
        }
        let decision = slot.as_mut().unwrap().scan(prefix, size);
        // A failed scan cannot poison the next identification.
        if decision == Decision::EngineError {
            *slot = None;
        }
        decision
    })
}

pub(super) fn bundled() -> anyhow::Result<&'static RuleSet> {
    BUNDLED
        .get_or_init(|| {
            RuleSet::load(super::DEFAULT_RULES, super::cache::default_directory().as_deref(), None)
                .map_err(|e| format!("{e:#}"))
        })
        .as_ref()
        .map_err(|error| anyhow::anyhow!("failed to load bundled rules: {error}"))
}

pub(super) fn scan_promoted(prefix: &[u8], size: u64) -> Option<ContentType> {
    bundled().ok()?.identify(prefix, size)
}
