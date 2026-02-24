---
id: TASK-ydj7q
title: Publish markplane-core to crates.io
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
created: 2026-02-13
updated: 2026-02-23
---

# Publish markplane-core to crates.io

## Description

Publish markplane crates to crates.io so developers can install from source with `cargo install markplane`. This is the Rust-native distribution path. Note that `cargo install` won't include the web UI by default since it doesn't run npm — this is the CLI-only install path.

## Acceptance Criteria

- [ ] `markplane-core` published to crates.io
- [ ] `markplane-cli` published to crates.io (depends on markplane-core)
- [ ] `markplane-mcp` published to crates.io (depends on markplane-core)
- [ ] `cargo install markplane-cli` installs a working `markplane` binary (CLI only, no web UI)
- [ ] README and docs clarify that `cargo install` gives CLI + MCP only; use GitHub Releases or Homebrew for the full binary with web UI

## Notes

- Need to set proper `package.repository`, `package.license`, `package.description` in each Cargo.toml
- Workspace dependency publishing order matters: core first, then cli and mcp
- Consider a `cargo-release` or `release-plz` integration for version bumping

## References

- [[EPIC-bb6pe]]
