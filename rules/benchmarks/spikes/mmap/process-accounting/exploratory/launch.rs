// Copyright 2026 Google LLC
// SPDX-License-Identifier: Apache-2.0
use std::{fs::File, process::{Command, Stdio}, time::{Instant, SystemTime, UNIX_EPOCH}};
fn unix() -> u128 { SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos() }
fn main() {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    let runs = std::env::var("MAGIKA_LAUNCH_RUNS").unwrap_or("1".into()).parse::<usize>().unwrap();
    for repetition in 0..runs {
    let mut command = Command::new(&args[2]);
    command.args(&args[3..]).stdin(Stdio::null())
        .stdout(File::create(&args[0]).unwrap()).stderr(File::create(&args[1]).unwrap());
    let start_unix = unix();
    let start = Instant::now();
    let mut child = command.spawn().unwrap();
    let spawn_ns = start.elapsed().as_nanos();
    let status = child.wait().unwrap();
    let elapsed_ns = start.elapsed().as_nanos();
    let end_unix = unix();
    let trace = std::fs::read_to_string(&args[1]).unwrap();
    let trace = if trace.trim().is_empty() { "null" } else { trace.trim() };
    println!("{{\"repetition\":{repetition},\"trace\":{trace},\"start_unix_ns\":{start_unix},\"end_unix_ns\":{end_unix},\"elapsed_ns\":{elapsed_ns},\"spawn_call_ns\":{spawn_ns}}}");
    assert!(status.success());
    }
}
