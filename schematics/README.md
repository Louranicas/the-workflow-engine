# schematics/

> **Back to:** [`../README.md`](../README.md) · [`../CLAUDE.md`](../CLAUDE.md) · [`../CLAUDE.local.md`](../CLAUDE.local.md)

**Status:** placeholder + cross-link only. Real diagrams live in dedicated homes.

This top-level `schematics/` directory exists for symmetry with other habitat services that carry diagrams at repo root, but **diagrams for `workflow-trace` live in two dedicated homes** to keep concerns separated:

- [`../ai_docs/schematics/`](../ai_docs/schematics/) — **design-time schematics** (Mermaid diagrams of module composition, cross-cluster synergies, data flow shapes, layer architecture). Read this for "how the codebase is structured."
- [`../ultramap/schematics/`](../ultramap/schematics/) — **operational flow maps** (DATA / CONTROL / CONTEXTUAL / INVARIANT views; master flow map). Read this for "how the codebase behaves at runtime."

If a schematic doesn't fit either home (e.g. a one-off whiteboard photograph or a third-party tool's export), it may land here. Default: send all new schematics to one of the two homes above. Do NOT duplicate diagrams across homes; cross-link instead.

> **Back to:** [`../README.md`](../README.md) · [`../CLAUDE.md`](../CLAUDE.md) · [`../CLAUDE.local.md`](../CLAUDE.local.md)
