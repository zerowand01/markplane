---
id: BACK-018
title: Implement plugin system for external integrations
status: backlog
priority: someday
type: feature
effort: xl
tags:
- plugins
- integrations
epic: EPIC-005
plan: null
depends_on: []
blocks: []
assignee: null
created: 2026-02-10
updated: 2026-02-10
---

# Implement plugin system for external integrations

## Description

As Markplane matures, users will want to integrate it with external systems — GitHub Issues sync, Slack notifications on status changes, CI/CD pipeline triggers, custom reports, etc. Rather than building every integration into core, a plugin system would allow community-driven extensions that hook into Markplane events and data.

This is the most ambitious feature on the roadmap and should only be tackled once the core system is stable and the most common integration patterns are understood.

## Acceptance Criteria

- [ ] Plugin discovery: Markplane finds plugins in `.markplane/plugins/` or a configured directory
- [ ] Plugin lifecycle: hooks for `on_create`, `on_update`, `on_status_change`, `on_sync`
- [ ] Plugins can read project data via the core library API
- [ ] Plugins can modify items (with validation) or create new items
- [ ] Plugin configuration via `config.yaml` plugins section
- [ ] At least one reference plugin (e.g., GitHub issue sync or Slack webhook)
- [ ] Plugin errors are isolated — a failing plugin doesn't crash Markplane
- [ ] Documentation for plugin authors

## Notes

The plugin mechanism could be one of several approaches: (1) shell-based hooks (simple scripts executed on events, like git hooks), (2) WASM plugins (sandboxed, portable, but complex), (3) Rust dynamic libraries (fast but platform-specific), or (4) a simple stdin/stdout protocol where plugins are executables that receive JSON events. Start with shell-based hooks (option 1) as the simplest approach — it's the git model and requires minimal infrastructure. WASM could be explored later for sandboxing and cross-platform portability.
