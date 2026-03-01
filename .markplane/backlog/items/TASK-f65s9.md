---
id: TASK-f65s9
title: Seed markplane init with starter content and onboarding plan
status: backlog
priority: medium
type: feature
effort: medium
epic: EPIC-c5uem
plan: null
depends_on: []
blocks: []
related: []
assignee: null
tags:
- onboarding
- init
position: a5V
created: 2026-02-21
updated: 2026-03-01
---

# Seed markplane init with starter content and onboarding plan

## Description

`markplane init` currently scaffolds an empty directory structure. New users see blank INDEX files and empty directories with no guidance on how to use the system. Instead, seed `init` with a small set of real items that serve dual purpose: concrete examples of well-formed entities AND an actionable onboarding workflow.

This also solves the "project migration" problem (importing from GitHub Issues, Jira, plain TODOs, etc.) without building rigid importers. A starter plan provides AI-readable context that any AI assistant can use to help migrate existing work into markplane.

## Acceptance Criteria

- [ ] `markplane init` creates a starter epic, task(s), plan, and note — real items with real IDs
- [ ] Starter items demonstrate correct format: YAML frontmatter, body structure, cross-references between items
- [ ] A starter plan (linked to a setup task) contains onboarding steps including guidance for importing existing work from other tools
- [ ] Starter items form a coherent onboarding workflow that a user can follow, complete, and archive
- [ ] Starter content is concise — each item stays within the ~2000 token file convention
- [ ] `markplane init --empty` (or similar flag) skips starter content for users who don't want it

## Notes

**Starter item set** (illustrative, refine during implementation):
- One epic: "Project Setup" — groups the onboarding tasks
- One task: "Review and customize your markplane setup" — walks through config, templates, conventions
- One plan: "Import existing work into markplane" — linked to the task, contains format reference and migration guidance for GitHub Issues, Jira, TODOs, etc. This plan IS the AI migration context — any AI assistant reading it via `markplane_show` gets full instructions.
- One note: "Project decisions" or similar — seeds the notes pattern, shows how to capture decisions

**Why real items, not docs**: Items use the system to teach the system. They appear in INDEX files, show up in queries, can be completed and archived. An AI assistant sees them through normal MCP tools and learns the format by example.

**Why not automated importers**: Every source tool has a different export format. The scope is unbounded and maintenance-heavy. An AI assistant reading the onboarding plan can handle any import format flexibly. 80% of the value, near-zero maintenance.

**Template relationship**: Starter items complement but don't replace templates. Templates define the skeleton for new items (`markplane add`). Starter items are filled-in examples showing what good items look like. When customizable templates are designed, starter items should align with the default templates.

## References

- [[TASK-4ed4i]] — Migration framework (version upgrades, separate concern)
