//! Alias listing tests

#![allow(clippy::unwrap_used)]

use super::alx;

#[test]
fn list_outputs_aliases() {
    // This test depends on having aliases defined in the shell
    // It should at least not crash
    let output = alx().arg("--list").output().unwrap();

    // Either succeeds with aliases or fails gracefully
    assert!(output.status.success() || output.status.code() == Some(2));
}

#[test]
fn list_with_shell_option() {
    // Test that --shell option is accepted
    let output = alx().args(["--list", "--shell", "zsh"]).output().unwrap();

    // Either succeeds with aliases or fails gracefully
    assert!(output.status.success() || output.status.code() == Some(2));
}

#[test]
fn list_with_json_format() {
    let output = alx().args(["--list", "--format", "json"]).output().unwrap();

    // Either succeeds with aliases or fails gracefully
    assert!(output.status.success() || output.status.code() == Some(2));

    // If successful, output should be valid JSON (starts with [ or is empty)
    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.trim().is_empty() || stdout.trim().starts_with('['));
    }
}

#[test]
fn list_with_json_pretty_format() {
    let output =
        alx().args(["--list", "--format", "json-pretty"]).output().unwrap();

    assert!(output.status.success() || output.status.code() == Some(2));
}
