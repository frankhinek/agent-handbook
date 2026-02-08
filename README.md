# agent-handbook

Handbook for agentic engineering: culture, practices, and operating procedures for single-agent and multi-agent software delivery.

Repo URL target: `https://github.com/frankhinek/agent-handbook`

`AGENTS.md` is the canonical contract for agent behavior and should be suitable for direct use as global `~/.codex/AGENTS.md`.

## Contents

1. `AGENTS.md`: core operating contract (principles, standards, evaluations).
2. `runbooks/`: repeatable operational workflows.
3. `skills/`: task-specific playbooks that extend the base contract.
4. `tooling/`: scripts, CLIs, and integration docs.

## Operating Model

1. Keep core expectations centralized in `AGENTS.md`.
2. Use runbooks for repeatable execution paths.
3. Use skills for specialized task workflows.
4. Evaluate outputs with the rubric in `AGENTS.md`.

## Getting Started

1. Copy `AGENTS.md` to your global agent config path (for Codex: `~/.codex/AGENTS.md`).
2. Add project-specific overrides in repository-local `AGENTS.md` files.
3. Author runbooks and skills for your highest-frequency workflows first.
