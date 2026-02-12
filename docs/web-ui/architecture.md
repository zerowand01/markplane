# Markplane Web UI Architecture

**Status**: Design Proposal
**Created**: 2026-02-12
**Updated**: 2026-02-12
**Related**: [[TASK-017]], [[PLAN-001]]

---

## 1. Overview

The Markplane web UI is a local-first interface for browsing and managing `.markplane/` project data. It runs via `markplane serve` on `localhost:4200`, providing a modern dark-themed dashboard, kanban board, task detail views, epic progress tracking, roadmap timeline, dependency graph, and search.

### Design Principles

1. **Local-first**: Reads/writes `.markplane/` files via the Rust backend. No external services.
2. **Real-time**: File changes (from CLI, MCP, or manual edits) reflect in the UI immediately.
3. **Embeddable**: The production build can be embedded in the Rust binary via `rust-embed`, requiring zero Node.js runtime.
4. **Progressively enhanced**: Server Components for fast initial load; client interactivity where needed.

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
| Framework | Next.js 15+ (App Router) | RSC support, file-based routing, static export, industry standard |
| UI library | shadcn/ui | Composable, accessible, Tailwind-native, customizable via copy-paste |
| Styling | Tailwind CSS v4 | OKLCH color space, `@theme inline`, CSS-first config, dark mode via class strategy |
| State/data | TanStack Query v5 | Purpose-built `useMutation` lifecycle for optimistic updates, `staleTime`/`gcTime` for smart caching, targeted `invalidateQueries` for WebSocket integration, first-party DevTools |
| Fonts | Geist Sans + Geist Mono | Vercel's font family — native Next.js integration, slightly condensed for dashboard density, modern developer tool aesthetic |
| Timeline | SVAR React Gantt | MIT licensed, React 19 compatible, gantt/swimlane for roadmap view |
| Animations | Framer Motion | Drag-and-drop springs, page transitions, status change animations |
| Markdown | `react-markdown` + `remark-gfm` | Runtime rendering of arbitrary `.md` content (not page-based MDX); supports GFM tables, checkboxes |
| Syntax highlighting | `shiki` (lazy) | Accurate, VS Code-compatible, tree-shakeable |
| Graph visualization | `@xyflow/react` (React Flow) | Purpose-built for node/edge graphs, interactive, well-maintained, MIT licensed |
| Icons | `lucide-react` | Already bundled with shadcn/ui, consistent, tree-shakeable |
| Date handling | `date-fns` | Lightweight, tree-shakeable, no locale bloat |
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
- **React Context**: Theme preference, sidebar collapsed state
- **URL search params**: Filter selections, view mode (kanban/list/table) — shareable via URL
- **Zustand** (if needed): Complex client state like command palette history, drag-in-progress state. Only add if React Context proves insufficient.

### Why Static Export?

`markplane serve` is a Rust HTTP server. The Next.js app is built once at release time into static HTML/CSS/JS, then embedded in the binary via `rust-embed`. At runtime, the Rust server serves static files and provides API endpoints. This means:
- Zero Node.js dependency for end users
- Single binary distribution
- API routes are handled by Rust, not Next.js

---

## 3. Project Structure

```
crates/markplane-web/               # New crate (or directory within existing)
├── ui/                              # Next.js project root
│   ├── next.config.ts
│   ├── tailwind.config.ts           # Minimal — most config via CSS @theme
│   ├── tsconfig.json
│   ├── package.json
│   │
│   ├── public/
│   │   └── logo.svg
│   │
│   ├── src/
│   │   ├── app/                     # Next.js App Router
│   │   │   ├── layout.tsx           # Root layout: ThemeProvider, sidebar, fonts
│   │   │   ├── page.tsx             # Dashboard (redirect or inline)
│   │   │   ├── globals.css          # Tailwind directives + OKLCH theme variables
│   │   │   │
│   │   │   ├── dashboard/
│   │   │   │   └── page.tsx         # Project overview dashboard
│   │   │   │
│   │   │   ├── backlog/
│   │   │   │   ├── page.tsx         # Kanban board + list view
│   │   │   │   └── [id]/
│   │   │   │       └── page.tsx     # Task detail view
│   │   │   │
│   │   │   ├── epics/
│   │   │   │   ├── page.tsx         # All epics overview
│   │   │   │   └── [id]/
│   │   │   │       └── page.tsx     # Epic detail with linked tasks
│   │   │   │
│   │   │   ├── roadmap/
│   │   │   │   └── page.tsx         # Timeline / swimlane view
│   │   │   │
│   │   │   ├── plans/
│   │   │   │   ├── page.tsx         # Plans list
│   │   │   │   └── [id]/
│   │   │   │       └── page.tsx     # Plan detail
│   │   │   │
│   │   │   ├── notes/
│   │   │   │   ├── page.tsx         # Notes list
│   │   │   │   └── [id]/
│   │   │   │       └── page.tsx     # Note detail
│   │   │   │
│   │   │   ├── graph/
│   │   │   │   └── page.tsx         # Dependency graph (React Flow)
│   │   │   │
│   │   │   └── search/
│   │   │       └── page.tsx         # Full-text search with filters
│   │   │
│   │   ├── components/
│   │   │   ├── ui/                  # shadcn/ui primitives (Button, Card, Badge, etc.)
│   │   │   │
│   │   │   ├── layout/
│   │   │   │   ├── sidebar.tsx      # App sidebar navigation
│   │   │   │   ├── header.tsx       # Page header with breadcrumbs
│   │   │   │   ├── command-palette.tsx  # Cmd+K command palette
│   │   │   │   └── theme-toggle.tsx # Dark/light mode switch
│   │   │   │
│   │   │   ├── domain/              # Markplane-specific compound components
│   │   │   │   ├── kanban-board.tsx
│   │   │   │   ├── kanban-column.tsx
│   │   │   │   ├── task-card.tsx
│   │   │   │   ├── task-detail.tsx
│   │   │   │   ├── status-badge.tsx
│   │   │   │   ├── priority-indicator.tsx
│   │   │   │   ├── effort-badge.tsx
│   │   │   │   ├── epic-progress.tsx
│   │   │   │   ├── roadmap-timeline.tsx
│   │   │   │   ├── dependency-graph.tsx
│   │   │   │   ├── markdown-renderer.tsx
│   │   │   │   ├── item-references.tsx
│   │   │   │   ├── filter-bar.tsx
│   │   │   │   └── metrics-card.tsx
│   │   │   │
│   │   │   └── shared/              # Generic reusable components
│   │   │       ├── data-table.tsx
│   │   │       ├── empty-state.tsx
│   │   │       └── loading-skeleton.tsx
│   │   │
│   │   ├── lib/
│   │   │   ├── api.ts               # API client (fetch wrapper for /api/*)
│   │   │   ├── types.ts             # TypeScript types mirroring Rust models
│   │   │   ├── constants.ts         # Status colors, priority weights, routes
│   │   │   ├── utils.ts             # Shared utilities (cn(), formatDate, etc.)
│   │   │   └── hooks/
│   │   │       ├── use-tasks.ts     # TanStack Query: useTasks(filter?)
│   │   │       ├── use-task.ts      # TanStack Query: useTask(id)
│   │   │       ├── use-epics.ts     # TanStack Query: useEpics()
│   │   │       ├── use-epic.ts      # TanStack Query: useEpic(id)
│   │   │       ├── use-plans.ts     # TanStack Query: usePlans()
│   │   │       ├── use-notes.ts     # TanStack Query: useNotes()
│   │   │       ├── use-summary.ts   # TanStack Query: useSummary()
│   │   │       ├── use-graph.ts     # TanStack Query: useGraph(id)
│   │   │       ├── use-websocket.ts # WebSocket + TanStack Query invalidation
│   │   │       └── use-search.ts    # TanStack Query: useSearch(query)
│   │   │
│   │   └── styles/
│   │       └── (empty — all in globals.css)
│   │
│   └── out/                         # Static build output (gitignored)
│
└── src/                             # Rust: HTTP server + file watcher
    ├── server.rs                    # Axum/Actix HTTP server serving static + API
    ├── api.rs                       # REST API handlers (uses markplane-core)
    ├── websocket.rs                 # WebSocket handler for file change events
    └── embed.rs                     # rust-embed integration for static files
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
                    │   (axum or actix)    │
                    │                      │
                    │  GET /api/tasks      │──→ markplane_core::list_tasks()
                    │  GET /api/tasks/:id  │──→ markplane_core::read_item()
                    │  PATCH /api/tasks/:id│──→ markplane_core::update_status()
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
| `PATCH` | `/api/tasks/:id` | Update task fields | `update_status()`, `write_item()` |
| `PATCH` | `/api/tasks/:id/status` | Update status only | `update_status(id, status)` |
| `DELETE` | `/api/tasks/:id` | Archive task | `archive_item(id)` |

**Query parameters for `GET /api/tasks`:**
- `status` — comma-separated: `in-progress,planned`
- `priority` — comma-separated: `critical,high`
- `epic` — epic ID: `EPIC-001`
- `tags` — comma-separated: `ui,backend`
- `assignee` — assignee name
- `type` — item type: `bug,feature`
- `search` — full-text search across title and body

#### Epics

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/api/epics` | List all epics |
| `GET` | `/api/epics/:id` | Get epic detail with linked task summary |
| `POST` | `/api/epics` | Create epic |
| `PATCH` | `/api/epics/:id` | Update epic |

#### Plans & Notes

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/api/plans` | List plans |
| `GET` | `/api/plans/:id` | Get plan detail |
| `GET` | `/api/notes` | List notes |
| `GET` | `/api/notes/:id` | Get note detail |

#### Project-Level

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/api/summary` | Project summary (from `.context/summary.md`) |
| `GET` | `/api/metrics` | Project health metrics |
| `GET` | `/api/graph/:id` | Dependency graph for an item |
| `GET` | `/api/graph` | Full project dependency graph |
| `POST` | `/api/sync` | Trigger `markplane sync` |
| `GET` | `/api/search?q=...` | Full-text search across all items |
| `GET` | `/api/blocked` | Blocked items |

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

type TaskStatus = 'draft' | 'backlog' | 'planned' | 'in-progress' | 'done' | 'cancelled';
type EpicStatus = 'planned' | 'active' | 'done';
type PlanStatus = 'draft' | 'approved' | 'in-progress' | 'done';
type NoteStatus = 'draft' | 'active' | 'archived';
type Priority = 'critical' | 'high' | 'medium' | 'low' | 'someday';
type ItemType = 'feature' | 'bug' | 'enhancement' | 'chore' | 'research' | 'spike';
type Effort = 'xs' | 'small' | 'medium' | 'large' | 'xl';

interface Task {
  id: string;
  title: string;
  status: TaskStatus;
  priority: Priority;
  type: ItemType;
  effort: Effort;
  tags: string[];
  epic: string | null;
  plan: string | null;
  depends_on: string[];
  blocks: string[];
  assignee: string | null;
  created: string;  // ISO date
  updated: string;
  body: string;     // Rendered markdown body
}

interface Epic {
  id: string;
  title: string;
  status: EpicStatus;
  priority: Priority;
  started: string | null;
  target: string | null;
  tags: string[];
  depends_on: string[];
  body: string;
  // Computed by API:
  task_count: number;
  done_count: number;
  progress: number;  // 0.0 - 1.0
  status_breakdown: Record<TaskStatus, number>;  // e.g., { "in-progress": 3, "planned": 5, "done": 2 }
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
  type: string;
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
}

interface GraphNode {
  id: string;
  type: 'task' | 'epic' | 'plan' | 'note';
  title: string;
  status: string;
}

interface GraphEdge {
  source: string;
  target: string;
  relation: 'blocks' | 'depends_on' | 'implements' | 'epic' | 'plan' | 'related';
}
```

### 4.4 TanStack Query Hooks

```typescript
// lib/hooks/use-tasks.ts
import { useQuery } from '@tanstack/react-query';
import { fetcher } from '@/lib/api';
import type { Task, TaskStatus, Priority } from '@/lib/types';

interface TaskFilter {
  status?: TaskStatus[];
  priority?: Priority[];
  epic?: string;
  tags?: string[];
  assignee?: string;
}

export function useTasks(filter?: TaskFilter) {
  const params = new URLSearchParams();
  if (filter?.status) params.set('status', filter.status.join(','));
  if (filter?.priority) params.set('priority', filter.priority.join(','));
  if (filter?.epic) params.set('epic', filter.epic);
  // ...

  const query = params.toString();
  return useQuery<Task[]>({
    queryKey: ['tasks', filter],
    queryFn: () => fetcher(`/api/tasks${query ? `?${query}` : ''}`),
  });
}
```

```typescript
// lib/hooks/use-websocket.ts
import { useEffect } from 'react';
import { useQueryClient } from '@tanstack/react-query';

export function useWebSocket() {
  const queryClient = useQueryClient();

  useEffect(() => {
    const ws = new WebSocket(`ws://${window.location.host}/ws`);

    ws.onmessage = (event) => {
      const msg = JSON.parse(event.data);
      // msg: { type: "file_changed", entity: "task", id: "TASK-001", action: "modified" }
      // msg: { type: "sync_complete" }

      switch (msg.type) {
        case 'file_changed':
          // Invalidate the specific item and its list
          // Only queries with active observers will refetch
          queryClient.invalidateQueries({ queryKey: [msg.entity + 's', msg.id] });
          queryClient.invalidateQueries({ queryKey: [msg.entity + 's'] });
          queryClient.invalidateQueries({ queryKey: ['summary'] });
          break;
        case 'sync_complete':
          // Invalidate everything
          queryClient.invalidateQueries();
          break;
      }
    };

    return () => ws.close();
  }, [queryClient]);
}
```

---

## 5. Routing

| Route | Page | Description |
|-------|------|-------------|
| `/` | Redirect | Redirects to `/dashboard` |
| `/dashboard` | Dashboard | Project overview, metrics, active work, blocked items |
| `/backlog` | Backlog | Kanban board + list toggle, filterable |
| `/backlog/[id]` | Task Detail | Full task view: metadata sidebar + markdown body |
| `/epics` | Epics List | All epics with progress bars |
| `/epics/[id]` | Epic Detail | Epic info + linked tasks table + progress |
| `/roadmap` | Roadmap | Timeline/swimlane view of epics |
| `/plans` | Plans List | All implementation plans |
| `/plans/[id]` | Plan Detail | Plan content with linked tasks |
| `/notes` | Notes List | Research, ideas, decisions |
| `/notes/[id]` | Note Detail | Note content |
| `/graph` | Dependency Graph | Full project graph (React Flow) |
| `/graph?focus=[id]` | Focused Graph | Graph centered on a specific item |
| `/search` | Search | Full-text search with faceted filters |

### URL Design

Item URLs use the entity type + ID format (e.g., `/backlog/TASK-042`). This mirrors the `.markplane/` directory structure and makes URLs predictable. The `[id]` dynamic segment uses `generateStaticParams()` at build time — but since we're doing client-side data fetching, routes render a loading skeleton then fetch data via TanStack Query.

---

## 6. Component Architecture

### 6.1 Component Tree

```
<RootLayout>                          # app/layout.tsx
├── <ThemeProvider>                   # Dark/light mode (next-themes)
│   ├── <Sidebar>                    # Left nav: logo, navigation links, project name
│   │   ├── <SidebarNav>             # Link items with icons
│   │   └── <SidebarFooter>          # Theme toggle, settings
│   │
│   ├── <CommandPalette>             # Cmd+K overlay (shadcn/ui Command)
│   │
│   └── <main>
│       ├── <Header>                 # Breadcrumbs, page title, actions
│       └── {page content}           # Page-specific content
│
└── <WebSocketProvider>              # useWebSocket() at app root
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

- Drag-and-drop via `@dnd-kit/core` (lightweight, accessible)
- Drop triggers `PATCH /api/tasks/:id/status` to update the task
- TanStack Query optimistic updates via `useMutation` for instant feedback with rollback on failure
- **WIP limits**: Configurable per-column. Column header shows count/limit (e.g., "In Progress 3/5"). At capacity: amber border. Over capacity: red border with subtle pulse animation. Prevents drag-in when over limit (with override option).

#### TaskDetail
```
<TaskDetail>
├── <Header>                         # Title, status badge, actions (edit, archive)
├── <div className="flex">
│   ├── <main>                       # 2/3 width
│   │   ├── <MarkdownRenderer>       # Task body content
│   │   └── <ItemReferences>         # Linked items (clickable [[ID]] links)
│   │
│   └── <aside>                      # 1/3 width — metadata sidebar
│       ├── <StatusBadge>
│       ├── <PriorityIndicator>
│       ├── <EffortBadge>
│       ├── <EpicLink>
│       ├── <PlanLink>
│       ├── <TagList>
│       ├── <AssigneeField>
│       ├── <DependencyList>         # depends_on + blocks
│       └── <DateInfo>               # created, updated
```

#### DependencyGraph
```
<DependencyGraph>
├── <FilterBar>                      # Filter by type, status
├── <ReactFlow>                      # @xyflow/react
│   ├── <ItemNode>*                  # Custom node: shows ID, title, status color
│   ├── <Edge>*                      # Animated edges: blocks (red), depends_on (blue)
│   ├── <MiniMap>                    # Overview minimap
│   └── <Controls>                   # Zoom, fit, fullscreen
```

#### MarkdownRenderer

Renders markdown body content with `[[TASK-042]]` wiki-links transformed into entity-colored `<WikiLinkChip>` components with hover tooltip previews.

```typescript
// components/domain/markdown-renderer.tsx
'use client';

import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';
import { WikiLinkChip } from './wiki-link-chip';

const WIKI_LINK_REGEX = /\[\[([A-Z]+-\d+)\]\]/g;

function resolveRoute(id: string): string {
  const type = id.split('-')[0].toLowerCase();
  return type === 'task' ? 'backlog'
       : type === 'epic' ? 'epics'
       : type === 'plan' ? 'plans'
       : 'notes';
}

export function MarkdownRenderer({ content }: { content: string }) {
  // Pre-process: replace [[ID]] with markdown links tagged as wiki-links
  const processed = content.replace(
    WIKI_LINK_REGEX,
    (_, id) => `[${id}](/${resolveRoute(id)}/${id} "wikilink")`
  );

  return (
    <ReactMarkdown
      remarkPlugins={[remarkGfm]}
      className="prose prose-invert max-w-none"
      components={{
        // Render wiki-links as entity-colored chips with hover preview
        a: ({ href, children, title }) => {
          if (title === 'wikilink' && typeof children === 'string') {
            return <WikiLinkChip id={children} />;
          }
          return <a href={href}>{children}</a>;
        },
      }}
    >
      {processed}
    </ReactMarkdown>
  );
}
```

```typescript
// components/domain/wiki-link-chip.tsx
'use client';

import { useQuery } from '@tanstack/react-query';
import { fetcher } from '@/lib/api';
import Link from 'next/link';
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip';

const ENTITY_COLORS: Record<string, string> = {
  task: 'bg-blue-500/15 text-blue-400 border-blue-500/30',
  epic: 'bg-purple-500/15 text-purple-400 border-purple-500/30',
  plan: 'bg-amber-500/15 text-amber-400 border-amber-500/30',
  note: 'bg-green-500/15 text-green-400 border-green-500/30',
};

export function WikiLinkChip({ id }: { id: string }) {
  const type = id.split('-')[0].toLowerCase();
  const route = type === 'task' ? 'backlog' : type === 'epic' ? 'epics' : type === 'plan' ? 'plans' : 'notes';
  const colorClass = ENTITY_COLORS[type] || ENTITY_COLORS.task;

  // Lazy-fetch item preview on hover
  const { data, refetch } = useQuery({
    queryKey: [type + 's', id, 'preview'],
    queryFn: () => fetcher(`/api/${route === 'backlog' ? 'tasks' : route}/${id}`),
    enabled: false,
    staleTime: 60_000,
  });

  return (
    <Tooltip>
      <TooltipTrigger asChild>
        <Link
          href={`/${route}/${id}`}
          className={`inline-flex items-center px-1.5 py-0.5 rounded border text-xs font-mono font-medium ${colorClass} hover:opacity-80 transition-opacity`}
          onMouseEnter={() => refetch()}
        >
          {id}
        </Link>
      </TooltipTrigger>
      <TooltipContent side="top" className="max-w-xs">
        {data ? (
          <div className="text-sm">
            <p className="font-medium">{data.title}</p>
            <p className="text-muted-foreground text-xs mt-1">{data.status} · {data.priority}</p>
          </div>
        ) : (
          <p className="text-xs text-muted-foreground">Loading...</p>
        )}
      </TooltipContent>
    </Tooltip>
  );
}
```

### 6.3 shadcn/ui Components Used

| Component | Usage |
|-----------|-------|
| `Badge` | Status, priority, effort, tags |
| `Button` | Actions, navigation |
| `Card` | Task cards, metric cards, dashboard panels |
| `Command` | Command palette (Cmd+K) |
| `Dialog` | Create/edit item modals |
| `DropdownMenu` | Context menus, filter dropdowns |
| `Input` | Search, form fields |
| `Select` | Status/priority/effort selectors |
| `Separator` | Visual dividers |
| `Sheet` | Mobile sidebar |
| `Skeleton` | Loading states |
| `Table` | List views, epic task tables |
| `Tabs` | View toggles (kanban/list/table) |
| `Tooltip` | Hover info on badges, icons, and wiki-link chips |
| `Progress` | Epic completion bars |

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
- CLI runs `markplane status TASK-001 done` → file change → UI updates
- MCP server creates a task → file change → UI updates

Debouncing: File watcher events are debounced at 100ms to batch rapid changes (e.g., `markplane sync` writes multiple files).

---

## 9. Key Page Designs

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
│  Search    │  │ ▸ TASK-015 Implement user auth  high │    │
│            │  │ ▸ TASK-017 Add search to...    med  │    │
│            │  │ ▸ TASK-018 Fix pagination      high │    │
│            │  └─────────────────────────────────────┘    │
│            │                                              │
│            │  ┌─────────────────┐ ┌─────────────────┐    │
│            │  │ Blocked Items    │ │ Recent Done      │    │
│            │  │ TASK-012 ←TASK-8│ │ ✓ TASK-014 (2d) │    │
│            │  └─────────────────┘ └─────────────────┘    │
│            │                                              │
│            │  ┌─────────────────────────────────────┐    │
│            │  │ Epic Progress                        │    │
│            │  │ EPIC-001 Core CLI     ████████░░ 80%│    │
│            │  │ EPIC-002 MCP Server   ██████░░░░ 60%│    │
│            │  │ EPIC-003 Web UI       ██░░░░░░░░ 20%│    │
│            │  └─────────────────────────────────────┘    │
│            │                                              │
│            │  ┌─────────────────────────────────────┐    │
│            │  │ AI CONTEXT                    [stale]│    │
│            │  │                                      │    │
│            │  │ .context/summary.md content:          │    │
│            │  │ Project: markplane                    │    │
│            │  │ 15 tasks (3 in-progress, 5 planned)  │    │
│            │  │ 3 active epics                       │    │
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
│            │  │ TASK-015 │   │ TASK-020 │  │ TASK-025 │ │
│            │  │ User auth│   │ CSV exp  │  │ Dark mode│ │
│            │  │ 🔴 high  │   │ 🟡 med   │  │ 🟡 med   │ │
│            │  │ EPIC-001 │   │ EPIC-002 │  │ EPIC-003 │ │
│            │  ├──────────┤   ├──────────┤  ├──────────┤ │
│            │  │ TASK-017 │   │ TASK-021 │  │ TASK-026 │ │
│            │  │ Search   │   │ Form val │  │ Profile  │ │
│            │  │ 🟡 med   │   │ 🟢 low   │  │ 🟢 low   │ │
│            │  └──────────┘   └──────────┘  └──────────┘ │
└─────────────────────────────────────────────────────────┘
```

- Drag between columns updates status via `PATCH /api/tasks/:id/status`
- Filter bar persists state in URL query params for shareability
- List view alternative uses `<DataTable>` with sortable columns

### 9.3 Task Detail (`/backlog/TASK-042`)

```
┌─────────────────────────────────────────────────────────┐
│ [Sidebar]  │  Backlog > TASK-042                         │
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
│            │  │ → EPIC-003           │ │ [EPIC-003] → │  │
│            │  │ ← TASK-038           │ │              │  │
│            │  │ → TASK-045           │ │ Tags         │  │
│            │  │                      │ │ [ui][theming]│  │
│            │  │                      │ │              │  │
│            │  │                      │ │ Depends On   │  │
│            │  │                      │ │ TASK-038 →   │  │
│            │  │                      │ │              │  │
│            │  │                      │ │ Blocks       │  │
│            │  │                      │ │ TASK-045 →   │  │
│            │  └──────────────────────┘ └──────────────┘  │
└─────────────────────────────────────────────────────────┘
```

- Left: `<MarkdownRenderer>` with rendered body content
- Right: Metadata sidebar with editable fields (click status badge → dropdown to change)
- `[[ID]]` references in markdown body render as entity-colored `<WikiLinkChip>` components with hover tooltip previews
- Sidebar fields trigger `PATCH /api/tasks/:id` on change

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

The Rust binary uses `rust-embed` to include the `out/` directory contents:

```rust
#[derive(RustEmbed)]
#[folder = "crates/markplane-web/ui/out"]
struct WebAssets;
```

### 10.3 Runtime: `markplane serve`

```bash
markplane serve              # Start on http://localhost:4200
markplane serve --port 8080  # Custom port
markplane serve --open       # Open browser automatically
```

The Rust server:
1. Serves embedded static HTML/CSS/JS for all frontend routes
2. Handles `/api/*` requests using `markplane-core`
3. Runs WebSocket server on `/ws` for real-time file change notifications
4. Watches `.markplane/` via `notify` crate

### 10.4 Next.js Configuration

```typescript
// next.config.ts
import type { NextConfig } from 'next';

const nextConfig: NextConfig = {
  output: 'export',
  // All routes are client-rendered SPA-style
  // API calls go to the Rust backend
  trailingSlash: true,
  images: {
    unoptimized: true,  // No image optimization server needed
  },
};

export default nextConfig;
```

---

## 11. Performance Considerations

### Code Splitting
- Next.js automatically code-splits by route
- React Flow (dependency graph) is lazy-loaded only on `/graph` route
- `shiki` syntax highlighter loaded on-demand when code blocks are present

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

## 13. Implementation Phases

### Phase 1: Foundation
- Scaffold Next.js project with shadcn/ui + Tailwind v4
- Implement Rust HTTP server with `axum` serving static files + API
- Build core API endpoints: tasks CRUD, epics list, summary
- Implement sidebar layout, routing, theme system
- Dashboard page with summary data

### Phase 2: Core Views
- Backlog kanban board with drag-and-drop
- Task detail page with markdown rendering
- Epic overview with progress bars
- List/table view alternatives

### Phase 3: Advanced Features
- WebSocket file watching + TanStack Query invalidation
- Dependency graph with React Flow
- Full-text search
- Command palette (Cmd+K)
- Roadmap timeline view

### Phase 4: Polish
- Loading skeletons and error states
- Mobile responsive layout
- Keyboard navigation
- Performance optimization
- `rust-embed` integration + `markplane serve` command
