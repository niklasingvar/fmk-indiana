---
status: draft
purpose: Step-by-step release checklist for Indiana. Follow in order — each step gates the next.
approval: pending
---

# SHIP — cut a release

Pre-release checklist:
- [ ] `cargo test --release` — all tests green
- [ ] `git status` — working tree clean, on `main`
- [ ] Version bumped in `crates/core/Cargo.toml` and `crates/indiana/Cargo.toml`
- [ ] Changelog notes written (what's new since the last tag)

## 1. Build
```sh
cargo build --release --target aarch64-apple-darwin
```

## 2. Tarball
```sh
tar -czf indiana-aarch64-apple-darwin.tar.gz -C target/aarch64-apple-darwin/release indiana
shasum -a 256 indiana-aarch64-apple-darwin.tar.gz
```
Record the SHA256 hash.

Alternative: if building on Apple Silicon without `--target`:
```sh
make release
```
(same result; uses host target path `target/release/indiana`)

## 3. Tag
```sh
git tag -a v<VERSION> -m "v<VERSION> — <summary>"
git push origin v<VERSION>
```

## 4. GitHub release
- Go to https://github.com/niklasingvar/fmk-indiana/releases
- The tag auto-creates a draft release. Edit the release notes.
- Upload `indiana-aarch64-apple-darwin.tar.gz` as a release asset.

## 5. Update Homebrew formula
- Repo: `niklasingvar/homebrew-fmk-indiana`
- File: `Formula/fmk-indiana.rb`
- Update `url` to the new release tarball URL.
- Replace `sha256` with the hash from step 2.
- Commit and push.

## 6. Smoke test
```sh
brew upgrade fmk-indiana
indiana --version
indiana scan .
```
