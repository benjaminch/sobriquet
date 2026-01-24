//! Help flag tests

use super::alx;
use predicates::prelude::*;

#[test]
fn shows_help_with_flag() {
    alx()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Fuzzy finder for shell aliases"))
        .stdout(predicate::str::contains("--list"))
        .stdout(predicate::str::contains("--shell"))
        .stdout(predicate::str::contains("init"))
        .stdout(predicate::str::contains("generate"))
        .stdout(predicate::str::contains("audit"));
}

#[test]
fn shows_help_with_short_flag() {
    alx()
        .arg("-h")
        .assert()
        .success()
        .stdout(predicate::str::contains("Fuzzy finder for shell aliases"));
}
