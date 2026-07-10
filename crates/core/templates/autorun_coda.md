INDIANA AUTO-RUN — you were dispatched automatically to act on the single marker above. This run is autonomous: do not ask for confirmation. When you have acted on it:

1. Delete the marker line from `{path}` — the line at {path}:{line} bearing the `{marker}` marker. Remove only that line; leave the surrounding content intact.
2. Commit the change following the repository's commit discipline (see `docs/AGENT_COMMIT.md` if present): a small, focused, local commit. Do not push.

If you cannot complete the task, leave the marker line in place and stop; the daemon will mark it failed.
