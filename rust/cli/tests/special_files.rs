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

#![cfg(unix)]

use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

fn command() -> Command {
    Command::new(env!("CARGO_BIN_EXE_magika"))
}

#[test]
fn recursive_named_pipe_reports_error_and_preserves_regular_results() {
    let directory =
        std::env::temp_dir().join(format!("magika-recursive-fifo-{}", std::process::id()));
    let inputs = directory.join("inputs");
    std::fs::create_dir_all(&inputs).unwrap();
    std::fs::write(inputs.join("a.txt"), b"ordinary text before pipe\n").unwrap();
    let pipe = inputs.join("b.pipe");
    assert!(Command::new("mkfifo").arg(&pipe).status().unwrap().success());
    std::fs::write(inputs.join("c.txt"), b"ordinary text after pipe\n").unwrap();
    let output = directory.join("output.jsonl");
    let mut child = command()
        .args(["-r", "--jsonl", "--backend=cpu", "--threads=1", "--readers=1"])
        .arg(&inputs)
        .stdin(Stdio::null())
        .stdout(std::fs::File::create(&output).unwrap())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(10);
    let status = loop {
        if let Some(status) = child.try_wait().unwrap() {
            break status;
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            std::fs::remove_dir_all(&directory).unwrap();
            panic!("recursive FIFO blocked classification");
        }
        std::thread::sleep(Duration::from_millis(10));
    };
    let rows: Vec<serde_json::Value> = std::fs::read_to_string(output)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    std::fs::remove_dir_all(&directory).unwrap();
    assert_eq!(status.code(), Some(1));
    assert_eq!(rows.len(), 3);
    for (row, name) in rows.iter().zip(["a.txt", "b.pipe", "c.txt"]) {
        assert_eq!(row["path"].as_str().unwrap(), inputs.join(name).to_str().unwrap());
    }
    assert_eq!(rows[1]["result"]["status"], "unsupported_file_type");
    for index in [0, 2] {
        assert!(rows[index]["result"]["value"]["output"]["label"].is_string());
    }
}

#[test]
fn named_pipes_and_sockets_given_directly_are_rejected() {
    let directory = std::env::temp_dir().join(format!("magika-special-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    let pipe = directory.join("pipe");
    assert!(Command::new("mkfifo").arg(&pipe).status().unwrap().success());
    let link = directory.join("link");
    std::os::unix::fs::symlink(&pipe, &link).unwrap();
    let socket = directory.join("socket");
    let listener = std::os::unix::net::UnixListener::bind(&socket).unwrap();
    let mut child = command()
        .args(["--jsonl", "--backend=cpu"])
        .args([&pipe, &link, &socket])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(10);
    while child.try_wait().unwrap().is_none() {
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            std::fs::remove_dir_all(&directory).unwrap();
            panic!("a special file blocked classification");
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    let output = child.wait_with_output().unwrap();
    drop(listener);
    std::fs::remove_dir_all(&directory).unwrap();
    let rows: Vec<serde_json::Value> = String::from_utf8(output.stdout)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    assert_eq!(rows.len(), 3);
    for row in &rows {
        assert_eq!(row["result"]["status"], "unsupported_file_type", "{row}");
    }
}
