---
id: TASK-divg5
title: Build migration tooling for importing from existing docs
status: cancelled
priority: low
type: feature
effort: large
tags:
- cli
- migration
epic: EPIC-z8tdz
plan: null
depends_on: []
blocks: []
assignee: null
position: a4
created: 2026-02-10
updated: 2026-02-23
---

# Build migration tooling for importing from existing docs

## Description

Teams adopting Markplane likely have existing project documentation — GitHub issues, Jira exports, plain markdown files, TODO comments in code, or spreadsheets. Without migration tooling, they must manually recreate every item. A `markplane import` command would convert common formats into Markplane items, lowering the barrier to adoption.

This is an ecosystem feature that becomes important once Markplane is mature enough for real-world adoption. Priority is low because early adopters will start fresh, but it's essential for broader adoption.

## Acceptance Criteria

- [ ] `markplane import github --repo owner/repo` imports open GitHub issues as tasks
- [ ] `markplane import markdown <path>` imports a directory of markdown files as notes or tasks
- [ ] `markplane import csv <file>` imports from a CSV with columns mapped to frontmatter fields
- [ ] Import preserves original IDs as references in the body (e.g., "Imported from GitHub #42")
- [ ] Import assigns new Markplane IDs (does not reuse external IDs)
- [ ] Dry-run mode (`--dry-run`) shows what would be imported without writing files
- [ ] Import handles duplicates gracefully (skip or warn if title already exists)
- [ ] Labels/tags from source systems map to Markplane tags

## Notes

Start with the simplest format (CSV or markdown directory) and add GitHub/Jira later. GitHub import could use the `gh` CLI or GitHub API via `reqwest`. Jira export is typically CSV or JSON. Consider making importers pluggable so community contributors can add new sources. The import should create items in `draft` status so users can review before promoting to `backlog`.

## Cancelled

Superseded by [[TASK-f65s9]] (Seed markplane init with starter content and onboarding plan). That task takes an AI-native approach to migration: instead of building and maintaining rigid importers for each source format, it seeds `markplane init` with an onboarding plan containing migration guidance that any AI assistant can read and act on. This delivers ~80% of the value with near-zero maintenance, making dedicated import tooling unnecessary.
