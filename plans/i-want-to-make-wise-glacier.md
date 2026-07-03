---
status: superseded
purpose: Archived plan — /release slash command (executed).
approval: pending
---

# Add a `/release` slash command (Claude Code + Cursor)

## Context

We just built the Homebrew shipping pipeline and cut `v0.1.0`. Cutting the *next*
version should be a single command in either editor. This task adds a project-level
`/release` slash command, mirrored for both tools:

- Claude Code: `.claude/commands/release.md`
- Cursor: `.cursor/commands/release.md`

It drives the whole release: run tests → bump the version everywhere → commit →
(confirm) → tag + push → watch CI → verify the GitHub release **and** the Homebrew
tap bump. One-time setup is already done (tap repo `niklasingvar/homebrew-fmk-indiana`,
`HOMEBREW_TAP_TOKEN` secret, `.github/workflows/release.yml`).

### ⚠ Blocker uncovered while shipping v0.1.0

The v0.1.0 release run **failed at the tap-bump step**: it cloned the tap and committed
locally, then `git push` returned `403 — Permission to homebrew-fmk-indiana denied to
niklasingvar`. The `HOMEBREW_TAP_TOKEN` PAT can **clone (read) but not push (write)** —
it was created with *Contents: Read* only. Consequence: the GitHub release + both assets
exist, but the tap still carries zero-checksums, so **`brew install` fails today**. Every
future release hits the same wall until the token is fixed. (Also: the run *looked* green
because `gh run watch` was piped through `grep`, masking its exit code — the command must
avoid that.)

## Decisions (confirmed)

- Name `release` → `/release`. Project-level, committed so it travels with the repo.
- Agent-driven: it runs tests, ensures a clean tree + correct version bump, commits, and
  only tags/pushes after an **explicit confirm** (tag/push publishes a public release).

## Deliverables

1. `.claude/commands/release.md` — Claude Code command. Frontmatter: `description`,
   `argument-hint: <version, e.g. 0.1.1>`, and a scoped `allowed-tools`
   (`Bash(git:*)`, `Bash(gh:*)`, `Bash(cargo:*)`, `Bash(make:*)`, `Bash(npm:*)`,
   `Read`, `Edit`). Body = the runbook, using `$1`/`$ARGUMENTS` for the version.
2. `.cursor/commands/release.md` — Cursor command, same runbook as plain markdown
   (Cursor injects the typed text as context; instruct it to use the provided version
   or ask for one).

Keep the runbook wording identical across the two files so they don't drift.

## Runbook the command encodes

Preconditions → tests → bump → commit → confirm → tag/push → watch → **verify**:

0. **Preconditions**: on `main`, working tree clean, `gh auth status` ok. Read the current
   version from `MENULET/src-tauri/tauri.conf.json`. Take target `X.Y.Z` from the argument;
   require it to be greater than current (semver). Requires `HOMEBREW_TAP_TOKEN` with
   Contents: **write** (see blocker) — note this in the body.
1. **Tests**: `cargo test --release` (the release workflow does *not* run tests; gate here).
   Abort on failure.
2. **Bump** the version to `X.Y.Z` in all six manifests (the `version = "…"` / `"version":`
   line in each):
   - `crates/core/Cargo.toml`, `crates/indiana/Cargo.toml`, `crates/indiana-protocol/Cargo.toml`
   - `MENULET/src-tauri/Cargo.toml`, `MENULET/src-tauri/tauri.conf.json`, `MENULET/package.json`
   Refresh lockfiles: `cargo build --release` (updates `Cargo.lock`) and
   `npm install --prefix MENULET` (updates `MENULET/package-lock.json`).
3. **Optional pre-flight**: `make dist` to confirm the full bundle builds and print SHAs
   (slow; CI rebuilds regardless — offer to skip).
4. **Commit**: `chore(release): vX.Y.Z` with the manifest + lockfile changes, ending with
   the `Co-Authored-By: Claude Opus 4.8` trailer.
5. **CONFIRM GATE**: summarize the version, files changed, and that the next step publishes
   a public release. Proceed only on explicit yes.
6. **Tag + push**: `git push origin main && git tag vX.Y.Z && git push origin vX.Y.Z`.
7. **Watch CI without masking the exit code** (do NOT pipe `gh run watch` through grep/tail —
   that hid the v0.1.0 failure). Get the run id with
   `gh run list --workflow release.yml -L1 --json databaseId -q '.[0].databaseId'`, then
   `gh run watch <id> --repo niklasingvar/fmk-indiana --exit-status`.
8. **Verify the outcome explicitly** (this is where v0.1.0 silently broke):
   - release has `indiana-aarch64-apple-darwin.tar.gz` + `Indiana_X.Y.Z_aarch64.dmg`
     (`gh release view vX.Y.Z --json assets`),
   - the tap `Formula/indiana.rb` + `Casks/indiana-menulet.rb` now show version `X.Y.Z`
     and a **non-zero** sha256 (`gh api repos/niklasingvar/homebrew-fmk-indiana/contents/...`).
   - If the bump step failed, report it loudly and fall back to updating the tap manually
     from the job's SHA256 summary.
9. **Done**: print the install/upgrade commands for the user.

## Critical files

| File | Change |
|------|--------|
| `.claude/commands/release.md` | **new** — Claude Code `/release` runbook |
| `.cursor/commands/release.md` | **new** — Cursor `/release` runbook (same content) |

References (read, not modified): `.github/workflows/release.yml`, `Makefile` (`dist`),
`docs/DISTRO.md` (one-time setup).

## Prerequisite fix (blocker — must happen for any release, including v0.1.0)

Not part of the command, but required for it to work:
1. Regenerate the PAT with **Contents: Read and write** on `homebrew-fmk-indiana`
   (the current one is read-only), then re-set it:
   `gh secret set HOMEBREW_TAP_TOKEN --repo niklasingvar/fmk-indiana`.
2. Make v0.1.0 installable: re-run the failed bump (`gh run rerun --failed <run-id>`) or
   update the tap's two files' shas by hand from the run's SHA256 summary.
3. Rotate that PAT afterward — it was pasted in plaintext earlier.

## Verification

- `/release` appears in both the Claude Code and Cursor command menus and expands to the
  runbook.
- The encoded steps match the proven-and-now-debugged process (explicit tap-bump check;
  no exit-code masking).
- True end-to-end (after the token fix): run `/release 0.1.1`, confirm the tap bumps and
  `brew update && brew upgrade --cask indiana-menulet` pulls the new version.
