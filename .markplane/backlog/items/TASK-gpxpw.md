---
id: TASK-gpxpw
title: Build GitHub Actions release workflow with embedded web UI
status: draft
priority: high
type: chore
effort: large
epic: EPIC-bb6pe
plan: null
depends_on:
- TASK-yzftd
blocks:
- TASK-8xvxq
related: []
assignee: null
tags:
- ci
- release
position: a2
created: 2026-02-13
updated: 2026-02-15
---

# Build GitHub Actions release workflow with embedded web UI

## Description

Create a GitHub Actions release workflow triggered by version tags (e.g. `v0.1.0`). The workflow builds the Next.js frontend, then compiles the Rust binary with `--features embed-ui` for each target platform. Uploads the resulting binaries to a GitHub Release with checksums.

## Acceptance Criteria

- [ ] Triggered by pushing a `v*` tag
- [ ] Builds frontend: `npm install && npm run build` in `crates/markplane-web/ui/`
- [ ] Builds Rust binary with `--features embed-ui` for:
  - macOS arm64 (Apple Silicon)
  - macOS x86_64
  - Linux x86_64 (musl for static linking)
  - Windows x86_64
- [ ] Creates GitHub Release with binary assets and SHA256 checksums
- [ ] Release notes auto-generated from git log or changelog
- [ ] Binaries are named consistently (e.g. `markplane-v0.1.0-darwin-arm64.tar.gz`)

## Notes

- Use `cross` or `cargo-zigbuild` for cross-compilation if native runners aren't available for all targets
- Frontend build only needs to happen once — share the `out/` artifact across platform builds
- Consider using `actions/upload-artifact` to pass the frontend build between jobs
- macOS universal binary (arm64 + x86_64 via `lipo`) is an option to simplify

## References

- [[EPIC-bb6pe]]
- [[TASK-yzftd]]
- [[TASK-8xvxq]]
