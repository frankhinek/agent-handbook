# docs-list

Rust CLI inspired by [OpenClaw's `docs-list.js`](https://github.com/openclaw/openclaw/blob/main/scripts/docs-list.js).

## What It Does

- Scans `./docs` recursively for markdown files.
- Skips hidden paths and excluded dirs (`archive`, `research`).
- Reads frontmatter metadata:
  - `summary`
  - `read_when`
- Prints a sortable docs inventory for humans and agents.

## Build

From repository root:

```bash
cargo build --release --manifest-path tooling/docs-list/Cargo.toml
```

## Run

From repository root:

```bash
cargo run --manifest-path tooling/docs-list/Cargo.toml
```

If installed globally:

```bash
docs-list
```

## Output Rules

- Missing `docs/` directory: exits `1`.
- `docs` path is not a directory: exits `1`.
- Missing or invalid frontmatter: prints file with reason marker, for example:
  - `[missing front matter]`
  - `[unterminated front matter]`
  - `[summary key missing]`
  - `[summary is empty]`

## Verify

```bash
cargo nextest run --manifest-path tooling/docs-list/Cargo.toml
```
