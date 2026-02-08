---
summary: "Use docs-list to audit docs summaries and read_when triggers from the repo root."
read_when:
  - You need a quick inventory of docs pages and frontmatter quality.
  - You are adding or editing docs frontmatter and want parser-visible validation output.
title: "docs-list Tool"
---

# docs-list

`docs-list` scans the repository `docs/` directory and prints markdown paths,
summaries, and `read_when` hints.

## Run

```bash
docs-list
```

If not installed globally:

```bash
cargo run --manifest-path tooling/docs-list/Cargo.toml
```

## Frontmatter Expectations

- `summary` should be present and non-empty.
- `read_when` can be a bullet list or inline array.
- Missing or malformed frontmatter is surfaced in output so gaps are obvious.
