---
id: TASK-tvz3p
title: Add Linux ARM64 (aarch64) release target
status: backlog
priority: medium
type: chore
effort: small
epic: null
plan: null
depends_on: []
blocks: []
related: []
assignee: null
tags:
- release
- installation
position: a4
created: 2026-03-11
updated: 2026-03-11
---

# Add Linux ARM64 (aarch64) release target

## Description

We currently build for 4 targets (macOS x86/arm, Linux x86_64, Windows x86_64) but have no Linux ARM64 binary. The install script explicitly rejects `Linux arm64` with an error. This is a growing gap as AWS Graviton, ARM-based CI runners, and ARM servers become mainstream.

GitHub Actions now offers `ubuntu-24.04-arm` runners (GA, free for public repos since Aug 2025). This means we can add native ARM64 Linux builds with no cross-compilation complexity — just another matrix entry using the same pattern as our existing builds.

## Acceptance Criteria

- [ ] `aarch64-unknown-linux-musl` binary built in release workflow on `ubuntu-24.04-arm` runner
- [ ] Binary included in GitHub release artifacts with checksum
- [ ] `install.sh` accepts Linux arm64/aarch64 and downloads the correct binary
- [ ] Homebrew formula template includes `on_linux` + `on_arm` block for Linux ARM64
- [ ] Smoke test passes on ARM runner

## Notes

**Approach**: Use native `ubuntu-24.04-arm` runner (not cross-compilation or QEMU).

Files to change:
- `.github/workflows/release.yml` — add matrix entry `{target: aarch64-unknown-linux-musl, os: ubuntu-24.04-arm}`
- `.github/workflows/release.yml` — extend musl-tools install condition to include new target
- `.github/formula-template.rb` — add `on_linux` + `on_arm` SHA/URL block
- `install.sh` — remove Linux arm64 rejection, map to `aarch64-unknown-linux-musl` target

## References

- [[TASK-8xvxq]] (Homebrew tap), [[TASK-3zks9]] (install script)
- https://github.blog/changelog/2025-08-07-arm64-hosted-runners-for-public-repositories-are-now-generally-available/
