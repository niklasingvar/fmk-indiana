---
status: draft
purpose: Specify the on-disk format of a tracked indiana line — how ID and status live in the source byte-for-byte.
max_lines: 45
approval: pending
---

# IN_LINE — on-disk line format

> What Indiana writes into a tracked line. Identity: [IN_IDENTITY.md](IN_IDENTITY.md). Grammar: [IN_COMMANDS.md](IN_COMMANDS.md). Write rules: [IN_SCAN.md](IN_SCAN.md).

## The bracket
- A tracked indiana carries its ID in brackets on the command token: `::action[happy-otter] buy milk`.
- The bracket rides the token, directly after the command word, no space: `::action[id]`, never at line end.
- Ephemeral markers get no bracket — nothing is written ([IN_IDENTITY.md](IN_IDENTITY.md)).

## Why brackets
- The ID travels with the command token, not with the line's end — move the marker anywhere on the line, identity follows it.
- Plain text the parser already reads: strip `[...]` between the token and the message in one pass. No second parse, no end-of-line scan.
- Markers are visible source annotations by nature; the bracket needs no invisibility, unlike a rendered-document comment would.

## Status
- Open task: `::action[happy-otter]` — no status word.
- Resolved: `::action[happy-otter:done] buy milk` or `::action[happy-otter:failed] buy milk`.
- Status applies to `::action` / `::todo` only ([IN_IDENTITY.md](IN_IDENTITY.md)). No other kind takes one.
- `done` / `failed` are the only status words, after a `:` inside the bracket. Absence means open.

## Discipline
- Idempotent: a line that already has its bracket is left byte-identical on re-tag ([IN_SCAN.md](IN_SCAN.md): write chokepoint).
- The bracket moves with the line — copy the line to another file, identity and status travel with it.
- Parsing strips the bracket before reading the marker's message, so the ID never leaks into the payload.

## Repair (D7)
- Malformed brackets are repaired, not trusted. An id failing `[a-z]+-[a-z]+(-[0-9]+)?` gets a fresh id; an unknown status word is dropped to open. Must stay idempotent (a repaired line rescans clean).
