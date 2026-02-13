# Web UI Guide

Markplane includes a local web dashboard for visual project management. It runs entirely on your machine — no external services, no accounts. The Rust binary serves both the API and the static frontend.

## Quick Start

```bash
markplane serve         # Start on http://localhost:4200
markplane serve --open  # Start and open browser automatically
markplane serve --port 8080  # Custom port
```

> **Note**: `markplane serve` needs the web UI frontend to be available. If you built markplane with `--features embed-ui`, the frontend is baked into the binary and this just works. Otherwise, you need to build the frontend first — see [Production Build](#production-build).

## Pages

| Page | URL | Description |
|------|-----|-------------|
| Dashboard | `/dashboard` | Summary metrics, active work, blocked items, epic progress, AI context panel |
| Backlog | `/backlog` | Kanban board, list view, and table view with filtering |
| Epics | `/epics` | All epics with progress bars and status breakdowns |
| Roadmap | `/roadmap` | Epic progress cards grouped by status |
| Plans | `/plans` | Implementation plans with detail view |
| Notes | `/notes` | Research notes, ideas, and decisions |
| Graph | `/graph` | Interactive dependency graph |
| Search | `/search` | Full-text search across all items |

## Views

### Kanban Board (`/backlog`)

The default backlog view shows tasks organized in status columns: In Progress, Planned, Backlog, and Draft.

- **Drag-and-drop**: Drag cards between columns to change status. Updates are optimistic — the UI reflects the change immediately while the file is updated in the background.
- **WIP limits**: The In Progress column has a limit of 5 items. The column header turns red when exceeded.
- **Filters**: Filter by status, priority, epic, tags, and assignee. Active filters are shown as removable pills. Filter state is preserved in the URL.
- **View toggle**: Switch between Kanban, List, and Table views using the toggle in the header. The selection is preserved in the URL (`?view=list`).

### Task Detail

Click any task to open its detail panel. The panel shows:

- **Markdown body** rendered with syntax highlighting and clickable `[[TASK-042]]` wiki-link chips
- **Metadata sidebar** with editable fields: status, priority, effort, type, epic, tags, assignee, dependencies
- Click any metadata field to edit it inline via dropdown

### Dependency Graph (`/graph`)

An interactive node graph built with React Flow showing `blocks` and `depends_on` relationships.

- Nodes are color-coded by entity type and show status
- Pan, zoom, and click nodes to navigate
- Use `?focus=TASK-042` to center the graph on a specific item
- MiniMap in the corner for orientation

### Command Palette (`Cmd+K`)

Press `Cmd+K` (or `Ctrl+K`) to open the command palette. It provides:

- Fuzzy search across all tasks, epics, plans, and notes
- Navigation shortcuts to any page
- Quick actions like triggering a sync

### AI Context Panel

The dashboard includes an AI Context panel showing the `.context/summary.md` content — what AI coding tools see when they read your project. A freshness indicator shows whether the context is up to date, and a "Sync Now" button regenerates it.

## Real-Time Updates

The web UI stays in sync with changes made from any source:

- Edit a `.markplane/` file in your editor
- Run a CLI command (`markplane status TASK-001 done`)
- Use the MCP server from an AI coding tool

All changes are detected via filesystem watching and pushed to the browser over WebSocket within ~100ms. No manual refresh needed.

## Keyboard Shortcuts

### Navigation (press `g` then a letter)

| Shortcut | Action |
|----------|--------|
| `g` then `d` | Go to Dashboard |
| `g` then `b` | Go to Backlog |
| `g` then `e` | Go to Epics |
| `g` then `r` | Go to Roadmap |
| `g` then `p` | Go to Plans |
| `g` then `n` | Go to Notes |
| `g` then `g` | Go to Graph |
| `g` then `s` | Go to Search |

### Global

| Shortcut | Action |
|----------|--------|
| `Cmd+K` / `Ctrl+K` | Open command palette |
| `?` | Open command palette |
| `Escape` | Close panel/modal |

## Themes

Dark mode is the default. Toggle between dark, light, and system-aware modes using the theme switch in the sidebar footer.

The color system uses OKLCH for perceptually uniform colors. Status colors (blue for in-progress, green for done, amber for blocked) and priority colors (red for critical, orange for high, yellow for medium) are consistent across all views.

## Development Workflow

This section is for contributors working on the web UI frontend code itself. If you just want to use the web UI, see [Quick Start](#quick-start) and [Production Build](#production-build).

You need two terminals — one for the Rust API server, one for the Next.js dev server with hot reload:

```bash
# Terminal 1: Rust API server (from repo root)
cargo run -p markplane-cli -- serve --dev

# Terminal 2: Next.js dev server (from repo root)
cd crates/markplane-web/ui
npm install   # first time only
npm run dev
```

In dev mode (`--dev`), the Rust server only serves the API (no static files). The Next.js dev server runs on port 3000 and proxies `/api/*` and `/ws` requests to the Rust server on port 4200. You get hot reload for frontend changes this way.

## Production Build

The web UI is a static site (plain HTML/CSS/JS — no Node.js needed at runtime). You build it once with npm, then the Rust binary serves it.

### Option 1: Single binary with embedded UI (recommended)

This bakes the frontend into the `markplane` binary so there's nothing else to deploy:

```bash
# From the repo root:

# 1. Build the frontend
cd crates/markplane-web/ui
npm install
npm run build
cd ../../..    # back to repo root

# 2. Install markplane with the embedded UI
cargo install --path crates/markplane-cli --features embed-ui
```

The resulting `markplane` binary contains the full web UI. Run `markplane serve` from any project and it just works.

### Option 2: Separate frontend files

If you build without `--features embed-ui`, `markplane serve` looks for the pre-built frontend files at `crates/markplane-web/ui/out/` relative to where the repo is checked out. This is useful during development but means the binary isn't self-contained.

## Architecture

```
Browser (Next.js static export)
    |
    |-- REST API --> Rust HTTP server (axum)
    |                    |
    |                    +-- markplane-core (shared library)
    |                    |
    |                    +-- .markplane/ (filesystem)
    |
    +-- WebSocket <---> File watcher (notify crate)
```

- **Frontend**: Next.js 16 with Tailwind v4 and shadcn/ui, statically exported
- **Backend**: axum HTTP server with 16 REST endpoints + WebSocket
- **State management**: TanStack Query for server state, URL params for view state
- **File watching**: `notify` crate with 100ms debouncing via `notify-debouncer-mini`

### REST API

All responses follow the envelope format:

```
Success (single):  { "data": T }
Success (list):    { "data": T[], "meta": { "total": number } }
Error:             { "error": { "code": string, "message": string } }
```

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/summary` | Dashboard data: counts, active epics, in-progress/blocked tasks |
| GET | `/api/tasks` | List tasks with filters: status, priority, epic, tags, assignee, type |
| GET | `/api/tasks/:id` | Task detail with markdown body |
| POST | `/api/tasks` | Create task |
| PATCH | `/api/tasks/:id` | Update task fields |
| DELETE | `/api/tasks/:id` | Archive task |
| GET | `/api/epics` | List epics with progress and status breakdown |
| GET | `/api/epics/:id` | Epic detail with linked tasks |
| GET | `/api/plans` | List plans |
| GET | `/api/plans/:id` | Plan detail |
| GET | `/api/notes` | List notes |
| GET | `/api/notes/:id` | Note detail |
| POST | `/api/sync` | Trigger markplane sync |
| GET | `/api/search?q=...` | Full-text search |
| GET | `/api/graph` | Full dependency graph |
| GET | `/api/graph/:id` | Focused graph (2-hop neighborhood) |

### WebSocket Events

Connect to `ws://localhost:4200/ws` for real-time updates:

```json
{ "type": "file_changed", "entity": "task", "id": "TASK-001", "action": "modified" }
{ "type": "config_changed" }
{ "type": "sync_complete" }
{ "type": "connected", "version": "0.1.0" }
```

### Project Structure

```
crates/markplane-web/ui/          # Next.js project
  src/
    app/                           # Pages (App Router)
    components/
      ui/                          # shadcn/ui primitives
      domain/                      # Markplane-specific components
      layout/                      # App shell (sidebar, providers)
    lib/
      api.ts                       # API client
      types.ts                     # TypeScript types mirroring Rust
      constants.ts                 # Status/priority config
      hooks/                       # TanStack Query hooks

crates/markplane-cli/src/commands/
  serve.rs                         # HTTP server + API + WebSocket
```
