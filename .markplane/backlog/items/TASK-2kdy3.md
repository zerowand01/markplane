---
id: TASK-2kdy3
title: Add cargo-binstall metadata to markplane-cli
status: backlog
priority: low
type: chore
effort: xs
epic: null
plan: null
depends_on: []
blocks: []
related: []
assignee: null
tags:
- release
- installation
position: a5
created: 2026-03-11
updated: 2026-03-11
---

# Add cargo-binstall metadata to markplane-cli

## Description

`cargo-binstall` lets users install pre-built binaries via `cargo binstall markplane` instead of compiling from source. It works by reading `[package.metadata.binstall]` from the crate's Cargo.toml to find the download URL pattern. We already publish release binaries in the right format — we just need the metadata so binstall knows where to find them.

This is a zero-infra-cost improvement that becomes valuable once we publish to crates.io ([[TASK-ydj7q]]).

## Acceptance Criteria

- [ ] `crates/markplane-cli/Cargo.toml` has `[package.metadata.binstall]` section with correct URL template and binary name
- [ ] URL template matches our actual release artifact naming convention

## Notes

Example metadata:
```toml
[package.metadata.binstall]
pkg-url = "{ repo }/releases/download/v{ version }/markplane-v{ version }-{ target }.tar.gz"
bin-dir = "markplane"
pkg-fmt = "tgz"
```

## References

- [[TASK-ydj7q]] (publish to crates.io)
- https://github.com/cargo-bins/cargo-binstall
