//! Error handling tests

use super::alx;
use predicates::prelude::*;

#[test]
fn invalid_shell_rejected() {
    alx()
        .args(["--shell", "invalid"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid"));
}

#[test]
fn unknown_option_rejected() {
    alx()
        .arg("--unknown-option")
        .assert()
        .failure()
        .stderr(predicate::str::contains("unexpected argument"));
}

#[test]
fn unknown_subcommand_rejected() {
    alx()
        .arg("unknown-subcommand")
        .assert()
        .failure()
        .stderr(predicate::str::contains("unrecognized subcommand"));
}

#[test]
fn empty_query_accepted() {
    // Empty query should be valid
    let output = alx().args(["--query", "", "--list"]).output().unwrap();
    assert!(output.status.success() || output.status.code() == Some(2));
}

#[test]
fn color_option_accepts_valid_values() {
    for value in ["auto", "always", "never"] {
        let output =
            alx().args(["--color", value, "--list"]).output().unwrap();
        assert!(
            output.status.success() || output.status.code() == Some(2),
            "color={value} should be accepted"
        );
    }
}

#[test]
fn color_option_rejects_invalid_values() {
    alx()
        .args(["--color", "invalid"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid"));
}
