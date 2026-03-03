---
id: TASK-hvwsk
title: Configure GitHub branch protection for master
status: backlog
priority: high
type: chore
effort: xs
epic: EPIC-bb6pe
plan: null
depends_on:
- TASK-yzftd
blocks: []
related: []
assignee: null
tags:
- ci
- github
position: a0l
created: 2026-03-02
updated: 2026-03-02
---

# Configure GitHub branch protection for master

## Description

Once CI is live (TASK-yzftd), configure GitHub branch protection on `master` so that CI actually gates merges. Without this, CI runs but doesn't prevent broken code from landing. This is the step that connects the CI pipeline to quality enforcement.

## Acceptance Criteria

- [ ] `master` branch is protected
- [ ] CI status checks required to pass before merging
- [ ] Force pushes blocked on `master`
- [ ] Squash merge set as default (or only) merge strategy

## Notes

- This is a GitHub settings task, not code. Done via repo Settings → Branches → Branch protection rules.
- Can also be configured via `gh api` if you prefer CLI.
- Use PRs even as solo maintainer — creates a clean paper trail for releases and ensures CI runs in the same path it will for contributors.
- "Require branch to be up-to-date before merging" is optional and can be noisy — skip for now.

## References

- [[EPIC-bb6pe]]
- [[TASK-yzftd]]
