# Release v0.3.0 — CI failure postmortem + fix steps

**Status:** `v0.3.0` tag pushed 2026-07-04, CI run `28718564263` **failed** at
"Build Casablanca (.app + .dmg)". **Nothing public shipped** — the failure is
before "Publish GitHub release" and "Bump Homebrew tap" (both skipped), so there
is no GitHub release and the Homebrew tap is unchanged. The `v0.3.0` tag exists
but is inert and safe to delete/reuse.

## What failed

`npm ci` in the Casablanca build step rejected the committed lockfile:

```
npm error `npm ci` can only install packages when your package.json and
package-lock.json are in sync.
npm error Missing: esbuild@0.28.1 from lock file
npm error Missing: @esbuild/darwin-arm64@0.28.1 from lock file   (+ every other platform)
```

## Root cause — local vs CI toolchain mismatch (not a code bug)

| | Node | npm |
|---|------|-----|
| Dev machine (this repo) | v24.16.0 | 11.15.0 |
| CI (`actions/setup-node@v4`, `node-version: 20`) | 20 | 10.x |

- The committed `crates/casablanca/package-lock.json` pins `esbuild@^0.21.5`
  (0.21.x). A newer transitive constraint in the tree
  (`"esbuild": "^0.27.0 || ^0.28.0"`) makes a modern npm resolve `esbuild@0.28.1`,
  which is **not present** in the committed lock (`grep -c 0.28.1` → 0).
- npm 11 (local) tolerated the lock and let `npm ci` pass. npm 10 (CI) enforced
  strict lock/manifest agreement and aborted.
- **`make dist` gave false confidence**: its Casablanca step also runs `npm ci`,
  but under the local npm 11 — so it validated a lock CI would reject. The local
  pre-flight is only trustworthy when the local Node/npm major matches CI.

The same class of drift is the likely reason the earlier `v0.1.0` and `v0.2.0`
runs also show `failure` (different steps, same "works-locally" trap).

## Fix — regenerate the lock under CI's Node, then re-release

Do this deliberately; each `npm ci` verify is the gate.

### 1. Match CI's Node locally
```
nvm install 20 && nvm use 20      # or fnm/asdf equivalent; must be Node 20 / npm 10
node -v   # v20.x
npm -v    # 10.x
```

### 2. Regenerate the Casablanca lockfile from scratch
```
rm -rf crates/casablanca/node_modules crates/casablanca/package-lock.json
npm install --prefix crates/casablanca
```

### 3. Verify with the *exact* command CI runs (this is the real gate)
```
cd crates/casablanca && npm ci        # must complete clean, no EUSAGE
grep -c '0.28.1' package-lock.json     # expect > 0 now
cd ../..
```
If `npm ci` still errors, the lock and manifest still disagree — do **not**
proceed. (Do the same regen for `crates/menulet` if its lock was also generated
under npm 11 and you want to be safe: `rm -rf crates/menulet/node_modules
crates/menulet/package-lock.json && npm install --prefix crates/menulet && (cd
crates/menulet && npm ci)`.)

### 4. Commit the corrected lock
```
git add crates/casablanca/package-lock.json   # + crates/menulet/package-lock.json if regenerated
git commit -m "fix(casablanca): regenerate package-lock under Node 20 to match CI npm ci"
```

### 5. Re-release under the same v0.3.0 tag
Nothing public shipped under v0.3.0, so reuse the version:
```
git push origin main
git push origin :refs/tags/v0.3.0     # delete the remote tag
git tag -d v0.3.0                      # delete the local tag
git tag v0.3.0                         # re-tag at the fix commit
git push origin v0.3.0                 # re-trigger release.yml
```
Then watch: `gh run watch <id> --exit-status` (no grep/tail — it hides the exit
code). Expect it to pass "Build Casablanca", publish the release with all three
assets, and bump the tap with real (non-zero) SHAs.

## Prevention

**Implemented** (commit after the v0.3.0 fix):

1. **Pin Node so dev == CI.** Added `.nvmrc` (`20`) at repo root; the workflow's
   `setup-node` now reads `node-version-file: ".nvmrc"` instead of a hardcoded
   `20`, so CI and `nvm use` share one source of truth. (Deliberately did **not**
   add `"engines"` to the `package.json`s — that would change the manifests and
   force another lockfile regen for no safety `.nvmrc` doesn't already give.)
2. **Fail fast on lock drift, like the version guard.** Added a
   "Check npm lockfiles in sync" step in `.github/workflows/release.yml` (right
   after `setup-node`) that runs `npm ci --dry-run` for both `crates/menulet` and
   `crates/casablanca` before the ~10-min Rust/Tauri build — a stale lock now
   aborts in seconds.

**Still worth doing:**

3. **Note in the runbook** that `make dist` only validates the pipeline when the
   local Node/npm major matches CI; otherwise it can pass while CI fails.
4. GitHub is deprecating Node 20 on runners (build annotation warned). When you
   move CI to Node 22/24, regenerate all lockfiles under that version in the same
   commit and bump `.nvmrc` to match.

## The Homebrew tap step (separate failure, same release)

v0.3.0's re-run built and **published the GitHub release**, then failed at
"Bump Homebrew tap": `remote: Permission to niklasingvar/homebrew-fmk-indiana.git
denied to niklasingvar … 403`. The `HOMEBREW_TAP_TOKEN` secret exists (updated
2026-07-03) and is not expired, so the PAT lacks `Contents:write` on the tap repo
(or, if fine-grained, the tap repo isn't in its selected repositories). Fix the
secret so future releases bump the tap automatically. Until then, the tap can be
bumped manually with local git creds (the repo owner has write access) by copying
`dist/homebrew/*` into the tap and sed-filling version/url/sha from the published
assets' SHA256s.

## Files involved
- `crates/casablanca/package-lock.json` — the out-of-sync lock (fix target)
- `crates/casablanca/package.json` — manifest whose tree now wants esbuild 0.28.x
- `.github/workflows/release.yml` — `Build Casablanca` step runs `npm ci`; also
  where the prevention fail-fast lock check would go
- `Makefile` `dist:` target — the misleading local pre-flight
