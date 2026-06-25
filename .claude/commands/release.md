---
description: Cut a new Indiana release (CLI + menulet) to the Homebrew tap
argument-hint: <version, e.g. 0.1.1>
allowed-tools: Bash(git:*), Bash(gh:*), Bash(cargo:*), Bash(make:*), Bash(npm:*), Read, Edit
---

# Release Indiana

Cut a new release of Indiana (CLI + menulet) so testers can `brew upgrade`.
Target version: **$1** (e.g. `0.1.1`). If no version was given, ask for one.

Drive these steps in order. **Stop and report if any step fails — never continue past a
failure.** The irreversible, outward-facing steps are 6+; do not run them before the
confirm gate in step 5.

## 0. Preconditions
- Branch is `main` and the working tree is clean (`git status`). If not, stop.
- `gh auth status` is logged in.
- Read the current version from `MENULET/src-tauri/tauri.conf.json`. The target `$1` must
  be greater (semver). If not, stop.
- Relies on one-time setup: the tap repo `niklasingvar/homebrew-fmk-indiana`, the
  `HOMEBREW_TAP_TOKEN` secret (a PAT with **Contents: write** on the tap), and
  `.github/workflows/release.yml`. See `docs/DISTRO.md`.

## 1. Tests
- Run `cargo test --release`. The release workflow does **not** run tests, so this is the
  gate. Abort on any failure.

## 2. Bump the version
Set the version to `$1` in all six manifests (replace the existing `version` value):
- `crates/core/Cargo.toml`
- `crates/indiana/Cargo.toml`
- `crates/indiana-protocol/Cargo.toml`
- `MENULET/src-tauri/Cargo.toml`
- `MENULET/src-tauri/tauri.conf.json`
- `MENULET/package.json`

Refresh the lockfiles so they match:
- `cargo build --release` (updates `Cargo.lock`)
- `npm install --prefix MENULET` (updates `MENULET/package-lock.json`)

## 3. Optional pre-flight (offer to skip — it's slow)
- `make dist` builds the full bundle (CLI tarball + menulet `.dmg`) and prints both
  SHA256s. CI rebuilds these regardless, so this is only a local sanity check.

## 4. Commit
- Commit the manifest + lockfile changes with message `chore(release): v$1` and the
  `Co-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>` trailer.

## 5. Confirm before publishing
- Summarize: the new version, the files changed, and that the next step publishes a
  **public** GitHub release and updates the Homebrew tap.
- Proceed only after the user explicitly says yes.

## 6. Tag and push
```sh
git push origin main
git tag v$1
git push origin v$1
```
The pushed tag triggers `.github/workflows/release.yml`.

## 7. Watch CI (do NOT mask the exit code)
- Get the run id:
  `gh run list --workflow release.yml -L1 --json databaseId -q '.[0].databaseId'`
- Watch it: `gh run watch <id> --repo niklasingvar/fmk-indiana --exit-status`
- **Do not** pipe `gh run watch` through `grep`/`tail` — that hides its exit code and once
  let a failed release look green. Check the result directly.

## 8. Verify the outcome (a green run is not enough)
- Release assets exist:
  `gh release view v$1 --repo niklasingvar/fmk-indiana --json assets -q '.assets[].name'`
  → expect `indiana-aarch64-apple-darwin.tar.gz` and `Indiana_$1_aarch64.dmg`.
- The tap was bumped to **real** checksums (not zeros):
  `gh api repos/niklasingvar/homebrew-fmk-indiana/contents/Formula/indiana.rb -q '.content' | base64 -d | grep -E 'version|sha256'`
  and the same for `Casks/indiana-menulet.rb`.
- If the tap still shows `0000…` or the old version, the bump step failed (commonly the
  `HOMEBREW_TAP_TOKEN` lacks Contents: write). Report it, then fix by re-running the failed
  job (`gh run rerun --failed <id>`) after correcting the token, or update the two tap
  files' shas by hand from the run's SHA256 summary.

## 9. Done
Tell the user it's live and how to get it:
```sh
brew update
brew upgrade --cask indiana-menulet   # or: brew install --cask --no-quarantine indiana-menulet
brew upgrade indiana                   # CLI
```
