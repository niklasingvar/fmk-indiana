---
status: draft
purpose: Scratch note — Indiana test strategy in four paragraphs.
approval: pending
---

# Test strategy

Every requirement earns a named test. [docs/indiana/IN_TEST.md](docs/indiana/IN_TEST.md) maps each E-criterion to a concrete case; a requirement without a test is an aspiration, not a contract. Specs are the contract and code conforms to them — when behavior drifts, the missing or failing test is the signal, not a chat explanation.

Tests sit next to what they prove. Unit tests live in `src/` beside the code (`#[cfg(test)]`); integration tests live in `tests/` and drive fixture folders under `tests/fixtures/`. Each fixture is one scenario — folder as architecture — with markdown as the natural input shape, because markers and scopes are file-shaped problems. A shared `scan_fixture` helper walks a dir into an index; assertions hit fields, not stdout poetry.

Layering follows the engine: parser and fences first, then scope, scan/walk, ID injection and the write chokepoint, compiler bundles, daemon lifecycle, CLI/MCP faces, watch, Chief of Staff capture, and auto-run dispatch behind `test-support` with a mock ACP agent. Bug fixes start with a reproducing test; refactors keep the suite green before and after. Daemon tests wait on real readiness and polled scans, never a bare socket connect.

We do not re-test the OS, the menulet pixels, or external tool contracts. FSEvents delivery, rename atomicity, and fsync durability are platform promises; Tauri rendering belongs in the menulet; `rg` and clipboard APIs get smoke, not unit coverage. Performance targets are benchmarks, not pass/fail gates. Watch and auto-run suites stay in CI while fast and stable — timing flakes under parallel load are environmental until proven otherwise.
