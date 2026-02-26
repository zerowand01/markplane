---
id: EPIC-6zdf4
title: CLI & Developer Experience
status: later
priority: medium
started: null
target: null
tags: []
depends_on: []
created: 2026-02-10
updated: 2026-02-26
---

# CLI & Developer Experience

## Objective

Polish the day-to-day experience of using Markplane across CLI and MCP interfaces. The tool works, but has friction points — non-deterministic INDEX.md output, no `edit` command, AI agents can't reorder tasks without computing fractional indices, and a deprecated YAML dependency. This epic covers the quality-of-life improvements that make Markplane feel complete and professional.

## Key Results

- [x] Update and link commands support all property fields and cross-type entity linking
- [ ] AI agents can reorder tasks via a high-level `markplane_move` MCP tool without manual position math
- [ ] Generated INDEX.md sections sort deterministically (by date, priority, ID)
- [ ] Users can open any item in their editor with `markplane edit TASK-xxx`
- [ ] `serde_yaml` replaced with `serde_yaml_ng` (maintained fork, identical API)

## Notes

This epic spans both user-facing polish (edit command, deterministic output) and AI-agent ergonomics (move tool). The common thread is reducing friction for anyone interacting with Markplane — whether human or AI. Tasks range from xs to small effort, making this a good epic for incremental progress between larger efforts.
