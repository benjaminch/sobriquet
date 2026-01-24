# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |

## Reporting a Vulnerability

We take security vulnerabilities seriously. If you discover a security issue, please report it responsibly.

### How to Report

**Please do NOT report security vulnerabilities through public GitHub issues.**

Instead, please report them via one of the following methods:

1. **GitHub Security Advisories** (Preferred): Use [GitHub's private vulnerability reporting](https://github.com/benjaminch/alxrs/security/advisories/new)

2. **Email**: Contact the maintainer directly at the email address listed in the [Cargo.toml](Cargo.toml) file

### What to Include

Please include as much of the following information as possible:

- Type of vulnerability (e.g., command injection, path traversal, information disclosure)
- Full paths of source file(s) related to the vulnerability
- Location of the affected source code (tag/branch/commit or direct URL)
- Step-by-step instructions to reproduce the issue
- Proof-of-concept or exploit code (if possible)
- Impact of the issue, including how an attacker might exploit it

### Response Timeline

- **Initial Response**: Within 48 hours, we will acknowledge receipt of your report
- **Status Update**: Within 7 days, we will provide an initial assessment
- **Resolution**: We aim to resolve critical vulnerabilities within 30 days

### Disclosure Policy

- We will work with you to understand and resolve the issue quickly
- We will keep you informed of our progress
- We will credit you in the security advisory (unless you prefer to remain anonymous)
- We ask that you give us reasonable time to address the issue before public disclosure

## Security Best Practices for Users

### Alias Auditing

`alx` includes a built-in security audit feature to help you identify potential issues in your aliases:

```bash
# Check for embedded secrets and duplicates
alx audit

# Check for secrets only
alx audit secrets
```

This detects:
- Embedded API keys, tokens, and passwords
- AWS credentials
- Known secret formats (GitHub PATs, Slack tokens, etc.)

### Recommendations

1. **Review aliases regularly**: Use `alxrs audit` to check for accidentally committed secrets
2. **Use environment variables**: Instead of hardcoding secrets in aliases, reference environment variables
3. **Verify alias sources**: Only source alias files from trusted locations

## Security Features

- **No unsafe code**: The codebase uses `#![forbid(unsafe_code)]`
- **Dependency auditing**: We use `cargo-audit` in CI to check for known vulnerabilities
- **Minimal dependencies**: We keep dependencies to a minimum to reduce attack surface
- **MSRV 1.92**: We maintain a modern Rust version with latest security fixes
