---
id: TASK-pj4ga
title: Document create-then-edit workflow in MCP instructions
status: done
priority: medium
type: enhancement
effort: small
tags:
- mcp
- documentation
epic: EPIC-ji4z3
plan: null
depends_on:
- TASK-eduur
- TASK-7cucf
blocks: []
assignee: null
position: a2
created: 2026-02-10
updated: 2026-02-11
---

# Document create-then-edit workflow in MCP instructions

## Description

The MCP `instructions` field (TASK-eduur) needs to clearly explain the two-step workflow for creating items: first create the item (which scaffolds a template with placeholder content), then fill in the markdown body (via `markplane_write` from TASK-7cucf or direct file editing). Without this guidance, an LLM might create items and leave them with placeholder text, or not understand why items have empty descriptions.

## Acceptance Criteria

- [ ] Instructions text includes a section explaining the create-then-edit workflow
- [ ] Instructions mention that `markplane_add` creates a template with placeholder content
- [ ] Instructions explain how to use `markplane_write` to fill in body content
- [ ] Instructions note the draft → backlog status transition as the "item is fully defined" gate

## Notes

This is a content update to the instructions string built in TASK-eduur, not a code change. Blocked by TASK-eduur (instructions field must exist) and TASK-7cucf (write tool must exist to be documented).
