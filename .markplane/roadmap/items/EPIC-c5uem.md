---
id: EPIC-c5uem
title: "Core Architecture"
status: later
priority: medium
started: null
target: null
tags: []
depends_on: []
---

# Core Architecture

## Objective

Harden the foundation and evolve the core data model. This epic has two themes: **reliability** — making concurrent CLI/MCP usage safe and providing a migration path for data format changes — and **flexibility** — letting teams customize workflows and optionally adopt time-boxed planning. Both are prerequisites for Markplane being trustworthy in multi-user, long-lived project scenarios.

## Key Results

- [x] Web API uses core update and link methods (no inline read-modify-write)
- [ ] Atomic file writes and advisory locking prevent data loss from concurrent CLI + MCP access
- [ ] `markplane migrate` provides a reliable upgrade path for data format changes between versions
- [ ] Statuses, priorities, and effort sizes are defined in `config.yaml` and validated at runtime
- [ ] Default config matches current hardcoded values (zero migration burden)
- [ ] Sprint entity type available as an optional time-boxed container for tasks

## Notes

The reliability work ([[TASK-2tags]], [[TASK-4ed4i]]) should come before the flexibility work ([[TASK-us45u]], [[TASK-b54gy]]). Concurrency safety and migration framework are preconditions — configurable workflows will be the first migration that exercises the framework. Sprints remain optional; many teams prefer continuous flow. The larger tasks here deserve implementation plans before coding begins.
