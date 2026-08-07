class SerialCli < Formula
  desc "Universal serial port CLI tool optimized for AI interaction"
  homepage "https://github.com/transmit-bug/serial_cli"
  url "https://github.com/transmit-bug/serial_cli/archive/refs/tags/${VERSION}.tar.gz"
  sha256 "${SHA256}"
  license any_of: ["MIT", "Apache-2.0"]
  depends_on "rust" => :build
  def install
    system "cargo", "install", "--path", "."
  end
  test do
    system "#{bin}/serial-cli", "--version"
  end
end
