//! Version flag tests

use super::alx;
use predicates::prelude::*;

#[test]
fn shows_version_with_flag() {
    // Get the version from Cargo.toml at runtime
    let version = env!("CARGO_PKG_VERSION");
    alx()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("sobriquet"))
        .stdout(predicate::str::contains(version));
}

#[test]
fn shows_version_with_short_flag() {
    alx()
        .arg("-V")
        .assert()
        .success()
        .stdout(predicate::str::contains("sobriquet"));
}
