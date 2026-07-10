# Authoritative copy of the CLI formula. The release workflow copies this into
# the tap repo (niklasingvar/homebrew-fmk-indiana) and fills in url/sha256/version.
# Installs the standalone `indiana` CLI + daemon. The menulet cask bundles its own
# daemon, so this formula is only needed by terminal users (or power users who want
# a PATH binary the menulet will prefer when newer).
class Indiana < Formula
  desc "Scan markdown for :: markers and expose agent-readable payloads"
  homepage "https://github.com/niklasingvar/fmk-indiana"
  url "https://github.com/niklasingvar/fmk-indiana/releases/download/v0.1.0/indiana-aarch64-apple-darwin.tar.gz"
  version "0.1.0"
  sha256 "0000000000000000000000000000000000000000000000000000000000000000"
  license "MIT"

  depends_on arch: :arm64
  depends_on macos: :ventura
  # Auto-run (IN_AUTORUN.md) dispatches `::fix -a` markers to Claude Code's ACP
  # adapter, which the daemon launches via `npx -y @zed-industries/claude-code-acp`.
  # Node provides `npx`; the adapter itself is fetched and cached on first use, so
  # there is nothing to pin here. Auto-run is off by default (config.auto_run).
  depends_on "node"

  def install
    bin.install "indiana"
  end

  test do
    assert_match "indiana", shell_output("#{bin}/indiana --version")
  end
end
