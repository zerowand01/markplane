---
id: TASK-8xvxq
title: Create Homebrew tap for markplane
status: done
priority: high
type: chore
effort: medium
epic: EPIC-bb6pe
plan: null
depends_on: []
blocks: []
related: []
assignee: null
tags:
- release
- homebrew
position: a0
created: 2026-02-13
updated: 2026-03-10
---

# Create Homebrew tap for markplane

## Description

Create a Homebrew tap repository (`zerowand01/homebrew-markplane`) so users can install markplane with `brew install zerowand01/markplane/markplane`. The formula should download pre-built binaries from GitHub Releases rather than building from source, so users don't need Rust or Node.js.

## Acceptance Criteria

- [x] Separate repo `zerowand01/homebrew-markplane` created with `Formula/markplane.rb`
- [x] Formula uses `on_macos`/`on_linux` + `on_arm`/`on_intel` blocks for platform detection
- [x] Formula downloads correct binary for macOS arm64, macOS x86_64, Linux x86_64 (musl)
- [x] `brew install zerowand01/markplane/markplane` installs a working binary with embedded web UI
- [x] Formula includes `test` block (`markplane --version` check)
- [x] `brew audit --strict --new` passes
- [ ] `brew upgrade` works when new releases are published (automation in place; will verify on next release)
- [x] Release workflow updated with `update-homebrew` job that auto-updates the formula
- [x] `TAP_GITHUB_TOKEN` secret created for cross-repo push access
- [x] README.md Homebrew stub section (from [[TASK-3zks9]]) filled in with actual install command

## Implementation Summary

Completed 2026-03-10.

- **Tap repo**: `zerowand01/homebrew-markplane` with `Formula/markplane.rb` and `README.md`
- **Formula**: Uses `on_macos`/`on_linux` + `on_arm`/`on_intel` blocks; no explicit `version` line (extracted from URL per Homebrew convention)
- **Automation**: `.github/formula-template.rb` template + `update-homebrew` job in release workflow renders and pushes on each release
- **Secret**: `TAP_GITHUB_TOKEN` (fine-grained PAT, no expiration, scoped to tap repo only). See [[NOTE-29e3c]] for rotation procedure
- **Docs**: README.md updated, `docs/releasing.md` updated with Homebrew step and secrets table

## Key Decisions

- **No explicit `version` line** in formula — Homebrew extracts it from the URL; `brew audit` flags it as redundant
- **Linux target**: `x86_64-unknown-linux-musl` (fully static, matches release workflow)
- **No Linux ARM**: Not in release matrix yet; add later if needed
- **No shell completions**: `clap_complete` is a dep but no `completions` subcommand exists yet — follow-up task
- **No token expiration**: Low-risk token (can only write to single-file tap repo); optional calendar-based rotation recommended

## Future Work

- Add `completions` subcommand and `generate_completions_from_executable` to formula
- Add `strip`/`lto` release profile optimizations
- Submit to homebrew-core once popular enough for `brew install markplane` without tap

## References

- [[EPIC-bb6pe]]
- [[TASK-gpxpw]]
- [[TASK-3zks9]]
