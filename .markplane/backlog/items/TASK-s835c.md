---
id: TASK-s835c
title: Let markplane init write the editor MCP config
status: backlog
priority: medium
type: feature
effort: medium
epic: null
plan: null
depends_on: []
blocks: []
related: []
assignee: null
tags:
- onboarding
- mcp
- growth
position: aC
created: 2026-10-24
updated: 2026-10-24
---

# Let markplane init write the editor MCP config

## Context

Onboarding today is four separate steps: install, `markplane init`, `markplane serve`, then a hand-edited MCP config per editor (see `docs/mcp-setup.md`). The MCP step is where the "AI-native" value lives, and it is the step most likely to be skipped. Growth analysis (Oct 2026): Markplane has ~190 stars but near-zero inbound traffic; the closest comparable projects (Beads, Backlog.md) lead with a one-command setup.

## Exploration

`init::run(name, description, empty)` in `crates/markplane-cli/src/commands/init.rs` writes only `.markplane/`. There is no awareness of the host editor.

Editor config locations differ:

| Editor | Project-scoped file | Shape |
|---|---|---|
| Claude Code | `.mcp.json` | `{"mcpServers": {"markplane": {"command": "markplane", "args": ["mcp"]}}}` |
| Cursor | `.cursor/mcp.json` | same `mcpServers` shape |
| VS Code (Copilot) | `.vscode/mcp.json` | `{"servers": {"markplane": {"type": "stdio", "command": "markplane", "args": ["mcp"]}}}` |
| Windsurf | user-level only (`~/.codeium/windsurf/mcp_config.json`) | `mcpServers` shape |
| Zed | `.zed/settings.json` → `context_servers` | different shape |

Viable approach: a `--mcp <editor>` flag (repeatable, or `all`) on `init`, plus a standalone `markplane mcp install <editor>` for existing projects. Merge into an existing file rather than overwrite; never touch user-level config without an explicit flag. When no flag is given, detect `.cursor/`, `.vscode/`, `.claude/` and print the one-liner for whichever is present, so the hint is always visible even in non-interactive runs.

## Acceptance Criteria

- [ ] `markplane init --mcp claude` writes or merges `.mcp.json`; `cursor` and `vscode` write their equivalents; `all` writes all three
- [ ] `markplane mcp install <editor>` does the same for an already-initialized project
- [ ] Existing files are merged (other servers preserved), never clobbered; `--force` overwrites
- [ ] Plain `init` ends with a one-line "connect your AI" hint tailored to detected editors
- [ ] README quickstart collapses to: install, `markplane init --mcp claude`, done
- [ ] Integration tests cover merge, overwrite refusal, and each editor shape

## Notes

Related to the growth plan: MCP registry listing, Claude Code plugin packaging, and the one-command install story.
