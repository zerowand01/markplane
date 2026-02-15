---
id: TASK-023
title: Create Homebrew tap for markplane
status: draft
priority: medium
type: chore
effort: medium
tags:
- release
- homebrew
epic: EPIC-006
plan: null
depends_on:
- TASK-022
blocks: []
assignee: null
position: a0
created: 2026-02-13
updated: 2026-02-13
---

# Create Homebrew tap for markplane

## Description

Create a Homebrew tap repository (`homebrew-tap`) so users can install markplane with `brew install markplane/tap/markplane`. The formula should download pre-built binaries from GitHub Releases rather than building from source, so users don't need Rust or Node.js.

## Acceptance Criteria

- [ ] Separate repo `markplane/homebrew-tap` created
- [ ] Formula downloads the correct binary for the user's platform (macOS arm64/x86_64, Linux x86_64)
- [ ] `brew install markplane/tap/markplane` installs a working binary with embedded web UI
- [ ] `brew upgrade` works when new releases are published
- [ ] Formula updated automatically (or via script) when a new GitHub Release is created

## Notes

- Use Homebrew's binary bottle pattern — no source compilation
- Consider automating formula updates with a GitHub Action in the tap repo that triggers on new releases in the main repo
- Future: once popular enough, submit to homebrew-core for `brew install markplane` without tap

## References

- [[EPIC-006]]
- [[TASK-022]]
