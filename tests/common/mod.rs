//! Common test utilities and helpers for integration tests
//!
//! This module provides reusable test infrastructure inspired by uutils/coreutils:
//! - TestScenario: Fixture management with temp directories
//! - CommandExt: Fluent assertion helpers for Command
//! - Fixture helpers: Creating test files and directories

#![allow(dead_code)] // Test utilities may not all be used yet

use assert_cmd::Command;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

/// A test scenario that manages temporary directories and fixtures
///
/// Automatically cleans up on drop.
pub struct TestScenario {
    /// Temporary directory for this test
    pub tmpdir: TempDir,
    /// Name of the test scenario
    pub name: String,
}

impl TestScenario {
    /// Create a new test scenario
    pub fn new(name: &str) -> Self {
        let tmpdir = TempDir::new().expect("Failed to create temp dir");
        Self { tmpdir, name: name.to_owned() }
    }

    /// Get the path to the temporary directory
    pub fn tmpdir_path(&self) -> &Path {
        self.tmpdir.path()
    }

    /// Create a file in the temp directory with the given content
    pub fn create_file(&self, name: &str, content: &str) -> PathBuf {
        let path = self.tmpdir.path().join(name);
        fs::write(&path, content).expect("Failed to write test file");
        path
    }

    /// Create a directory in the temp directory
    pub fn create_dir(&self, name: &str) -> PathBuf {
        let path = self.tmpdir.path().join(name);
        fs::create_dir_all(&path).expect("Failed to create test directory");
        path
    }

    /// Create a file with the given path relative to tmpdir
    pub fn create_nested_file(&self, path: &str, content: &str) -> PathBuf {
        let full_path = self.tmpdir.path().join(path);
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent)
                .expect("Failed to create parent directories");
        }
        fs::write(&full_path, content).expect("Failed to write nested file");
        full_path
    }

    /// Get a Command for the sobriquet binary
    #[allow(clippy::unused_self)]
    pub fn cmd(&self) -> Command {
        use assert_cmd::cargo::cargo_bin_cmd;
        cargo_bin_cmd!("sobriquet")
    }

    /// Get a Command for the sobriquet binary with env vars set to tmpdir
    pub fn cmd_with_env(&self) -> Command {
        let mut cmd = self.cmd();
        cmd.env("HOME", self.tmpdir.path());
        cmd.env("XDG_CONFIG_HOME", self.tmpdir.path().join(".config"));
        cmd
    }

    /// Read a file from the temp directory
    pub fn read_file(&self, name: &str) -> String {
        let path = self.tmpdir.path().join(name);
        fs::read_to_string(&path).expect("Failed to read file")
    }

    /// Check if a file exists in the temp directory
    pub fn file_exists(&self, name: &str) -> bool {
        self.tmpdir.path().join(name).exists()
    }
}

/// Extension trait for Command to add fluent assertion helpers
pub trait CommandExt {
    /// Assert command succeeds and returns the Output
    fn succeeds(&mut self) -> std::process::Output;

    /// Assert command fails and returns the Output
    fn fails(&mut self) -> std::process::Output;

    /// Assert command succeeds with specific stdout content
    fn succeeds_with_stdout(&mut self, expected: &str)
    -> std::process::Output;

    /// Assert command succeeds and stdout contains the given string
    fn succeeds_with_stdout_containing(
        &mut self,
        expected: &str,
    ) -> std::process::Output;

    /// Assert command succeeds and stderr contains the given string
    fn succeeds_with_stderr_containing(
        &mut self,
        expected: &str,
    ) -> std::process::Output;

    /// Assert command fails with specific exit code
    fn fails_with_code(&mut self, code: i32) -> std::process::Output;

    /// Assert command fails with stderr containing the given string
    fn fails_with_stderr_containing(
        &mut self,
        expected: &str,
    ) -> std::process::Output;

    /// Assert command output (success or failure) with no assertions on exit code
    fn run_no_check(&mut self) -> std::process::Output;
}

impl CommandExt for Command {
    fn succeeds(&mut self) -> std::process::Output {
        let output = self.output().expect("Failed to execute command");
        assert!(
            output.status.success(),
            "Command failed when it should have succeeded.\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        output
    }

    fn fails(&mut self) -> std::process::Output {
        let output = self.output().expect("Failed to execute command");
        assert!(
            !output.status.success(),
            "Command succeeded when it should have failed.\nstdout: {}",
            String::from_utf8_lossy(&output.stdout)
        );
        output
    }

    fn succeeds_with_stdout(
        &mut self,
        expected: &str,
    ) -> std::process::Output {
        let output = self.succeeds();
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert_eq!(
            stdout.trim(),
            expected.trim(),
            "stdout did not match expected"
        );
        output
    }

    fn succeeds_with_stdout_containing(
        &mut self,
        expected: &str,
    ) -> std::process::Output {
        let output = self.succeeds();
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains(expected),
            "stdout did not contain '{expected}'\nActual stdout: {stdout}"
        );
        output
    }

    fn succeeds_with_stderr_containing(
        &mut self,
        expected: &str,
    ) -> std::process::Output {
        let output = self.succeeds();
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains(expected),
            "stderr did not contain '{expected}'\nActual stderr: {stderr}"
        );
        output
    }

    fn fails_with_code(&mut self, code: i32) -> std::process::Output {
        let output = self.fails();
        let actual_code = output.status.code().expect("No exit code");
        assert_eq!(
            actual_code, code,
            "Exit code {actual_code} did not match expected code {code}"
        );
        output
    }

    fn fails_with_stderr_containing(
        &mut self,
        expected: &str,
    ) -> std::process::Output {
        let output = self.fails();
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains(expected),
            "stderr did not contain '{expected}'\nActual stderr: {stderr}"
        );
        output
    }

    fn run_no_check(&mut self) -> std::process::Output {
        self.output().expect("Failed to execute command")
    }
}

/// Helper to get the sobriquet command
pub fn alx() -> Command {
    use assert_cmd::cargo::cargo_bin_cmd;
    cargo_bin_cmd!("sobriquet")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scenario_creates_tmpdir() {
        let ts = TestScenario::new("test");
        assert!(ts.tmpdir_path().exists());
    }

    #[test]
    fn test_scenario_creates_file() {
        let ts = TestScenario::new("test");
        let path = ts.create_file("test.txt", "hello");
        assert!(path.exists());
        assert_eq!(ts.read_file("test.txt"), "hello");
    }

    #[test]
    fn test_scenario_creates_dir() {
        let ts = TestScenario::new("test");
        let path = ts.create_dir("testdir");
        assert!(path.exists());
        assert!(path.is_dir());
    }

    #[test]
    fn test_scenario_creates_nested_file() {
        let ts = TestScenario::new("test");
        let path = ts.create_nested_file("a/b/c.txt", "nested");
        assert!(path.exists());
        assert_eq!(ts.read_file("a/b/c.txt"), "nested");
    }
}
