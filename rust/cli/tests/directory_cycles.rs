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

fn rows(stdout: Vec<u8>) -> Vec<serde_json::Value> {
    String::from_utf8(stdout)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect()
}

#[test]
#[cfg(unix)]
fn recursive_symlink_cycles_report_once_and_keep_other_inputs() {
    let directory = std::env::temp_dir().join(format!("magika-cycle-{}", std::process::id()));
    std::fs::create_dir_all(directory.join("tree/sub")).unwrap();
    std::fs::write(directory.join("tree/a.txt"), b"ordinary text\n").unwrap();
    std::os::unix::fs::symlink("..", directory.join("tree/sub/back")).unwrap();
    // A separate alias to the same subtree is legitimate; cycle detection must be
    // scoped to ancestors, not a global visited set that silently drops inputs.
    std::os::unix::fs::symlink("tree", directory.join("alias")).unwrap();
    let output = command()
        .args(["-r", "--jsonl", "--backend=cpu"])
        .arg(directory.join("tree"))
        .arg(directory.join("alias"))
        .output()
        .unwrap();
    let links = command()
        .args(["-r", "--jsonl", "--no-dereference", "--backend=cpu"])
        .arg(directory.join("tree"))
        .arg(directory.join("alias"))
        .output()
        .unwrap();
    std::fs::remove_dir_all(&directory).unwrap();
    assert_eq!(output.status.code(), Some(1));
    let followed = rows(output.stdout);
    assert_eq!(followed.len(), 4, "repeated files through a directory cycle");
    for (row, suffix) in
        followed.iter().zip(["tree/a.txt", "tree/sub/back", "alias/a.txt", "alias/sub/back"])
    {
        assert_eq!(row["path"], directory.join(suffix).to_str().unwrap());
    }
    for index in [1, 3] {
        assert_eq!(followed[index]["result"]["status"], "directory_cycle");
    }
    assert!(links.status.success());
    let unfollowed = rows(links.stdout);
    assert_eq!(unfollowed.len(), 3);
    for index in [1, 2] {
        assert_eq!(unfollowed[index]["result"]["value"]["output"]["label"], "symlink");
    }
}
