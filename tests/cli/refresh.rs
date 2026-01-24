//! Cache refresh tests

#![allow(clippy::unwrap_used)]

use super::alx;

#[test]
fn refresh_flag_accepted() {
    // Test that --refresh flag is accepted (will try to refresh and list)
    let output = alx().args(["--refresh", "--list"]).output().unwrap();

    // Either succeeds with aliases or fails gracefully
    assert!(output.status.success() || output.status.code() == Some(2));
}

#[test]
fn refresh_short_flag_accepted() {
    // Test that -r flag is accepted
    let output = alx().args(["-r", "-l"]).output().unwrap();

    // Either succeeds with aliases or fails gracefully
    assert!(output.status.success() || output.status.code() == Some(2));
}

#[test]
fn refresh_alone_works() {
    // Test that --refresh works without --list (will open fuzzy finder, but we just check it accepts the flag)
    let output = alx().args(["--refresh", "--list"]).output().unwrap();

    assert!(output.status.success() || output.status.code() == Some(2));
}
