---
status: draft
purpose: Quick local testing steps for Indiana.
approval: pending
---

# Indiana

## Test locally

## Quick dev test

Use this when you just want to see Indiana run without installing anything.

Terminal 1 — start the server (monitors nothing yet):
```sh
make serve
```

Terminal 2 — make a fixture, select it, then read it:
```sh
make scratch
make add
make scan
make copy
```

`make add` tells the running server to monitor `tmp/indiana-test`; the server scans it immediately. `make scan` / `make copy` then read the server's live state.

Why this path:
- Uses the release profile, so behavior is close to production.
- Does not copy anything into `~/.local/bin`.
- Uses ignored `tmp/indiana-test` inside this repo, so ID injection cannot touch real notes.

## Real local install test

Use this when you want the real CLI shape: `indiana serve`, `indiana scan`, `indiana copy`.

### 1. Build release binary
```sh
cargo build --release
```

### 2. Install on PATH
```sh
mkdir -p ~/.local/bin
cp target/release/indiana ~/.local/bin/indiana
```

Verify:
```sh
indiana --help
```

If `indiana` is not found, add this to your shell config:
```sh
export PATH="$HOME/.local/bin:$PATH"
```

Then open a new terminal.

Why not `install -m 755`:
- `install -m 755 source dest` means copy the binary and set executable permissions.
- Cargo already builds `target/release/indiana` as executable.
- `cp` is simpler and enough here.

### 3. Create a scratch folder
```sh
mkdir -p tmp/indiana-test
cat > tmp/indiana-test/review.md <<'EOF'
This line needs work ::fix tighten wording

::action follow up on this
Next block of context for the action.
EOF
```

Note: a normal scan/server may inject IDs into `::action` / `::todo` lines. Use a scratch folder first.

### 4. Select folder to monitor
```sh
indiana add tmp/indiana-test
```

### 5. Start server
```sh
indiana serve
```

Keep this terminal running. Stop with `Ctrl-C`.

Alternative one-off server, without saving config:
```sh
indiana serve tmp/indiana-test
```

### 6. Test from another terminal
```sh
indiana scan
```

Copy compiled bundle:
```sh
indiana copy
```

Read structured JSON:
```sh
indiana scan --json
```

Read-only standalone scan, no ID writes:
```sh
indiana scan --read-only tmp/indiana-test
```

### 7. Add more markers
Edit `tmp/indiana-test/review.md`, save, then run:
```sh
indiana scan
```

The server watches the folder and updates after save.
