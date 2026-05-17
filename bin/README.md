# bin/

> **Back to:** [`../README.md`](../README.md) · [`../CLAUDE.md`](../CLAUDE.md) · [`../CLAUDE.local.md`](../CLAUDE.local.md)

**Status:** placeholder. Both binary subdirs exist; their `main.rs` files land post-G9.

This directory is the **top-level binary directory** for the workflow-trace two-binary split. The split itself (rather than a single-binary CLI with subcommands) is a deliberate architectural decision per [[Modules Synergy Clusters and Feature Verification S1001982]] § Two-binary architecture — it prevents read-heavy crystallisation work from accidentally triggering dispatch (the class-of-bug that killed the `habitat-loop-engine` ancestor). Both binaries share the in-crate `workflow_core` library (types, schemas, namespace constants) via the ORAC single-Cargo pattern, not the LCM cross-workspace pattern.

The two subdirectories:

- [`wf-crystallise/`](wf-crystallise/) — read-heavy binary owning modules **m1-m23 + m40-m42**; ingest substrate, observe habitat, correlate, propose variants, emit substrate feedback. See [`wf-crystallise/README.md`](wf-crystallise/README.md).
- [`wf-dispatch/`](wf-dispatch/) — write-heavy binary owning modules **m30-m33**; curated bank, selector, Conductor dispatch, 4-agent verifier. See [`wf-dispatch/README.md`](wf-dispatch/README.md).

Both binaries' `Cargo.toml` `[[bin]]` declarations land post-G9 inside the (currently nonexistent) root `Cargo.toml`. HOLD-v2 still forbids both `.rs` source files and the root `Cargo.toml` itself.

> **Back to:** [`../README.md`](../README.md) · [`../CLAUDE.md`](../CLAUDE.md) · [`../CLAUDE.local.md`](../CLAUDE.local.md)
