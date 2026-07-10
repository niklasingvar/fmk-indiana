# Release plan — cut v0.3.0 (ship Casablanca + release hardening)

## Context

The latest git tag is **v0.2.0**. `main` (HEAD `d08c883`) is several commits ahead with
work that has **never shipped under a tag**:

- The entire **Casablanca** Electron editor + its brew cask (`19f7e78`, `02aefc0`).
- Release-pipeline **hardening** just landed this session: a fail-fast manifest/tag
  version guard and a DRY Makefile macro (`5cae6d9`), plus a `.gitignore` cleanup (`d08c883`).

There is also a **version drift**: 6 of the 7 version manifests are at `0.2.0`, but
`crates/casablanca/package.json` is still at its scaffold value `0.1.0`. The release
process expects **one shared version** across all seven; the CI fail-fast guard in
`release.yml` checks both `menulet/src-tauri/tauri.conf.json` and
`crates/casablanca/package.json` against the tag, so re-tagging today would abort on
Casablanca. Bumping all seven to the new version fixes the drift as a side effect.

**Goal:** cut **v0.3.0**, shipping Casablanca for the first time and the release
hardening, with all seven manifests aligned. Minor bump because a brand-new app ships.

> The two deferred cleanups are intentionally **out of scope** and will not be touched:
> the 3× `sed`-block "refactor" in `release.yml` (blocks genuinely differ — risk of
> shipping wrong SHAs for no gain) and the cosmetic `dist:dir` / doc-xattr duplication.

## Note on the automated path

An authoritative runbook already exists as the **`/release` command**
(`.claude/commands/release.md`, mirrored at `.cursor/commands/release.md`). Execution
should be driven by that command with argument `0.3.0`; this plan documents what it will
do so the steps are reviewable. Do **not** follow the stale `docs/SHIP.md` (bumps only 2
manifests, wrong formula name — superseded).

## Steps

### 1. Preconditions
- On `main`, clean working tree (currently clean, all work committed).
- `gh auth status` OK; the `HOMEBREW_TAP_TOKEN` secret is set on the repo (expires
  2026-07-10 per project memory — still valid today, 2026-07-04).

### 2. Gate: tests
- `cargo test --release` — **CI does not run tests**, so this is the only correctness gate.

### 3. Bump all 7 manifests to `0.3.0`
Set the version field in each (current value → `0.3.0`):

| Path | Field | From |
|------|-------|------|
| `crates/core/Cargo.toml` | `version = ` (line 3) | 0.2.0 |
| `crates/indiana/Cargo.toml` | `version = ` (line 3) | 0.2.0 |
| `crates/indiana-protocol/Cargo.toml` | `version = ` (line 3) | 0.2.0 |
| `crates/menulet/src-tauri/Cargo.toml` | `version = ` (line 5) | 0.2.0 |
| `crates/menulet/src-tauri/tauri.conf.json` | `"version"` (line 4) | 0.2.0 |
| `crates/menulet/package.json` | `"version"` (line 4) | 0.2.0 |
| `crates/casablanca/package.json` | `"version"` (line 3) | **0.1.0** ← fixes drift |

**Do not** hand-edit: root `Cargo.toml` (no `version` key — workspace carries only
edition/license/authors), the `dist/homebrew/*` templates (CI overwrites version+sha
into the tap repo), or any lockfile directly.

Then refresh lockfiles via builds, not by hand:
- `cargo build --release` → refreshes root `Cargo.lock`.
- `npm install --prefix crates/menulet` → `crates/menulet/package-lock.json` (+ its
  `src-tauri/Cargo.lock` refreshes as a build side-effect; do **not** run
  `cargo generate-lockfile` on it).
- `npm install --prefix crates/casablanca` → `crates/casablanca/package-lock.json`.

Verify all seven now read `0.3.0` before committing.

### 4. Local pre-flight (recommended — untested pipeline)
- `make dist` — builds the CLI tarball + menulet `.dmg` + Casablanca `.dmg` and prints
  all three SHA256s. This is the **first real exercise** of the new fail-fast guard's
  sibling logic and the Casablanca DMG build; catch a break here, before the public tag.
- Slow (~several min). Skippable only if trusting CI end-to-end.

### 5. Commit
- `chore(release): v0.3.0` with the `Co-Authored-By: Claude Opus 4.8` trailer.
- Include the 7 manifests + all refreshed lockfiles.

### 6. CONFIRM GATE (irreversible/public beyond this point)
- Summarize the diff and the three artifacts that will be published; proceed to tag
  push only on **explicit** user yes.

### 7. Tag & push (triggers the release)
- `git push origin main`
- `git tag v0.3.0 && git push origin v0.3.0` → fires `.github/workflows/release.yml`.

### 8. Watch CI
- `gh run watch <id> --exit-status` (do **not** pipe through grep/tail — that hides the
  exit code). The new **"Check manifest versions match tag"** guard runs first; if any
  manifest was missed it aborts in seconds instead of after a ~10-min build.

### 9. Verify published output
- GitHub release `v0.3.0` has: `indiana-aarch64-apple-darwin.tar.gz`,
  `Indiana_0.3.0_aarch64.dmg`, `Casablanca_0.3.0_aarch64.dmg`.
- Tap repo `niklasingvar/homebrew-fmk-indiana` got a `indiana 0.3.0` commit with **real**
  SHA256s (not the zeroed template values) in the formula + both casks.
- If the tap step 403s: the `HOMEBREW_TAP_TOKEN` PAT lacks `Contents:write` or expired.

### 10. Smoke-test install (optional)
- `brew update && brew upgrade --cask indiana-menulet indiana-casablanca && brew upgrade indiana`.

## Verification summary

- **Pre-tag:** `cargo test --release` green; all 7 manifests show `0.3.0`; `make dist`
  produces all three artifacts locally with SHA256s.
- **Post-tag:** `gh run watch` exits 0; three named assets on the GitHub release; tap
  repo has a new commit with real (non-zero) SHAs.

## Open decisions (defaulted; override on review)

- **Version = 0.3.0** (minor). Alternative considered: 0.2.1 (patch) — rejected because
  a brand-new app (Casablanca) ships here.
- **Run `make dist` pre-flight = yes.** Safer given the pipeline hardening has never run
  in a real release. Skip only to save time and trust CI.
