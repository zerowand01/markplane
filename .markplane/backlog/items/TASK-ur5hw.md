---
id: TASK-ur5hw
title: Build web UI with React and Tailwind
status: done
priority: someday
type: feature
effort: xl
tags:
- web-ui
epic: EPIC-z8tdz
plan: PLAN-f79x3
depends_on: []
blocks: []
assignee: null
position: a0
created: 2026-02-10
updated: 2026-02-13
---
# Build web UI with React and Tailwind

## Description

Build a local-first web dashboard for Markplane, served via `markplane serve` on `localhost:4200`. The UI is a Next.js static export embedded in the Rust binary via `rust-embed`, requiring zero Node.js runtime. The Rust HTTP server (axum) serves static files, a REST API backed by `markplane-core`, and a WebSocket endpoint for real-time file change notifications.

The complete design is documented across three design docs:
- `docs/web-ui/research-brief.md` — UI research: PM tool analysis, design trends, shadcn/ui ecosystem, color palettes
- `docs/web-ui/architecture.md` — Technical architecture: project structure, data layer, API design, component architecture, theme system, real-time updates, build/deployment
- `docs/web-ui/visual-design.md` — Visual system: color tokens, typography (Geist Sans/Mono), spacing, component specs, wireframes, animations, accessibility

### Key Technical Decisions
- **Stack**: Next.js 15 (App Router) + Tailwind v4 + shadcn/ui + TanStack Query v5
- **Fonts**: Geist Sans + Geist Mono (Vercel's font family)
- **Deployment**: Static export embedded in Rust binary via `rust-embed` — single binary, zero Node.js
- **Data**: TanStack Query for server state, REST API backed by `markplane-core`, WebSocket for real-time invalidation
- **Theme**: OKLCH color space, dark theme default, `next-themes` for switching
- **Graph**: `@xyflow/react` (React Flow) with Dagre layout for dependency DAG
- **Kanban**: `@dnd-kit` with optimistic mutations and WIP limits
- **Timeline**: SVAR React Gantt for roadmap view
- **Animations**: Framer Motion for drag-drop, transitions, status changes

## Acceptance Criteria

### Core Infrastructure
- [ ] Next.js project scaffolded in `crates/markplane-web/ui/` with shadcn/ui + Tailwind v4
- [ ] Rust HTTP server (axum) serving static files + REST API + WebSocket
- [ ] `markplane serve` command starts the local server
- [ ] Static export embedded in binary via `rust-embed` for production builds
- [ ] Dark/light theme with dark as default (OKLCH color system)

### Views & Pages
- [ ] Dashboard: summary cards, active work, blocked items, epic progress, AI context panel
- [ ] Backlog kanban board with drag-and-drop status changes and WIP limits
- [ ] Backlog list/table view alternatives with sorting and filtering
- [ ] Task detail: markdown body with WikiLinkChip rendering + metadata sidebar
- [ ] Epic overview with progress bars and status breakdown
- [ ] Epic detail with linked tasks table
- [ ] Roadmap timeline/swimlane view (SVAR React Gantt)
- [ ] Dependency graph (React Flow + Dagre)
- [ ] Command palette (Cmd+K) for navigation, actions, and search
- [ ] Full-text search with faceted filters
- [ ] Plans and Notes list/detail views

### Real-Time & Interactivity
- [ ] WebSocket file watching with TanStack Query targeted invalidation
- [ ] Optimistic mutations for status changes, drag-and-drop, inline edits
- [ ] Keyboard navigation (vim-style shortcuts)
- [ ] Responsive design (desktop and tablet)

### API Endpoints
- [ ] Tasks CRUD (`/api/tasks`, `/api/tasks/:id`, `/api/tasks/:id/status`)
- [ ] Epics CRUD (`/api/epics`, `/api/epics/:id`)
- [ ] Plans and Notes read (`/api/plans`, `/api/notes`)
- [ ] Summary (`/api/summary`), metrics, graph, search, blocked, sync

## Notes

Implementation follows a 4-phase plan — see linked PLAN item for details. The web UI is a new `markplane-web` crate (or directory) alongside the existing CLI and MCP crates. Development uses a dual-terminal workflow: Rust backend with `cargo watch` + Next.js dev server with API proxy.
