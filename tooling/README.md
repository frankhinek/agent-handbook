# Tooling Reference

Scripts, CLIs, and integrations used by agents.

Document for each tool:

1. What problem it solves.
2. Usage examples.
3. Required permissions and credentials.
4. Failure modes and troubleshooting.

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
