# Quick Release Guide

## TL;DR - Release in 30 Seconds

```bash
# One command to rule them all
make release VERSION=0.3.0

# Or directly
./scripts/bump-version.sh 0.3.0
```

That's it! The script will:
1. ✅ Update `Cargo.toml` version
2. ✅ Update `Cargo.lock`
3. ✅ Run tests
4. ✅ Create commit: `chore: release version 0.3.0`
5. ✅ Create tag: `v0.3.0`
6. ✅ Push to GitHub (asks first)

Then automatically:
- 🤖 Build binaries (Linux x3, macOS x2)
- 🤖 Publish to crates.io
- 🤖 Update Homebrew formula
- 🤖 Create GitHub release

## Usage Examples

### Using Make (Recommended)
```bash
make release VERSION=0.3.0
```

### Using Script Directly
```bash
./scripts/bump-version.sh 0.3.0
```

### See What Changed
```bash
# After running bump-version.sh
git show HEAD              # See the commit
git show v0.3.0            # See the tag
git diff v0.2.0..v0.3.0    # Compare versions
```

### Manual Push (if you said "no" to auto-push)
```bash
git push origin main --follow-tags
```

## What Gets Updated Automatically

### 1. Cargo.toml
```diff
- version = "0.2.0"
+ version = "0.3.0"
```

### 2. Git Commit
```
chore: release version 0.3.0
```

### 3. Git Tag
```
v0.3.0 (annotated tag with message "Release 0.3.0")
```

### 4. Cargo.lock
All dependencies locked to new version

## Release Checklist (Done Automatically)

When you push the tag, GitHub Actions will:

- [ ] Validate version matches Cargo.toml
- [ ] Build for Linux x86_64 (musl)
- [ ] Build for Linux aarch64
- [ ] Build for Linux armv7
- [ ] Build for macOS x86_64 (Intel)
- [ ] Build for macOS aarch64 (Apple Silicon)
- [ ] Create GitHub release with changelog
- [ ] Upload all binaries with SHA256 checksums
- [ ] Publish to crates.io
- [ ] Update Homebrew formula in homebrew-tap repo
- [ ] Make release public (not draft)

## Common Commands

```bash
# Check current version
grep '^version' Cargo.toml

# See all tags
git tag -l

# See recent releases
gh release list

# Check release workflow status
gh run list --workflow=release.yml

# Test Homebrew install after release
brew install benjaminch/tap/sobriquet

# Test crates.io install after release
cargo install sobriquet
```

## Troubleshooting

### "Version already exists"
```bash
# Check existing tags
git tag -l

# Use next version number
./scripts/bump-version.sh 0.3.1
```

### "Tests failed"
The script automatically reverts changes. Fix tests, then try again.

### "Uncommitted changes"
The script will warn you but allows continuing. Commit or stash first:
```bash
git stash
./scripts/bump-version.sh 0.3.0
git stash pop
```

### Want to undo?
If you haven't pushed yet:
```bash
# Undo commit and tag
git reset --hard HEAD~1
git tag -d v0.3.0

# Revert Cargo.toml
git checkout HEAD~1 -- Cargo.toml Cargo.lock
```

## Pro Tips

### Test Before Releasing
```bash
make check          # Run fmt, lint, test
make test-all       # Run all tests
cargo build --release  # Ensure release builds
```

### Preview What Would Change
```bash
# Dry run (manual)
git diff
cargo build --release
cargo test
```

### Schedule Releases
```bash
# Release every Friday at 10 AM
# (Add to your calendar/crontab)
make release VERSION=x.y.z
```

## Full Release Example

```bash
# 1. Ensure working directory is clean
git status

# 2. Run checks
make check

# 3. Bump version and create release
make release VERSION=0.3.0
# Or: ./scripts/bump-version.sh 0.3.0

# 4. Script asks: "Push to origin now? (y/N)"
# Type 'y' and press Enter

# 5. Wait ~10 minutes for automation

# 6. Verify release
gh release view v0.3.0
brew install benjaminch/tap/sobriquet
```

## See Also

- [RELEASING.md](./RELEASING.md) - Detailed release documentation
- [GitHub Actions](https://github.com/benjaminch/sobriquet/actions) - Monitor workflows
- [Releases](https://github.com/benjaminch/sobriquet/releases) - All releases
