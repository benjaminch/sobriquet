//! Security audit tests

use super::alx;
use predicates::prelude::*;

#[test]
fn audit_runs_successfully() {
    alx()
        .arg("audit")
        .assert()
        .success()
        .stdout(predicate::str::contains("Scanned"));
}

#[test]
fn audit_all_is_default() {
    // Running without kind should be same as 'all'
    let output_default = alx().arg("audit").output().unwrap();
    let output_all = alx().args(["audit", "all"]).output().unwrap();

    assert_eq!(output_default.status, output_all.status);
}

#[test]
fn audit_secrets_only() {
    alx()
        .args(["audit", "secrets"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Scanned"));
}

#[test]
fn audit_duplicates_only() {
    alx()
        .args(["audit", "duplicates"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Scanned"));
}

#[test]
fn audit_shows_in_help() {
    alx()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("audit"))
        .stdout(predicate::str::contains("security"));
}

#[test]
fn audit_rejects_invalid_kind() {
    alx()
        .args(["audit", "invalid"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid"));
}

#[test]
fn audit_shows_summary() {
    // Should always show either "No issues found" or "aliases passed all checks"
    alx().arg("audit").assert().success().stdout(
        predicate::str::contains("No issues found")
            .or(predicate::str::contains("aliases passed all checks")),
    );
}
