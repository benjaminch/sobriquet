//! Shell initialization script tests

use super::alx;
use predicates::prelude::*;

#[test]
fn generates_zsh_init() {
    alx()
        .args(["init", "zsh"])
        .assert()
        .success()
        .stdout(predicate::str::contains("sobriquet()"))
        .stdout(predicate::str::contains("print -z"))
        .stdout(predicate::str::contains("command sobriquet"))
        .stdout(predicate::str::contains("audit")); // Verify audit is in passthrough list
}

#[test]
fn generates_bash_init() {
    alx()
        .args(["init", "bash"])
        .assert()
        .success()
        .stdout(predicate::str::contains("sobriquet()"))
        .stdout(predicate::str::contains("READLINE_LINE"))
        .stdout(predicate::str::contains("audit"));
}

#[test]
fn generates_fish_init() {
    alx()
        .args(["init", "fish"])
        .assert()
        .success()
        .stdout(predicate::str::contains("function sobriquet"))
        .stdout(predicate::str::contains("commandline"))
        .stdout(predicate::str::contains("audit"));
}

#[test]
fn rejects_invalid_shell() {
    alx()
        .args(["init", "invalid"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid"));
}
