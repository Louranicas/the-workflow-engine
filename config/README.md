# config/

> **Back to:** [`../README.md`](../README.md) · [`../CLAUDE.md`](../CLAUDE.md) · [`../CLAUDE.local.md`](../CLAUDE.local.md)

**Status:** placeholder + 3 `.template` files. Real `.toml` configs land post-G9.

This directory is the **runtime TOML configuration root** for `workflow-trace`. Post-G9 it carries: `default.toml` (baseline runtime config; loaded first), `production.toml` (production overrides), `devenv-service.toml` (devenv.toml fragment for `~/.config/devenv/devenv.toml` registration). Pre-G9, the `.template` files in this directory ([`default.toml.template`](default.toml.template) · [`production.toml.template`](production.toml.template) · [`devenv-service.toml.template`](devenv-service.toml.template)) document the planned shape. The `.template` suffix is **load-bearing** — it ensures no service-loader or hook accidentally picks them up as live config. Real configs MUST drop the suffix and pass [`../ai_docs/optimisation-v7/STANDARDS/`](../ai_docs/optimisation-v7/STANDARDS/) validation gates before landing.

> **Back to:** [`../README.md`](../README.md) · [`../CLAUDE.md`](../CLAUDE.md) · [`../CLAUDE.local.md`](../CLAUDE.local.md)
