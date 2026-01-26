# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Safe preview mode: commands shown without expansion by default
- Dynamic toggle with Ctrl+X to expand variables and command substitutions on-demand
- Preview expansion configuration option (`preview_expand_details`)
- Automatic secret masking in preview (tokens, API keys, passwords)
- Enhanced security with secret detection in audit command
- Improved command breakdown with expandable details

### Changed

- Preview now shows raw commands by default for security
- Better handling of environment variables in preview
- More comprehensive command analysis and warnings

## [0.1.0] - 2025-01-24

### Added

- Initial release
- Interactive fuzzy finder for shell aliases using skim
- Support for zsh, bash, and fish shells
- Automatic shell detection (tries zsh first, then bash)
- `--list` flag to list all aliases without interactive selection
- `--shell` flag to specify which shell to use
- `init` subcommand to generate shell integration scripts
  - `sobriquet init zsh` - Generate zsh integration
  - `sobriquet init bash` - Generate bash integration  
  - `sobriquet init fish` - Generate fish integration
- `generate` subcommand for shell completions and man page
  - `sobriquet generate complete-zsh` - Zsh completions
  - `sobriquet generate complete-bash` - Bash completions
  - `sobriquet generate complete-fish` - Fish completions
  - `sobriquet generate man` - Man page
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

[unreleased]: https://github.com/benjaminch/sobriquet/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/benjaminch/sobriquet/releases/tag/v0.1.0
