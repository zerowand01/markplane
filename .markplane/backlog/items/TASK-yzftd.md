---
id: TASK-yzftd
title: Set up GitHub Actions CI pipeline
status: draft
priority: high
type: chore
effort: medium
epic: EPIC-bb6pe
plan: null
depends_on: []
blocks:
- TASK-gpxpw
related: []
assignee: null
tags:
- ci
- release
position: a1
created: 2026-02-13
updated: 2026-02-21
---

# Set up GitHub Actions CI pipeline

## Description

Create a GitHub Actions CI workflow that runs on every push and PR. Should validate the full stack: both Rust workspace crates (`markplane-core`, `markplane-cli`) and the Next.js frontend (`crates/markplane-web/ui/`). This is the foundation for the release workflow — we need confidence that the build is clean before cutting releases.

## Acceptance Criteria

- [ ] Workflow triggers on push to main and on PRs
- [ ] Runs `cargo fmt --check` (formatting gate)
- [ ] Runs `cargo clippy --all-targets --all-features -- -D warnings`
- [ ] Runs `cargo test --workspace`
- [ ] Runs `npm install && npm run build` in `crates/markplane-web/ui/`
- [ ] Runs on ubuntu-latest (Linux), optionally macOS for cross-platform validation
- [ ] Uses Rust toolchain caching for fast CI times
- [ ] Uses Node.js with npm caching

## Notes

- Rust stable toolchain (edition 2024, rust-version 1.93.0)
- Node.js version should match what's in the Next.js project
- Consider matrix strategy for multiple OS targets if needed for release prep

## References

- [[EPIC-bb6pe]]
- [[TASK-gpxpw]]
