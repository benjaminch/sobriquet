# Release Process

This document describes the automated release process for sobriquet.

## Quick Release (Fully Automated)

The easiest way to create a release:

```bash
./scripts/bump-version.sh 0.3.0
```

This script will:
1. Update version in `Cargo.toml`
2. Update `Cargo.lock`
3. Update `Formula/sobriquet.rb` (URL to new version)
4. Run tests
5. Create a commit
6. Create a git tag
7. Push to GitHub (optional)

Once pushed, the release workflow automatically:
- Builds binaries for all platforms
- Creates GitHub release
- Publishes to crates.io

## Alternative: Manual Release with PR

If you want to review changes before releasing:

1. Go to [Actions → Prepare Release](https://github.com/benjaminch/sobriquet/actions/workflows/prepare-release.yml)
2. Click "Run workflow"
3. Enter the new version (e.g., `0.3.0`)
4. Review the auto-created PR
5. Merge the PR when ready
6. Create and push the tag manually:
   ```bash
   git checkout main
   git pull
   git tag -a v0.3.0 -m "Release 0.3.0"
   git push origin v0.3.0
   ```

## What Happens Automatically

### On Tag Push (`v*.*.*`)

The `release.yml` workflow triggers and:

1. **Creates Draft Release**
   - Validates version matches Cargo.toml
   - Generates changelog from commits
   - Creates GitHub release (draft)

2. **Builds Binaries** (parallel)
   - Linux: x86_64, aarch64, armv7
   - macOS: x86_64 (Intel), aarch64 (Apple Silicon)
   - Uploads binaries with SHA256 checksums

3. **Publishes to crates.io**
   - Uses `CARGO_REGISTRY_TOKEN` secret

4. **Publishes Release**
   - Changes draft to published

## Required Secrets

Configure these in [Settings → Secrets and variables → Actions](https://github.com/benjaminch/sobriquet/settings/secrets/actions):

- `CARGO_REGISTRY_TOKEN`: Token from [crates.io](https://crates.io/settings/tokens)
  - Create with: "Publish new crates and updates"
  
- `GITHUB_TOKEN`: Automatically provided by GitHub
  - Used for: Creating releases, updating Homebrew tap

## Homebrew Installation

After release, users can install via:

```bash
brew install benjaminch/sobriquet/sobriquet
```

## Version Scheme

We follow [Semantic Versioning](https://semver.org/):

- **MAJOR** (1.0.0): Breaking changes
- **MINOR** (0.3.0): New features, backwards compatible
- **PATCH** (0.2.1): Bug fixes, backwards compatible

## Troubleshooting

### Build Fails

Check the [Actions tab](https://github.com/benjaminch/sobriquet/actions) for logs.

Common issues:
- Version mismatch between tag and Cargo.toml
- Tests failing
- Cross-compilation issues

### crates.io Publish Fails

Check that:
- `CARGO_REGISTRY_TOKEN` is set correctly
- Version doesn't already exist on crates.io
- Cargo.toml is valid

## Rollback

If a release has issues:

1. **Delete the release** (on GitHub)
2. **Delete the tag**:
   ```bash
   git tag -d v0.3.0
   git push origin :refs/tags/v0.3.0
   ```
3. **Fix issues and re-release**

Note: You cannot re-publish the same version to crates.io. You'll need to bump to the next patch version.

## Post-Release Checklist

After a successful release:

- [ ] Test Homebrew installation: `brew install benjaminch/sobriquet/sobriquet`
- [ ] Test crates.io installation: `cargo install sobriquet`
- [ ] Verify GitHub release has all binaries
- [ ] Test binary downloads work
- [ ] Update documentation if needed
- [ ] Announce on social media (optional)

## Example Release Timeline

```
10:00 - Run: ./scripts/bump-version.sh 0.3.0
10:01 - Version commit created and pushed
10:02 - Tag pushed, release workflow triggered
10:05 - Binaries built for all platforms
10:08 - Published to crates.io
10:10 - GitHub release published
10:15 - Test installations (brew, cargo)
```

Total time: ~15 minutes (mostly automated)
