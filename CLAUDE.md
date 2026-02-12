# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Markplane is an AI-native, markdown-first project management system. "Mark" (markdown) + "plane" (control plane). The repo itself is the project manager — no database, no SaaS. Files are the source of truth, git is the changelog.

**Status**: Design phase. The complete specification lives in `docs/ai-native-pm-system-design.md`. No source code has been written yet.

**Planned tech stack**: Rust CLI binary + MCP server wrapping a shared core library. Optional React + Tailwind web UI.

## Architecture

The system is structured around a `.markplane/` directory that lives inside any repository:

```
CLI binary ──→ Core Library (Rust) ←── MCP Server (stdio/SSE)
                     │
              .markplane/ (markdown files)
```

### Key architectural concepts

- **Filesystem as database**: Each work item is an individual markdown file with YAML frontmatter. No SQL database.
- **INDEX.md router pattern**: Every directory has an INDEX.md that serves as a routing layer. AI reads the index (~200 tokens), then loads only relevant files.
- **AI context layer**: `.context/` directory contains generated summaries compressing project state for AI consumption (~1000 tokens for full project state).
- **ID system**: `{PREFIX}-{NUMBER}` — `EPIC-NNN`, `TASK-NNN`, `PLAN-NNN`, `NOTE-NNN`. IDs are permanent, sequential, never reused. Prefix determines directory location.
- **Cross-references**: `[[TASK-042]]` wiki-style syntax. Prefix resolves to type and directory.

### Directory modules (within `.markplane/`)

| Directory | Purpose | Entity prefix |
|-----------|---------|---------------|
| `roadmap/` | Strategic epics & phases | `EPIC` |
| `backlog/` | Work items (primary) | `TASK` |
| `plans/` | Implementation details | `PLAN` |
| `notes/` | Research, ideas, decisions | `NOTE` |
| `templates/` | Document templates | — |
| `.context/` | Generated AI summaries | — |

### Status workflows

- **Tasks**: `draft → backlog → planned → in-progress → done → archive` (also `cancelled`)
- **Epics**: `planned → active → done`
- **Plans**: `draft → approved → in-progress → done → archive`

### Lifecycle flow

```
ideas.md / NOTE-xxx → TASK-xxx → PLAN-xxx → Done → Archive
```

## Build & Development

No build system exists yet. When implemented:

- **Language**: Rust (for CLI speed, single binary distribution)
- **Crates**: `pulldown-cmark` or `comrak` (markdown), `serde_yaml` (YAML), `notify` (file watching)
- **Distribution**: `brew install markplane`, `cargo install markplane`, or as a project dev dependency

### Planned CLI commands

```bash
markplane init                         # Scaffold .markplane/ structure
markplane add "title" --type --priority  # Create task
markplane show TASK-042                # View item
markplane ls --status in-progress      # List/filter items
markplane status TASK-042 in-progress  # Update status
markplane sync                         # Regenerate INDEX.md files + .context/
markplane context --item TASK-042      # Generate focused AI context bundle
markplane check                        # Validate cross-references
markplane promote NOTE-007             # Note → task
markplane plan TASK-042                # Create linked implementation plan
```

### MCP server

The MCP server exposes typed tools (`markplane_summary`, `markplane_query`, `markplane_show`, `markplane_add`, `markplane_update`, `markplane_start`, `markplane_done`, `markplane_sync`, etc.) providing structured API access for AI coding tools. Configured via `~/.claude/mcp.json` or `.cursor/mcp.json`.

## File Conventions

- Individual files stay under ~2,000 tokens (AI-optimized)
- YAML frontmatter for all structured metadata (`id`, `title`, `status`, `priority`, `type`, `effort`, `tags`, `epic`, `depends_on`, `blocks`, etc.)
- Config in YAML (`config.yaml`), not JSON
- Archive subdirectories preserve completed items (not deleted)
- Special non-ID files: `notes/ideas.md` (quick capture), `notes/decisions.md` (decision log)

## Agent Teams

Experimental agent teams are enabled via `.claude/settings.json` (`CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS=1`).

## Project Management
This project uses Markplane for project management. Key files:
- `.markplane/INDEX.md` - Navigation entry point
- `.markplane/.context/summary.md` - Current project state
- `.markplane/backlog/INDEX.md` - All work items
- `.markplane/plans/INDEX.md` - Implementation plans
When working on a task, read the relevant task item and its linked plan first.
