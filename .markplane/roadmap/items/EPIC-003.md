---
id: EPIC-003
title: "Core Architecture"
status: planned
priority: medium
started: null
target: null
tags: []
depends_on: []
---

# Core Architecture

## Objective

Evolve the core data model to support team customization and time-boxed work planning. Currently statuses, priorities, and effort sizes are hardcoded Rust enums, and there's no concept of sprints or iterations. Configurable workflows let teams tailor Markplane to their process; sprints add a time-boxed planning layer for teams that want it.

## Key Results

- [ ] Statuses, priorities, and effort sizes are defined in `config.yaml` and validated at runtime
- [ ] Default config matches current hardcoded values (zero migration burden)
- [ ] Sprint entity type available as an optional time-boxed container for backlog items

## Notes

These are the two largest architectural changes on the roadmap. Configurable workflows (BACK-009) touches models, serialization, validation, CLI, and MCP — it's a deep refactor. Sprints (BACK-010) adds a new entity type. Both should be designed carefully and likely deserve implementation plans before coding begins. Sprints should remain optional — many teams prefer continuous flow.
