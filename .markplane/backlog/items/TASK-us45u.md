---
id: TASK-us45u
title: Implement configurable workflows for statuses, priorities, and effort sizes
status: backlog
priority: medium
type: feature
effort: large
tags:
- core
- config
epic: EPIC-c5uem
plan: null
depends_on: []
blocks: []
assignee: null
position: a4
created: 2026-02-10
updated: 2026-02-10
---

# Implement configurable workflows for statuses, priorities, and effort sizes

## Description

The design spec envisions statuses, priorities, and effort sizes as configurable values defined in `config.yaml`, but the current implementation hardcodes them as Rust enums (`Status`, `EpicStatus`, `PlanStatus`, `Priority`, `Effort`). This means users cannot customize their workflow — for example, adding a `review` status, removing `someday` priority, or using t-shirt sizes different from `xs/small/medium/large/xl`. Moving to config-driven workflows would allow teams to tailor Markplane to their process.

This is a large architectural change that touches models, serialization, validation, CLI display, and MCP tool schemas. The hardcoded enums would be replaced with string-based values validated against the config at runtime.

## Acceptance Criteria

- [ ] `config.yaml` supports `workflows` section defining valid statuses per entity type
- [ ] `config.yaml` supports `priorities` list defining valid priority values
- [ ] `config.yaml` supports `effort_sizes` list defining valid effort values
- [ ] Default config provides the current hardcoded values (backward compatible)
- [ ] Validation rejects unknown status/priority/effort values on write
- [ ] CLI `--status`, `--priority` tab completion reads from config
- [ ] MCP tool schemas dynamically reflect configured values
- [ ] `markplane init` scaffolds the default workflow config
- [ ] All existing tests pass with default config values
- [ ] Migration path: existing projects without workflow config use built-in defaults

## Notes

Consider a phased approach: first add the config schema and parsing with defaults matching current enums, then swap enum-based validation to config-based validation. The hardest part is replacing strongly-typed enums with validated strings throughout the codebase while maintaining type safety. A `ValidatedStatus(String)` newtype pattern could help. The MCP `instructions` field (TASK-eduur) should dynamically list available statuses once this lands.

## References

- Design spec workflow section: `docs/ai-native-pm-system-design.md`
