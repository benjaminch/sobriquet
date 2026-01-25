class Sobriquet < Formula
  desc "Fuzzy finder for shell aliases"
  homepage "https://github.com/benjaminch/sobriquet"
  url "https://github.com/benjaminch/sobriquet/archive/refs/tags/v0.1.1.tar.gz"
  sha256 "0000000000000000000000000000000000000000000000000000000000000000"
  license "MIT"

  bottle do
    root_url "https://github.com/benjaminch/sobriquet/releases/download/v0.1.1"
    sha256 cellar: :any_skip_relocation, arm64_sequoia:  "0000000000000000000000000000000000000000000000000000000000000000"
    sha256 cellar: :any_skip_relocation, arm64_sonoma:   "0000000000000000000000000000000000000000000000000000000000000000"
    sha256 cellar: :any_skip_relocation, arm64_ventura:  "0000000000000000000000000000000000000000000000000000000000000000"
    sha256 cellar: :any_skip_relocation, x86_64_linux:   "0000000000000000000000000000000000000000000000000000000000000000"
  end

  depends_on "rust" => :build

  def install
    system "cargo", "install", "--locked", "--root", prefix, "--path", "."
  end

  def post_install
    shell_integration = Utils.popen_read("#{bin}/sobriquet init bash")
    puts "To set up shell integration, add this to your shell profile:"
    puts "eval \"$(#{bin}/sobriquet init zsh)\"   # for zsh"
    puts "eval \"$(#{bin}/sobriquet init bash)\"  # for bash"
    puts "#{bin}/sobriquet init fish | source      # for fish"
  end

  test do
    system "#{bin}/sobriquet", "--version"
  end
end
