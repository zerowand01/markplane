---
id: NOTE-83hku
title: 'Architecture decision: separate repo for OpenClaw plugin'
status: active
type: decision
related:
- TASK-3pgfu
tags:
- openclaw
- architecture
created: 2026-03-27
updated: 2026-03-27
---

# Architecture decision: separate repo for OpenClaw plugin

## Decision

The OpenClaw plugin (`markplane-memory`) lives in a separate npm package repository ([zerowand01/markplane-memory](https://github.com/zerowand01/markplane-memory)), not in the main Markplane Rust repo.

## Context

Markplane is a Rust project (edition 2024, workspace with `markplane-core` and `markplane-cli`). The OpenClaw plugin is a TypeScript npm package (~60 lines). Integrating it into the Rust repo would require adding Node.js/npm tooling to the build system.

## Rationale

- **Build system isolation**: Adding npm/TypeScript tooling to a Rust workspace creates disproportionate complexity for ~60 lines of glue code.
- **Independent versioning**: OpenClaw's plugin API may change independently of Markplane's release cycle. The plugin can adapt without requiring a Markplane release.
- **Low coupling**: The plugin reads `.context/` files from disk and calls `appendSystemContext`. It has zero dependency on Markplane's internals — no imports, no bundling.
- **Contributor access**: Plugin contributors don't need to understand Markplane's Rust codebase.
- **Naming**: Using `markplane-memory` (not `openclaw-markplane`) avoids potential trademark issues and positions the plugin by function rather than platform.

## Alternatives considered

- **Monorepo with npm workspace**: Rejected — adds Node.js as a build dependency for all Markplane contributors, even those who don't touch the plugin.
- **Rust-based plugin**: Not possible — OpenClaw plugins must be npm packages with a JS entry point.
- **No plugin (manual setup only)**: Viable but inferior — the `AGENTS.md` instruction approach is unreliable across compaction. The `before_prompt_build` hook is the one thing that cannot be replicated through config alone.
