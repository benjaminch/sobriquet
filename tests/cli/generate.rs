//! Shell completions and man page generation tests

use super::alx;
use predicates::prelude::*;

#[test]
fn generates_zsh_completions() {
    alx()
        .args(["generate", "complete-zsh"])
        .assert()
        .success()
        .stdout(predicate::str::contains("#compdef sobriquet"));
}

#[test]
fn generates_bash_completions() {
    alx()
        .args(["generate", "complete-bash"])
        .assert()
        .success()
        .stdout(predicate::str::contains("_sobriquet"));
}

#[test]
fn generates_fish_completions() {
    alx()
        .args(["generate", "complete-fish"])
        .assert()
        .success()
        .stdout(predicate::str::contains("complete"));
}

#[test]
fn generates_man_page() {
    alx()
        .args(["generate", "man"])
        .assert()
        .success()
        .stdout(predicate::str::contains(".TH SOBRIQUET 1"))
        .stdout(predicate::str::contains(".SH NAME"))
        .stdout(predicate::str::contains(".SH SYNOPSIS"))
        .stdout(predicate::str::contains(".SH DESCRIPTION"))
        .stdout(predicate::str::contains("audit")); // Verify audit is documented
}

#[test]
fn rejects_invalid_kind() {
    alx()
        .args(["generate", "invalid"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid"));
}
