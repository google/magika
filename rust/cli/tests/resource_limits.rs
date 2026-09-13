// Copyright 2026 Google LLC
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

use std::process::Command;

fn command() -> Command {
    Command::new(env!("CARGO_BIN_EXE_magika"))
}

#[test]
fn invalid_resource_limits_exit_without_panic_or_output() {
    for (flag, limit) in [("--batch-size", 64), ("--threads", 256), ("--readers", 256)] {
        for value in [usize::MAX, limit + 1, 0] {
            let output = command().args([flag, &value.to_string(), "sample"]).output().unwrap();
            assert_eq!(output.status.code(), Some(2), "{flag}={value}");
            assert!(output.stdout.is_empty());
            let error = String::from_utf8_lossy(&output.stderr);
            assert!(error.contains(flag), "{error}");
            assert!(!error.contains("panicked"), "{error}");
        }
    }
}

#[test]
fn resource_limits_accept_their_bounds() {
    let sample = concat!(env!("CARGO_MANIFEST_DIR"), "/../../tests_data/basic/rust/code.rs");
    for (flag, limit) in [("--batch-size", 64), ("--threads", 256), ("--readers", 256)] {
        for value in [1, limit] {
            let output = command()
                .args([flag, &value.to_string(), "--backend=cpu", "--label", sample])
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "{flag}={value}: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
    }
}
