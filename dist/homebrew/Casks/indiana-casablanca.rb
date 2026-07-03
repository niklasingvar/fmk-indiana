# Authoritative copy of the Casablanca cask. The release workflow copies this into
# the tap repo (niklasingvar/homebrew-fmk-indiana) and fills in version/sha256.
#
# Casablanca is the editor: open a vault, edit markdown inline, tag `::` markers,
# hit "Copy all" (shells out to the `indiana` CLI). It does NOT bundle the daemon
# — `depends_on` the `indiana` formula, and talks to whichever daemon is already
# running (the menulet's, or `indiana serve` / `indiana service install`).
#
# The build is UNSIGNED, so strip quarantine after install (Homebrew 6.x dropped
# `--no-quarantine` as a working CLI flag — see docs/DISTRO.md):
#   brew install --cask indiana-casablanca
#   xattr -dr com.apple.quarantine /Applications/Casablanca.app
cask "indiana-casablanca" do
  version "0.1.0"
  sha256 "0000000000000000000000000000000000000000000000000000000000000000"

  url "https://github.com/niklasingvar/fmk-indiana/releases/download/v#{version}/Casablanca_#{version}_aarch64.dmg"
  name "Casablanca"
  desc "Indiana's markdown editor — inline WYSIWYG, :: marker tagging, Copy all"
  homepage "https://github.com/niklasingvar/fmk-indiana"

  depends_on macos: :ventura
  depends_on arch: :arm64
  depends_on formula: "niklasingvar/fmk-indiana/indiana"

  app "Casablanca.app"

  zap trash: [
    "~/Library/Application Support/Casablanca",
  ]
end
