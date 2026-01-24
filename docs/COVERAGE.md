# Code Coverage Tracking

This project uses automated code coverage tracking to identify untested code paths and maintain code quality.

## Overview

- **Tool**: [cargo-tarpaulin](https://github.com/xd009642/tarpaulin) - Rust code coverage measurement
- **Platform**: [codecov.io](https://codecov.io/) - Continuous coverage tracking and reporting
- **Minimum Threshold**: 55% overall coverage, 70% for new code
- **Current Coverage**: See [codecov dashboard](https://codecov.io/gh/benjaminch/alxrsrs)

## Current Coverage by File

| File | Coverage | Lines |
|------|----------|-------|
| `src/config.rs` | 100% | 29/29 ✅ |
| `src/shell.rs` | 97% | 72/74 |
| `src/source.rs` | 81% | 108/134 |
| `src/alias.rs` | 95% | 87/92 |
| `src/audit.rs` | 80% | 130/162 |
| `src/cli.rs` | 50% | 88/174 |
| `src/preview.rs` | 28% | 55/197 |
| `src/stats.rs` | 29% | 63/220 |
| `src/main.rs` | 50% | 3/6 |
| **Overall** | **58%** | **635/1088** |

## Coverage Workflow

Coverage is automatically measured on:
- Every push to main/master
- Every pull request
- Daily scheduled runs

### Local Coverage Measurement

To check coverage locally before submitting a PR:

```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Generate coverage report
cargo tarpaulin --timeout 600

# Generate HTML report
cargo tarpaulin --out Html --output-dir coverage

# Check specific thresholds
cargo tarpaulin --timeout 600 | grep coverage
```

### View Coverage Report

```bash
# Open HTML coverage report (after running with --out Html)
open coverage/index.html

# Or view online at codecov.io
# https://codecov.io/gh/benjaminch/alxrsrs
```

## Coverage Thresholds

| Category | Minimum | Target |
|----------|---------|--------|
| **Overall** | 55% | 70%+ |
| **New Code** | 70% | 80%+ |
| **Public API** | 85% | 95%+ |
| **Critical Paths** | 80% | 100% |

The CI will **warn** if coverage drops but won't fail the build unless it falls below the minimum threshold.

## Areas for Improvement

### Low Coverage Files

1. **stats.rs** (29%)
   - Statistics generation and formatting
   - Many edge cases and formatting variations
   - Recommendation: Add tests for edge cases and time calculations

2. **preview.rs** (28%)
   - Rich preview generation
   - Many conditional branches for different command types
   - Recommendation: Add snapshot tests for different command patterns

3. **cli.rs** (50%)
   - Command-line argument parsing and handling
   - Many subcommand variations
   - Recommendation: Add more integration tests for each subcommand

### High Coverage Files

✅ **config.rs** (100%)
- Configuration parsing and defaults
- All code paths tested

✅ **shell.rs** (97%)
- Shell detection and initialization
- Comprehensive shell-specific tests

## How Coverage Works in CI

### Step 1: Generate Coverage
```bash
cargo tarpaulin --out xml --output-dir coverage
```

### Step 2: Upload to codecov.io
```bash
codecov --files ./coverage/cobertura.xml
```

### Step 3: Status Checks
- ✅ Coverage doesn't drop significantly
- ✅ New code has adequate coverage
- ⚠️ Warning if approaching minimum threshold

### Step 4: Badge Updates
The coverage badge in the README updates automatically with the latest data.

## Interpreting Coverage Reports

### Line Coverage
Shows % of lines executed during tests.

### Branch Coverage
Shows % of conditional branches taken.

### Uncovered Lines
Listed by file/line number, helps identify gaps:
```
|| src/alias.rs: 43, 174, 182-183, 187
|| src/stats.rs: 65, 74, 105-108, 110-113
```

## Best Practices

### Writing Testable Code

1. **Keep functions focused** - Single responsibility = easier to test
2. **Use dependency injection** - Makes mocking easier
3. **Avoid tight coupling** - Testable modules are loosely coupled
4. **Separate logic from I/O** - Test logic without file/network calls

### Writing Better Tests

1. **Test happy path** - Normal case should work
2. **Test error paths** - What happens on failure?
3. **Test edge cases** - Boundary conditions, empty inputs, etc.
4. **Use descriptive names** - Test name should describe what it tests
5. **One assertion per test** - Clearer failure messages

### Example: Adding Coverage

Before (low coverage):
```rust
fn format_time(seconds: u64) -> String {
    if seconds < 60 {
        format!("{}s", seconds)
    } else if seconds < 3600 {
        format!("{}m", seconds / 60)
    } else {
        format!("{}h", seconds / 3600)
    }
}
```

After (high coverage):
```rust
fn format_time(seconds: u64) -> String {
    if seconds < 60 {
        format!("{}s", seconds)
    } else if seconds < 3600 {
        format!("{}m", seconds / 60)
    } else {
        format!("{}h", seconds / 3600)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_seconds() {
        assert_eq!(format_time(30), "30s");
        assert_eq!(format_time(0), "0s");
        assert_eq!(format_time(59), "59s");
    }

    #[test]
    fn test_format_minutes() {
        assert_eq!(format_time(60), "1m");
        assert_eq!(format_time(3599), "59m");
    }

    #[test]
    fn test_format_hours() {
        assert_eq!(format_time(3600), "1h");
        assert_eq!(format_time(86400), "24h");
    }
}
```

## Known Limitations

### Code Not Covered in Tests

1. **Error handling for I/O operations** - Hard to mock file system/network
2. **Platform-specific code** - Different coverage on Windows/macOS/Linux
3. **Interactive UI components** - UI testing is complex
4. **Unused code paths** - Dead code shows as uncovered

### Coverage Tools Limitations

- **Macros** - Tarpaulin may miss coverage in macro-generated code
- **Inline assembly** - Not covered by LLVM instrumentation
- **FFI calls** - Can't instrument foreign code

## Improving Coverage

To improve coverage metrics:

1. **Review uncovered lines** - Are they important?
2. **Add focused tests** - Don't just test for coverage's sake
3. **Refactor complex functions** - Smaller functions are easier to test
4. **Add snapshot tests** - Useful for complex output

## Resources

- [Tarpaulin Documentation](https://github.com/xd009642/tarpaulin)
- [codecov.io Documentation](https://docs.codecov.io/)
- [Rust Testing Guide](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [Coverage Best Practices](https://martinfowler.com/bliki/TestCoverage.html)

## Contributing

When contributing code:

1. Run coverage locally: `cargo tarpaulin`
2. Aim for 70%+ coverage on new code
3. Don't sacrifice code quality for coverage metrics
4. Add tests for new functionality
5. Review uncovered lines in your changes

Good coverage = confidence in code quality! ✨
