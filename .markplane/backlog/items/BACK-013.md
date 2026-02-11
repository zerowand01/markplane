---
id: BACK-013
title: Add clipboard context output
status: backlog
priority: low
type: feature
effort: xs
tags:
- cli
- context
epic: EPIC-004
plan: null
depends_on: []
blocks: []
assignee: null
created: 2026-02-10
updated: 2026-02-10
---

# Add clipboard context output

## Description

When using AI tools that don't support MCP (e.g., ChatGPT web, or pasting context into any chat window), users need to manually copy project context. A `--clipboard` flag on `markplane context` would pipe the generated context directly to the system clipboard, saving the copy step.

This is a tiny quality-of-life feature — just pipe stdout to `pbcopy` (macOS) or equivalent.

## Acceptance Criteria

- [ ] `markplane context --clipboard` copies the context output to the system clipboard
- [ ] Works on macOS (`pbcopy`), Linux (`xclip` or `xsel`), and Windows (`clip.exe`)
- [ ] Prints a confirmation message (e.g., "Context copied to clipboard (1,234 chars)")
- [ ] Can be combined with other context flags (e.g., `--item BACK-042 --clipboard`)
- [ ] Falls back gracefully if clipboard tool is not available

## Notes

Use `std::process::Command` to invoke the platform clipboard utility. Detection order: `pbcopy` (macOS), `xclip -selection clipboard` (Linux), `xsel --clipboard` (Linux fallback), `clip.exe` (Windows/WSL). This is a CLI-only feature — MCP clients handle their own clipboard.
