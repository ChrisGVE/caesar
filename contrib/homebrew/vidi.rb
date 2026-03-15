# This file is a template. The tap auto-updater replaces version and sha256 values.
class Vidi < Formula
  desc "Universal terminal file viewer"
  homepage "https://github.com/ChrisGVE/vidi"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/ChrisGVE/vidi/releases/download/v#{version}/vidi-v#{version}-aarch64-apple-darwin.tar.gz"
      sha256 "PLACEHOLDER"
    end

    on_intel do
      url "https://github.com/ChrisGVE/vidi/releases/download/v#{version}/vidi-v#{version}-x86_64-apple-darwin.tar.gz"
      sha256 "PLACEHOLDER"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/ChrisGVE/vidi/releases/download/v#{version}/vidi-v#{version}-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "PLACEHOLDER"
    end

    on_intel do
      url "https://github.com/ChrisGVE/vidi/releases/download/v#{version}/vidi-v#{version}-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "PLACEHOLDER"
    end
  end

  def install
    bin.install "vidi"
  end
end
