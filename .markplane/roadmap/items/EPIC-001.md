---
id: EPIC-001
title: MCP Protocol Improvements
status: done
priority: high
started: null
target: null
tags: []
depends_on: []
---

# MCP Protocol Improvements

## Objective

Make the MCP server a complete, spec-compliant interface for AI tools to manage Markplane projects. Currently the server is functional but missing key protocol features — the `instructions` field that teaches LLMs how to use the tools, resource templates for plans and notes, the correct protocol version, and a tool for writing markdown body content. Without these, LLMs can create items but can't fully manage them.

## Key Results

- [x] LLMs receive project-specific guidance via the `instructions` field in the initialize response
- [x] All four entity types (BACK, EPIC, PLAN, NOTE) are accessible as MCP resources
- [x] LLMs can create items AND fill in their markdown content entirely through MCP tools
- [x] Protocol version reports the current spec (2025-11-25)

## Notes

This is the highest-priority epic because it directly impacts how well AI tools can work with Markplane. The `instructions` field (BACK-001) is the single most impactful improvement — it's the difference between an LLM guessing how to use tools and being explicitly taught. BACK-006 (documenting the create-then-edit workflow) is blocked by BACK-001 and BACK-005.
