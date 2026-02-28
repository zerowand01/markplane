---
id: TASK-7f8ev
title: Web UI visual polish
status: done
priority: medium
type: enhancement
effort: xs
epic: EPIC-4bjs9
plan: null
depends_on: []
blocks: []
related: []
assignee: null
tags:
- web-ui
position: Zzd
created: 2026-02-23
updated: 2026-02-25
---

# Web UI visual polish

## Description

Three visual improvements bundled together.

**Detail sheet max width:** The resizable detail sheet is capped at `MAX_WIDTH = 960px` in `resizable-sheet-content.tsx`. On wide monitors this feels cramped. Change to a hybrid approach: compute max width as `Math.min(1400, window.innerWidth * 0.8)` so the drag ceiling scales with the monitor (e.g. ~1536px on a 1920px display, ~1152px on 1440px). The inline style already uses `min(${width}px, 75vw)` for rendering, so MAX_WIDTH just needs to be a reasonable upper bound for the drag handle — a pure vw constant won't work cleanly since the resize logic needs a pixel value to clamp against during pointer events.

**Inline code font:** Both `markdown-renderer.tsx` and `markdown-editor.tsx` style inline code with `prose-code:bg-muted prose-code:rounded prose-code:text-sm` but don't explicitly set the font. Add `prose-code:font-mono` to both so Geist Mono (already loaded and wired as `--font-mono` via Tailwind v4 theme in `globals.css`) is used instead of relying on the browser's default monospace stack. Also add a subtle border (`ring-1 ring-border/50` or `border border-border/50`).

**Hide decorative backticks:** The Tailwind Typography plugin adds `::before` and `::after` pseudo-elements with backtick characters on `<code>` elements. These are currently visible in both the markdown renderer and TipTap editor rich-text view. Add `prose-code:before:content-[''] prose-code:after:content-['']` to both files to strip them.

## Acceptance Criteria

- [ ] Detail sheet max width uses hybrid approach: `Math.min(1400, window.innerWidth * 0.8)`
- [ ] Inline code uses `font-mono` (Geist Mono) and has a subtle border
- [ ] Decorative backticks hidden via `prose-code:before:content-[''] prose-code:after:content-['']`
- [ ] Inline code styling matches between markdown renderer and TipTap editor
- [ ] Both light and dark themes render inline code attractively

## References
