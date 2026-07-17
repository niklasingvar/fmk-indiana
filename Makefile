TEST_DIR ?= tmp/indiana-test
HOST := $(shell rustc -vV | sed -n 's/^host: //p')
SIDECAR := crates/menulet/src-tauri/binaries/indiana-$(HOST)
BIN := target/release/indiana

.PHONY: build scratch serve add scan copy install help menulet casablanca sidecar-copy release dist
# make invokes recipes via /bin/sh, which never sources ~/.zshrc, so nvm's
# npm/node are missing even when they work fine in an interactive shell.
# Load nvm explicitly before any npm-shaped recipe.
NVM_INIT = export NVM_DIR="$$HOME/.nvm"; [ -s "$$NVM_DIR/nvm.sh" ] && . "$$NVM_DIR/nvm.sh" && nvm use default;
# Extract the first "version": "x.y.z" from a JSON manifest.
json_version = $(shell sed -n 's/.*"version": *"\([^"]*\)".*/\1/p' $(1) | head -1)
VERSION := $(call json_version,crates/menulet/src-tauri/tauri.conf.json)
DMG := crates/menulet/src-tauri/target/release/bundle/dmg/Indiana_$(VERSION)_aarch64.dmg
CBL_VERSION := $(call json_version,crates/casablanca/package.json)
CBL_DMG := crates/casablanca/dist/Casablanca_$(CBL_VERSION)_aarch64.dmg

help:
	@echo "make scratch  Create a test markdown folder"
	@echo "make serve    Run release server (monitors nothing until you add)"
	@echo "make add      Tell the running server to monitor TEST_DIR and scan it"
	@echo "make scan     Scan the running server's monitored folders"
	@echo "make copy     Copy compiled bundle from the running server"
	@echo "make install  Copy release binary to ~/.local/bin/indiana"
	@echo "make menulet  Build daemon, bundle as sidecar, launch menulet (tauri dev)"
	@echo "make casablanca  Launch the Casablanca editor (electron-vite dev)"

build:
	cargo build --release

scratch:
	mkdir -p "$(TEST_DIR)"
	printf '%s\n\n%s\n%s\n' \
		'This line needs work ::fix tighten wording' \
		'::action follow up on this' \
		'Next block of context for the action.' \
		> "$(TEST_DIR)/review.md"

serve: build
	cargo run --release -- serve

add: build
	cargo run --release -- add "$(TEST_DIR)"

scan: build
	cargo run --release -- scan

copy: build
	cargo run --release -- copy

install: build
	mkdir -p "$(HOME)/.local/bin"
	cp "$(BIN)" "$(HOME)/.local/bin/indiana"

# Copy the just-built daemon into the menulet sidecar slot.
sidecar-copy: build
	mkdir -p "$(dir $(SIDECAR))"
	cp "$(BIN)" "$(SIDECAR)"

# Build the daemon, refresh the bundled sidecar, launch the menulet (foreground).
menulet: sidecar-copy
	$(NVM_INIT) cd crates/menulet && npm install && npm run dev

# Build Indiana, then launch Casablanca with the repo binary for dev integration.
casablanca: build
	$(NVM_INIT) cd crates/casablanca && npm install && INDIANA_BIN="$(abspath $(BIN))" npm run dev
.PHONY: release
release: build
	tar -czf indiana-aarch64-apple-darwin.tar.gz -C target/release indiana
	@echo "SHA256:"
	@shasum -a 256 indiana-aarch64-apple-darwin.tar.gz

# Local dry-run of the full release bundle: CLI tarball + menulet .dmg + Casablanca
# .dmg, with the SHA256s the Homebrew tap needs. Mirrors .github/workflows/release.yml
# so you can validate a build before pushing a tag.
dist: sidecar-copy
	$(NVM_INIT) cd crates/menulet && npm ci && npm run build
	$(NVM_INIT) cd crates/casablanca && npm ci && npm run dist
	tar -czf indiana-aarch64-apple-darwin.tar.gz -C target/release indiana
	@echo ""
	@echo "== Release artifacts (menulet v$(VERSION), casablanca v$(CBL_VERSION)) =="
	@shasum -a 256 indiana-aarch64-apple-darwin.tar.gz
	@shasum -a 256 "$(DMG)"
	@shasum -a 256 "$(CBL_DMG)"
