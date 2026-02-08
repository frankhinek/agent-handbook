# Tooling Reference

Scripts, CLIs, and integrations used by agents.

Document for each tool:

1. What problem it solves.
2. Usage examples.
3. Required permissions and credentials.
4. Failure modes and troubleshooting.

---

## docs-list
List markdown docs with `summary` and `read_when` metadata.

**Location**: `tooling/docs-list`

**Commands**:
```bash
docs-list
cargo run --manifest-path tooling/docs-list/Cargo.toml
cargo run --manifest-path tooling/docs-list/Cargo.toml | head -n 20
```

**Requirements**:
- Run from repository root.
- `docs/` directory must exist.

**Failure modes**:
- Exits with `docs:list: missing docs directory. Run from repo root.`
- Exits with `docs:list: docs path is not a directory.`
- Prints per-file metadata issues:
  - `[missing front matter]`
  - `[unterminated front matter]`
  - `[summary key missing]`
  - `[summary is empty]`

---

## gh
GitHub CLI for PRs, issues, CI, releases.

**Usage**: `gh help`

When someone shares a GitHub URL, use `gh` to read it:
```bash
gh issue view <url> --comments
gh pr view <url> --comments --files
gh run list / gh run view <id>
```
