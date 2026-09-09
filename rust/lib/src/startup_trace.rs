// Copyright 2026 Google LLC
// SPDX-License-Identifier: Apache-2.0
//! Opt-in structured timings for startup diagnostics.

use std::sync::{Mutex, OnceLock};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use serde::Serialize;

static ENABLED: OnceLock<bool> = OnceLock::new();
static START: OnceLock<Instant> = OnceLock::new();
static EVENTS: Mutex<Vec<Event>> = Mutex::new(Vec::new());

fn enabled() -> bool {
    *ENABLED.get_or_init(|| std::env::var_os("MAGIKA_STARTUP_TRACE").is_some_and(|v| v == "1"))
}

#[derive(Serialize)]
struct Event {
    name: &'static str,
    thread: String,
    start_ns: u128,
    duration_ns: u128,
}

/// A measured interval; nested and concurrent intervals must not be summed.
pub struct Span {
    name: &'static str,
    start: Instant,
}
/// Starts an interval only when explicitly enabled.
pub fn span(name: &'static str) -> Option<Span> {
    enabled().then(|| {
        START.get_or_init(Instant::now);
        Span { name, start: Instant::now() }
    })
}
impl Drop for Span {
    fn drop(&mut self) {
        let duration_ns = self.start.elapsed().as_nanos();
        let event = Event {
            name: self.name,
            thread: std::thread::current().name().unwrap_or("unnamed").to_owned(),
            start_ns: self.start.duration_since(*START.get().unwrap()).as_nanos(),
            duration_ns,
        };
        EVENTS.lock().unwrap().push(event);
    }
}

/// Owns the process trace, emitting one JSON object after all main-local resources drop.
pub struct Session {
    entry_unix_ns: u128,
    start: Instant,
}
impl Session {
    /// Call as the first statement in main.
    pub fn begin() -> Option<Self> {
        let entry_unix_ns = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let start = Instant::now();
        if !enabled() {
            return None;
        }
        START.get_or_init(|| start);
        Some(Self { entry_unix_ns, start })
    }
}
impl Drop for Session {
    fn drop(&mut self) {
        let elapsed_ns = self.start.elapsed().as_nanos();
        let mut events = EVENTS.lock().unwrap();
        events.sort_by_key(|event| event.start_ns);
        eprintln!(
            "{}",
            serde_json::json!({"magika_startup_trace": 1, "main_entry_unix_ns": self.entry_unix_ns,
            "main_elapsed_ns": elapsed_ns, "events": *events})
        );
    }
}
