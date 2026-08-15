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

use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};

pub(crate) fn exec<T>(mut future: impl Future<Output = T>) -> T {
    let future = unsafe { Pin::new_unchecked(&mut future) };
    let waker = panic_waker();
    let mut context = Context::from_waker(&waker);
    match future.poll(&mut context) {
        Poll::Ready(x) => x,
        Poll::Pending => unreachable!(),
    }
}

fn panic_waker() -> Waker {
    const PANIC_WAKER: RawWakerVTable = RawWakerVTable::new(clone, wake, wake, drop);
    fn clone(p: *const ()) -> RawWaker {
        RawWaker::new(p, &PANIC_WAKER)
    }
    fn wake(_: *const ()) {
        unreachable!()
    }
    fn drop(_: *const ()) {}
    let raw = clone(std::ptr::null());
    unsafe { Waker::from_raw(raw) }
}
