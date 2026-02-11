---
id: BACK-014
title: Build migration tooling for importing from existing docs
status: backlog
priority: low
type: feature
effort: large
tags:
- cli
- migration
epic: EPIC-005
plan: null
depends_on: []
blocks: []
assignee: null
created: 2026-02-10
updated: 2026-02-10
---

# Build migration tooling for importing from existing docs

## Description

Teams adopting Markplane likely have existing project documentation — GitHub issues, Jira exports, plain markdown files, TODO comments in code, or spreadsheets. Without migration tooling, they must manually recreate every item. A `markplane import` command would convert common formats into Markplane items, lowering the barrier to adoption.

This is an ecosystem feature that becomes important once Markplane is mature enough for real-world adoption. Priority is low because early adopters will start fresh, but it's essential for broader adoption.

## Acceptance Criteria

- [ ] `markplane import github --repo owner/repo` imports open GitHub issues as backlog items
- [ ] `markplane import markdown <path>` imports a directory of markdown files as notes or backlog items
- [ ] `markplane import csv <file>` imports from a CSV with columns mapped to frontmatter fields
- [ ] Import preserves original IDs as references in the body (e.g., "Imported from GitHub #42")
- [ ] Import assigns new Markplane IDs (does not reuse external IDs)
- [ ] Dry-run mode (`--dry-run`) shows what would be imported without writing files
- [ ] Import handles duplicates gracefully (skip or warn if title already exists)
- [ ] Labels/tags from source systems map to Markplane tags

## Notes

Start with the simplest format (CSV or markdown directory) and add GitHub/Jira later. GitHub import could use the `gh` CLI or GitHub API via `reqwest`. Jira export is typically CSV or JSON. Consider making importers pluggable so community contributors can add new sources. The import should create items in `draft` status so users can review before promoting to `backlog`.
