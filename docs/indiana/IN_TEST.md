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
| [IN_SCAN.md](IN_SCAN.md) | Explicit frontmatter property comments are markers; other YAML stays inert | `test_frontmatter_property_comment_marker`, `test_frontmatter_ordinary_comments_and_values_stay_ignored`, `test_frontmatter_property_comment_scope_and_identity` |
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
| [IN_SCOPE.md](IN_SCOPE.md) | Section: marker alone before an ATX heading targets section until equal/higher heading | `test_scope_section` — `::k`, then `## Intro`, then text, then `## Next` → scope is Intro section |
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
| [IN_SCAN.md](IN_SCAN.md) | `::ignore` opts a file out: no markers, no warnings, no ID injection | `test_ignored_file_contributes_nothing`; forms and negatives: `test_file_ignored_frontmatter`, `test_file_ignored_first_line_comment`, `test_file_ignored_negative_cases`, `test_ignore_token_is_not_a_marker` |

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
| [IN_IDENTITY.md](IN_IDENTITY.md) | Malformed id repaired with fresh one | `test_bracket_repair_bad_id` — `::action[not-valid] do it` → written with valid id |
| [IN_IDENTITY.md](IN_IDENTITY.md) | Unknown status dropped to open | `test_bracket_repair_bad_status` — `::action[valid-id:unknown] do it` → written as `::action[valid-id] do it` |
| [IN_IDENTITY.md](IN_IDENTITY.md) | --read-only suppresses writes | `test_scan_read_only_no_write` — `::action do thing` with --read-only → no file modified |

## E7 — Compiler: copy bundle
| Ref | Requirement | Test |
|-----|-------------|------|
| [IN_PRD.md](IN_PRD.md) | `indiana copy` returns all commands as one bundle | `test_copy_all_commands` — fixture with 3 markers → bundle contains 3 compiled prompts |
| [IN_PRINCIPLES.md](IN_PRINCIPLES.md) | Content is data, not code — prompt templates live separately | `test_prompt_templates_external` — changing `::hate` wording does not require recompiling the core |
| [IN_COMMANDS.md](IN_COMMANDS.md) | `::hate` → canned explainer prompt | `test_compile_hate` — `::h` compiles to "The user tagged this as hate. Explain why." |
| [IN_COMMANDS.md](IN_COMMANDS.md) | `::fix <msg>` → "Fix this." + msg | `test_compile_fix` — `::fix the loop condition` → prompt includes "Fix this. the loop condition" |
| [IN_COMMANDS.md](IN_COMMANDS.md) | `::question <msg>` → "The user asks: <msg>. Answer it." | `test_compile_question` — message passed through verbatim |
| [IN_SCOPE.md](IN_SCOPE.md) | Resolved scope travels into the bundle | `test_scope_in_bundle` — inline marker's line text appears in the compiled output |
| [IN_FOLDER.md](IN_FOLDER.md) | Folder template overrides embedded default for its owning root | `test_compile_with_roots_uses_owning_root_template` — two roots, one override → only that root changes prompt |
| [IN_FOLDER.md](IN_FOLDER.md) | Bad folder template falls back to embedded default with warning | `test_compile_with_roots_bad_template_warns_and_falls_back` |

## E8 — Daemon: lifecycle
| Ref | Requirement | Test |
|-----|-------------|------|
| [IN_DAEMON.md](IN_DAEMON.md) | One daemon binds socket; second fails cleanly | `test_socket_single_bind` — start daemon A, try to start daemon B → B exits with "already running" |
| [IN_DAEMON.md](IN_DAEMON.md) | Stale socket detected and cleaned | `test_stale_socket` — create a socket file, no daemon behind it → daemon connects, gets refused, unlinks, binds |
| [IN_DAEMON.md](IN_DAEMON.md) | Config lives in `~/.indiana/config.json` | `test_config_persists` — add a folder via CLI, restart daemon → folder still monitored |
| [IN_DAEMON.md](IN_DAEMON.md) | Empty config monitors nothing | `test_serve_empty_no_folders` — serve with no config → scan reports zero markers |
| [IN_DAEMON.md](IN_DAEMON.md) | Live `add` watches + rescans without restart | `test_live_add_autoscan` — add a folder to a running daemon → its markers appear with no restart |
| [IN_DAEMON.md](IN_DAEMON.md) | Live `add` is idempotent | `test_live_add_idempotent` — re-adding a monitored folder → "already monitoring" |
| [IN_DAEMON.md](IN_DAEMON.md) | Client disconnect doesn't lose state | `test_client_reconnect` — CLI queries, disconnects, reconnects → same counts |
| [IN_FOLDER.md](IN_FOLDER.md) | Monitoring a folder initializes `.indiana/` without overwriting | `test_cli_add_scaffolds_folder_templates_idempotently`, `test_live_add_autoscan` |

Harness invariants (uphold for any new daemon test):
- Readiness is a real round-trip, not a socket connect. Wait via `wait_ready` (polls `indiana status`, which talks only to the daemon). The listener backlog accepts a connection before the daemon can serve it, so a bare connect is "bound", not "ready" — trusting it raced tests under parallel load.
- Read daemon state through `scan_json`, which runs `indiana scan` from `home` (no `.md`). `scan` with no path falls back to a standalone cwd scan when the daemon doesn't answer; running from a marker-free dir makes that fallback empty instead of a silent scan of the test runner's cwd.
- Assert on content with `wait_until` (polling), never a single `scan_json` call — the initial scan may lag the socket under load.

## E9 — Invariants
| Ref | Requirement | Test |
|-----|-------------|------|
| [IN_PRINCIPLES.md](IN_PRINCIPLES.md) | Source is truth: delete derived scratch/index state, rescan, marker state is byte-identical | `test_source_is_truth` — scan, delete index dir, rescan → identical results |
| [IN_PRINCIPLES.md](IN_PRINCIPLES.md) | One marker table drives everything | `test_marker_table_single_source` — adding a marker kind updates parser, compiler, and identity in one place |
| [IN_PRINCIPLES.md](IN_PRINCIPLES.md) | Core computes, faces render | `test_faces_never_compute` — verify CLI and MCP output both derive from the same compiled payload, neither re-parses |
| [IN_PRINCIPLES.md](IN_PRINCIPLES.md) | Write path is a single chokepoint | `test_write_path_single_function` — grep for file writes outside the chokepoint; fail if any exist |

## E10 — CLI
| Ref | Requirement | Test |
|-----|-------------|------|
| [IN_PRD.md](IN_PRD.md) | `indiana scan` lists every marker | `test_cli_scan` — run against fixture, stdout contains all markers |
| [IN_PRD.md](IN_PRD.md) | `indiana copy` puts bundle on clipboard | `test_cli_copy` — run against fixture, clipboard contains compiled bundle |
| [IN_DAEMON.md](IN_DAEMON.md) | `indiana service install` registers launchd plist | `test_cli_service_install` — plist created at `~/Library/LaunchAgents/…`, valid XML |
| [IN_FOLDER.md](IN_FOLDER.md) | `indiana copy <path>` uses repo-local templates | `test_cli_copy_uses_folder_template` |
| [IN_FOLDER.md](IN_FOLDER.md) | User can edit generated prompt templates for existing commands | `test_cli_add_then_user_template_edit_affects_copy` |
| [IN_FOLDER.md](IN_FOLDER.md) | `indiana templates refresh <path>` creates missing templates without overwriting edits | `test_cli_templates_refresh_restores_missing_without_overwrite` |
| [IN_MCP.md](IN_MCP.md) | MCP reads daemon compiled payload with folder templates | `test_mcp_read_payload` |

## E11 — Watch (FSEvents)
| Ref | Requirement | Test |
|-----|-------------|------|
| [IN_SCAN.md](IN_SCAN.md) | New file with markers detected within ~500ms | `test_watch_new_file` — write a `.md` with `::h` into monitored dir → marker detected within 800ms (with tolerance) |
| [IN_SCAN.md](IN_SCAN.md) | Modified file re-scanned | `test_watch_modify` — append `::l` to existing file → new marker detected |
| [IN_SCAN.md](IN_SCAN.md) | Deleted file → markers removed from index | `test_watch_delete` — delete a file with markers → markers gone from counts |
| [IN_SCAN.md](IN_SCAN.md) | Burst of writes debounced to single rescan | `test_watch_debounce` — write 10 files in rapid succession → one rescan, not ten |

## E12 — Chief of Staff tracker + action log
| Ref | Requirement | Test |
|-----|-------------|------|
| [COS_PRD.md](../chief-of-staff/COS_PRD.md) | `indiana task add` → `list` → `done` round-trip; done leaves the default view | `test_add_list_done_round_trip` |
| [COS_PRD.md](../chief-of-staff/COS_PRD.md) | `--queue` filters; `--json` emits stable task rows | `test_queue_filter_and_json_shape` |
| [COS_PRD.md](../chief-of-staff/COS_PRD.md) | `done` on a missing id is a clean failure | `test_done_unknown_id_fails` |
| [COS_PRD.md](../chief-of-staff/COS_PRD.md) | Hand-edited tracker lines survive machine writes byte-for-byte | `test_hand_edited_file_survives`, `test_unknown_lines_survive_rewrites` |
| [COS_PRD.md](../chief-of-staff/COS_PRD.md) | A scan captures `::todo`/`::task` into Agent, `::action` into Human, with origin + source id | `test_scan_capture_visible_to_task_list`, `test_capture_routes_queues_and_logs` |
| [COS_PRD.md](../chief-of-staff/COS_PRD.md) | Capture is idempotent: a second build is write-free | `test_capture_idempotent_across_builds` |
| [COS_PRD.md](../chief-of-staff/COS_PRD.md) | Capture is opt-in: a plain write scan mints no chief-of-staff files | `test_capture_off_without_opt_in` |
| [COS_PRD.md](../chief-of-staff/COS_PRD.md) | A duplicated marker id captures one row, not two | `test_duplicate_id_captures_once` |
| [COS_PRD.md](../chief-of-staff/COS_PRD.md) | An unreadable file hides its markers without resolving their tasks | `test_unreadable_file_does_not_resolve` |
| [COS_PRD.md](../chief-of-staff/COS_PRD.md) | Marker removal resolves the task; `:working`/`:failed` mirror onto it | `test_marker_removed_resolves_task`, `test_marker_status_mirrors_onto_task` |
| [COS_PRD.md](../chief-of-staff/COS_PRD.md) | Human state edits stand; hand-added rows are never reconciled | `test_human_done_stands_while_bare_marker_lives`, `test_human_added_task_untouched_by_reconcile` |
| [COS_PRD.md](../chief-of-staff/COS_PRD.md) | `indiana log` tails in order; `-n` limits; `--json` is machine-shaped | `test_log_tail_order_and_json` |
| [COS_PRD.md](../chief-of-staff/COS_PRD.md) | Dispatch lifecycle (`claimed`/`done`) lands in the repo's log | `test_autorun_success_resolves_and_commits` |

## E13 — Auto-run dispatch (ACP)
Behind `--features test-support`; driven against a mock ACP agent for determinism.
| Ref | Requirement | Test |
|-----|-------------|------|
| [IN_AUTORUN.md](IN_AUTORUN.md) | `-a` / `--auto` parses to `auto`, stripped from the message; only on directives | `test_auto_flag_short`, `test_auto_flag_ignored_on_non_directive`, `test_auto_runnable_directives_only` |
| [IN_AUTORUN.md](IN_AUTORUN.md), [IN_LINE.md](IN_LINE.md) | Claim mints an id + `working`; flags stay in source, idempotent | `test_claim_working_mints_id_and_keeps_flag`, `test_claim_working_idempotent`, `test_claim_keeps_flags_in_source_order` |
| [IN_AUTORUN.md](IN_AUTORUN.md) | A `::fix -a` marker is claimed, dispatched, resolved, and committed | `test_autorun_success_resolves_and_commits` |
| [IN_AUTORUN.md](IN_AUTORUN.md) | A turn that leaves the marker → `failed`, not re-dispatched | `test_autorun_failure_marks_failed` |
| [IN_AUTORUN.md](IN_AUTORUN.md) | `config.auto_run` off leaves the marker untouched | `test_autorun_disabled_leaves_marker` |
| [IN_AUTORUN.md](IN_AUTORUN.md) | Permission requests are auto-granted (allow-always preferred) | `test_grant_permission_prefers_allow_always` |
| [IN_AUTORUN.md](IN_AUTORUN.md) | ACP form question becomes a live job, accepts a human response, and resumes the same turn | `test_autorun_question_pauses_and_resumes` |
| [IN_AUTORUN.md](IN_AUTORUN.md) | Repo-local `model` is selected through ACP before the prompt | `test_autorun_selects_repo_model` |
| [IN_AUTORUN_ARCHITECTURE.md](IN_AUTORUN_ARCHITECTURE.md) | One save burst launches one turn; distinct markers in one repo run serially | `test_autorun_debounces_save_burst_to_one_turn`, `test_autorun_serializes_turns_per_repo` |
| [IN_COMMANDS.md](IN_COMMANDS.md) | Positive numeric flags become group metadata and are stripped from messages; `-a` may coexist and is consumed on claim in either order | `test_numeric_group_flag`, `test_numeric_groups_support_multiple_labels`, `test_numeric_group_coexists_with_auto_in_either_order`, `test_claim_group_retains_group_and_strips_auto_in_either_order` |
| [IN_AUTORUN.md](IN_AUTORUN.md) | Status reports sorted group counts; grouped Copy filters the payload; Run dispatches all members as one turn | `test_group_summary_copy_and_run_one_turn` |
| [IN_AUTORUN.md](IN_AUTORUN.md) | A grouped turn that leaves markers marks every surviving member failed | `test_group_failure_marks_all_survivors_failed` |

## What not to test
- OS behavior: FSEvents delivery, `rename` atomicity, `fsync` durability — these are OS contracts, not Indiana's.
- Tauri/NSPanel rendering: visual tests go in the menulet, not in Indiana core.
- Performance targets (sub-500ms): these are benchmarks, not pass/fail tests. A separate `cargo bench` suite.
- External tool behavior: `rg` output format, clipboard API on macOS — integration smoke, not unit coverage.

## Open
- E11 watch tests may be flaky by nature. Keep in CI while fast and stable; move to manual-only if they start failing nondeterministically.
- E13 auto-run tests are watch- and process-driven (spawn a daemon + a mock ACP agent), so the same flakiness caveat applies. They are deterministic single-threaded; treat parallel timing failures as environmental, not regressions.
