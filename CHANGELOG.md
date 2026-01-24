# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2025-01-24

### Added

- Initial release
- Interactive fuzzy finder for shell aliases using skim
- Support for zsh, bash, and fish shells
- Automatic shell detection (tries zsh first, then bash)
- `--list` flag to list all aliases without interactive selection
- `--shell` flag to specify which shell to use
- `init` subcommand to generate shell integration scripts
  - `alx init zsh` - Generate zsh integration
  - `alx init bash` - Generate bash integration  
  - `alx init fish` - Generate fish integration
- `generate` subcommand for shell completions and man page
  - `alx generate complete-zsh` - Zsh completions
  - `alx generate complete-bash` - Bash completions
  - `alx generate complete-fish` - Fish completions
  - `alx generate man` - Man page
- Proper error handling with meaningful error messages
- Exit codes: 0 (success), 1 (user cancelled), 2 (error)
- Cross-platform support (macOS, Linux, Windows)
- GitHub Actions CI/CD pipeline
  - Automated testing on Linux, macOS, and Windows
  - Cross-compilation for multiple targets
  - Automated releases with binary artifacts
- Comprehensive test suite (32 unit tests)
- Performance benchmarks
- Man page documentation
- Semantic commit enforcement

### Security

- No unsafe code (`#![forbid(unsafe_code)]`)
- Static linking for portable binaries

[unreleased]: https://github.com/benjaminch/alx/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/benjaminch/alx/releases/tag/v0.1.0
