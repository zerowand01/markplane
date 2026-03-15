---
id: TASK-t672g
title: Expand release smoke tests beyond --version
status: backlog
priority: medium
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
- ci
position: a3V
created: 2026-03-11
updated: 2026-03-11
---

# Expand release smoke tests beyond --version

## Description

The release workflow's smoke test only runs `markplane --version`. This catches linking errors but not broken functionality — a binary could pass `--version` while core features like `init`, `sync`, or `mcp` are broken due to missing embedded assets or runtime issues.

Adding a quick functional smoke test (`markplane init` + `markplane sync` in a temp directory) would catch these failures before they ship to users.

## Acceptance Criteria

- [ ] Release smoke test runs `markplane init` in a temp directory and verifies it succeeds
- [ ] Release smoke test runs `markplane sync` after init and verifies it succeeds
- [ ] Tests clean up temp directory after completion

## Notes

Keep it minimal — the goal is catching "binary is fundamentally broken" not comprehensive integration testing (that's what the test suite is for). A temp dir with `markplane init && markplane sync` covers the critical path.

## References
