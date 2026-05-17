# hooks/

> **Back to:** [`../README.md`](../README.md) · [`../CLAUDE.md`](../CLAUDE.md) · [`../CLAUDE.local.md`](../CLAUDE.local.md)

**Status:** placeholder. Likely remains empty.

This directory is the home for **Cargo build hooks** (`build.rs` artefacts, build-script helpers, codegen scaffolding) IF any are needed. Current expectation: **none**. The m8 `m8_povm_build_prereq` module (Cluster D — [`../ai_specs/modules/cluster-D/m8_povm_build_prereq.md`](../ai_specs/modules/cluster-D/m8_povm_build_prereq.md)) carries a tiny refuse-build assertion that may land as a `build.rs` in the root or as part of the m8 module crate — TBD at G7 Zen audit. If `build.rs` is chosen, it would conventionally live at repo root not here; this directory is for any **shared build-script helpers** the codebase grows over time. Do NOT confuse with [`../.claude/hooks/`](../.claude/hooks/) which is the **Claude Code hook configuration** (PreToolUse / PostToolUse / Stop handlers — entirely different machinery). Do NOT confuse with workspace-charter habitat hooks at `~/.claude/settings.json`.

> **Back to:** [`../README.md`](../README.md) · [`../CLAUDE.md`](../CLAUDE.md) · [`../CLAUDE.local.md`](../CLAUDE.local.md)
