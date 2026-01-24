# Homebrew Distribution Setup

This document explains how to set up `alx` for distribution via Homebrew.

## Overview

The `alx` project uses a two-repository approach for Homebrew distribution:

1. **Main Repository** (`benjaminch/alx`) - Contains the source code and release automation
2. **Homebrew Tap** (`benjaminch/homebrew-alx`) - Contains Homebrew formulas and is updated automatically during releases

## Automated Release Process

When you push a semantic version tag (e.g., `1.0.0`), the CI/CD pipeline automatically:

1. Creates a GitHub release with pre-built binaries for multiple platforms
2. Publishes the package to crates.io
3. Triggers an update workflow in the Homebrew tap repository
4. Updates the formula with new checksums and release URLs

## Manual Setup (First Time Only)

### Prerequisites

- A GitHub account
- `brew` installed locally
- Permissions to create/manage GitHub repositories

### Step 1: Create the Homebrew Tap Repository

```bash
# Create a new repository named "homebrew-alx" on GitHub
# Clone it locally
git clone https://github.com/YOUR_USERNAME/homebrew-alx.git
cd homebrew-alx
```

### Step 2: Create the Base Formula Structure

```bash
mkdir -p Formula
cat > Formula/alx.rb << 'EOF'
# frozen_string_literal: true

class Alx < Formula
  desc "Fuzzy finder for shell aliases"
  homepage "https://github.com/benjaminch/alx"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/benjaminch/alx/releases/download/1.0.0/alx-1.0.0-aarch64-apple-darwin.tar.gz"
      sha256 "SHA256_HASH_HERE"
    end
    on_intel do
      url "https://github.com/benjaminch/alx/releases/download/1.0.0/alx-1.0.0-x86_64-apple-darwin.tar.gz"
      sha256 "SHA256_HASH_HERE"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/benjaminch/alx/releases/download/1.0.0/alx-1.0.0-armv7-unknown-linux-gnueabihf.tar.gz"
      sha256 "SHA256_HASH_HERE"
    end
    on_intel do
      url "https://github.com/benjaminch/alx/releases/download/1.0.0/alx-1.0.0-x86_64-unknown-linux-musl.tar.gz"
      sha256 "SHA256_HASH_HERE"
    end
  end

  def install
    bin.install "alx-1.0.0-*/alx"
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/alx --version")
  end
end
EOF
```

### Step 3: Create the Update Workflow

Create `.github/workflows/update-formula.yml`:

```yaml
name: Update Formula

on:
  workflow_dispatch:
    inputs:
      version:
        description: 'Version to update to'
        required: true
        type: string

permissions:
  contents: write

jobs:
  update:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout
        uses: actions/checkout@v4

      - name: Download release assets
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        run: |
          version="${{ inputs.version }}"
          mkdir -p /tmp/assets
          
          # Download SHA256 files
          gh release download "$version" \
            --repo benjaminch/alx \
            --pattern "*.sha256" \
            --dir /tmp/assets

      - name: Calculate checksums
        run: |
          version="${{ inputs.version }}"
          
          # Extract checksums from downloaded files
          aarch64_darwin=$(cat /tmp/assets/alx-$version-aarch64-apple-darwin.tar.gz.sha256 | awk '{print $1}')
          x86_64_darwin=$(cat /tmp/assets/alx-$version-x86_64-apple-darwin.tar.gz.sha256 | awk '{print $1}')
          armv7_linux=$(cat /tmp/assets/alx-$version-armv7-unknown-linux-gnueabihf.tar.gz.sha256 | awk '{print $1}')
          x86_64_linux=$(cat /tmp/assets/alx-$version-x86_64-unknown-linux-musl.tar.gz.sha256 | awk '{print $1}')
          
          # Create environment variables
          echo "AARCH64_DARWIN=$aarch64_darwin" >> $GITHUB_ENV
          echo "X86_64_DARWIN=$x86_64_darwin" >> $GITHUB_ENV
          echo "ARMV7_LINUX=$armv7_linux" >> $GITHUB_ENV
          echo "X86_64_LINUX=$x86_64_linux" >> $GITHUB_ENV

      - name: Update formula
        env:
          VERSION: ${{ inputs.version }}
          AARCH64_DARWIN: ${{ env.AARCH64_DARWIN }}
          X86_64_DARWIN: ${{ env.X86_64_DARWIN }}
          ARMV7_LINUX: ${{ env.ARMV7_LINUX }}
          X86_64_LINUX: ${{ env.X86_64_LINUX }}
        run: |
          cat > Formula/alx.rb << 'EOF'
# frozen_string_literal: true

class Alx < Formula
  desc "Fuzzy finder for shell aliases"
  homepage "https://github.com/benjaminch/alx"
  license "MIT"
  version "$VERSION"

  on_macos do
    on_arm do
      url "https://github.com/benjaminch/alx/releases/download/$VERSION/alx-$VERSION-aarch64-apple-darwin.tar.gz"
      sha256 "$AARCH64_DARWIN"
    end
    on_intel do
      url "https://github.com/benjaminch/alx/releases/download/$VERSION/alx-$VERSION-x86_64-apple-darwin.tar.gz"
      sha256 "$X86_64_DARWIN"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/benjaminch/alx/releases/download/$VERSION/alx-$VERSION-armv7-unknown-linux-gnueabihf.tar.gz"
      sha256 "$ARMV7_LINUX"
    end
    on_intel do
      url "https://github.com/benjaminch/alx/releases/download/$VERSION/alx-$VERSION-x86_64-unknown-linux-musl.tar.gz"
      sha256 "$X86_64_LINUX"
    end
  end

  def install
    bin.install "alx-$VERSION-*/alx"
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/alx --version")
  end
end
EOF

      - name: Commit and push
        run: |
          git config user.name "github-actions"
          git config user.email "github-actions@github.com"
          git add Formula/alx.rb
          git commit -m "chore: update alx formula to ${{ inputs.version }}"
          git push
```

### Step 4: Add PAT Token

In the main `benjaminch/alx` repository:

1. Go to Settings → Secrets and variables → Actions
2. Create a new secret called `HOMEBREW_TAP_TOKEN`
3. Generate a GitHub Personal Access Token with `repo` and `workflow` permissions
4. Paste it as the secret value

### Step 5: Create Initial Release

```bash
# Bump version in Cargo.toml and commit
git tag 1.0.0
git push origin main
git push origin 1.0.0
```

This will trigger the release workflow which will:
1. Build binaries
2. Create GitHub release
3. Update the Homebrew formula automatically

## Installation for Users

Once set up, users can install `alx` via Homebrew:

```bash
# Add tap
brew tap benjaminch/homebrew-alx

# Install
brew install alx

# Upgrade
brew upgrade alx
```

## Troubleshooting

### Formula Update Fails

- Check that the `HOMEBREW_TAP_TOKEN` is valid
- Verify the workflow file exists in the tap repository
- Check GitHub Actions logs in the tap repository

### Checksum Mismatch

- Ensure SHA256 files are uploaded with release assets
- Verify the checksums haven't changed between downloads

### Installation Fails

- Run `brew doctor` to check for issues
- Try `brew tap --force-auto-update benjaminch/homebrew-alx`
- Check Formula/alx.rb for syntax errors with `brew audit Formula/alx.rb`

## References

- [Homebrew Formula Cookbook](https://docs.brew.sh/Formula-Cookbook)
- [Homebrew TAP](https://docs.brew.sh/Taps)
- [GitHub Releases Documentation](https://docs.github.com/en/repositories/releasing-projects-on-github/managing-releases-in-a-repository)
