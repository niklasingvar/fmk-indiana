INDIANA BATCH RUN — you were dispatched manually to act on all {count} markers in group -{group}. This run is autonomous: do not ask for confirmation. Treat the group as one unit and make one commit.

When you have acted on every marker, delete each marker line below. Remove only those lines; leave surrounding content intact:

{targets}

Commit the complete batch following the repository's commit discipline (see `docs/AGENT_COMMIT.md` if present). Do not push.

If you cannot complete a marker, leave its line in place and stop; the daemon will mark every surviving group member failed.
