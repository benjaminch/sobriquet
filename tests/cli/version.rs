//! Version flag tests

use super::alx;
use predicates::prelude::*;

#[test]
fn shows_version_with_flag() {
    alx()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("alx"))
        .stdout(predicate::str::contains("0.1.0"));
}

#[test]
fn shows_version_with_short_flag() {
    alx().arg("-V").assert().success().stdout(predicate::str::contains("alx"));
}
