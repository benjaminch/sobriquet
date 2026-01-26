class Sobriquet < Formula
  desc "Fuzzy finder for shell aliases"
  homepage "https://github.com/benjaminch/sobriquet"
  url "https://github.com/benjaminch/sobriquet/archive/refs/tags/v0.2.0.tar.gz"
  sha256 "c0bb22d8c5aa536074685e3ad2d9b47bda2f40abc52d4ac278e50704fb49a9b8"
  license "MIT"

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
