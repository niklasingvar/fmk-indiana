# Authoritative copy of the menulet cask. The release workflow copies this into
# the tap repo (niklasingvar/homebrew-fmk-indiana) and fills in version/sha256.
# The .app bundles the `indiana` daemon as a Tauri sidecar, so installing this
# cask alone gives a fully working menubar app — no separate CLI install needed.
#
# The build is UNSIGNED, so strip quarantine after install (Homebrew 6.x dropped
# `--no-quarantine` as a working CLI flag — see docs/DISTRO.md):
#   brew install --cask indiana-menulet
#   xattr -dr com.apple.quarantine /Applications/Indiana.app
cask "indiana-menulet" do
  version "0.1.0"
  sha256 "0000000000000000000000000000000000000000000000000000000000000000"

  url "https://github.com/niklasingvar/fmk-indiana/releases/download/v#{version}/Indiana_#{version}_aarch64.dmg"
  name "Indiana"
  desc "Menubar view onto the Indiana marker server"
  homepage "https://github.com/niklasingvar/fmk-indiana"

  depends_on macos: :ventura
  depends_on arch: :arm64

  app "Indiana.app"

  zap trash: [
    "~/.indiana",
    "~/Library/LaunchAgents/com.niklas.indiana.plist",
  ]
end
