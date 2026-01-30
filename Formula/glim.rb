class Glim < Formula
  desc "Next-generation Git Diff CLI tool with semantic zooming and AI integration"
  homepage "https://github.com/kanywst/glim"
  url "https://github.com/kanywst/glim/archive/refs/tags/v0.1.0.tar.gz"
  sha256 "0000000000000000000000000000000000000000000000000000000000000000"
  license "MIT"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args
  end

  test do
    assert_match "glim #{version}", shell_output("#{bin}/glim --version")
  end
end
