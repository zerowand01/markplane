---
id: BACK-006
title: Document create-then-edit workflow in MCP instructions
status: backlog
priority: medium
type: enhancement
effort: small
tags:
- mcp
- documentation
epic: EPIC-001
plan: null
depends_on:
- BACK-001
- BACK-005
blocks: []
assignee: null
created: 2026-02-10
updated: 2026-02-10
---

# Document create-then-edit workflow in MCP instructions

## Description

The MCP `instructions` field (BACK-001) needs to clearly explain the two-step workflow for creating items: first create the item (which scaffolds a template with placeholder content), then fill in the markdown body (via `markplane_write` from BACK-005 or direct file editing). Without this guidance, an LLM might create items and leave them with placeholder text, or not understand why items have empty descriptions.

## Acceptance Criteria

- [ ] Instructions text includes a section explaining the create-then-edit workflow
- [ ] Instructions mention that `markplane_add` creates a template with placeholder content
- [ ] Instructions explain how to use `markplane_write` to fill in body content
- [ ] Instructions note the draft → backlog status transition as the "item is fully defined" gate

## Notes

This is a content update to the instructions string built in BACK-001, not a code change. Blocked by BACK-001 (instructions field must exist) and BACK-005 (write tool must exist to be documented).
