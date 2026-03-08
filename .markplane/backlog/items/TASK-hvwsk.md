---
id: TASK-hvwsk
title: Configure GitHub branch protection for master
status: done
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
updated: 2026-03-07
---

# Configure GitHub branch protection for master

## Description

Once CI is live (TASK-yzftd), configure GitHub branch protection on `master` so that CI actually gates merges. Without this, CI runs but doesn't prevent broken code from landing. This is the step that connects the CI pipeline to quality enforcement.

## Acceptance Criteria

- [x] Use GitHub **rulesets** (not legacy branch protection rules)
- [x] `master` branch is protected (cannot be deleted)
- [x] CI status checks required to pass before merging
- [x] Force pushes blocked on `master`
- [x] Linear history required
- [x] Squash merge set as default merge strategy (repo-level Settings → General → Pull Requests)
- [x] Owner/maintainer bypass enabled for direct `.markplane/` pushes to master
- [x] Set enforcement to **Active**
- [x] Enable "Automatically delete head branches" (repo-level Settings → General → Pull Requests)

## Decisions

- **Rulesets over branch protection rules**: GitHub recommends rulesets as the successor to legacy branch protection rules. Rulesets support multiple stacking rules, evaluation mode (dry-run), and are more flexible.
- **Squash merge is a repo-level setting**: Not part of rulesets — configured separately under Settings → General → Pull Requests.
- **Owner bypass for `.markplane/` workflows**: Owners get ruleset bypass so project management updates (task status, notes, triage) can be pushed directly to master without a PR. GitHub rulesets don't support per-path exemptions, so this relies on discipline — code changes still go through PRs. Task status updates tied to code work should ride in the feature branch PR.
- **Skip "require branch to be up-to-date before merging"**: Optional and noisy — not worth it for now.

## Notes

- This is a GitHub settings task, not code. Done via repo Settings → Rules → Rulesets.
- Can also be configured via `gh ruleset create` CLI.
- PRs create a clean paper trail for releases and ensure CI runs consistently for all contributors.
- Consider requiring conversation resolution on PRs as the contributor base grows.

## References

- [[EPIC-bb6pe]]
- [[TASK-yzftd]]
