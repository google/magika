// Copyright 2026 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     https://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Checks that declaring a maximum batch only changes which plans are resident.

use magika_tract_runtime::{BackendRequest, Runtime};

const FEATURE_SIZE: usize = 2048;

/// Builds a deterministic feature tensor that is not uniform across the batch.
fn features(batch: usize) -> Vec<i32> {
    (0..batch * FEATURE_SIZE).map(|index| (index % 257) as i32).collect()
}

#[test]
fn a_declared_maximum_serves_larger_requests_with_the_same_scores() {
    let batch = 20;
    let input = features(batch);

    let mut every_class = Runtime::new(BackendRequest::Cpu).unwrap().session().unwrap();
    let reference = every_class.run(&input, batch).unwrap();

    // Twenty items cannot be served by the classes at or below eight unless the request is
    // decomposed over exactly those, so this fails outright if routing reaches for a larger class.
    let mut up_to_eight =
        Runtime::with_max_batch(BackendRequest::Cpu, 8).unwrap().session().unwrap();
    let routed = up_to_eight.run(&input, batch).unwrap();

    assert_eq!(routed.len(), reference.len());
    for (index, (routed, reference)) in routed.iter().zip(&reference).enumerate() {
        assert!(
            (routed - reference).abs() < 1e-5,
            "score {index} differs: {routed} against {reference}"
        );
    }
}

#[test]
fn a_maximum_below_every_class_still_serves_requests() {
    let batch = 3;
    let input = features(batch);
    let mut single = Runtime::with_max_batch(BackendRequest::Cpu, 1).unwrap().session().unwrap();
    assert_eq!(single.run(&input, batch).unwrap().len() % batch, 0);
}

#[test]
fn a_zero_maximum_is_rejected() {
    assert!(Runtime::with_max_batch(BackendRequest::Cpu, 0).is_err());
}
