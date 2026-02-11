---
id: BACK-017
title: Build web UI with React and Tailwind
status: backlog
priority: someday
type: feature
effort: xl
tags:
- web-ui
epic: EPIC-005
plan: null
depends_on: []
blocks: []
assignee: null
created: 2026-02-10
updated: 2026-02-10
---

# Build web UI with React and Tailwind

## Description

Markplane is currently CLI and MCP only. While the markdown-first approach works well for developers and AI tools, a visual dashboard would help non-technical stakeholders, provide at-a-glance project overviews, and make Markplane accessible to a wider audience. The design spec mentions an optional React + Tailwind web UI as a future capability.

This is a large, separate frontend project that communicates with Markplane data either by reading `.markplane/` files directly (for local use) or via the MCP SSE transport (BACK-016) for a richer integration.

## Acceptance Criteria

- [ ] React + Tailwind project scaffolded in `web/` directory
- [ ] Dashboard view showing project summary, active work, and blocked items
- [ ] Backlog list view with filtering by status, priority, tags, and epic
- [ ] Item detail view rendering markdown content with frontmatter metadata
- [ ] Epic view showing epic progress with linked backlog items
- [ ] Status updates possible from the UI (calls markplane commands or MCP)
- [ ] Responsive design (desktop and tablet)
- [ ] Development server with hot reload
- [ ] Production build outputs static files that can be served locally

## Notes

Start read-only (just visualizing data) and add write capabilities later. The simplest initial approach is reading `.markplane/` files directly via a local dev server, without requiring MCP SSE. Consider whether the web UI should be a separate npm package or bundled with the Rust binary (e.g., embedded static files served by an axum endpoint). For the latter approach, `rust-embed` or `include_dir` could embed the built frontend assets.
