//! Usage statistics tests

use super::alx;
use predicates::prelude::*;

#[test]
fn stats_command_works() {
    // Stats command should succeed, showing either stats or "no stats yet" message
    alx().arg("stats").assert().success().stdout(
        predicate::str::contains("sobriquet statistics")
            .or(predicate::str::contains("No statistics yet")),
    );
}

#[test]
fn stats_clear_works() {
    alx()
        .args(["stats", "clear"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Statistics cleared"));
}

#[test]
fn stats_shows_usage_info() {
    // After clearing, should show "no stats yet"
    alx().args(["stats", "clear"]).assert().success();

    alx()
        .arg("stats")
        .assert()
        .success()
        .stdout(predicate::str::contains("No statistics yet"));
}
