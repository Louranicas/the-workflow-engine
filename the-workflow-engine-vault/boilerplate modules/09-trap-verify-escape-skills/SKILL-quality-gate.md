---
name: quality-gate
description: ULTRAPLATE quality gate enforcement meta-skill. Runs the mandatory 4-stage quality gate (cargo check -> clippy -> pedantic -> test) with zero tolerance. Use before any commit, deployment, or module completion.
allowed-tools:
  - Bash
  - Read
  - Edit
---

# Quality Gate

Mandatory 4-stage quality gate for all ULTRAPLATE Rust code.

## Gate Sequence (MANDATORY ORDER)

```bash
CARGO_TARGET_DIR=./target cargo check 2>&1 | tail -20 && \
cargo clippy -- -D warnings 2>&1 | tail -20 && \
cargo clippy -- -D warnings -W clippy::pedantic 2>&1 | tail -20 && \
CARGO_TARGET_DIR=./target cargo test --lib --release 2>&1 | tail -30
```

## Rules

- **Zero tolerance:** 0 errors AND 0 warnings at EVERY stage
- **Order matters:** Each stage must pass before proceeding
- **Test density:** 50 tests per module minimum
- **Standards:** No unwrap() outside tests, no unsafe, doc comments on public items

## Source Skills

- `the_code_synthor_v7/.claude/skills/compliance-validation/SKILL.md`
- `sphere_vortex_framework/.claude/skills/compilation-compliance/SKILL.md`
- `the-orchestrator/the tool maker/.claude/skills/quality-gate-enforcer/SKILL.md`
- `the-orchestrator/the tool maker/.claude/skills/rust-compilation-enforcer/SKILL.md`
- `the_code_synthor_v7/.claude/skills/module-testing/SKILL.md`


---

> Vault navigation: [[../../BOILERPLATE_INDEX|BOILERPLATE_INDEX]] · [[../../README|boilerplate modules README]] · [[../../../HOME|HOME]] · [[../../../MASTER_INDEX|MASTER_INDEX]]
> Reference-only clone — see [[../../BOILERPLATE_INDEX]] for upstream source + target-module mapping.
