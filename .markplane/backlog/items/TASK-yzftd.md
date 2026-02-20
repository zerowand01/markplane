---
id: TASK-yzftd
title: Set up GitHub Actions CI pipeline
status: draft
priority: high
type: chore
effort: medium
tags:
- ci
- release
epic: EPIC-bb6pe
plan: null
depends_on: []
blocks:
- TASK-gpxpw
assignee: null
position: a1
created: 2026-02-13
updated: 2026-02-19
---

# Set up GitHub Actions CI pipeline

## Description

Create a GitHub Actions CI workflow that runs on every push and PR. Should validate the full stack: Rust (all 3 crates) and the Next.js frontend. This is the foundation for the release workflow — we need confidence that the build is clean before cutting releases.

## Acceptance Criteria

- [ ] Workflow triggers on push to main and on PRs
- [ ] Runs `cargo clippy --all-targets --all-features -- -D warnings`
- [ ] Runs `cargo test --workspace` (all 230 tests)
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
