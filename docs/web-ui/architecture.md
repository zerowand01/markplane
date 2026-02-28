# Markplane Web UI Architecture

**Status**: Implemented
**Created**: 2026-02-12
**Updated**: 2026-02-21
**Related**: [[TASK-ur5hw]], [[PLAN-f79x3]]

---

## 1. Overview

The Markplane web UI is a local-first interface for browsing and managing `.markplane/` project data. It runs via `markplane serve` on `localhost:4200`, providing a modern dark-themed dashboard, kanban board, task detail views, epic progress tracking, roadmap timeline, dependency graph, and search.

### Design Principles

1. **Local-first**: Reads/writes `.markplane/` files via the Rust backend. No external services.
2. **Real-time**: File changes (from CLI, MCP, or manual edits) reflect in the UI immediately.
3. **Embeddable**: The production build can be embedded in the Rust binary via `rust-embed`, requiring zero Node.js runtime.
4. **Keyboard-first**: Chord navigation (`g`+letter), command palette (`Cmd+K`), keyboard shortcuts for all key actions.

### Architecture Summary

```
Browser (React + Next.js)
    │
    ├── Initial HTML ← Static export (embedded in Rust binary)
    │
    ├── REST API calls ──→ Rust HTTP server (localhost:4200/api/*)
    │                           │
    │                           ├── markplane-core (shared library)
    │                           │
    │                           └── .markplane/ (filesystem)
    │
    └── WebSocket ←──────→ Rust WS server (localhost:4200/ws)
                               │
                               └── notify crate (file watcher)
```

---

## 2. Technology Decisions

| Technology | Choice | Rationale |
|-----------|--------|-----------|
| Framework | Next.js 16 (App Router) | File-based routing, static export, industry standard |
| UI library | shadcn/ui + Radix UI | Composable, accessible, Tailwind-native, customizable via copy-paste |
| Styling | Tailwind CSS v4 | OKLCH color space, `@theme inline`, CSS-first config, dark mode via class strategy |
| State/data | TanStack Query v5 | Purpose-built `useMutation` lifecycle for optimistic updates, `staleTime`/`gcTime` for smart caching, targeted `invalidateQueries` for WebSocket integration |
| Fonts | Geist Sans + Geist Mono | Vercel's font family — native Next.js integration, slightly condensed for dashboard density, modern developer tool aesthetic |
| Animations | Framer Motion | Page transitions, status change animations |
| Markdown reading | `react-markdown` + `remark-gfm` | Runtime rendering of arbitrary `.md` content (not page-based MDX); supports GFM tables, checkboxes |
| Markdown editing | TipTap (`@tiptap/react`) | Rich text editor with markdown source view, task lists, wiki-link syntax support |
| Graph visualization | `@xyflow/react` (React Flow) + `elkjs` | React Flow for interactive canvas; ELK.js for hierarchical/orthogonal auto-layout (better than Dagre for complex dependency graphs) |
| Drag-and-drop | `@dnd-kit/core` + `@dnd-kit/sortable` | Lightweight, accessible drag-and-drop for kanban board and roadmap |
| Icons | `lucide-react` | Already bundled with shadcn/ui, consistent, tree-shakeable |
| Toasts | `sonner` | Lightweight toast notifications for mutation feedback |
| Command palette | `cmdk` (via shadcn Command) | Keyboard-first navigation and search |
| Ordering | `fractional-indexing` | Stable ordering for kanban card drag-and-drop position |
| Build output | Static export (`output: 'export'`) | Embeddable in Rust binary; no Node.js server required at runtime |

### Why Not MDX?

MDX compiles markdown to React components at build time, requiring files to be known ahead of time. Markplane items are user-created files read at runtime from the filesystem. `react-markdown` handles this natively with zero build step.

### Why TanStack Query Over SWR?

1. **Optimistic updates**: `useMutation` with `onMutate`/`onError`/`onSettled` lifecycle makes kanban drag-and-drop, status changes, and inline edits feel instant with proper rollback on failure. SWR's `mutate()` is simpler but less structured for complex mutations.
2. **WebSocket invalidation**: `queryClient.invalidateQueries({ queryKey: ['tasks'] })` only refetches queries with active observers — perfect for targeted file-change events. SWR's `mutate()` with matcher functions works but is less ergonomic.
3. **Cache control**: `staleTime` (30s) prevents unnecessary refetches during navigation while WebSocket handles freshness. `gcTime` (5min) garbage-collects cache entries for queries with no active observers, keeping memory usage bounded.
4. **DevTools**: First-party React Query DevTools (`@tanstack/react-query-devtools`) is invaluable during development for inspecting cache state, query status, and refetch triggers.
5. **Community**: Larger ecosystem with more examples for patterns we need (optimistic mutations, WebSocket integration, dependent queries for graph traversal).

The ~9KB bundle size difference (SWR ~4KB vs TQ ~13KB) is acceptable given TQ's superior mutation handling and WebSocket integration.

### Client-Only State

TanStack Query handles all server state (API data, caching, revalidation). For client-only state:
- **React Context**: Theme preference (next-themes), sidebar collapsed state (shadcn SidebarProvider)
- **URL search params**: Active detail sheet (e.g., `?task=TASK-rm6d3`), view mode (kanban/list)

### Why Static Export?

`markplane serve` is a Rust HTTP server. The Next.js app is built once at release time into static HTML/CSS/JS, then embedded in the binary via `rust-embed`. At runtime, the Rust server serves static files and provides API endpoints. This means:
- Zero Node.js dependency for end users
- Single binary distribution
- API routes are handled by Rust, not Next.js

---

## 3. Project Structure

```
crates/markplane-web/ui/               # Next.js project root
│   ├── next.config.ts
│   ├── postcss.config.mjs           # Tailwind v4 via @tailwindcss/postcss
│   ├── tsconfig.json
│   ├── package.json
│   │
│   ├── public/
│   │
│   ├── src/
│   │   ├── app/                     # Next.js App Router
│   │   │   ├── layout.tsx           # Root layout: providers, sidebar, fonts
│   │   │   ├── page.tsx             # Redirect to /dashboard
│   │   │   ├── globals.css          # Tailwind directives + OKLCH theme variables
│   │   │   │
│   │   │   ├── dashboard/
│   │   │   │   └── page.tsx         # Project overview dashboard
│   │   │   │
│   │   │   ├── backlog/
│   │   │   │   └── page.tsx         # Kanban board + list view (detail via Sheet)
│   │   │   │
│   │   │   ├── roadmap/
│   │   │   │   └── page.tsx         # Epic roadmap with progress
│   │   │   │
│   │   │   ├── plans/
│   │   │   │   └── page.tsx         # Plans list (detail via Sheet)
│   │   │   │
│   │   │   ├── notes/
│   │   │   │   └── page.tsx         # Notes list (detail via Sheet)
│   │   │   │
│   │   │   ├── graph/
│   │   │   │   └── page.tsx         # Dependency graph (React Flow + ELK)
│   │   │   │
│   │   │   ├── archive/
│   │   │   │   └── page.tsx         # Archived items with restore action
│   │   │   │
│   │   │   └── settings/
│   │   │       └── page.tsx         # Project settings (type configuration)
│   │   │
│   │   ├── components/
│   │   │   ├── ui/                  # shadcn/ui primitives (22 components)
│   │   │   │
│   │   │   ├── layout/
│   │   │   │   ├── app-sidebar.tsx      # App sidebar navigation + theme toggle
│   │   │   │   ├── command-palette.tsx  # Cmd+K / ? command palette (includes create actions)
│   │   │   │   ├── command-palette-wrapper.tsx  # Keyboard shortcut handler
│   │   │   │   ├── global-create-dialog.tsx  # Handles create-item events from command palette
│   │   │   │   ├── mobile-header.tsx    # Mobile responsive header
│   │   │   │   └── providers.tsx        # React Query + Next Themes providers
│   │   │   │
│   │   │   ├── domain/              # Markplane-specific compound components
│   │   │   │   ├── create-dialog.tsx        # Reusable creation dialog (task/epic/plan/note)
│   │   │   │   ├── task-detail-sheet.tsx     # Task slide-over panel
│   │   │   │   ├── epic-detail-sheet.tsx     # Epic slide-over panel
│   │   │   │   ├── plan-detail-sheet.tsx     # Plan slide-over panel
│   │   │   │   ├── note-detail-sheet.tsx     # Note slide-over panel
│   │   │   │   ├── task-card.tsx             # Kanban/list task card
│   │   │   │   ├── status-badge.tsx
│   │   │   │   ├── priority-indicator.tsx
│   │   │   │   ├── epic-progress.tsx
│   │   │   │   ├── metrics-card.tsx
│   │   │   │   ├── markdown-renderer.tsx     # Read-only markdown with wiki-links
│   │   │   │   ├── markdown-editor.tsx       # TipTap rich text + markdown source
│   │   │   │   ├── graph-view.tsx            # React Flow + ELK dependency graph
│   │   │   │   ├── wiki-link-chip.tsx        # Clickable [[ID]] chip
│   │   │   │   ├── tiptap-wiki-link.ts       # TipTap wiki-link extension
│   │   │   │   ├── inline-edit.tsx           # In-place text editing
│   │   │   │   ├── tag-editor.tsx            # Tag management
│   │   │   │   ├── entity-combobox.tsx       # Searchable entity selector
│   │   │   │   ├── entity-ref-editor.tsx     # Edit entity relationships
│   │   │   │   ├── field-row.tsx             # Consistent field display
│   │   │   │   ├── empty-state.tsx
│   │   │   │   ├── error-boundary.tsx
│   │   │   │   ├── page-transition.tsx       # Framer Motion transitions
│   │   │   │   └── resizable-sheet-content.tsx
│   │   │   │
│   │   │   └── providers.tsx        # Additional provider setup
│   │   │
│   │   ├── lib/
│   │   │   ├── api.ts               # API client (fetch wrapper for /api/*)
│   │   │   ├── types.ts             # TypeScript types mirroring Rust models
│   │   │   ├── constants.ts         # Status configs, priority configs, nav items, prefix routing
│   │   │   ├── utils.ts             # Shared utilities (cn(), etc.)
│   │   │   └── hooks/
│   │   │       ├── use-tasks.ts     # useTasks(), useTask(), useArchivedTasks()
│   │   │       ├── use-epics.ts     # useEpics(), useEpic()
│   │   │       ├── use-plans.ts     # usePlans(), usePlan()
│   │   │       ├── use-notes.ts     # useNotes(), useNote()
│   │   │       ├── use-summary.ts   # useSummary()
│   │   │       ├── use-graph.ts     # useGraph(focusId?)
│   │   │       ├── use-search.ts    # useSearch(query)
│   │   │       ├── use-websocket.ts # WebSocket + TanStack Query invalidation
│   │   │       ├── use-mutations.ts # All mutations (create, update, archive, link)
│   │   │       └── use-keyboard-nav.ts # Keyboard chord navigation (g+letter)
│   │   │
│   │   └── hooks/
│   │       └── use-mobile.ts        # Mobile viewport detection
│   │
│   └── out/                         # Static build output (gitignored)

crates/markplane-cli/src/commands/
    └── serve.rs                     # Axum HTTP server + WebSocket + file watcher
```

### Component Organization Rationale

- **`ui/`**: Raw shadcn/ui primitives. Never import domain logic. Copied via `npx shadcn@latest add`.
- **`domain/`**: Markplane-specific components that compose `ui/` primitives with business logic. The kanban board, task cards, status badges — all here.
- **`layout/`**: App shell components (sidebar, header, command palette). These compose `ui/` and `domain/` components.
- **`shared/`**: Generic components not specific to Markplane (data tables, empty states, skeletons).

---

## 4. Data Layer

### 4.1 Static Export + Rust API (Primary Architecture)

Since the Next.js app is statically exported, **all data fetching happens client-side** via TanStack Query hooks calling the Rust HTTP API. There are no Next.js API routes or server-side data fetching at runtime.

```
┌────────────────────────────────────────────────────┐
│                    Browser                          │
│                                                     │
│  TanStack Query ──fetch()──→ localhost:4200/api/*    │
│                              │                      │
│  WebSocket ←──ws──→ localhost:4200/ws               │
└────────────────────────────────────────────────────┘
                               │
                    ┌──────────▼──────────┐
                    │   Rust HTTP Server   │
                    │      (axum)         │
                    │                      │
                    │  GET /api/tasks      │──→ markplane_core::list_tasks()
                    │  GET /api/tasks/:id  │──→ markplane_core::read_item()
                    │  PATCH /api/tasks/:id│──→ markplane_core::update_task()
                    │  POST /api/tasks     │──→ markplane_core::create_task()
                    │  GET /api/epics      │──→ markplane_core::list_epics()
                    │  GET /api/summary    │──→ .context/summary.md
                    │  GET /api/graph/:id  │──→ build_reference_graph()
                    │  POST /api/sync      │──→ project.sync_all()
                    │  WS  /ws            │──→ notify file watcher
                    │                      │
                    │  GET /*              │──→ rust-embed (static files)
                    └──────────────────────┘
```

### 4.2 REST API Design

The Rust HTTP server exposes a REST API that mirrors `markplane-core` operations. All responses are JSON.

#### Tasks (Backlog Items)

| Method | Path | Description | Maps to |
|--------|------|-------------|---------|
| `GET` | `/api/tasks` | List tasks with filters | `list_tasks(filter)` |
| `GET` | `/api/tasks/:id` | Get task detail | `read_item(id)` |
| `POST` | `/api/tasks` | Create task | `create_task(...)` |
| `PATCH` | `/api/tasks/:id` | Update task fields (status, priority, body, etc.) | `update_task()` + `link_items()` |
| `DELETE` | `/api/tasks/:id` | Archive task | `archive_item(id)` |

**Query parameters for `GET /api/tasks`:**
- `status` — comma-separated: `in-progress,planned`
- `priority` — comma-separated: `critical,high`
- `epic` — epic ID: `EPIC-xa7r2`
- `tags` — comma-separated: `ui,backend`
- `assignee` — assignee name
- `type` — item type: `bug,feature`
- `search` — full-text search across title and body

#### Epics

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/api/epics` | List all epics with progress metrics |
| `GET` | `/api/epics/:id` | Get epic detail with linked task summary |
| `POST` | `/api/epics` | Create epic |
| `PATCH` | `/api/epics/:id` | Update epic fields (title, status, priority, dates, tags, body) |

#### Plans & Notes

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/api/plans` | List plans |
| `GET` | `/api/plans/:id` | Get plan detail |
| `POST` | `/api/plans` | Create plan (optionally linked to a task) |
| `PATCH` | `/api/plans/:id` | Update plan fields (title, status, body) |
| `GET` | `/api/notes` | List notes |
| `GET` | `/api/notes/:id` | Get note detail |
| `POST` | `/api/notes` | Create note |
| `PATCH` | `/api/notes/:id` | Update note fields (title, status, tags, body) |

#### Archive

| Method | Path | Description |
|--------|------|-------------|
| `POST` | `/api/items/:id/archive` | Archive any item (task, epic, plan, note) |
| `POST` | `/api/items/:id/unarchive` | Restore an archived item |

All list endpoints (`GET /api/tasks`, `/api/epics`, `/api/plans`, `/api/notes`) accept an `?archived=true` query parameter to return archived items instead of active ones.

#### Relationships

| Method | Path | Description |
|--------|------|-------------|
| `POST` | `/api/link` | Add or remove a relationship between two items |

The link endpoint accepts `{ from, to, relation, remove? }` where `relation` is one of: `blocks`, `depends_on`, `epic`, `plan`, `implements`, `related`. Set `remove: true` to remove an existing link.

#### Project-Level

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/api/summary` | Project summary with counts, now epics, tasks, blocked items, context |
| `GET` | `/api/graph/:id` | Dependency graph for an item (2 hops) |
| `GET` | `/api/graph` | Full project dependency graph |
| `POST` | `/api/sync` | Trigger `markplane sync` (regenerate INDEX.md + .context/) |
| `GET` | `/api/search?q=...` | Full-text search across all items (min 2 chars) |
| `GET` | `/api/config` | Get project configuration (item types, note types) |
| `PATCH` | `/api/config` | Update project configuration (validates, trims, lowercases, deduplicates) |

#### Response Format

All API responses follow a consistent envelope:

```typescript
// Success
{ "data": T }

// Error
{ "error": { "code": string, "message": string } }

// List with metadata
{ "data": T[], "meta": { "total": number } }
```

### 4.3 TypeScript Types

Mirror the Rust data model exactly:

```typescript
// lib/types.ts

type StatusCategory = 'draft' | 'backlog' | 'planned' | 'active' | 'completed' | 'cancelled';
type TaskWorkflow = Record<StatusCategory, string[]>;  // configurable via workflows.task in config.yaml
type EpicStatus = 'now' | 'next' | 'later' | 'done';
type PlanStatus = 'draft' | 'approved' | 'in-progress' | 'done';
type NoteStatus = 'draft' | 'active' | 'archived';
type Priority = 'critical' | 'high' | 'medium' | 'low' | 'someday';
type TaskType = string;  // configurable via task_types in config.yaml
type Effort = 'xs' | 'small' | 'medium' | 'large' | 'xl';
type NoteType = 'research' | 'analysis' | 'idea' | 'decision' | 'meeting';

interface Task {
  id: string;
  title: string;
  status: string;  // configurable — resolved to StatusCategory via TaskWorkflow
  priority: Priority;
  type: TaskType;
  effort: Effort;
  tags: string[];
  epic: string | null;
  plan: string | null;
  depends_on: string[];
  blocks: string[];
  assignee: string | null;
  position: string | null;  // Fractional index for kanban ordering
  created: string;  // ISO date
  updated: string;
  body: string;     // Markdown body content
}

interface Epic {
  id: string;
  title: string;
  status: EpicStatus;
  priority: Priority;
  started: string | null;
  target: string | null;
  tags: string[];
  body: string;
  // Computed by API:
  task_count: number;
  done_count: number;
  progress: number;  // 0.0 - 1.0
  status_breakdown: Record<string, number>;  // keyed by task status strings
}

interface Plan {
  id: string;
  title: string;
  status: PlanStatus;
  implements: string[];
  epic: string | null;
  created: string;
  updated: string;
  body: string;
}

interface Note {
  id: string;
  title: string;
  note_type: NoteType;
  status: NoteStatus;
  tags: string[];
  related: string[];
  created: string;
  updated: string;
  body: string;
}

interface ProjectSummary {
  name: string;
  description: string;
  counts: {
    total: number;
    in_progress: number;
    planned: number;
    backlog: number;
    draft: number;
    done: number;
    blocked: number;
  };
  active_epics: Epic[];
  in_progress_tasks: Task[];
  blocked_tasks: Task[];
  recent_completions: Task[];
  next_up_tasks: Task[];
  context_summary: string | null;       // Content of .context/summary.md
  context_last_synced: string | null;   // ISO 8601 timestamp
}

interface GraphNode {
  id: string;
  type: 'task' | 'epic' | 'plan' | 'note';
  title: string;
  status: string;
  priority: string;
  tags: string[];
}

interface GraphEdge {
  source: string;
  target: string;
  relation: 'blocks' | 'depends_on' | 'implements' | 'epic' | 'related';
}
```

### 4.4 TanStack Query Hooks

**Query hooks** (`lib/hooks/`):

| Hook | File | Purpose |
|------|------|---------|
| `useTasks(filter?)` | `use-tasks.ts` | Query tasks with optional status/priority/epic/tags/assignee filters |
| `useTask(id)` | `use-tasks.ts` | Single task query |
| `useArchivedTasks()` | `use-tasks.ts` | Archived tasks query |
| `useEpics(filter?)` | `use-epics.ts` | Epic queries with optional filter |
| `useEpic(id)` | `use-epics.ts` | Single epic query |
| `usePlans(filter?)` | `use-plans.ts` | Plan queries |
| `usePlan(id)` | `use-plans.ts` | Single plan query |
| `useNotes(filter?)` | `use-notes.ts` | Note queries |
| `useNote(id)` | `use-notes.ts` | Single note query |
| `useSummary()` | `use-summary.ts` | Dashboard project summary |
| `useGraph(focusId?)` | `use-graph.ts` | Dependency graph (all or focused on entity) |
| `useSearch(query)` | `use-search.ts` | Full-text search (min 2 chars) |
| `useConfig()` | `use-config.ts` | Project configuration (item types, note types) |

**Mutation hooks** (`use-mutations.ts`):

| Hook | Purpose |
|------|---------|
| `useCreateTask()` | Create new task |
| `useCreateEpic()` | Create new epic |
| `useCreatePlan()` | Create new plan (optionally linked to a task) |
| `useCreateNote()` | Create new note |
| `useUpdateTask()` | Update task properties (optimistic + rollback) |
| `useUpdateEpic()` | Update epic properties |
| `useUpdatePlan()` | Update plan properties |
| `useUpdateNote()` | Update note properties |
| `useArchiveItem()` | Archive a single item |
| `useBatchArchive()` | Batch archive multiple items |
| `useUnarchiveItem()` | Restore an archived item |
| `useUpdateConfig()` | Update project configuration (optimistic + rollback) |

**Infrastructure hooks**:

| Hook | File | Purpose |
|------|------|---------|
| `useWebSocket()` | `use-websocket.ts` | WebSocket with auto-reconnect + TanStack Query cache invalidation |
| `useKeyboardNav()` | `use-keyboard-nav.ts` | Keyboard chord navigation (`g`+letter) + `?` for command palette |
| `useMobile()` | `hooks/use-mobile.ts` | Mobile viewport detection |

All mutation hooks use TanStack Query's `useMutation` with `onMutate`/`onError`/`onSettled` lifecycle for optimistic updates with rollback on failure, plus `sonner` toast notifications for success/error feedback. Update and archive mutations show a toast with an "Undo" action button (5-second window) that reverts the change via a second API call.

---

## 5. Routing

| Route | Page | Description |
|-------|------|-------------|
| `/` | Redirect | Redirects to `/dashboard` |
| `/dashboard` | Dashboard | Project overview, metrics, active work, blocked items |
| `/backlog` | Backlog | Kanban board + list view, filterable. Task detail via Sheet slide-over |
| `/roadmap` | Roadmap | Epic roadmap with progress bars and status breakdowns |
| `/plans` | Plans | All implementation plans. Plan detail via Sheet slide-over |
| `/notes` | Notes | Research, ideas, decisions. Note detail via Sheet slide-over |
| `/graph` | Dependency Graph | Full project graph (React Flow + ELK layout) |
| `/archive` | Archive | Archived items with restore action |
| `/settings` | Settings | Project configuration: task types, note types |

### URL Design

Item detail views use Sheet slide-over panels driven by URL query parameters (e.g., `/backlog?task=TASK-rm6d3`). This avoids full-page navigation for quick viewing/editing while keeping URLs shareable. The command palette (`Cmd+K` or `?`) provides full-text search across all entities.

---

## 6. Component Architecture

### 6.1 Component Tree

```
<RootLayout>                          # app/layout.tsx
├── <Providers>                      # React Query + Next Themes (layout/providers.tsx)
│   ├── <SidebarProvider>            # shadcn sidebar state
│   │   ├── <AppSidebar>             # Left nav: logo, navigation links, theme toggle
│   │   ├── <CommandPaletteWrapper>  # Keyboard shortcut handler (Cmd+K, ?, g+letter)
│   │   ├── <MobileHeader>           # Mobile responsive header
│   │   │
│   │   └── <main>
│   │       └── {page content}       # Page-specific content
│   │
│   └── <WebSocket>                  # useWebSocket() at app root
│   └── <KeyboardNav>               # useKeyboardNav() chord navigation
│   └── <Toaster>                    # sonner toast notifications
```

### 6.2 Key Domain Components

#### KanbanBoard
```
<KanbanBoard>
├── <FilterBar>                      # Status, priority, epic, tag filters
├── <ViewToggle>                     # Kanban / List / Table switch
└── <div className="flex gap-4">     # Horizontal scroll container
    ├── <KanbanColumn status="in-progress" wipLimit={5}>
    │   └── <TaskCard>*              # Draggable cards
    ├── <KanbanColumn status="planned">
    ├── <KanbanColumn status="backlog">
    └── <KanbanColumn status="draft">
```

- Drag-and-drop via `@dnd-kit/core` + `@dnd-kit/sortable` (lightweight, accessible)
- Drop triggers `PATCH /api/tasks/:id` to update status and position
- `fractional-indexing` for stable card ordering within columns
- TanStack Query optimistic updates via `useMutation` for instant feedback with rollback on failure
- **WIP limits**: Configurable per-column (e.g., 5 for In Progress). Column header shows count/limit.

#### TaskDetailSheet
```
<TaskDetailSheet>                     # Sheet slide-over (resizable)
├── <Header>                         # Title (inline-editable), status badge, archive button
├── <div className="flex">
│   ├── <main>                       # 2/3 width
│   │   ├── <MarkdownEditor>         # TipTap rich text + markdown source toggle
│   │   └── (wiki-links rendered as clickable chips)
│   │
│   └── <aside>                      # 1/3 width — metadata sidebar
│       ├── <StatusBadge> (dropdown)
│       ├── <PriorityIndicator> (dropdown)
│       ├── <EffortBadge> (dropdown)
│       ├── <TaskType> (dropdown)
│       ├── <EpicLink> (EntityCombobox)
│       ├── <PlanLink> (EntityCombobox)
│       ├── <TagEditor>
│       ├── <AssigneeField> (InlineEdit)
│       ├── <EntityRefEditor>         # depends_on, blocks, implements, related
│       └── <DateInfo>               # created, updated
```

#### GraphView
```
<GraphView>
├── <Toolbar>                        # Layout direction (TB/LR), layer toggles (Epics/Plans/Notes)
├── <FilterBar>                      # Priority, epic, tag multi-select filters + show completed toggle
├── <ReactFlow>                      # @xyflow/react + elkjs for auto-layout
│   ├── <ItemNode>*                  # Custom node: ID, title, status, priority color
│   ├── <EpicGroup>*                 # Compound grouping for epic containers
│   ├── <Edge>*                      # Styled edges: blocks (animated), depends_on, epic (dashed), implements, related
│   ├── <MiniMap>                    # Overview minimap
│   ├── <Controls>                   # Zoom, fit view
│   └── <Legend>                     # Edge type legend
```

#### Markdown Rendering & Editing

Two components handle markdown:

- **`MarkdownRenderer`** (read-only): Uses `react-markdown` + `remark-gfm` to render markdown body content. `[[TASK-rm6d3]]` wiki-links are pre-processed and transformed into entity-colored `<WikiLinkChip>` components with click-to-navigate behavior.

- **`MarkdownEditor`** (editing): Uses TipTap (`@tiptap/react`) with a dual-mode interface:
  - **Rich text mode**: WYSIWYG editing with formatting toolbar (bold, italic, strikethrough, headings, lists, task lists, code blocks, blockquotes, links). Uses `@tiptap/starter-kit`, `@tiptap/extension-task-list`, `@tiptap/extension-task-item`, `@tiptap/extension-link`.
  - **Markdown source mode**: Raw markdown editing with `tiptap-markdown` for serialization.
  - Custom `tiptap-wiki-link.ts` extension for `[[ID]]` syntax support.

Both components use `WikiLinkChip` for rendering entity references as interactive, entity-colored inline chips.

### 6.3 shadcn/ui Components Used (22 installed)

| Component | Usage |
|-----------|-------|
| `Badge` | Status, priority, effort, tags |
| `Button` | Actions, navigation |
| `Card` | Task cards, metric cards, dashboard panels |
| `Command` | Command palette (Cmd+K / ?) |
| `Dialog` | Confirmation dialogs |
| `DropdownMenu` | Context menus, status/priority selectors |
| `Input` | Search, form fields |
| `Popover` | Inline editors, entity selectors |
| `Select` | Status/priority/effort selectors |
| `Separator` | Visual dividers |
| `Sheet` | Detail slide-over panels (task, epic, plan, note) |
| `Sidebar` | Main navigation sidebar with collapsible state |
| `Skeleton` | Loading states |
| `Table` | List views, epic task tables |
| `Tabs` | View toggles (kanban/list) |
| `Textarea` | Multi-line text input |
| `Tooltip` | Hover info on badges, icons |
| `Progress` | Epic completion bars |
| `Sonner` | Toast notification config (via sonner) |
| `Markplane Logo` | Custom SVG logo component |

---

## 7. Theme System

### 7.1 OKLCH Color Variables

Tailwind v4 + shadcn/ui use OKLCH color space. Dark mode is the default, with light mode opt-in via class strategy.

```css
/* app/globals.css */
@import "tailwindcss";

@theme inline {
  --color-background: var(--background);
  --color-foreground: var(--foreground);
  --color-card: var(--card);
  --color-card-foreground: var(--card-foreground);
  --color-primary: var(--primary);
  --color-primary-foreground: var(--primary-foreground);
  --color-secondary: var(--secondary);
  --color-secondary-foreground: var(--secondary-foreground);
  --color-muted: var(--muted);
  --color-muted-foreground: var(--muted-foreground);
  --color-accent: var(--accent);
  --color-accent-foreground: var(--accent-foreground);
  --color-destructive: var(--destructive);
  --color-border: var(--border);
  --color-ring: var(--ring);

  /* Markplane semantic colors */
  --color-status-draft: var(--status-draft);
  --color-status-backlog: var(--status-backlog);
  --color-status-planned: var(--status-planned);
  --color-status-in-progress: var(--status-in-progress);
  --color-status-done: var(--status-done);
  --color-status-cancelled: var(--status-cancelled);
  --color-status-active: var(--status-active);

  --color-priority-critical: var(--priority-critical);
  --color-priority-high: var(--priority-high);
  --color-priority-medium: var(--priority-medium);
  --color-priority-low: var(--priority-low);
}

/* Dark theme (default) */
:root {
  --background: oklch(0.14 0.005 260);
  --foreground: oklch(0.95 0.01 260);
  --card: oklch(0.18 0.005 260);
  --card-foreground: oklch(0.95 0.01 260);
  --primary: oklch(0.65 0.18 250);       /* Blue accent */
  --primary-foreground: oklch(0.98 0 0);
  --secondary: oklch(0.22 0.01 260);
  --secondary-foreground: oklch(0.90 0.01 260);
  --muted: oklch(0.22 0.005 260);
  --muted-foreground: oklch(0.65 0.01 260);
  --accent: oklch(0.25 0.01 260);
  --accent-foreground: oklch(0.95 0.01 260);
  --destructive: oklch(0.55 0.2 25);
  --border: oklch(0.28 0.005 260);
  --ring: oklch(0.65 0.18 250);

  /* Status colors (dark) */
  --status-draft: oklch(0.55 0.01 260);
  --status-backlog: oklch(0.60 0.08 260);
  --status-planned: oklch(0.70 0.15 280);
  --status-in-progress: oklch(0.70 0.18 250);
  --status-done: oklch(0.70 0.16 150);
  --status-cancelled: oklch(0.50 0.01 260);
  --status-active: oklch(0.70 0.18 250);

  /* Priority colors (dark) */
  --priority-critical: oklch(0.60 0.22 25);
  --priority-high: oklch(0.70 0.17 50);
  --priority-medium: oklch(0.75 0.14 85);
  --priority-low: oklch(0.65 0.10 200);
}

/* Light theme */
.light {
  --background: oklch(0.98 0.005 260);
  --foreground: oklch(0.15 0.01 260);
  --card: oklch(1.0 0 0);
  --card-foreground: oklch(0.15 0.01 260);
  --primary: oklch(0.55 0.2 250);
  --primary-foreground: oklch(0.98 0 0);
  --secondary: oklch(0.94 0.005 260);
  --secondary-foreground: oklch(0.30 0.01 260);
  --muted: oklch(0.94 0.005 260);
  --muted-foreground: oklch(0.50 0.01 260);
  --accent: oklch(0.94 0.005 260);
  --accent-foreground: oklch(0.20 0.01 260);
  --destructive: oklch(0.55 0.22 25);
  --border: oklch(0.88 0.005 260);
  --ring: oklch(0.55 0.2 250);

  /* Status colors (light) — slightly adjusted for contrast */
  --status-draft: oklch(0.60 0.01 260);
  --status-backlog: oklch(0.50 0.10 260);
  --status-planned: oklch(0.50 0.18 280);
  --status-in-progress: oklch(0.50 0.2 250);
  --status-done: oklch(0.50 0.18 150);
  --status-cancelled: oklch(0.55 0.01 260);
  --status-active: oklch(0.50 0.2 250);

  --priority-critical: oklch(0.50 0.25 25);
  --priority-high: oklch(0.55 0.2 50);
  --priority-medium: oklch(0.55 0.16 85);
  --priority-low: oklch(0.50 0.12 200);
}
```

### 7.2 Dark Mode Implementation

Use `next-themes` for system-aware dark/light switching, with TanStack Query provider and Geist fonts:

```typescript
// app/layout.tsx
import { ThemeProvider } from 'next-themes';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { ReactQueryDevtools } from '@tanstack/react-query-devtools';
import { GeistSans } from 'geist/font/sans';
import { GeistMono } from 'geist/font/mono';

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 30_000,       // 30s — WebSocket handles freshness
      gcTime: 5 * 60_000,      // 5min — clean up unused queries
      refetchOnWindowFocus: false, // WebSocket handles this
      retry: 1,                // Local server, minimal retries
    },
  },
});

export default function RootLayout({ children }) {
  return (
    <html lang="en" className={`${GeistSans.variable} ${GeistMono.variable}`} suppressHydrationWarning>
      <body className="font-sans">
        <QueryClientProvider client={queryClient}>
          <ThemeProvider attribute="class" defaultTheme="dark" enableSystem>
            {children}
          </ThemeProvider>
          <ReactQueryDevtools initialIsOpen={false} />
        </QueryClientProvider>
      </body>
    </html>
  );
}
```

Dark is the default. Users can toggle to light or system-aware. Preference persists in `localStorage`. Geist Sans is used for all UI text (`font-sans`), with Geist Mono available via `font-mono` for code blocks, IDs, and monospaced content.

---

## 8. Real-Time Updates

### 8.1 WebSocket Architecture

The Rust backend runs a WebSocket server alongside the HTTP API. It uses the `notify` crate to watch `.markplane/` for file changes and broadcasts structured events to connected clients.

```
                    notify crate
                        │
                        ▼
              ┌─────────────────┐
              │  File Watcher    │ watches: .markplane/**/*.md, config.yaml
              │  (debounced)     │
              └────────┬────────┘
                       │
                       ▼
              ┌─────────────────┐
              │ Event Processor  │ Parse changed path → entity type + ID
              └────────┬────────┘
                       │
                       ▼
              ┌─────────────────┐
              │ WebSocket Broker │ Broadcast to all connected clients
              └─────────────────┘
```

### 8.2 Event Types

```typescript
// WebSocket messages from server → client
type WsEvent =
  | { type: 'file_changed'; entity: 'task' | 'epic' | 'plan' | 'note'; id: string; action: 'created' | 'modified' | 'deleted' }
  | { type: 'config_changed' }
  | { type: 'sync_complete' }
  | { type: 'connected'; version: string }
```

### 8.3 Client Integration

The `useWebSocket()` hook at the app root listens for events and triggers TanStack Query invalidation for affected resources. This means:
- User edits a file in their editor → Rust detects change → WS event → TanStack Query refetches active observers → UI updates
- CLI runs `markplane status TASK-fq2x8 done` → file change → UI updates
- MCP server creates a task → file change → UI updates

Debouncing: File watcher events are debounced at 100ms to batch rapid changes (e.g., `markplane sync` writes multiple files).

---

## 9. Key Page Designs

> **Note**: The wireframes below are design references. The implementation uses Sheet slide-over panels for item detail views rather than dedicated detail pages. The actual sidebar navigation is: Dashboard, Backlog, Plans, Notes, Roadmap, Graph (no separate Epics or Search pages — search is via the command palette).

### 9.1 Dashboard (`/dashboard`)

```
┌─────────────────────────────────────────────────────────┐
│ [Sidebar]  │  Dashboard                                  │
│            │                                              │
│  Dashboard │  ┌──────────┐ ┌──────────┐ ┌──────────┐    │
│  Backlog   │  │ In Prog  │ │ Planned  │ │ Blocked  │    │
│  Epics     │  │    3     │ │    5     │ │    2     │    │
│  Roadmap   │  └──────────┘ └──────────┘ └──────────┘    │
│  Plans     │                                              │
│  Notes     │  ┌─────────────────────────────────────┐    │
│  Graph     │  │ Active Work                          │    │
│  Search    │  │ ▸ TASK-hj6r9 Implement user auth  high │    │
│            │  │ ▸ TASK-nt6j4 Add search to...    med  │    │
│            │  │ ▸ TASK-kp2m5 Fix pagination      high │    │
│            │  └─────────────────────────────────────┘    │
│            │                                              │
│            │  ┌─────────────────┐ ┌─────────────────┐    │
│            │  │ Blocked Items    │ │ Recent Done      │    │
│            │  │ TASK-vn8k4 ←T-vn8k4│ │ ✓ TASK-gt3w7 (2d) │    │
│            │  └─────────────────┘ └─────────────────┘    │
│            │                                              │
│            │  ┌─────────────────────────────────────┐    │
│            │  │ Epic Progress                        │    │
│            │  │ EPIC-xa7r2 Core CLI     ████████░░ 80%│    │
│            │  │ EPIC-kb4n9 MCP Server   ██████░░░░ 60%│    │
│            │  │ EPIC-gc8t5 Web UI       ██░░░░░░░░ 20%│    │
│            │  └─────────────────────────────────────┘    │
│            │                                              │
│            │  ┌─────────────────────────────────────┐    │
│            │  │ AI CONTEXT                    [stale]│    │
│            │  │                                      │    │
│            │  │ .context/summary.md content:          │    │
│            │  │ Project: markplane                    │    │
│            │  │ 15 tasks (3 in-progress, 5 planned)  │    │
│            │  │ 3 now epics                          │    │
│            │  │ Last sync: 2m ago                    │    │
│            │  │                                      │    │
│            │  │ [Sync Now]                           │    │
│            │  └─────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────┘
```

Data: `GET /api/summary` provides all dashboard data in one request.

The **AI Context** panel surfaces the `.context/summary.md` content — Markplane's key differentiator as an AI-native project management system. It shows a "stale" indicator when the context summary is out of date relative to recent file changes (detected via the last sync timestamp vs. latest file modification). A "Sync Now" button triggers `POST /api/sync` to regenerate the context layer.

### 9.2 Backlog Kanban (`/backlog`)

```
┌─────────────────────────────────────────────────────────┐
│ [Sidebar]  │  Backlog                     [Kanban|List]  │
│            │  [Filter: Status▾ Priority▾ Epic▾ Tags▾]    │
│            │                                              │
│            │  In Progress(3)  Planned(5)   Backlog(12)   │
│            │  ┌──────────┐   ┌──────────┐  ┌──────────┐ │
│            │  │ TASK-hj6r9 │   │ TASK-wq4t8 │  │ TASK-bm9v6 │ │
│            │  │ User auth│   │ CSV exp  │  │ Dark mode│ │
│            │  │ 🔴 high  │   │ 🟡 med   │  │ 🟡 med   │ │
│            │  │ EPIC-xa7r2 │   │ EPIC-kb4n9 │  │ EPIC-gc8t5 │ │
│            │  ├──────────┤   ├──────────┤  ├──────────┤ │
│            │  │ TASK-nt6j4 │   │ TASK-xr7n3 │  │ TASK-cs5k2 │ │
│            │  │ Search   │   │ Form val │  │ Profile  │ │
│            │  │ 🟡 med   │   │ 🟢 low   │  │ 🟢 low   │ │
│            │  └──────────┘   └──────────┘  └──────────┘ │
└─────────────────────────────────────────────────────────┘
```

- Drag between columns updates status via `PATCH /api/tasks/:id/status`
- Filter bar persists state in URL query params for shareability
- List view alternative uses `<DataTable>` with sortable columns

### 9.3 Task Detail (Sheet slide-over, e.g., `/backlog?task=TASK-rm6d3`)

```
┌─────────────────────────────────────────────────────────┐
│ [Sidebar]  │  Backlog > TASK-rm6d3                         │
│            │                                              │
│            │  ┌──────────────────────┐ ┌──────────────┐  │
│            │  │                      │ │ Status       │  │
│            │  │ # Add dark mode      │ │ [in-progress]│  │
│            │  │                      │ │              │  │
│            │  │ ## Description       │ │ Priority     │  │
│            │  │ Implement system-    │ │ [🔴 high]    │  │
│            │  │ aware dark mode...   │ │              │  │
│            │  │                      │ │ Effort       │  │
│            │  │ ## Acceptance Crit.  │ │ [medium]     │  │
│            │  │ - [x] Respects OS   │ │              │  │
│            │  │ - [ ] Manual toggle  │ │ Type         │  │
│            │  │ - [ ] WCAG 2.2 AA   │ │ [feature]    │  │
│            │  │                      │ │              │  │
│            │  │ ## References        │ │ Epic         │  │
│            │  │ → EPIC-gc8t5           │ │ [EPIC-gc8t5] → │  │
│            │  │ ← TASK-wp7v2           │ │              │  │
│            │  │ → TASK-bg8t1           │ │ Tags         │  │
│            │  │                      │ │ [ui][theming]│  │
│            │  │                      │ │              │  │
│            │  │                      │ │ Depends On   │  │
│            │  │                      │ │ TASK-wp7v2 →   │  │
│            │  │                      │ │              │  │
│            │  │                      │ │ Blocks       │  │
│            │  │                      │ │ TASK-bg8t1 →   │  │
│            │  └──────────────────────┘ └──────────────┘  │
└─────────────────────────────────────────────────────────┘
```

- Left: `<MarkdownEditor>` (TipTap) with rich text + markdown source toggle
- Right: Metadata sidebar with editable fields (click status badge → dropdown to change)
- `[[ID]]` references render as entity-colored `<WikiLinkChip>` components
- Sidebar fields trigger `PATCH /api/tasks/:id` on change
- Relationship changes (epic, depends_on, blocks) use `POST /api/link`

---

## 10. Build & Deployment

### 10.1 Development Workflow

```bash
# Terminal 1: Run Rust backend with hot-reload
cargo watch -x 'run -p markplane-cli -- serve --dev'

# Terminal 2: Run Next.js dev server with proxy to Rust API
cd crates/markplane-web/ui
npm run dev    # next dev, proxies /api/* and /ws to localhost:4200
```

In development, Next.js runs its own dev server (e.g., port 3000) and proxies API/WS requests to the Rust server. This gives full HMR + Fast Refresh for the frontend.

### 10.2 Production Build

```bash
# Step 1: Build the Next.js static export
cd crates/markplane-web/ui
npm run build   # next build → generates out/ directory

# Step 2: Build the Rust binary (embeds static files)
cargo build --release -p markplane-cli
```

The Rust binary uses `rust-embed` (behind the `embed-ui` feature flag) to include the `out/` directory contents. Without the feature flag, static files are served from the filesystem at `crates/markplane-web/ui/out/`.

### 10.3 Runtime: `markplane serve`

```bash
markplane serve              # Start on http://localhost:4200
markplane serve --port 8080  # Custom port
markplane serve --open       # Open browser automatically
markplane serve --dev        # Dev mode: API only, no static files (for Next.js dev server proxy)
```

The Rust server (axum):
1. Serves embedded static HTML/CSS/JS for all frontend routes (with SPA fallback to `index.html`)
2. Handles `/api/*` requests using `markplane-core`
3. Runs WebSocket server on `/ws` for real-time file change notifications
4. Watches `.markplane/` via `notify` crate (debounced at 100ms)
5. Runs initial `sync_all()` on startup to ensure INDEX.md and .context/ are fresh

### 10.4 Next.js Configuration

```typescript
// next.config.ts
const nextConfig = {
  output: 'export',           // Static export for single-binary embedding
  trailingSlash: true,        // Required for static export routing
  images: {
    unoptimized: true,        // No image optimization server needed
  },
  // Dev mode: proxy API and WS to Rust backend
  rewrites: async () => ({
    fallback: [
      { source: '/api/:path*', destination: 'http://localhost:4200/api/:path*' },
    ],
  }),
  env: {
    NEXT_PUBLIC_WS_URL: process.env.NODE_ENV === 'development'
      ? 'ws://localhost:4200/ws' : undefined,
  },
};
```

---

## 11. Performance Considerations

### Code Splitting
- Next.js automatically code-splits by route
- React Flow + ELK (dependency graph) loaded only on `/graph` route
- TipTap editor loaded only in detail sheet panels

### Bundle Size Targets
- Initial load (dashboard): < 150KB gzipped
- React Flow chunk: ~80KB gzipped (lazy)
- Total: < 300KB gzipped for full app

### Data Fetching
- TanStack Query caches all responses in memory; page navigation reuses cached data with `staleTime` control
- `gcTime` (5 min default) garbage-collects cache entries for queries with no active observers
- WebSocket triggers targeted invalidation (only queries with active observers refetch)
- Dashboard summary is a single API call (~1KB response)
- Task lists paginate at 100 items (configurable)

### Rendering
- Markdown rendering happens client-side per-page (not per-list-item)
- Task cards in kanban show title + metadata only (no body rendering)
- `react-markdown` is tree-shaken to include only GFM plugin

---

## 12. Future Enhancements

These are explicitly out of scope for v1 but inform the architecture:

| Enhancement | Architectural Implication |
|------------|--------------------------|
| MCP SSE transport | Add SSE endpoint to Rust server; UI could switch from REST to SSE |
| Collaborative editing | WebSocket already in place; add OT/CRDT for concurrent body edits |
| Git integration | Add `GET /api/git/blame/:id`, `POST /api/git/commit` endpoints |
| Offline support | Static export + service worker; queue mutations when offline |
| Custom themes | Theme variables are CSS custom properties; load user themes from config |
| Plugin widgets | Dashboard panels could load external components via dynamic import |

---

## 13. Implementation Status

All phases are complete:

- **Foundation**: Next.js 16 + shadcn/ui + Tailwind v4 scaffolded. Axum HTTP server with 22 API endpoints. Sidebar layout, routing, theme system. Dashboard with project summary.
- **Core Views**: Backlog kanban with dnd-kit drag-and-drop + fractional indexing. Sheet slide-over detail panels for all entity types. TipTap markdown editor with rich text + source modes. Epic roadmap with progress bars and drag-and-drop between Now/Next/Later columns.
- **Advanced Features**: WebSocket file watching with auto-reconnect. React Flow + ELK dependency graph with layer toggles and multi-filter system. Full-text search via command palette. Keyboard chord navigation (g+letter).
- **Polish**: Loading skeletons, error boundaries, toast notifications. Mobile responsive layout. `rust-embed` integration behind `embed-ui` feature flag. Archive view with batch archive support.
