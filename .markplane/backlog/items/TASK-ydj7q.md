---
id: TASK-ydj7q
title: Publish markplane crates to crates.io
status: backlog
priority: low
type: chore
effort: small
tags:
- release
epic: EPIC-bb6pe
plan: null
depends_on: []
blocks: []
assignee: null
position: a0
related: []
created: 2026-02-13
updated: 2026-02-24
---

# Publish markplane crates to crates.io

## Description

Publish markplane crates to crates.io so developers can install from source with `cargo install markplane`. Two crates: `markplane-core` (library) and `markplane-cli` (binary — includes CLI, MCP server, and web server).

By default `cargo install markplane` gives CLI + MCP only. For the full binary with embedded web UI, users can `cargo install markplane --features embed-ui` — this works without Node.js because the pre-built `out/` directory is included in the crate package.

## Acceptance Criteria

- [ ] `markplane-core` published to crates.io
- [ ] `markplane-cli` published to crates.io (depends on markplane-core)
- [ ] `cargo install markplane` installs a working binary (CLI + MCP)
- [ ] `cargo install markplane --features embed-ui` installs binary with embedded web UI
- [ ] Pre-built `out/` included in crate package via `package.include` so embed-ui works without Node.js
- [ ] README and docs clarify install options: brew/GitHub Releases for full binary, `cargo install` for CLI-only or `--features embed-ui` for full

## Notes

- Need to set proper `package.repository`, `package.license`, `package.description` in each Cargo.toml
- Workspace dependency publishing order matters: core first, then cli
- Consider a `cargo-release` or `release-plz` integration for version bumping
- Primary install paths (brew, GitHub Releases) always include the web UI — the feature flag is mainly a crates.io concern
- Include `out/` in crate tarball by adding it to `package.include` in markplane-cli's Cargo.toml

## References

- [[EPIC-bb6pe]]
