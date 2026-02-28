---
id: EPIC-bb6pe
title: Release & Distribution
status: later
priority: high
started: null
target: null
related: []
tags: []
created: 2026-02-10
updated: 2026-02-26
---

# Release & Distribution

## Objective

Make markplane easy to install for end users. No one should need to run npm or build from source just to use it. CI validates every PR, release builds produce ready-to-go binaries, and package managers handle installation.

## Key Results

- [ ] PRs are validated by CI (cargo test, clippy, frontend build)
- [ ] Tagged releases produce binaries with embedded web UI for macOS (arm64/x86), Linux (x86), and Windows (x86)
- [ ] `brew install markplane/tap/markplane` works
- [ ] `cargo install markplane` works from crates.io

## Notes

- Release binaries need the full build pipeline: npm install + npm build + cargo build with embed-ui
- Homebrew formula pulls from GitHub Release artifacts — depends on the release workflow being solid first
- crates.io publishing is lower priority since it requires users to build from source (no embed-ui by default)
