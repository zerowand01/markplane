# Markplane

AI-native, markdown-first project management. Your repo is the project manager.

Markplane stores all project management data as markdown files with YAML frontmatter inside a `.markplane/` directory. No database, no SaaS — files are the source of truth, git is the changelog.

## Features

- **Filesystem as database** — Every work item is an individual markdown file. Browse, edit, and grep your project data with standard tools.
- **Markdown + YAML frontmatter** — Structured metadata (status, priority, effort, tags) in YAML; free-form details in markdown.
- **AI-optimized context layer** — Generated `.context/` summaries compress full project state into ~1000 tokens for AI consumption.
- **Cross-references** — `[[BACK-042]]` wiki-style links between items, with validation via `markplane check`.
- **INDEX.md router pattern** — Each directory has an INDEX.md that lets AI agents load only what they need (~200 tokens per index).
- **MCP server** — Structured tool access for AI coding assistants (Claude, Cursor, etc.) via JSON-RPC over stdio.
- **Dependency tracking** — `blocks` / `depends_on` relationships with visual dependency graphs.
- **Built-in workflows** — Status progressions for backlog items, epics, plans, and notes.

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

Requires Rust 1.93.0 or later.

## Quick Start

```bash
# Initialize in your project
markplane init --name "My Project"

# Create work items
markplane epic "Phase 1: Core Features" --priority high
markplane add "Implement user auth" --type feature --priority high --epic EPIC-001
markplane add "Fix login redirect" --type bug --priority critical --tags "auth,urgent"

# View and manage
markplane ls                            # List backlog items
markplane ls epics                      # List epics
markplane show BACK-001                 # View item details
markplane start BACK-001                # Set to in-progress + assign to you
markplane done BACK-001                 # Mark as done

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
├── config.yaml           # Project settings and ID counters
├── INDEX.md              # Root navigation
├── roadmap/              # Epics (EPIC-NNN)
├── backlog/              # Work items (BACK-NNN)
├── plans/                # Implementation plans (PLAN-NNN)
├── notes/                # Research, ideas, decisions (NOTE-NNN)
├── kb/                   # Knowledge base
├── templates/            # Document templates
└── .context/             # AI-generated summaries
```

## Status Workflows

| Type | Statuses |
|------|----------|
| Backlog | `draft` → `backlog` → `planned` → `in-progress` → `done` (also `cancelled`) |
| Epic | `planned` → `active` → `done` (also `paused`) |
| Plan | `draft` → `approved` → `in-progress` → `done` |
| Note | `draft` → `active` → `archived` |

## Documentation

- [Getting Started Guide](docs/getting-started.md) — Step-by-step tutorial
- [CLI Reference](docs/cli-reference.md) — Complete command documentation
- [Design Specification](docs/ai-native-pm-system-design.md) — Full architecture and design

## MCP Integration

Markplane includes an MCP server for AI tool integration. Configure it in your AI tool's MCP settings:

```json
{
  "mcpServers": {
    "markplane": {
      "command": "markplane-mcp",
      "args": []
    }
  }
}
```

The MCP server exposes 15 tools and 5 resources for reading and managing project data programmatically.

## License

Apache-2.0. See [LICENSE](LICENSE) for details.
