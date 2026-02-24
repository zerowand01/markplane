---
id: PLAN-f79x3
title: Web UI implementation plan
status: done
implements:
- TASK-ur5hw
epic: EPIC-z8tdz
created: 2026-02-12
updated: 2026-02-13
---
# Web UI Implementation Plan

4-phase plan for building the Markplane web dashboard. Each phase produces a usable increment. Phases are sequential — each builds on the previous.

**Design docs**: `docs/web-ui/research-brief.md`, `docs/web-ui/architecture.md`, `docs/web-ui/visual-design.md`

---

## Phase 1: Foundation

**Goal**: Scaffold the project, get a basic dashboard rendering data from the Rust API.

### Deliverables

1. **Next.js project scaffold**
   - Initialize in `crates/markplane-web/ui/`
   - Configure: Tailwind v4, shadcn/ui, `output: 'export'`, Geist fonts
   - Install: `@tanstack/react-query`, `next-themes`, `lucide-react`
   - Set up globals.css with OKLCH theme variables (dark + light)

2. **Rust HTTP server**
   - Add `axum` + `tokio` to markplane-cli dependencies
   - `markplane serve` command with `--port` and `--open` flags
   - Serve static files from embedded assets (or proxy in dev mode)
   - Core API endpoints:
     - `GET /api/summary` — project summary (dashboard data)
     - `GET /api/tasks` — list tasks with filter query params
     - `GET /api/tasks/:id` — task detail
     - `GET /api/epics` — list epics with progress
     - `GET /api/epics/:id` — epic detail with linked tasks
     - `POST /api/sync` — trigger markplane sync

3. **App shell**
   - Root layout with `QueryClientProvider`, `ThemeProvider`, Geist fonts
   - Sidebar navigation (shadcn Sidebar component)
   - Theme toggle (dark/light/system)
   - Basic routing: `/dashboard`, `/backlog`, `/epics`

4. **Dashboard page**
   - Summary metric cards (open tasks, active, blocked, sprint %)
   - Active work section (in-progress tasks)
   - Blocked items section
   - Epic progress bars
   - AI Context panel (`.context/summary.md` content + stale indicator + Sync Now)

### Exit Criteria
- `markplane serve` starts and renders the dashboard with real project data
- Dark/light theme switching works
- Sidebar navigation between dashboard, backlog, and epics pages

---

## Phase 2: Core Views

**Goal**: Build the primary work management views — kanban, task detail, epic detail.

### Deliverables

1. **Backlog kanban board** (`/backlog`)
   - Kanban columns by status (In Progress, Planned, Backlog, Draft)
   - TaskCard components with priority indicator, epic chip, tags, assignee
   - Drag-and-drop via `@dnd-kit` — drop updates status via `PATCH /api/tasks/:id/status`
   - Optimistic mutations with TanStack Query `useMutation`
   - WIP limits per column with visual indicators
   - Filter bar (status, priority, epic, tags, assignee)
   - View toggle: Kanban / List / Table

2. **Task detail page** (`/backlog/[id]`)
   - MarkdownRenderer with WikiLinkChip for `[[ID]]` references
   - Metadata sidebar: status, priority, effort, type, epic, plan, tags, assignee, dependencies, dates
   - Inline editing: click status/priority badges to change via dropdown
   - API endpoints: `PATCH /api/tasks/:id` for field updates

3. **List/table view** (`/backlog?view=list`)
   - shadcn DataTable with TanStack Table
   - Sortable columns: title, status, priority, effort, epic, assignee, updated
   - Row click navigates to task detail

4. **Epic views**
   - Epic overview (`/epics`): all epics with progress bars and status breakdown
   - Epic detail (`/epics/[id]`): markdown body + linked tasks table with status counts
   - API: epic `status_breakdown` field in response

5. **Additional API endpoints**
   - `POST /api/tasks` — create task
   - `PATCH /api/tasks/:id` — update task fields
   - `DELETE /api/tasks/:id` — archive task
   - `GET /api/plans`, `GET /api/plans/:id` — plans
   - `GET /api/notes`, `GET /api/notes/:id` — notes

### Exit Criteria
- Can browse, filter, and drag tasks between kanban columns
- Task detail renders markdown with clickable WikiLinkChips
- Can change status/priority from the UI
- Epic progress is visible with per-status breakdown

---

## Phase 3: Advanced Features

**Goal**: Add real-time updates, dependency graph, search, command palette, and roadmap timeline.

### Deliverables

1. **WebSocket real-time updates**
   - Rust: `notify` crate watches `.markplane/` for file changes
   - Rust: WebSocket endpoint at `/ws` broadcasts structured events
   - Client: `useWebSocket()` hook at app root
   - TanStack Query targeted invalidation on file change events
   - Debouncing at 100ms for rapid changes (e.g., during `markplane sync`)

2. **Dependency graph** (`/graph`)
   - `@xyflow/react` (React Flow) with Dagre hierarchical layout
   - Custom `ItemNode` component: ID, title, status color, entity-colored border
   - Animated edges: blocks (red), depends_on (blue)
   - MiniMap + Controls (zoom, fit, fullscreen)
   - Focus mode: `/graph?focus=TASK-rm6d3` centers on a specific item
   - API: `GET /api/graph/:id`, `GET /api/graph`

3. **Command palette** (Cmd+K)
   - shadcn Command component (built on cmdk)
   - Fuzzy search across all items
   - Grouped results: Tasks, Epics, Plans, Notes, Actions
   - Actions: navigate, change status, create task, trigger sync
   - Keyboard shortcut hints alongside each action
   - Recent items section

4. **Full-text search** (`/search`)
   - Search input with debounced query
   - Faceted filters: entity type, status, priority, tags
   - Results with highlighted matches
   - API: `GET /api/search?q=...`

5. **Roadmap timeline** (`/roadmap`)
   - SVAR React Gantt or equivalent timeline component
   - Swimlanes by epic
   - Zoomable time axis (week/month/quarter)
   - Dependency arrows between items
   - Today marker line

6. **Plans and Notes views**
   - Plans list (`/plans`) and detail (`/plans/[id]`)
   - Notes list (`/notes`) and detail (`/notes/[id]`)
   - Both with MarkdownRenderer and metadata sidebar

### Exit Criteria
- File changes from CLI/MCP/editor reflect in UI within 200ms
- Dependency graph renders and is interactive
- Cmd+K navigates anywhere and performs actions
- Search finds items across title and body

---

## Phase 4: Polish

**Goal**: Production-ready quality — loading states, error handling, mobile, performance, packaging.

### Deliverables

1. **Loading & error states**
   - Skeleton loading components for every page/section
   - Error boundaries with retry buttons
   - Empty states with paper airplane illustrations and contextual messages
   - Toast notifications for mutations (created, updated, error)

2. **Keyboard navigation**
   - Global: `g` then `d` (dashboard), `g` then `b` (backlog), etc.
   - Kanban: arrow keys to move between cards, `Enter` to open, `m` to move
   - List: `j`/`k` to navigate rows, `Enter` to open
   - Detail: `s` change status, `p` change priority, `e` edit
   - Escape to close modals/panels

3. **Mobile responsive layout**
   - Sidebar collapses to hamburger menu
   - Kanban: single-column swipe on mobile, 2-3 columns on tablet
   - Tables: card layout on small screens
   - Touch-friendly drag targets

4. **Performance optimization**
   - Code splitting: React Flow lazy-loaded on `/graph`
   - Shiki syntax highlighter loaded on-demand
   - Bundle size targets: <150KB initial, <300KB total (gzipped)
   - Task list pagination at 100 items

5. **Production packaging**
   - `rust-embed` integration to embed `out/` directory
   - `markplane serve` serves embedded static files + API
   - Build script: `npm run build` then `cargo build --release`
   - Verify single-binary distribution works

6. **Animations** (Framer Motion)
   - Drag-and-drop: spring physics on grab/release
   - Page transitions: subtle fade/slide
   - Status changes: color transition on badges
   - Loading spinner: paper airplane orbit animation

### Exit Criteria
- All pages have loading skeletons and error states
- Keyboard navigation works for common workflows
- Usable on tablet (1024px+) and mobile (375px+)
- Single `markplane` binary serves the full web UI
- Bundle size within targets
