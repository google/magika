// Copyright 2026 Google LLC
// SPDX-License-Identifier: Apache-2.0

//! Backpressure from ordered output to file traversal, including output shutdown.

use std::sync::{Condvar, Mutex};

#[derive(Debug, Default)]
struct State {
    next: usize,
    closed: bool,
}

#[derive(Debug, Default)]
pub(crate) struct Progress {
    state: Mutex<State>,
    changed: Condvar,
}

impl Progress {
    /// Waits without polling until the order fits the window, or output closes.
    pub(crate) fn wait(&self, order: usize, max_dist: usize) -> bool {
        let state = self.state.lock().unwrap();
        let state = self
            .changed
            .wait_while(state, |state| !state.closed && order.saturating_sub(state.next) > max_dist)
            .unwrap();
        !state.closed
    }

    /// Announces actual backpressure before sleeping. Notify outside the lock: the callback
    /// may send to a bounded channel whose consumer also needs ordered output to advance.
    pub(crate) fn wait_with_notify(
        &self, order: usize, max_dist: usize, notify: impl FnOnce(usize) -> bool,
    ) -> bool {
        let state = self.state.lock().unwrap();
        if state.closed || order.saturating_sub(state.next) <= max_dist {
            return !state.closed;
        }
        let blocked_on = state.next;
        drop(state);
        notify(blocked_on) && self.wait(order, max_dist)
    }

    pub(crate) fn advance(&self, next: usize) {
        self.state.lock().unwrap().next = next;
        self.changed.notify_one();
    }

    pub(crate) fn close(&self) {
        self.state.lock().unwrap().closed = true;
        self.changed.notify_all();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{mpsc, Arc};
    use std::time::Duration;

    #[test]
    fn backpressure_notification_does_not_hold_the_output_lock() {
        let progress = Progress::default();
        assert!(progress.wait_with_notify(5, 4, |blocked_on| {
            assert_eq!(blocked_on, 0);
            progress.advance(1);
            true
        }));
        assert!(progress.wait_with_notify(5, 4, |_| panic!("window is not full")));
        progress.close();
        assert!(!progress.wait_with_notify(6, 4, |_| panic!("output is closed")));
    }

    #[test]
    fn window_waits_for_ordered_progress() {
        let progress = Arc::new(Progress::default());
        assert!(progress.wait(4, 4));
        let (sender, receiver) = mpsc::channel();
        let worker = {
            let progress = progress.clone();
            std::thread::spawn(move || sender.send(progress.wait(5, 4)).unwrap())
        };
        assert!(receiver.recv_timeout(Duration::from_millis(20)).is_err());
        progress.advance(1);
        assert!(receiver.recv_timeout(Duration::from_secs(1)).unwrap());
        worker.join().unwrap();
    }

    #[test]
    fn closing_output_releases_waiters_and_stays_closed() {
        let progress = Arc::new(Progress::default());
        let (sender, receiver) = mpsc::channel();
        let worker = {
            let progress = progress.clone();
            std::thread::spawn(move || sender.send(progress.wait(5, 4)).unwrap())
        };
        assert!(receiver.recv_timeout(Duration::from_millis(20)).is_err());
        progress.close();
        assert!(!receiver.recv_timeout(Duration::from_secs(1)).unwrap());
        worker.join().unwrap();
        progress.advance(usize::MAX);
        assert!(!progress.wait(0, 4));
        assert!(!progress.wait(usize::MAX, usize::MAX));
    }
}
