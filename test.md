---
status: draft
purpose: Scratch note — Indiana test strategy in four paragraphs.
approval: pending
---
# Test strategy

Every requirement earns a named test. [docs/indiana/IN_TEST.md](docs/indiana/IN_TEST.md) maps each E-criterion to a case; no test means aspiration, not contract. Specs rule; a missing or failing test signals drift.

Tests sit next to what they prove: unit tests in `src/` (`#[cfg(test)]`), integration in `tests/` against fixture folders. Each fixture is one scenario; assert fields via `scan_fixture`, not stdout poetry.

Layering follows the engine: parser → scope → scan → write chokepoint → compiler → daemon → CLI/MCP → watch → CoS → auto-run (mock ACP). Bug fixes start with a reproducing test; refactors stay green.

Skip OS promises, menulet pixels, and external tool contracts — smoke only. Performance is benchmarks, not gates. Watch and auto-run stay in CI while fast; timing flakes are environmental until proven otherwise.