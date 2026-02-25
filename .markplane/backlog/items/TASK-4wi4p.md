---
id: TASK-4wi4p
title: Add documentation viewer to web UI
status: backlog
priority: medium
type: feature
effort: large
tags:
- web-ui
- docs
epic: EPIC-4bjs9
plan: null
depends_on: []
blocks: []
assignee: null
position: a7
created: 2026-02-24
updated: 2026-02-24
---

# Add documentation viewer to web UI

## Description

Add a `/docs` page to the web UI that renders a curated set of user-facing markdown documentation in a searchable, polished viewer. Markplane's docs already exist as markdown files in the repo — this feature surfaces them inside the web UI so users can reference guides, CLI commands, and AI workflows without leaving the dashboard.

Docs stay at their existing paths (README.md, docs/*.md). The viewer reads them from disk via a new API endpoint — no copying, no special directory.

### Included docs (6)

| Doc | Path | Purpose |
|-----|------|---------|
| README | `README.md` | Product overview and installation |
| Getting Started | `docs/getting-started.md` | Onboarding tutorial |
| CLI Reference | `docs/cli-reference.md` | Command lookup reference |
| MCP Setup | `docs/mcp-setup.md` | AI tool configuration |
| AI Integration | `docs/ai-integration.md` | Context layer, token budgets, AI workflows |
| Web UI Guide | `docs/web-ui-guide.md` | Web dashboard usage and keyboard shortcuts |

### Excluded docs (not user-facing)

- `docs/architecture.md` — contributor/internal
- `docs/file-format.md` — internals reference (YAML schemas, ID system)
- `docs/web-ui/architecture.md`, `docs/web-ui/visual-design.md` — contributor docs
- `docs/archive/*` — archived design specs

## Acceptance Criteria

- [ ] `/docs` page accessible from the sidebar navigation
- [ ] Sidebar doc list showing all 6 docs with titles
- [ ] Rendered markdown content area with syntax highlighting for code blocks
- [ ] Full-text search across all docs with highlighted matches
- [ ] Deep-linkable URLs (e.g. `/docs?page=cli-reference`)
- [ ] Keyboard shortcut `g` then `?` navigates to docs
- [ ] Real-time updates when doc files change on disk (via existing WebSocket)
- [ ] Responsive layout (sidebar collapses on mobile)

## Notes

- Docs live at their existing repo paths — no special directory, no copies
- The curated list could live in config.yaml (e.g. `web_docs` key) or be hardcoded initially
- TipTap is already in the project for markdown editing; a read-only markdown renderer may be simpler (e.g. react-markdown or reuse the existing rendering pipeline)
- Search could be client-side fuzzy search given the small corpus (~6 docs)
- The existing `documentation_paths` config and `list_documentation_files()` in core could be extended or a new endpoint added alongside it

## References

- Existing infra: `documentation_paths` in config.yaml, `list_documentation_files()` in `project.rs`
- Web UI patterns: Sheet slide-overs, TanStack Query hooks, chord keyboard nav
