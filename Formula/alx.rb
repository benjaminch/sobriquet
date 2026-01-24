# frozen_string_literal: true

class Sobriquet < Formula
  desc "Fuzzy finder for shell aliases"
  homepage "https://github.com/benjaminch/sobriquet"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/benjaminch/sobriquet/releases/download/VERSION/sobriquet-VERSION-aarch64-apple-darwin.tar.gz"
      sha256 "SHA256_ARM64"
    end
    on_intel do
      url "https://github.com/benjaminch/sobriquet/releases/download/VERSION/sobriquet-VERSION-x86_64-apple-darwin.tar.gz"
      sha256 "SHA256_X86_64"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/benjaminch/sobriquet/releases/download/VERSION/sobriquet-VERSION-armv7-unknown-linux-gnueabihf.tar.gz"
      sha256 "SHA256_ARMV7"
    end
    on_intel do
      url "https://github.com/benjaminch/sobriquet/releases/download/VERSION/sobriquet-VERSION-x86_64-unknown-linux-musl.tar.gz"
      sha256 "SHA256_LINUX_X86_64"
    end
  end

  def install
    # Extract binary from archive
    bin.install "sobriquet-VERSION-*/sobriquet"
  end

  def post_install
    # Generate shell completions
    bash_completion.install_symlink lib/"sobriquet/completions/sobriquet.bash" => "sobriquet"
    zsh_completion.install_symlink lib/"sobriquet/completions/_sobriquet" => "_sobriquet"
    fish_completion.install_symlink lib/"sobriquet/completions/sobriquet.fish" => "sobriquet.fish"
  end

  test do
    # Verify binary works
    assert_match version.to_s, shell_output("#{bin}/sobriquet --version")
  end
end
