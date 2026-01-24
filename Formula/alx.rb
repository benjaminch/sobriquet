# frozen_string_literal: true

class Alxrs < Formula
  desc "Fuzzy finder for shell aliases"
  homepage "https://github.com/benjaminch/alxrs"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/benjaminch/alxrs/releases/download/VERSION/alxrs-VERSION-aarch64-apple-darwin.tar.gz"
      sha256 "SHA256_ARM64"
    end
    on_intel do
      url "https://github.com/benjaminch/alxrs/releases/download/VERSION/alxrs-VERSION-x86_64-apple-darwin.tar.gz"
      sha256 "SHA256_X86_64"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/benjaminch/alxrs/releases/download/VERSION/alxrs-VERSION-armv7-unknown-linux-gnueabihf.tar.gz"
      sha256 "SHA256_ARMV7"
    end
    on_intel do
      url "https://github.com/benjaminch/alxrs/releases/download/VERSION/alxrs-VERSION-x86_64-unknown-linux-musl.tar.gz"
      sha256 "SHA256_LINUX_X86_64"
    end
  end

  def install
    # Extract binary from archive
    bin.install "alxrs-VERSION-*/alxrs"
  end

  def post_install
    # Generate shell completions
    bash_completion.install_symlink lib/"alxrs/completions/alxrs.bash" => "alxrs"
    zsh_completion.install_symlink lib/"alxrs/completions/_alxrs" => "_alxrs"
    fish_completion.install_symlink lib/"alxrs/completions/alxrs.fish" => "alxrs.fish"
  end

  test do
    # Verify binary works
    assert_match version.to_s, shell_output("#{bin}/alxrs --version")
  end
end
