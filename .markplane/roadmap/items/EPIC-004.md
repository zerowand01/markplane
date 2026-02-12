---
id: EPIC-004
title: "Context and AI Integration"
status: planned
priority: medium
started: null
target: null
tags: []
depends_on: []
---

# Context and AI Integration

## Objective

Make the `.context/` layer smarter and more useful for AI-assisted development workflows. Currently context generation is project-wide only — there's no way to get a focused context bundle for a single item or filter by domain. Adding per-item context bundles, domain-filtered generation, and clipboard output gives AI tools exactly the context they need, when they need it.

## Key Results

- [ ] `markplane context --item TASK-042` produces a focused bundle with the item, its epic, dependencies, and related items
- [ ] `markplane context --tag mcp` produces domain-filtered context relevant to a specific area
- [ ] `markplane context --clipboard` copies context to the system clipboard for pasting into non-MCP AI tools

## Notes

The existing `extract_references()` function and `QueryFilter` infrastructure provide the building blocks. Per-item context (TASK-011) is the most impactful — it's what an AI needs when starting work on a specific task. Domain filtering (TASK-012) builds on QueryFilter. Clipboard output (TASK-013) is trivial but enables workflows with AI tools that don't support MCP.
