# Getting Started with Markplane

This guide walks you through setting up Markplane and managing your project using the web UI, AI assistants, and CLI.

## Prerequisites

See the [Markplane README](https://github.com/zerowand01/markplane#installation) for installation instructions. Verify the installation by running:

```bash
markplane --version
```

## 1. Initialize Your Project

Navigate to the root of your repository and run:

```bash
markplane init --name "My App"
```

Output:

```
Initialized Markplane project: My App

  Seeded with starter content (1 epic, 2 tasks, 1 plan, 1 note)

  .markplane/
  ├── config.yaml
  ├── INDEX.md
  ├── roadmap/          (EPIC-xxxxx)
  ├── backlog/          (TASK-xxxxx)
  ├── plans/            (PLAN-xxxxx)
  ├── notes/            (NOTE-xxxxx)
  ├── templates/
  └── .context/

Next steps:
  markplane ls                  # See your starter tasks
  markplane show TASK-xxxxx   # Review your setup checklist
  markplane dashboard           # Project overview
```

This creates the `.markplane/` directory structure with config, templates, index files, and special note files (`ideas.md`, `decisions.md`). By default it also seeds the project with starter content — an onboarding epic, setup tasks, a migration plan, and a decision log note — that demonstrates correct format and gives you something to work with immediately. Use `markplane init --empty` to skip starter content.

### How Items Relate

Markplane organizes work into four item types:

- **Epics** — Strategic goals or phases that group related tasks (*the why*).
- **Tasks** — Your primary work units (bugs, features, chores, spikes, etc.). Tasks make up your backlog and move through status columns on the kanban board (*the what*).
- **Plans** — An optional tool for planning out complex tasks, multi-task implementations, or standalone planning — however your workflow needs them (*the how*).
- **Notes** — Research, ideas, analysis, decisions, etc. that don't need to be tracked as work. Notes can optionally be promoted to tasks if they become actionable.

## 2. Start the Web UI

Launch the local web dashboard to see your project:

```bash
markplane serve --open
```

This opens `http://localhost:4200` in your browser. You'll see the seeded starter content on the dashboard — an epic, tasks, a plan, and a note.

The web UI provides:

- **Dashboard** — Project overview with summary metrics, active work, blocked items, and epic progress
- **Backlog** — Kanban board with drag-and-drop between status columns, plus list and table views. Filter by priority, type, tags, epic, and assignee
- **Roadmap** — Epics with progress tracking and linked tasks
- **Plans** — Implementation plans with status tracking
- **Notes** — Free-form notes for research, ideas, decisions, or anything else
- **Graph** — Interactive visual graph of your project's items and their relationships
- **Archive** — Browse and restore completed items
- **Search** — Full-text search across all items with `Cmd+K` command palette
- **Settings** — Configure task types, note types, status workflows, and documentation paths
- **Real-time updates** — Changes from CLI, MCP, or file edits appear instantly

Explore the seeded content to understand the structure — drag tasks between kanban columns, click items to view and edit details, and check the graph view.

See the [Web UI Guide](web-ui-guide.md) for keyboard shortcuts, views, and more.

## 3. Connect Your AI Assistant

Markplane includes a built-in MCP server that lets AI coding assistants manage your project directly. Connect your assistant, then just tell it what to do in natural language.

You can configure MCP per-user (your local editor) or project-wide (shared with your team). Example with Claude Code:

**Per-user** — adds Markplane to your local editor configuration:

```bash
claude mcp add --transport stdio markplane -- markplane mcp
```

**Project-wide** — add a `.mcp.json` file at the repo root so every team member gets the integration automatically:

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

Once connected, your AI assistant can create items, update statuses, set up dependencies, query your backlog, and more — using natural language. For example:

- "Create a high-priority feature task for user authentication, linked to EPIC-xa7r2"
- "Show me what's currently blocked"
- "Mark TASK-fq2x8 as done"
- "Create a plan for the login page task"
- "What tasks are in progress?"

See the [MCP Setup Guide](mcp-setup.md) for other editors, configuration options, and the full tool and resource catalog.

### How AI works with Markplane

Because every item is a plain markdown file, your AI assistant has two ways to interact with your project:

- **MCP tools** for structural operations — creating items, updating status/priority/tags, linking dependencies, querying the backlog, and syncing. These ensure frontmatter stays valid and relationships are maintained correctly.
- **Direct file editing** for content — writing descriptions, acceptance criteria, plan details, and notes. The AI can read and edit `.markplane/` files just like any other file in your repo.

This is by design. Use MCP tools (or the CLI) for metadata and structure; edit the markdown body directly for content.

### CLAUDE.md

If you use Claude Code, Markplane has a built-in command to populate your `CLAUDE.md` with project context:

```bash
markplane claude-md
```

This tells Claude where to find project state and how to use Markplane.

## 4. Working with Markplane

With the web UI running and your AI connected, here's how the pieces work together.

### Creating items

You can create items from any interface:

- **AI**: "Create a bug task for the broken login redirect, priority critical, tag it auth"
- **Web UI**: Use the "+" button on the kanban board or any list view
- **CLI**: `markplane add "Fix login redirect" --type bug --priority critical --tags auth`

Epics, notes, and plans:

- **AI**: "Create an epic for the authentication system" / "Add a research note about OAuth providers" / "Create a plan for TASK-fq2x8"
- **Web UI**: Use the "+" button on the respective page
- **CLI**: `markplane epic "Auth System" --priority high` / `markplane note "OAuth research" --type research` / `markplane plan TASK-fq2x8`

### Available options

- **Task types**: `feature`, `bug`, `enhancement`, `chore`, `research`, `spike` (configurable in `config.yaml` or web UI Settings)
- **Priorities**: `critical`, `high`, `medium`, `low`, `someday`
- **Effort sizes**: `xs`, `small`, `medium`, `large`, `xl`
- **Note types**: `research`, `analysis`, `idea`, `decision`, `meeting` (configurable)

### Managing workflow

Tasks move through configurable statuses: `draft` → `backlog` → `planned` → `in-progress` → `done` (also `cancelled`). Epics use: `later` → `next` → `now` → `done`.

- **AI**: "Start working on TASK-fq2x8" / "Move the auth epic to now"
- **Web UI**: Drag cards between kanban columns, or edit status inline in detail views
- **CLI**: `markplane start TASK-fq2x8` / `markplane done TASK-fq2x8` / `markplane status EPIC-xa7r2 now`

### Dependencies and relationships

Link items that depend on each other. Both directions are automatically maintained — adding a `depends-on` link also adds the reverse `blocks` link.

- **AI**: "TASK-sv8r2 depends on TASK-d4p7m" / "Link TASK-fq2x8 to NOTE-vt3k8 as related"
- **Web UI**: Add dependencies in the item detail view
- **CLI**: `markplane link TASK-sv8r2 TASK-d4p7m -r depends-on`

Relation types: `blocks`, `depends-on`, `epic`, `plan`, `implements`, `related`.

### Archiving

Move completed items out of the active views:

- **AI**: "Archive TASK-fq2x8" / "Archive all done tasks"
- **Web UI**: Archive items individually or in bulk, or browse and restore archived items on the Archive page
- **CLI**: `markplane archive TASK-fq2x8` / `markplane archive --all-done`

Archived items are preserved (not deleted) and can be restored from the AI, web UI, or CLI (`markplane unarchive`).

### Keeping things in sync

Markplane auto-syncs index files and AI context summaries on `init`, `mcp` startup, and `serve` startup. You can also sync manually:

```bash
markplane sync       # Regenerate INDEX.md + .context/
markplane check      # Validate cross-references
```

### Connect project documentation

Bridge your repo's `docs/` directory with Markplane so AI agents and navigation can discover both project management and technical docs. Configure via the web UI Settings page or directly in `config.yaml`:

```yaml
# .markplane/config.yaml
documentation_paths:
  - docs
```

When `markplane sync` runs, it adds links to your docs in the root `INDEX.md` and `.context/summary.md`.

## CLI Reference

The CLI is a power-user interface for everything Markplane can do. A few commonly used commands:

```bash
markplane ls                          # List tasks
markplane ls epics                    # List epics
markplane ls --priority high,critical # Filter by priority
markplane ls --tags auth              # Filter by tag
markplane show TASK-fq2x8            # View item details
markplane update TASK-fq2x8 --assignee @daniel --add-tag "sprint-3"
markplane dashboard                   # Project overview in terminal
markplane metrics                     # Status and priority breakdowns
markplane graph TASK-sv8r2            # Dependency graph for an item
markplane stale --days 14             # Find items not updated recently
```

See the [CLI Reference](cli-reference.md) for complete command documentation.

## Next Steps

- Explore the seeded starter content to understand the structure
- Add `.markplane/` to version control to share project state with your team
- Customize task types, note types, and status workflows in `config.yaml` or web UI Settings
- Explore the `.markplane/templates/` directory to customize document templates
