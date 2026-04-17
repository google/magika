use std::io::{BufRead, BufReader, Read};
use std::path::PathBuf;
use std::process::{Command, Stdio};

#[test]
fn exits_cleanly_when_stdout_pipe_closes_early() {
    let input_dir =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..").join("tests_data/basic");

    let mut child = Command::new(env!("CARGO_BIN_EXE_magika"))
        .arg("-r")
        .arg(&input_dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn magika CLI");

    // Read one line so the CLI starts writing, then close the pipe early.
    let mut stdout = BufReader::new(child.stdout.take().expect("missing stdout pipe"));
    let mut first_line = String::new();
    let bytes_read = stdout.read_line(&mut first_line).expect("failed to read CLI output");
    assert!(bytes_read > 0, "expected at least one line of CLI output");
    drop(stdout);

    let status = child.wait().expect("failed to wait for magika");
    let mut stderr = String::new();
    BufReader::new(child.stderr.take().expect("missing stderr pipe"))
        .read_to_string(&mut stderr)
        .expect("failed to read stderr");

    assert!(
        status.success(),
        "expected success after broken pipe, got {status}; stderr:\n{stderr}",
    );
    assert!(stderr.is_empty(), "expected no stderr output, got:\n{stderr}");
}
