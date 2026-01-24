//! Configuration tests

use super::alx;
use predicates::prelude::*;

#[test]
fn config_shows_path() {
    alx()
        .arg("config")
        .assert()
        .success()
        .stdout(predicate::str::contains("config.toml"));
}

#[test]
fn config_path_contains_alx() {
    alx()
        .arg("config")
        .assert()
        .success()
        .stdout(predicate::str::contains("sobriquet"));
}
