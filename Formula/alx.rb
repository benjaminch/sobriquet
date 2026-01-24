# frozen_string_literal: true

class Alx < Formula
  desc "Fuzzy finder for shell aliases"
  homepage "https://github.com/benjaminch/alx"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/benjaminch/alx/releases/download/VERSION/alx-VERSION-aarch64-apple-darwin.tar.gz"
      sha256 "SHA256_ARM64"
    end
    on_intel do
      url "https://github.com/benjaminch/alx/releases/download/VERSION/alx-VERSION-x86_64-apple-darwin.tar.gz"
      sha256 "SHA256_X86_64"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/benjaminch/alx/releases/download/VERSION/alx-VERSION-armv7-unknown-linux-gnueabihf.tar.gz"
      sha256 "SHA256_ARMV7"
    end
    on_intel do
      url "https://github.com/benjaminch/alx/releases/download/VERSION/alx-VERSION-x86_64-unknown-linux-musl.tar.gz"
      sha256 "SHA256_LINUX_X86_64"
    end
  end

  def install
    # Extract binary from archive
    bin.install "alx-VERSION-*/alx"
  end

  def post_install
    # Generate shell completions
    bash_completion.install_symlink lib/"alx/completions/alx.bash" => "alx"
    zsh_completion.install_symlink lib/"alx/completions/_alx" => "_alx"
    fish_completion.install_symlink lib/"alx/completions/alx.fish" => "alx.fish"
  end

  test do
    # Verify binary works
    assert_match version.to_s, shell_output("#{bin}/alx --version")
  end
end
