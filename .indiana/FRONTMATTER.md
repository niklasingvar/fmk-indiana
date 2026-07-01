# Default markdown frontmatter

Indiana markdown files open with the block below. `indiana frontmatter --write`
prepends it to any `.md` file missing frontmatter; `indiana frontmatter` (no
flag) only reports. Edit the block to change the default for this repo.

```yaml
---
status: draft
purpose: TODO
approval: pending
---
```

The lint walk respects `.gitignore` and skips `.indiana/`, `.git/`, `target/`,
`node_modules/`, and `skills/` (third-party skill content has its own
frontmatter convention).

Fields:
- `status` — `draft` or `approved`
- `purpose` — one line on what the file is for; `TODO` marks new files for a human to fill
- `approval` — `pending` or `approved`
