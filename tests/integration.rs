//! Integration tests for the alx CLI
//!
//! Tests are organized by functional scope in the `cli` module:
//! - `help` - Help flag tests
//! - `version` - Version flag tests  
//! - `init` - Shell initialization script tests
//! - `generate` - Completions and man page tests
//! - `list` - Alias listing tests
//! - `stats` - Usage statistics tests
//! - `config` - Configuration tests
//! - `refresh` - Cache refresh tests
//! - `audit` - Security audit tests
//! - `error_handling` - Error handling tests

#![allow(clippy::unwrap_used)]

mod cli;
