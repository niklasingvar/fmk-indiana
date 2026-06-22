TEST_DIR ?= tmp/indiana-test
BIN := target/release/indiana

.PHONY: build scratch serve add scan copy install help

help:
	@echo "make scratch  Create a test markdown folder"
	@echo "make serve    Run release server (monitors nothing until you add)"
	@echo "make add      Tell the running server to monitor TEST_DIR and scan it"
	@echo "make scan     Scan the running server's monitored folders"
	@echo "make copy     Copy compiled bundle from the running server"
	@echo "make install  Copy release binary to ~/.local/bin/indiana"

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

.PHONY: release
release: build
	tar -czf indiana-aarch64-apple-darwin.tar.gz -C target/release indiana
	@echo "SHA256:"
	@shasum -a 256 indiana-aarch64-apple-darwin.tar.gz
