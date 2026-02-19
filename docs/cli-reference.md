# CLI Reference

Complete reference for every `markplane` command.

## Global

```
markplane [COMMAND] [OPTIONS]
markplane --version
markplane --help
```

---

## add

Create a new task.

```
markplane add <TITLE> [OPTIONS]
```

**Arguments:**

| Argument | Description |
|----------|-------------|
| `<TITLE>` | Title of the item (required) |

**Options:**

| Option | Default | Description |
|--------|---------|-------------|
| `--type <TYPE>` | `feature` | Item type: `feature`, `bug`, `enhancement`, `chore`, `research`, `spike` |
| `--priority <PRIORITY>` | `medium` | Priority: `critical`, `high`, `medium`, `low`, `someday` |
| `--effort <EFFORT>` | `medium` | Effort estimate: `xs`, `small`, `medium`, `large`, `xl` |
| `--epic <EPIC_ID>` | — | Parent epic ID (e.g. `EPIC-001`) |
| `--tags <TAGS>` | — | Comma-separated tags (e.g. `auth,backend`) |

New items are created with status `draft`.

**Example:**

```bash
markplane add "Implement dark mode" --type feature --priority high --effort large --epic EPIC-003 --tags "ui,theming"
# Created TASK-001 — Implement dark mode
```

---

## archive

Move done/cancelled items to archive subdirectories.

```
markplane archive [OPTIONS]
```

**Options:**

| Option | Default | Description |
|--------|---------|-------------|
| `--dry-run` | `false` | Preview what would be archived without making changes |

Items are eligible for archiving when their status is `done` (or `cancelled` if `keep_cancelled` is `false` in config) and they haven't been updated within the `auto_archive_after_days` threshold (default: 30 days).

**Example:**

```bash
markplane archive --dry-run
# → Would archive 2 item(s):
#   TASK-001 Implement login page (done)
#   TASK-005 Remove deprecated API (done)

markplane archive
# ✓ Archived 2 item(s).
```

---

## assign

Assign a task to a user.

```
markplane assign <ID> <USER>
```

**Arguments:**

| Argument | Description |
|----------|-------------|
| `<ID>` | Task ID (e.g. `TASK-042`) |
| `<USER>` | Username to assign (leading `@` is stripped automatically) |

Currently only supported for tasks (`TASK-NNN`).

**Example:**

```bash
markplane assign TASK-001 @daniel
# TASK-001 assigned to daniel
```

---

## check

Validate cross-references and find broken links.

```
markplane check [OPTIONS]
```

**Options:**

| Option | Default | Description |
|--------|---------|-------------|
| `--orphans` | `false` | Also show orphan items (items with no incoming references) |

Scans all markdown files for `[[ITEM-NNN]]` references and verifies that each referenced item exists. Exits with a non-zero status if broken references are found.

**Example:**

```bash
markplane check
# ✓ No broken references found.

markplane check --orphans
# ✓ No broken references found.
# ! 1 orphan item(s) (no incoming references):
#   TASK-007
```

---

## claude-md

Output a CLAUDE.md integration snippet for AI coding tools.

```
markplane claude-md
```

Prints a markdown snippet pointing AI assistants to relevant Markplane files. Add the output to your project's `CLAUDE.md`.

---

## context

Regenerate `.context/` files or generate focused context for a specific item.

```
markplane context [OPTIONS]
```

**Options:**

| Option | Default | Description |
|--------|---------|-------------|
| `--item <ID>` | — | Generate focused context for a specific item |
| `--focus <TAG>` | — | Generate context focused on a domain/tag |

When `--item` is specified, prints the item along with its linked epic, plan, and dependencies. Otherwise, regenerates all `.context/` summary files.

**Example:**

```bash
markplane context
# ✓ Context files regenerated in .context/

markplane context --item TASK-001
# Prints item details, linked epic, plan, and dependencies
```

---

## dashboard

Show project dashboard overview.

```
markplane dashboard
```

Displays:
- Items currently in progress (with assignees)
- Blocked items and what blocks them
- Active epics with completion percentages
- Summary counts (open, in-progress, blocked, critical)

**Example:**

```bash
markplane dashboard
# ✈  My App — Project Dashboard
# ══════════════════════════════════════════════════════
#
# In Progress
#   TASK-002 Set up JWT handling (high @alice)
#
# 3 open items | 1 in-progress | 0 blocked | 1 critical
```

---

## done

Mark an item as done.

```
markplane done <ID>
```

**Arguments:**

| Argument | Description |
|----------|-------------|
| `<ID>` | Item ID (e.g. `TASK-042`, `PLAN-001`) |

Works with tasks, epics, plans, and notes. Sets the status to `done`.

**Example:**

```bash
markplane done TASK-001
# TASK-001 → done
```

---

## epic

Create a new epic.

```
markplane epic <TITLE> [OPTIONS]
```

**Arguments:**

| Argument | Description |
|----------|-------------|
| `<TITLE>` | Epic title (required) |

**Options:**

| Option | Default | Description |
|--------|---------|-------------|
| `--priority <PRIORITY>` | `medium` | Priority: `critical`, `high`, `medium`, `low`, `someday` |

New epics are created with status `planned`.

**Example:**

```bash
markplane epic "Phase 2: API Layer" --priority high
# Created EPIC-002 — Phase 2: API Layer
```

---

## graph

Show dependency graph for an item.

```
markplane graph <ID> [OPTIONS]
```

**Arguments:**

| Argument | Description |
|----------|-------------|
| `<ID>` | Item ID (e.g. `TASK-042`) |

**Options:**

| Option | Default | Description |
|--------|---------|-------------|
| `--depth <DEPTH>` | `3` | Maximum depth to traverse |

Displays outgoing references (dependencies) as an indented tree and incoming references (what references this item). Circular references are detected and marked with `↻`.

**Example:**

```bash
markplane graph TASK-004 --depth 2
# Dependency graph for TASK-004
#
# TASK-004
#   └─ TASK-002
#     └─ TASK-001
#
# Referenced by:
#   TASK-006 → TASK-004
```

---

## init

Initialize a new `.markplane/` structure in the current directory.

```
markplane init [OPTIONS]
```

**Options:**

| Option | Default | Description |
|--------|---------|-------------|
| `--name <NAME>` | Current directory name | Project name |
| `--description <DESC>` | `""` | Project description |

Creates the full directory structure, `config.yaml`, INDEX.md files, templates, and special note files (`ideas.md`, `decisions.md`). Fails if `.markplane/` already exists.

**Example:**

```bash
markplane init --name "My App" --description "A web application"
# Initialized Markplane project: My App
```

---

## link

Add a dependency link between tasks.

```
markplane link <ID> [OPTIONS]
```

**Arguments:**

| Argument | Description |
|----------|-------------|
| `<ID>` | Source task ID |

**Options:**

| Option | Description |
|--------|-------------|
| `--blocks <TARGET_ID>` | Target item that the source blocks |
| `--depends-on <TARGET_ID>` | Target item that the source depends on |

At least one of `--blocks` or `--depends-on` must be specified. Both can be used together. Reverse links are automatically maintained on the target item.

Currently only supported for tasks (`TASK-NNN`).

**Example:**

```bash
markplane link TASK-003 --depends-on TASK-001
# TASK-003 depends on TASK-001

markplane link TASK-002 --blocks TASK-005
# TASK-002 blocks TASK-005
```

---

## ls

List items with optional filtering.

```
markplane ls [KIND] [OPTIONS]
```

**Subcommands:**

| Kind | Description |
|------|-------------|
| *(none)* | List tasks (default) |
| `epics` | List epics |
| `plans` | List plans |
| `notes` | List notes |

**Options (tasks only):**

| Option | Description |
|--------|-------------|
| `--status <VALUES>` | Filter by status (comma-separated) |
| `--priority <VALUES>` | Filter by priority (comma-separated) |
| `--epic <EPIC_ID>` | Filter by epic ID |
| `--tags <VALUES>` | Filter by tags (comma-separated, items must have at least one) |
| `--assignee <USER>` | Filter by assignee |
| `--type <VALUES>` | Filter by item type (comma-separated) |

**Examples:**

```bash
markplane ls
markplane ls --status in-progress,planned
markplane ls --priority critical,high --tags auth
markplane ls --epic EPIC-001
markplane ls epics
markplane ls plans
markplane ls notes
```

---

## metrics

Show project metrics and statistics.

```
markplane metrics
```

Displays:
- Backlog status distribution (total, in-progress, planned, backlog, draft, done, cancelled)
- Priority distribution for open items
- Epic progress with visual progress bars
- Plan counts (active vs completed)

---

## mcp

Run the MCP (Model Context Protocol) server over stdio.

```
markplane mcp [OPTIONS]
```

**Options:**

| Option | Default | Description |
|--------|---------|-------------|
| `--project <PATH>` | Current directory | Path to the project directory containing `.markplane/` |

Starts a JSON-RPC 2.0 server that reads requests from stdin and writes responses to stdout. Diagnostic messages go to stderr. This is intended to be launched by AI coding tools (Claude Code, Cursor, etc.) — not run interactively.

See the [MCP Setup Guide](mcp-setup.md) for configuration details including tool and resource catalogs.

**Example:**

```bash
# Smoke test
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}' | markplane mcp

# Configure for Claude Code
claude mcp add --transport stdio markplane -- markplane mcp
```

---

## note

Create a new note.

```
markplane note <TITLE> [OPTIONS]
```

**Arguments:**

| Argument | Description |
|----------|-------------|
| `<TITLE>` | Note title (required) |

**Options:**

| Option | Default | Description |
|--------|---------|-------------|
| `--type <TYPE>` | `idea` | Note type: `research`, `analysis`, `idea`, `decision`, `meeting` |
| `--tags <TAGS>` | — | Comma-separated tags |

New notes are created with status `draft`.

**Example:**

```bash
markplane note "Evaluate caching strategies" --type research --tags "performance,cache"
# Created NOTE-001 — Evaluate caching strategies
```

---

## plan

Create a linked implementation plan for a task.

```
markplane plan <ID> [OPTIONS]
```

**Arguments:**

| Argument | Description |
|----------|-------------|
| `<ID>` | Task ID (e.g. `TASK-042`) |

**Options:**

| Option | Default | Description |
|--------|---------|-------------|
| `--title <TITLE>` | `"Implementation plan for <item title>"` | Custom plan title |

Creates a plan that references the task via `implements` and links the plan back to the item's `plan` field. Inherits the item's epic.

Only works with tasks (`TASK-NNN`).

**Example:**

```bash
markplane plan TASK-001
# Created PLAN-001 — Implementation plan for Implement login page
# Linked to TASK-001

markplane plan TASK-002 --title "JWT architecture design"
# Created PLAN-002 — JWT architecture design
# Linked to TASK-002
```

---

## promote

Promote a note to a task.

```
markplane promote <ID> [OPTIONS]
```

**Arguments:**

| Argument | Description |
|----------|-------------|
| `<ID>` | Note ID (e.g. `NOTE-007`) |

**Options:**

| Option | Default | Description |
|--------|---------|-------------|
| `--priority <PRIORITY>` | `medium` | Priority for the new task |
| `--effort <EFFORT>` | `medium` | Effort estimate for the new task |

Creates a new task with the note's title and tags, using item type `feature`. Only works with notes (`NOTE-NNN`).

**Example:**

```bash
markplane promote NOTE-001 --priority high --effort small
# Promoted NOTE-001 → TASK-003 — Evaluate caching strategies
```

---

## serve

Start the web UI server.

```
markplane serve [OPTIONS]
```

**Options:**

| Option | Default | Description |
|--------|---------|-------------|
| `--port <PORT>` | `4200` | Port to listen on |
| `--open` | `false` | Open browser automatically after starting |
| `--dev` | `false` | Dev mode: API only, no static file serving (use with Next.js dev server) |

Starts an HTTP server that serves the web dashboard and a REST API backed by `markplane-core`. Also runs a WebSocket server at `/ws` that broadcasts real-time file change events.

In production builds compiled with `--features embed-ui`, the web UI is embedded in the binary. Otherwise, it serves static files from `crates/markplane-web/ui/out/`.

**Example:**

```bash
markplane serve
# Markplane web UI starting on http://localhost:4200

markplane serve --port 8080 --open
# Starts on port 8080 and opens the browser

markplane serve --dev
# API only on :4200 — run `npm run dev` in ui/ for the frontend
```

---

## show

Show details of an item.

```
markplane show <ID>
```

**Arguments:**

| Argument | Description |
|----------|-------------|
| `<ID>` | Item ID (e.g. `TASK-042`, `EPIC-001`, `PLAN-003`, `NOTE-007`) |

Displays all metadata fields and the markdown body. Works with all item types. Also finds archived items.

---

## stale

List items not updated within a given number of days.

```
markplane stale [OPTIONS]
```

**Options:**

| Option | Default | Description |
|--------|---------|-------------|
| `--days <DAYS>` | `30` | Number of days threshold |

Only shows items that are not `done` or `cancelled`.

**Example:**

```bash
markplane stale --days 14
# ! 2 item(s) not updated in 14 days:
#
#  ID       | Title            | Status  | Last Updated | Days Stale
#  TASK-003 | Fix password ... | draft   | 2026-01-20   | 20
```

---

## start

Start working on an item. Sets status to `in-progress` and assigns to you.

```
markplane start <ID> [OPTIONS]
```

**Arguments:**

| Argument | Description |
|----------|-------------|
| `<ID>` | Item ID (e.g. `TASK-042`) |

**Options:**

| Option | Default | Description |
|--------|---------|-------------|
| `--user <USER>` | `$USER` env var (or `"me"`) | Assignee name |

For tasks, sets both status and assignee. For other item types, only updates status.

**Example:**

```bash
markplane start TASK-001
# TASK-001 → in-progress (assigned to daniel)

markplane start TASK-002 --user alice
# TASK-002 → in-progress (assigned to alice)
```

---

## status

Update the status of an item.

```
markplane status <ID> <NEW_STATUS>
```

**Arguments:**

| Argument | Description |
|----------|-------------|
| `<ID>` | Item ID (e.g. `TASK-042`) |
| `<NEW_STATUS>` | New status value |

**Valid statuses by type:**

| Prefix | Valid statuses |
|--------|---------------|
| `TASK` | `draft`, `backlog`, `planned`, `in-progress`, `done`, `cancelled` |
| `EPIC` | `planned`, `active`, `done` |
| `PLAN` | `draft`, `approved`, `in-progress`, `done` |
| `NOTE` | `draft`, `active`, `archived` |

**Example:**

```bash
markplane status TASK-001 in-progress
# TASK-001 → in-progress

markplane status EPIC-001 active
# EPIC-001 → active
```

---

## sync

Regenerate INDEX.md files and `.context/` summaries.

```
markplane sync
```

Updates all INDEX.md routing files and regenerates AI context summaries in `.context/`. Run this after making bulk changes.

**Example:**

```bash
markplane sync
# Syncing...
# ✓ All INDEX.md files and .context/ summaries regenerated.
```

---

## tag

Add tags to an item.

```
markplane tag <ID> <TAGS>
```

**Arguments:**

| Argument | Description |
|----------|-------------|
| `<ID>` | Item ID (e.g. `TASK-042`, `NOTE-001`) |
| `<TAGS>` | Comma-separated tags to add |

Tags are additive — existing tags are preserved, duplicates are ignored. Currently supported for tasks and notes.

**Example:**

```bash
markplane tag TASK-001 "frontend,urgent"
# TASK-001 tagged with: frontend, urgent
```
