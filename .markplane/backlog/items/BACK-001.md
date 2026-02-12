---
id: BACK-001
title: Add instructions field to MCP initialize response
status: done
priority: high
type: feature
effort: small
tags:
- mcp
- ai-integration
epic: EPIC-001
plan: null
depends_on: []
blocks:
- BACK-006
assignee: null
created: 2026-02-10
updated: 2026-02-11
---

# Add instructions field to MCP initialize response

## Description

The MCP spec (2025-11-25) defines an `instructions` field in the initialize response that provides free-form natural language guidance to the LLM about how to use the server. Currently our `handle_initialize()` returns only `protocolVersion`, `capabilities`, and `serverInfo` — no instructions. Without this, the LLM only learns about Markplane from the 15 short tool description strings, with no higher-level context about entity types, workflows, or recommended usage patterns.

The instructions should be built dynamically from the project's `config.yaml` so they include the project name and are future-proof for configurable workflows.

## Acceptance Criteria

- [ ] `handle_initialize()` returns an `instructions` field in the response
- [ ] Instructions describe: what Markplane is, the 4 entity types (BACK/EPIC/PLAN/NOTE), status workflows, recommended tool call sequence, cross-reference syntax
- [ ] Instructions include the project name from config.yaml
- [ ] Instructions are built dynamically (not a static string) so they can reflect config in the future
- [ ] Existing MCP integration tests still pass
- [ ] New test validates instructions field is present and non-empty

## Notes

The instructions field is a universal solution — it works for Claude, Cursor, Windsurf, and any MCP client, unlike client-specific approaches like CLAUDE.md. The `claude-md` command remains valuable for project-specific tips but the generic guidance belongs in `instructions`.

## References

- MCP spec lifecycle: https://modelcontextprotocol.io/specification/2025-11-25/basic/lifecycle
- MCP schema (TypeScript source of truth): https://github.com/modelcontextprotocol/specification/blob/main/schema/2025-11-25/schema.ts
