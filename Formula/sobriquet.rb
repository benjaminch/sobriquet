class Sobriquet < Formula
  desc "Fuzzy finder for shell aliases"
  homepage "https://github.com/benjaminch/sobriquet"
  url "https://github.com/benjaminch/sobriquet/archive/refs/tags/v0.4.1.tar.gz"
  sha256 "9d775a4ae963e09f5986c57e79a6a3ed6bf81ad395c7f1d3abc84e7947c611b8"
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
