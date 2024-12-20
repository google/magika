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

use std::fs::Metadata;
use std::future::Future;
use std::path::Path;
use std::pin::Pin;
use std::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};

use ndarray::Array2;

use crate::input::AsyncInputApi;
use crate::Result;

pub(crate) fn exec<T>(mut future: impl Future<Output = T>) -> T {
    let future = unsafe { Pin::new_unchecked(&mut future) };
    let waker = panic_waker();
    let mut context = Context::from_waker(&waker);
    match future.poll(&mut context) {
        Poll::Ready(x) => x,
        Poll::Pending => unreachable!(),
    }
}

pub(crate) trait Env {
    type File: AsyncInputApi;
    async fn symlink_metadata(path: &Path) -> Result<Metadata>;
    async fn open(path: &Path) -> Result<Self::File>;
    async fn ort_session_run(
        session: &ort::session::Session, input: Array2<i32>,
    ) -> Result<ort::session::SessionOutputs>;
}

pub(crate) enum SyncEnv {}
impl Env for SyncEnv {
    type File = std::fs::File;

    async fn symlink_metadata(path: &Path) -> Result<Metadata> {
        Ok(std::fs::symlink_metadata(path)?)
    }

    async fn open(path: &Path) -> Result<Self::File> {
        Ok(std::fs::File::open(path)?)
    }

    async fn ort_session_run(
        session: &ort::session::Session, input: Array2<i32>,
    ) -> Result<ort::session::SessionOutputs> {
        Ok(session.run(ort::inputs!("bytes" => input)?)?)
    }
}

pub(crate) enum AsyncEnv {}
impl Env for AsyncEnv {
    type File = tokio::fs::File;

    async fn symlink_metadata(path: &Path) -> Result<Metadata> {
        Ok(tokio::fs::symlink_metadata(path).await?)
    }

    async fn open(path: &Path) -> Result<Self::File> {
        Ok(tokio::fs::File::open(path).await?)
    }

    async fn ort_session_run(
        session: &ort::session::Session, input: Array2<i32>,
    ) -> Result<ort::session::SessionOutputs> {
        Ok(session.run_async(ort::inputs!("bytes" => input)?)?.await?)
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
