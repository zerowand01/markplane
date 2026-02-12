---
id: EPIC-005
title: "Platform and Ecosystem"
status: planned
priority: low
started: null
target: null
tags: []
depends_on: []
---

# Platform and Ecosystem

## Objective

Expand Markplane beyond CLI and MCP into a broader platform — migration tooling for adoption, real-time file watching, SSE transport for web clients, a visual web UI, and a plugin system for external integrations. These features transform Markplane from a developer tool into a platform that serves teams with diverse needs and workflows.

## Key Results

- [ ] Teams can import existing work items from GitHub Issues, CSV, or markdown directories
- [ ] MCP server supports SSE transport for browser-based and remote clients
- [ ] A web dashboard provides visual project overview and item management
- [ ] Plugin system enables community-built integrations without modifying core

## Notes

This is the most ambitious and lowest-priority epic — all items here are future work that should only be tackled once the core system (EPIC-001 through EPIC-004) is mature and stable. The web UI (TASK-017) and plugin system (TASK-018) are particularly large efforts. SSE transport (TASK-016) is a prerequisite for the web UI. File watching (TASK-015) improves the MCP experience but isn't blocking anything.
