---
id: EPIC-z8tdz
title: "Platform and Ecosystem"
status: later
priority: low
started: null
target: null
tags: []
depends_on: []
related: []
created: 2026-02-10
updated: 2026-02-26
---

# Platform and Ecosystem

## Objective

Expand Markplane's reach beyond the current CLI + MCP + web UI stack. The core product is functional — this epic is about adoption and extensibility: tooling to import existing work items, real-time file watching for live MCP updates, SSE transport for remote/browser MCP clients, auto-sync for derived files, and a plugin system for community-built integrations. These features take Markplane from "works for the author" to "works for teams."

## Key Results

- [ ] Teams can import existing work items from GitHub Issues, CSV, or markdown directories
- [ ] MCP server detects file changes and pushes real-time notifications to clients
- [ ] MCP server supports SSE transport for browser-based and remote clients
- [ ] Derived files (INDEX.md, .context/) auto-regenerate after mutations in server mode
- [ ] Plugin system enables community-built integrations without modifying core

## Notes

This is the most ambitious and lowest-priority epic. All items here are future work for after the core system ([[EPIC-a5vs9]], [[EPIC-c5uem]], [[EPIC-bb6pe]]) is mature. The plugin system ([[TASK-2u963]]) is the largest single task on the roadmap. Import tooling ([[TASK-divg5]]) becomes important for adoption once Markplane is stable enough for real-world use. File watching ([[TASK-v5b6b]]) and auto-sync ([[TASK-ksw5c]]) are incremental server improvements. SSE transport ([[TASK-7jei5]]) enables new deployment models but isn't blocking current workflows.
