class Markplane < Formula
  desc "AI-native, markdown-first project management"
  homepage "https://github.com/zerowand01/markplane"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/zerowand01/markplane/releases/download/v%%VERSION%%/markplane-v%%VERSION%%-aarch64-apple-darwin.tar.gz"
      sha256 "%%SHA_AARCH64_APPLE_DARWIN%%"
    end
    on_intel do
      url "https://github.com/zerowand01/markplane/releases/download/v%%VERSION%%/markplane-v%%VERSION%%-x86_64-apple-darwin.tar.gz"
      sha256 "%%SHA_X86_64_APPLE_DARWIN%%"
    end
  end

  on_linux do
    on_intel do
      url "https://github.com/zerowand01/markplane/releases/download/v%%VERSION%%/markplane-v%%VERSION%%-x86_64-unknown-linux-musl.tar.gz"
      sha256 "%%SHA_X86_64_UNKNOWN_LINUX_MUSL%%"
    end
  end

  def install
    bin.install "markplane"
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/markplane --version")
  end
end
