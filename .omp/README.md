# .omp/

OMP native context directory (priority 100, shadows all other providers).

- `AGENTS.md` is a symlink to `../CLAUDE.md` — the project's single source of instructions. OMP auto-discovers `.omp/AGENTS.md` and injects it into the session's opening context. Keeping the root `CLAUDE.md` as the canonical file avoids duplication; the symlink bridges the two conventions.
