---
status: draft
purpose: Map every requirement to a test. Drift is a missing test — IN_PRINCIPLES.md.
max_lines: 80
approval: pending
---

# IN_TEST — test map

> Each E-criterion below names a requirement from the specs and a concrete test that proves it. A requirement without a test is at best an aspiration — [IN_PRINCIPLES.md](IN_PRINCIPLES.md): spec is the contract, code conforms.

## Structure
- Unit tests live in `src/` beside the code they test (Rust `#[cfg(test)] mod tests`).
- Integration tests live in `tests/` and point at fixture directories under `tests/fixtures/`.
- Fixtures are markdown files — folder as architecture. Each fixture dir is one scenario; its README (a markdown file, naturally) states the expected outcome.
- Test harness: a helper function `scan_fixture(dir) -> Index` that walks the dir and returns the parsed index; tests assert on the index fields.

## E1 — Parser: marker detection
| Ref | Requirement | Test |
|-----|-------------|------|
| [IN_COMMANDS.md](IN_COMMANDS.md) | `::<cmd>` at column 0 is a marker | `test_marker_column_zero` — file with `::h` alone on a line → detected as hate |
| [IN_COMMANDS.md](IN_COMMANDS.md) | `::<cmd>` inline at end of line is a marker | `test_marker_inline` — file with `Some text ::l` → detected as love, scope is the line |
| [IN_COMMANDS.md](IN_COMMANDS.md) | Short and long forms are equivalent | `test_marker_long_form` — `::hate` resolves to same kind as `::h` |
| [IN_COMMANDS.md](IN_COMMANDS.md) | Optional/required message follows the token | `test_marker_with_message` — `::fix rename this` → kind=fix, message="rename this" |
| [IN_COMMANDS.md](IN_COMMANDS.md) | Two or more `::` on one line → skip, warn | `test_marker_ambiguous_line` — `::h ::l` on one line → line skipped, warning emitted |
| [IN_LINE.md](IN_LINE.md) | Bracket is stripped before parsing | `test_marker_bracket_stripped` — `::action[bear-mouse] do it` → kind=action, message="do it", id="bear-mouse" |

## E2 — Parser: code fences
| Ref | Requirement | Test |
|-----|-------------|------|
| [IN_SCAN.md](IN_SCAN.md) | Marker inside ` ``` ` fence is ignored | `test_fence_backtick` — fixture with ` ``` ` open, `::h` inside, ` ``` ` close → marker ignored |
| [IN_SCAN.md](IN_SCAN.md) | Marker inside `~~~` fence is ignored | `test_fence_tilde` — same with `~~~` fences |
| [IN_SCAN.md](IN_SCAN.md) | ` ``` ` and `~~~` tracked independently | `test_fence_independent` — ` ``` ` opens, `~~~` appears inside, ` ``` ` closes → `~~~` still open, markers after still ignored |
| [IN_SCAN.md](IN_SCAN.md) | Unclosed fence → markers after are ignored | `test_fence_unclosed` — ` ``` ` opens, never closes, file has `::h` at end → marker ignored |
| (decided here) | YAML frontmatter `---` is a fence | `test_fence_yaml_frontmatter` — `---` opens, `::h` inside, `---` closes → marker ignored |
| (decided here) | Indented `::` (column ≥ 4) is ignored | `test_indented_ignored` — `    ::h` at column 4 → not a marker |
| [IN_SCAN.md](IN_SCAN.md) | Marker inside an inline code span is ignored | `test_inline_code_ignored` — `` `::hate` `` in prose → not a marker; CommonMark run matching covers a triple ``` shown inline (`test_inline_code_span_with_backtick_run`) |

## E3 — Parser: stateless per line
| Ref | Requirement | Test |
|-----|-------------|------|
| [IN_PRINCIPLES.md](IN_PRINCIPLES.md) | Parsing a line is a pure function of line + fence state | `test_parse_line_pure` — same input line always produces same output; property test with random lines |
| [IN_SCAN.md](IN_SCAN.md) | One indiana per line | `test_one_marker_per_line` — fixture with one marker per line across 50 lines → all detected |

## E4 — Scope resolution
| Ref | Requirement | Test |
|-----|-------------|------|
| [IN_SCOPE.md](IN_SCOPE.md) | Inline: marker at end of content line targets that line | `test_scope_inline` — `Fix this ::f` → scope is the line text (minus the marker and tail) |
| [IN_SCOPE.md](IN_SCOPE.md) | Next-row: marker alone on a line targets the next block until blank line | `test_scope_next_row` — `::n` on own line, then a paragraph of 3 lines, then blank → scope is the 3-line paragraph |
| [IN_SCOPE.md](IN_SCOPE.md) | Section: marker on heading targets section until equal/higher heading | `test_scope_section` — `## Intro ::k`, then text, then `## Next` → scope is text under Intro |
| [IN_SCOPE.md](IN_SCOPE.md) | Most-specific wins: inline inside section keeps own span | `test_scope_most_specific` — `## Section ::k` with an inline `::f` on a line inside → the `::f` scope is its line only, not the section |
| [IN_SCOPE.md](IN_SCOPE.md) | Spans never cross file boundaries | `test_scope_file_bound` — next-row marker at end of file → scope stops at EOF, no error |
| [IN_SCOPE.md](IN_SCOPE.md) | Range: deferred to later phase | No test yet. Placeholder: `test_scope_range_deferred` — `::end` not implemented, does not crash |

## E5 — Scan: full walk
| Ref | Requirement | Test |
|-----|-------------|------|
| [IN_SCAN.md](IN_SCAN.md) | Startup: full walk of all markdown under repo root | `test_full_walk` — fixture dir with 3 `.md` files across 2 subdirs, one `.txt` file → all 3 `.md` scanned, `.txt` ignored |
| [IN_SCAN.md](IN_SCAN.md) | Exclude `.indiana/` from walk | `test_exclude_indiana_dir` — fixture with `.indiana/scratch.md` containing `::h` → not in results |
| [IN_SCAN.md](IN_SCAN.md) | `rg`-style: column-0 and inline only | Already covered by E1 |
| [IN_SCAN.md](IN_SCAN.md) | Non-markdown files skipped | `test_skip_non_markdown` — `.txt`, `.rs`, `.json` files with `::h` → none detected |

## E6 — ID injection: write path
| Ref | Requirement | Test |
|-----|-------------|------|
| [IN_LINE.md](IN_LINE.md) | Tracked indiana gets `[<id>]` on first sight | `test_id_first_injection` — fresh `::action do thing` → written as `::action[happy-otter] do thing` |
| [IN_LINE.md](IN_LINE.md) | Already-tagged line is left byte-identical | `test_id_idempotent` — line with existing bracket → rescan leaves it unchanged (byte comparison) |
| [IN_IDENTITY.md](IN_IDENTITY.md) | Only `::action` / `::todo` get IDs | `test_id_only_tracked` — `::hate` and `::love` in fixture → no tail written |
| [IN_LINE.md](IN_LINE.md) | Status: `done` / `failed` inside bracket | `test_status_done` — `::action[happy-otter:done] buy milk` → status=done |
| [IN_SCAN.md](IN_SCAN.md) | Atomic write: temp → fsync → rename | `test_write_atomic` — verify temp file exists briefly, then renamed; original never corrupted |
| [IN_SCAN.md](IN_SCAN.md) | mtime guard: file changed under us → abort | `test_mtime_guard` — modify file between scan and injection → injection aborted, file re-queued |
| [IN_SCAN.md](IN_SCAN.md) | Own-write suppressed for ~500ms | `test_suppress_own_write` — after injection, the FSEvent on that path does not trigger a rescan within 500ms |

## E7 — Compiler: copy bundle
| Ref | Requirement | Test |
|-----|-------------|------|
| [IN_PRD.md](IN_PRD.md) | `indiana copy` returns all commands as one bundle | `test_copy_all_commands` — fixture with 3 markers → bundle contains 3 compiled prompts |
| [IN_PRINCIPLES.md](IN_PRINCIPLES.md) | Content is data, not code — prompt templates live separately | `test_prompt_templates_external` — changing `::hate` wording does not require recompiling the core |
| [IN_COMMANDS.md](IN_COMMANDS.md) | `::hate` → canned explainer prompt | `test_compile_hate` — `::h` compiles to "The user tagged this as hate. Explain why." |
| [IN_COMMANDS.md](IN_COMMANDS.md) | `::fix <msg>` → "Fix this." + msg | `test_compile_fix` — `::fix the loop condition` → prompt includes "Fix this. the loop condition" |
| [IN_COMMANDS.md](IN_COMMANDS.md) | `::question <msg>` → "The user asks: <msg>. Answer it." | `test_compile_question` — message passed through verbatim |
| [IN_SCOPE.md](IN_SCOPE.md) | Resolved scope travels into the bundle | `test_scope_in_bundle` — inline marker's line text appears in the compiled output |

## E8 — Daemon: lifecycle
| Ref | Requirement | Test |
|-----|-------------|------|
| [IN_DAEMON.md](IN_DAEMON.md) | One daemon binds socket; second fails cleanly | `test_socket_single_bind` — start daemon A, try to start daemon B → B exits with "already running" |
| [IN_DAEMON.md](IN_DAEMON.md) | Stale socket detected and cleaned | `test_stale_socket` — create a socket file, no daemon behind it → daemon connects, gets refused, unlinks, binds |
| [IN_DAEMON.md](IN_DAEMON.md) | Config lives in `~/.indiana/config.json` | `test_config_persists` — add a folder via CLI, restart daemon → folder still monitored |
| [IN_DAEMON.md](IN_DAEMON.md) | Client disconnect doesn't lose state | `test_client_reconnect` — CLI queries, disconnects, reconnects → same counts |

## E9 — Invariants
| Ref | Requirement | Test |
|-----|-------------|------|
| [IN_PRINCIPLES.md](IN_PRINCIPLES.md) | Source is truth: delete `.indiana/`, rescan, state is byte-identical | `test_source_is_truth` — scan, delete index dir, rescan → identical results |
| [IN_PRINCIPLES.md](IN_PRINCIPLES.md) | One marker table drives everything | `test_marker_table_single_source` — adding a marker kind updates parser, compiler, and identity in one place |
| [IN_PRINCIPLES.md](IN_PRINCIPLES.md) | Core computes, faces render | `test_faces_never_compute` — verify CLI and MCP output both derive from the same compiled payload, neither re-parses |
| [IN_PRINCIPLES.md](IN_PRINCIPLES.md) | Write path is a single chokepoint | `test_write_path_single_function` — grep for file writes outside the chokepoint; fail if any exist |

## E10 — CLI
| Ref | Requirement | Test |
|-----|-------------|------|
| [IN_PRD.md](IN_PRD.md) | `indiana scan` lists every marker | `test_cli_scan` — run against fixture, stdout contains all markers |
| [IN_PRD.md](IN_PRD.md) | `indiana copy` puts bundle on clipboard | `test_cli_copy` — run against fixture, clipboard contains compiled bundle |
| [IN_DAEMON.md](IN_DAEMON.md) | `indiana service install` registers launchd plist | `test_cli_service_install` — plist created at `~/Library/LaunchAgents/…`, valid XML |

## E11 — Watch (FSEvents)
| Ref | Requirement | Test |
|-----|-------------|------|
| [IN_SCAN.md](IN_SCAN.md) | New file with markers detected within ~500ms | `test_watch_new_file` — write a `.md` with `::h` into monitored dir → marker detected within 800ms (with tolerance) |
| [IN_SCAN.md](IN_SCAN.md) | Modified file re-scanned | `test_watch_modify` — append `::l` to existing file → new marker detected |
| [IN_SCAN.md](IN_SCAN.md) | Deleted file → markers removed from index | `test_watch_delete` — delete a file with markers → markers gone from counts |
| [IN_SCAN.md](IN_SCAN.md) | Burst of writes debounced to single rescan | `test_watch_debounce` — write 10 files in rapid succession → one rescan, not ten |

## What not to test
- OS behavior: FSEvents delivery, `rename` atomicity, `fsync` durability — these are OS contracts, not Indiana's.
- Tauri/NSPanel rendering: visual tests go in the menulet, not in Indiana core.
- Performance targets (sub-500ms): these are benchmarks, not pass/fail tests. A separate `cargo bench` suite.
- External tool behavior: `rg` output format, clipboard API on macOS — integration smoke, not unit coverage.

## Open
- Whether E6 (ID injection) tests run against real temp dirs or an in-memory filesystem abstraction. Real temp dirs are simpler and test the actual OS path; they also leave cleanup cruft on failure.
- Whether E11 (watch) tests are flaky by nature and belong in CI at all, or are manual-only.
