// Copyright 2026 Google LLC
// SPDX-License-Identifier: Apache-2.0

use std::cell::RefCell;
use std::sync::{Arc, OnceLock};

use super::preprocess::{Blocks, Synthetic};
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
    // Stream B, rebuilt in place per input so facts packs never allocate per identification.
    static SYNTHETIC: RefCell<Option<Synthetic>> = const { RefCell::new(None) };
}

/// Identifies one input from its held blocks, preprocessing them into this thread's stream B
/// first when `preprocess` (only databases that scan facts need to).
pub(super) fn identify(
    database: &Arc<native::Database>, blocks: &Blocks<'_>, preprocess: bool,
) -> Decision {
    let Blocks { prefix, size, .. } = *blocks;
    if !preprocess {
        return scan(database, None, prefix, size);
    }
    SYNTHETIC.with(|cell| {
        let Ok(mut slot) = cell.try_borrow_mut() else { return Decision::EngineError };
        let synthetic = slot.get_or_insert_with(Synthetic::new);
        synthetic.prepare(blocks);
        scan(database, Some(synthetic), prefix, size)
    })
}

/// Scans one input on this thread's worker. `synthetic` is stream B, prepared only for a
/// database that scans facts.
pub(super) fn scan(
    database: &Arc<native::Database>, synthetic: Option<&Synthetic>, prefix: &[u8], size: u64,
) -> Decision {
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
        let decision = slot.as_mut().unwrap().scan(synthetic, prefix, size);
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

/// Identifies one input with the bundled pack; a pack that failed to load abstains.
pub(super) fn scan_promoted(prefix: &[u8], size: u64, tail: Option<&[u8]>) -> Option<ContentType> {
    bundled().ok()?.identify(prefix, size, tail)
}
