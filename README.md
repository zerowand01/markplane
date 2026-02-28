# Markplane

AI-native, markdown-first project management. Your repo is the project manager.

Markplane stores all project management data as markdown files with YAML frontmatter inside a `.markplane/` directory. No database, no SaaS — files are the source of truth, git is the changelog.

## Features

- **Filesystem as database** — Every work item is an individual markdown file. Browse, edit, and grep your project data with standard tools.
- **Markdown + YAML frontmatter** — Structured metadata (status, priority, effort, tags) in YAML; free-form details in markdown.
- **AI-optimized context layer** — Generated `.context/` summaries compress full project state into ~1000 tokens for AI consumption.
- **Cross-references** — `[[TASK-rm6d3]]` wiki-style links between items, with validation via `markplane check`.
- **INDEX.md router pattern** — Each directory has an INDEX.md that lets AI agents load only what they need (~200 tokens per index).
- **Web UI** — Local dashboard with kanban board, dependency graph, markdown rendering, real-time updates, and dark/light themes via `markplane serve`.
- **MCP server** — Structured tool access for AI coding assistants (Claude, Cursor, etc.) via JSON-RPC over stdio.
- **Dependency tracking** — `blocks` / `depends_on` relationships and bidirectional `related` links with visual dependency graphs.
- **Archive management** — Archive completed items across all entity types with easy restore.
- **Built-in workflows** — Status progressions for tasks, epics, plans, and notes.

## Installation

```bash
cargo install markplane
```

> **Note**: Not yet published to crates.io. For now, clone and build from source:
>
> ```bash
> git clone https://github.com/your-org/markplane.git
> cd markplane
> cargo install --path crates/markplane-cli
> ```

To include the web UI (`markplane serve`), build the frontend first, then install with `embed-ui`:

```bash
cd crates/markplane-web/ui && npm install && npm run build && cd ../../..
cargo install --path crates/markplane-cli --features embed-ui
```

Requires Rust 1.93.0 or later. The web UI build also requires Node.js 18+.

## Quick Start

```bash
# Initialize in your project
markplane init --name "My Project"

# Create work items
markplane epic "Phase 1: Core Features" --priority high
markplane add "Implement user auth" --type feature --priority high --epic EPIC-xa7r2
markplane add "Fix login redirect" --type bug --priority critical --tags "auth,urgent"

# View and manage
markplane ls                            # List tasks
markplane ls epics                      # List epics
markplane show TASK-fq2x8                 # View item details
markplane start TASK-fq2x8                # Set to in-progress + assign to you
markplane done TASK-fq2x8                 # Mark as done

# Keep things organized
markplane sync                          # Regenerate INDEX.md + .context/
markplane check                         # Validate cross-references
markplane dashboard                     # Project overview
markplane metrics                       # Statistics and progress
```

## Project Structure

After `markplane init`, your repo gets a `.markplane/` directory:

```
.markplane/
├── config.yaml           # Project settings
├── INDEX.md              # Root navigation
├── roadmap/              # Epics — strategic goals and phases (EPIC-xxxxx)
├── backlog/              # Tasks — the "what" to do (TASK-xxxxx)
├── plans/                # Plans — the "how" to do it (PLAN-xxxxx)
├── notes/                # Notes — research, ideas, decisions (NOTE-xxxxx)
├── templates/            # Document templates
└── .context/             # AI-generated summaries
```

## Status Workflows

| Type | Statuses | Configurable? |
|------|----------|---------------|
| Task | `draft` → `backlog` → `planned` → `in-progress` → `done` (also `cancelled`) | Yes — via `config.yaml` `workflows.task` |
| Epic | `later` → `next` → `now` → `done` | No |
| Plan | `draft` → `approved` → `in-progress` → `done` | No |
| Note | `draft` → `active` → `archived` | No |

Task statuses are fully configurable. Each status maps to one of six **status categories** (`draft`, `backlog`, `planned`, `active`, `completed`, `cancelled`) that control system behavior (kanban columns, progress tracking, archive eligibility). Add custom statuses like `in-review`, `in-qa`, or `deployed` by placing them under the appropriate category in `config.yaml`.

## Web UI

Markplane includes a local web dashboard for visual project management.

```bash
markplane serve         # Start on http://localhost:4200
markplane serve --open  # Start and open browser
```

The web UI provides:

- **Dashboard** — Summary metrics, active work, blocked items, epic progress, AI context panel
- **Kanban board** — Drag-and-drop task management with status columns, filters, and WIP limits
- **List and table views** — Sortable, filterable alternatives to the kanban
- **Task and epic detail** — Markdown rendering with clickable `[[TASK-rm6d3]]` wiki-links, inline status/priority editing
- **Dependency graph** — Interactive node graph (React Flow) showing blocks/depends_on relationships
- **Command palette** — `Cmd+K` to search and navigate anywhere
- **Full-text search** — Search across all items with highlighted matches
- **Real-time updates** — Changes from CLI, MCP, or file edits appear instantly via WebSocket
- **Dark/light themes** — Dark-first design with system-aware theme switching

See the [Web UI Guide](docs/web-ui-guide.md) for development workflow, keyboard shortcuts, and architecture details.

## Documentation

- [Getting Started Guide](docs/getting-started.md) — Step-by-step tutorial
- [CLI Reference](docs/cli-reference.md) — Complete command documentation
- [Web UI Guide](docs/web-ui-guide.md) — Web dashboard usage and development
- [MCP Setup Guide](docs/mcp-setup.md) — AI tool integration
- [Architecture](docs/architecture.md) — System design and crate structure

## MCP Integration

Markplane includes an MCP server for AI tool integration.

**Claude Code** (recommended — uses the `claude mcp add` command):

```bash
claude mcp add --transport stdio markplane -- markplane mcp
```

**Project-wide** (shared with your team via `.mcp.json` at repo root):

```json
{
  "mcpServers": {
    "markplane": {
      "command": "markplane",
      "args": ["mcp"]
    }
  }
}
```

See the [MCP Setup Guide](docs/mcp-setup.md) for scopes, configuration options, and the full tool and resource catalog.

## License

Apache-2.0. See [LICENSE](LICENSE) for details.
