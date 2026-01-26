# Automation Summary

This document provides an overview of all automation in the sobriquet project.

## Release Automation (Full End-to-End)

### Single Command Release

```bash
make release VERSION=0.3.0
```

**What happens:**

1. **Local (your machine)**
   - Updates `Cargo.toml` → `version = "0.3.0"`
   - Updates `Cargo.lock`
   - Runs all tests
   - Creates commit: `chore: release version 0.3.0`
   - Creates tag: `v0.3.0`
   - Pushes to GitHub

2. **GitHub Actions (automatic)**
   - ✅ Validates version matches Cargo.toml
   - ✅ Generates changelog from git commits
   - ✅ Creates draft GitHub release
   
3. **Build Binaries (parallel, ~5 minutes)**
   - ✅ Linux x86_64 (musl)
   - ✅ Linux aarch64 (ARM64)
   - ✅ Linux armv7 (32-bit ARM)
   - ✅ macOS x86_64 (Intel)
   - ✅ macOS aarch64 (Apple Silicon)
   - ✅ Each with SHA256 checksum
   - ✅ Uploaded to GitHub release

4. **Publish to crates.io**
   - ✅ Publishes crate
   - ✅ Users can: `cargo install sobriquet`

5. **Update Homebrew Formula**
   - ✅ Checks out `benjaminch/homebrew-tap`
   - ✅ Downloads source tarball
   - ✅ Calculates SHA256
   - ✅ Updates `Formula/sobriquet.rb`
   - ✅ Commits and pushes
   - ✅ Users can: `brew install benjaminch/tap/sobriquet`

6. **Finalize Release**
   - ✅ Makes GitHub release public (not draft)

**Total time:** ~10 minutes (automated)

## CI/CD Workflows

### On Every Push/PR

#### 1. CI (`ci.yml`)
- Runs on: Linux, macOS, Windows
- Tests: Unit + Integration
- Lints: rustfmt + clippy
- Checks: All features, MSRV

#### 2. Semantic PR (`semantic-pr.yml`)
- Validates PR title follows [Conventional Commits](https://www.conventionalcommits.org/)
- Types: `feat`, `fix`, `docs`, `chore`, `refactor`, etc.

#### 3. Commitlint (`commitlint.yml`)
- Validates all commit messages
- Ensures lowercase subjects
- Checks format: `type: description`

#### 4. Coverage (`coverage.yml`)
- Generates code coverage
- Uploads to Codecov
- Comments on PRs with coverage diff

### On Release Tags (`v*.*.*`)

#### Release Workflow (`release.yml`)
Full automated release as described above.

## Local Development Automation

### Git Hooks (Pre-commit)

Install once:
```bash
./scripts/install-hooks.sh
```

Runs before every commit:
- ✅ `cargo fmt --check` - Enforces code formatting
- ✅ `cargo clippy` - Catches common mistakes

### Makefile Commands

Quick access to common tasks:

```bash
make build          # Build debug
make build-release  # Build release
make test           # Run tests
make test-all       # Run all tests
make lint           # Run clippy
make fmt            # Format code
make check          # Run fmt-check + lint + test
make release        # Create new release
make docs           # Build and open docs
make bench          # Run benchmarks
make install        # Install locally
```

## Scripts

### `scripts/bump-version.sh`

The core release script:

```bash
./scripts/bump-version.sh 0.3.0
```

Features:
- ✅ Validates version format (semver)
- ✅ Checks for uncommitted changes
- ✅ Updates Cargo.toml
- ✅ Updates Cargo.lock
- ✅ Runs full test suite
- ✅ Runs clippy
- ✅ Creates commit
- ✅ Creates annotated git tag
- ✅ Optionally pushes to origin
- ✅ Rolls back on any error

### `scripts/install-hooks.sh`

Installs git pre-commit hooks:

```bash
./scripts/install-hooks.sh
```

## Workflow Triggers

| Workflow | Trigger | Purpose |
|----------|---------|---------|
| `ci.yml` | Push, PR | Test on all platforms |
| `release.yml` | Tag `v*.*.*` | Build and publish release |
| `prepare-release.yml` | Manual | Create release PR |
| `semantic-pr.yml` | PR opened/edited | Validate PR title |
| `commitlint.yml` | Push to PR | Validate commit messages |
| `coverage.yml` | Push, PR | Generate coverage report |

## Required Secrets

Configure in [Settings → Secrets](https://github.com/benjaminch/sobriquet/settings/secrets/actions):

| Secret | Purpose | How to Get |
|--------|---------|------------|
| `CARGO_REGISTRY_TOKEN` | Publish to crates.io | [crates.io/settings/tokens](https://crates.io/settings/tokens) |
| `GITHUB_TOKEN` | Create releases, update Homebrew | Auto-provided by GitHub |

## Release Channels

After a release, users can install via:

### Homebrew (macOS/Linux)
```bash
brew install benjaminch/tap/sobriquet
```

### Cargo (All platforms)
```bash
cargo install sobriquet
```

### Direct Download (GitHub Releases)
```bash
# Example for Linux x86_64
curl -LO https://github.com/benjaminch/sobriquet/releases/download/v0.3.0/sobriquet-0.3.0-x86_64-unknown-linux-musl.tar.gz
tar xzf sobriquet-0.3.0-x86_64-unknown-linux-musl.tar.gz
./sobriquet-0.3.0-x86_64-unknown-linux-musl/sobriquet --version
```

## Automation Philosophy

1. **One Command Release** - `make release VERSION=x.y.z`
2. **Safe by Default** - Tests run before creating tags
3. **Automatic Rollback** - Reverts on failure
4. **Multi-Platform** - Builds for 5 platforms automatically
5. **Fast Feedback** - CI runs on every push
6. **Quality Gates** - Linting, formatting, tests required

## Monitoring

### GitHub Actions
- View workflow runs: https://github.com/benjaminch/sobriquet/actions
- Filter by workflow: Click workflow name
- Check logs: Click on any run

### Release Status
```bash
# View recent releases
gh release list

# View specific release
gh release view v0.3.0

# View workflow runs
gh run list --workflow=release.yml
```

### Installation Status
```bash
# Check Homebrew
brew info benjaminch/tap/sobriquet

# Check crates.io
cargo search sobriquet
```

## Troubleshooting Automation

### Release Failed

1. Check [Actions tab](https://github.com/benjaminch/sobriquet/actions)
2. Click on failed workflow
3. Expand failed step
4. Fix issue
5. Delete tag and re-release:
   ```bash
   git tag -d v0.3.0
   git push origin :refs/tags/v0.3.0
   make release VERSION=0.3.0
   ```

### Homebrew Update Failed

Check `homebrew-tap` job logs:
- Repository exists: `benjaminch/homebrew-tap`
- Has correct permissions
- Formula is valid Ruby syntax

### crates.io Publish Failed

- Verify `CARGO_REGISTRY_TOKEN` is set
- Check version doesn't exist already
- Ensure Cargo.toml is valid

### CI Failed on PR

Pre-commit hooks should catch most issues. If CI fails:
```bash
make check  # Run locally
cargo fmt   # Fix formatting
cargo clippy --fix  # Fix clippy warnings
cargo test  # Fix failing tests
```

## Future Enhancements

Potential additions:

- [ ] Automatic changelog in GitHub releases (✅ **Done!**)
- [ ] Windows builds (removed due to Unix-only deps)
- [ ] Docker images
- [ ] Snap packages (Linux)
- [ ] Chocolatey packages (Windows)
- [ ] AUR package (Arch Linux)
- [ ] Release notes from PR descriptions
- [ ] Automatic dependency updates (Dependabot)
- [ ] Performance benchmarks in CI
- [ ] Cross-platform integration tests

## Questions?

- See [docs/QUICK_RELEASE.md](QUICK_RELEASE.md) for quick reference
- See [docs/RELEASING.md](RELEASING.md) for detailed docs
- Open an issue for automation problems
