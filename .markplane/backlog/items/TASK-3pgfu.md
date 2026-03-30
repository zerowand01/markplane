---
id: TASK-3pgfu
title: OpenClaw plugin for context injection (markplane-memory)
status: done
priority: high
type: feature
effort: medium
epic: EPIC-z8tdz
plan: null
depends_on: []
blocks: []
related:
- NOTE-83hku
assignee: null
tags:
- openclaw
- integration
position: a5
created: 2026-03-27
updated: 2026-03-29
---

# OpenClaw plugin for context injection (markplane-memory)

## Description

OpenClaw's hardcoded bootstrap file list does not support custom context files. The only reliable way to inject Markplane's compressed project state into the agent's system prompt on every turn is via a plugin's `before_prompt_build` hook. Without this, agents must spend tool calls reading context files, and any `AGENTS.md` instruction to do so can be lost during compaction.

The plugin (`@zerowand/markplane-memory`) is a thin npm package in a separate repo ([zerowand01/markplane-memory](https://github.com/zerowand01/markplane-memory)) that reads `.markplane/.context/` files and returns them via `appendSystemContext`. It also bundles a SKILL.md that teaches the agent Markplane usage patterns.

## Acceptance Criteria

- [x] Plugin installs via `openclaw plugins install @zerowand/markplane-memory`
- [x] `.context/summary.md` appears in system prompt on every turn (confirmed by asking the agent)
- [x] Configurable: users can choose which context files to inject
- [x] Bundled SKILL.md appears in the agent's skill list
- [x] README documents MCP server registration and optional compaction flush config
- [x] Graceful no-op when `.markplane/` is not initialized (no errors)

## Notes

- See [[NOTE-83hku]] for the architecture decision on why this is a separate repo
- Plugin is ~60 lines of runtime code — intentionally minimal
- MCP server registration is a manual config edit; compaction flush prompt is optional
- Published as `@zerowand/markplane-memory` on npm (v0.1.1)
- `peerDependencies` verified against OpenClaw SDK 2026.3.24

## References

- Repository: https://github.com/zerowand01/markplane-memory
- OpenClaw plugin SDK: `before_prompt_build` hook with `appendSystemContext`
- OpenClaw Issue #9491: feature request for custom injected workspace files
