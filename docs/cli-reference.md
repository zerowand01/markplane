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
| `--type <TYPE>` | *(first in config)* | Task type (configurable via `config.yaml`, default: first in `task_types` list) |
| `--priority <PRIORITY>` | `medium` | Priority: `critical`, `high`, `medium`, `low`, `someday` |
| `--effort <EFFORT>` | `medium` | Effort estimate: `xs`, `small`, `medium`, `large`, `xl` |
| `--epic <EPIC_ID>` | — | Parent epic ID (e.g. `EPIC-xa7r2`) |
| `--tags <TAGS>` | — | Comma-separated tags (e.g. `auth,backend`) |
| `--template <NAME>` | — | Template name override (e.g. `bug`). Uses type-based or kind defaults if omitted. |

New items are created with status `draft`.

**Example:**

```bash
markplane add "Implement dark mode" --type feature --priority high --effort large --epic EPIC-gc8t5 --tags "ui,theming"
# Created TASK-fq2x8 — Implement dark mode

markplane add "Login crash on Safari" --type bug --template bug
# Created TASK-kn4r7 — Login crash on Safari (uses bug template)
```

---

## archive

Move items to archive subdirectories. Works with all entity types (tasks, epics, plans, notes).

```
markplane archive [ID] [OPTIONS]
```

**Arguments:**

| Argument | Description |
|----------|-------------|
| `<ID>` | Item ID to archive (e.g. `TASK-rm6d3`, `EPIC-xa7r2`) |

**Options:**

| Option | Default | Description |
|--------|---------|-------------|
| `--all-done` | `false` | Archive all completed items across all types |
| `--dry-run` | `false` | Preview what would be archived without making changes |

When archiving a single item, provide the ID. When using `--all-done`, all done/cancelled tasks, done epics, done plans, and archived-status notes are moved to their respective `archive/` subdirectories.

Archived items retain their status (no "archived" status) and remain resolvable via `[[ID]]` cross-references and `markplane show`.

When an item is archived, inbound references from active items are automatically cleaned up. For example, if task A blocks task B and you archive A, the `depends_on: [A]` entry is removed from B's frontmatter. This prevents ghost references to archived items. The cleanup is best-effort — if a file is locked, the archive still succeeds. Note that unarchiving does not restore cleaned-up references; re-link them manually if needed.

**Example:**

```bash
# Archive a single item
markplane archive TASK-fq2x8
# ✓ Archived TASK-fq2x8

# Preview batch archive
markplane archive --all-done --dry-run
# → Would archive 2 item(s):
#   TASK-fq2x8 Implement login page (done)
#   TASK-jt9w4 Remove deprecated API (done)

# Batch archive all completed items
markplane archive --all-done
# ✓ Archived 2 item(s).
```

---

## unarchive

Restore an archived item back to active items.

```
markplane unarchive <ID>
```

**Arguments:**

| Argument | Description |
|----------|-------------|
| `<ID>` | Item ID to restore (e.g. `TASK-rm6d3`) |

Moves the item from `archive/` back to `items/`. Works with all entity types.

**Example:**

```bash
markplane unarchive TASK-fq2x8
# ✓ Restored TASK-fq2x8
```

---

## check

Validate cross-references, task statuses, reciprocal link integrity, and dependency cycles.

```
markplane check [OPTIONS]
```

**Options:**

| Option | Default | Description |
|--------|---------|-------------|
| `--orphans` | `false` | Also show orphan items (items with no incoming references) |
| `--fix` | `false` | Repair asymmetric reciprocal links |

Performs four validations:

1. **Broken references** — scans all markdown files for `[[ITEM-xxxxx]]` references and verifies each referenced item exists.
2. **Task statuses** — checks each task's status is recognized by the configured workflow.
3. **Reciprocal links** — verifies blocks/depends_on, plan/implements, and related links are symmetric on both sides.
4. **Dependency cycles** — detects circular dependencies in the blocks/depends_on graph (e.g. A blocks B blocks A).

Exits with a non-zero status if issues are found. Use `--fix` to automatically repair asymmetric reciprocal links (safe — uses idempotent `link_items` logic). Cycles and broken references must be resolved manually.

The `--orphans` flag scans both active and archived items as reference sources. An active item referenced only by an archived item is not considered an orphan.

**Example:**

```bash
markplane check
# ✓ No broken references found.
# ✓ All task statuses are valid.
# ✓ All reciprocal links are symmetric.
# ✓ No dependency cycles found.

markplane check --orphans
# ✓ No broken references found.
# ✓ All task statuses are valid.
# ✓ All reciprocal links are symmetric.
# ✓ No dependency cycles found.
# ! 1 orphan item(s) (no incoming references):
#   TASK-nq5w2

markplane check --fix
# ✗ 1 asymmetric link(s):
#   TASK-abc12 has blocks: TASK-def34 but TASK-def34 is missing depends_on: TASK-abc12
#   ✓ Repaired blocks TASK-abc12 → TASK-def34

# Cycle detection example:
# ✗ 1 dependency cycle(s):
#   TASK-abc12 → TASK-def34 → TASK-abc12
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

Regenerate `.context/` files or print a specific context view.

```
markplane context [OPTIONS]
```

**Options:**

| Option | Default | Description |
|--------|---------|-------------|
| `--focus <AREA>` | — | Generate and print a specific context view (`active-work`, `blocked`, `metrics`, `summary`) |

With no flags, regenerates all `.context/` summary files. With `--focus`, generates the requested view and prints it to stdout.

**Example:**

```bash
markplane context
# ✓ Context files regenerated in .context/

markplane context --focus active-work
# Prints active work context to stdout

markplane context --focus metrics
# Prints project metrics to stdout
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
- Now epics with completion percentages
- Summary counts (open, in-progress, blocked, critical)

**Example:**

```bash
markplane dashboard
# ✈  My App — Project Dashboard
# ══════════════════════════════════════════════════════
#
# In Progress
#   TASK-d4p7m Set up JWT handling (high @alice)
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
| `<ID>` | Item ID (e.g. `TASK-rm6d3`, `PLAN-ya8v2`) |

Works with tasks, epics, plans, and notes. Sets the status to `done`.

**Example:**

```bash
markplane done TASK-fq2x8
# TASK-fq2x8 → done
```

---

## edit

Open an item in your editor.

```
markplane edit <ID>
```

**Arguments:**

| Argument | Description |
|----------|-------------|
| `<ID>` | Item ID (e.g. `TASK-rm6d3`, `EPIC-xa7r2`, `PLAN-wk7n1`, `NOTE-dq6m1`) |

Resolves the item ID to its file path and opens it in `$EDITOR` (falls back to `$VISUAL`, then `vi`). Works with all entity types and archived items.

**Example:**

```bash
markplane edit TASK-fq2x8
# Opens .markplane/backlog/items/TASK-fq2x8.md in $EDITOR
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

New epics are created with status `later`.

**Example:**

```bash
markplane epic "Phase 2: API Layer" --priority high
# Created EPIC-kb4n9 — Phase 2: API Layer
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
| `<ID>` | Item ID (e.g. `TASK-rm6d3`) |

**Options:**

| Option | Default | Description |
|--------|---------|-------------|
| `--depth <DEPTH>` | `3` | Maximum depth to traverse |

Displays outgoing references (dependencies) as an indented tree and incoming references (what references this item). Circular references are detected and marked with `↻`.

**Example:**

```bash
markplane graph TASK-sv8r2 --depth 2
# Dependency graph for TASK-sv8r2
#
# TASK-sv8r2
#   └─ TASK-d4p7m
#     └─ TASK-fq2x8
#
# Referenced by:
#   TASK-mp3v8 → TASK-sv8r2
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
| `--empty` | `false` | Skip starter content (create empty project) |

Creates the full directory structure, `config.yaml`, INDEX.md files, templates, and special note files (`ideas.md`, `decisions.md`). By default, seeds the project with starter content (1 epic, 2 tasks, 1 plan, 1 note) that demonstrates correct format and provides an onboarding workflow. Use `--empty` to skip this. Fails if `.markplane/` already exists.

**Examples:**

```bash
markplane init --name "My App"
# Initialized Markplane project: My App
# Seeded with starter content (1 epic, 2 tasks, 1 plan, 1 note)

markplane init --name "My App" --empty
# Initialized Markplane project: My App (no starter content)
```

---

## link

Link two items with a typed relationship.

```
markplane link <FROM> <TO> -r <RELATION> [--remove]
```

**Arguments:**

| Argument | Description |
|----------|-------------|
| `<FROM>` | Source item ID |
| `<TO>` | Target item ID |

**Options:**

| Option | Description |
|--------|-------------|
| `-r, --relation <RELATION>` | Relationship type (required): `blocks`, `depends-on`, `epic`, `plan`, `implements`, `related` |
| `--remove` | Remove the link instead of adding it |

Each relation type has specific source/target type constraints:

| Relation | From | To | Effect |
|----------|------|----|--------|
| `blocks` | TASK | TASK | Adds to `from.blocks[]` and `to.depends_on[]` |
| `depends-on` | TASK | TASK | Adds to `from.depends_on[]` and `to.blocks[]` |
| `epic` | TASK | EPIC | Sets `from.epic` to target |
| `plan` | TASK | PLAN | Sets `from.plan` to target and adds to `plan.implements[]` |
| `implements` | PLAN | TASK | Adds to `from.implements[]` and sets `task.plan` |
| `related` | any | any | Adds to `from.related[]` and `to.related[]` (bidirectional) |

Self-links are rejected. Invalid source/target type combinations return an error. Adding a duplicate link is a no-op. Removing a non-existent link is a no-op.

**Example:**

```bash
# Task dependency
markplane link TASK-hn5k3 TASK-fq2x8 -r depends-on

# Task blocks another task
markplane link TASK-d4p7m TASK-jt9w4 -r blocks

# Assign a task to an epic
markplane link TASK-fq2x8 EPIC-kb4n9 -r epic

# Link a task to a plan (bidirectional)
markplane link TASK-fq2x8 PLAN-wk7n1 -r plan

# Related links — any type to any type (bidirectional)
markplane link TASK-fq2x8 NOTE-vt3k8 -r related
markplane link TASK-hn5k3 TASK-d4p7m -r related
markplane link EPIC-xa7r2 EPIC-kb4n9 -r related

# Remove a link
markplane link TASK-hn5k3 TASK-fq2x8 -r depends-on --remove
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
| `--archived` | Show archived items instead of active ones |

**Examples:**

```bash
markplane ls
markplane ls --status in-progress,planned
markplane ls --priority critical,high --tags auth
markplane ls --epic EPIC-xa7r2
markplane ls epics
markplane ls plans
markplane ls notes
markplane ls --archived             # Archived tasks
markplane ls epics --archived       # Archived epics
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
| `--type <TYPE>` | *(first in config)* | Note type (configurable via `config.yaml`, default: first in `note_types` list) |
| `--tags <TAGS>` | — | Comma-separated tags |
| `--template <NAME>` | — | Template name override (e.g. `research`, `analysis`). Uses type-based defaults if omitted. |

New notes are created with status `draft`.

**Example:**

```bash
markplane note "Evaluate caching strategies" --type research --tags "performance,cache"
# Created NOTE-vt3k8 — Evaluate caching strategies
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
| `<ID>` | Task ID (e.g. `TASK-rm6d3`) |

**Options:**

| Option | Default | Description |
|--------|---------|-------------|
| `--title <TITLE>` | `"Implementation plan for <item title>"` | Custom plan title |
| `--template <NAME>` | — | Template name override (e.g. `refactor`). Defaults to `implementation`. |

Creates a plan that references the task via `implements` and links the plan back to the item's `plan` field.

Only works with tasks (`TASK-xxxxx`).

**Example:**

```bash
markplane plan TASK-4ed4i
# Created PLAN-r9m2b — Implementation plan for Implement login page
# Linked to TASK-4ed4i

markplane plan TASK-d4p7m --title "JWT architecture design"
# Created PLAN-mf5t9 — JWT architecture design
# Linked to TASK-d4p7m
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
| `<ID>` | Note ID (e.g. `NOTE-dq6m1`) |

**Options:**

| Option | Default | Description |
|--------|---------|-------------|
| `--priority <PRIORITY>` | `medium` | Priority for the new task |
| `--effort <EFFORT>` | `medium` | Effort estimate for the new task |

Creates a new task with the note's title and tags, using the default item type from `config.yaml`. Only works with notes (`NOTE-xxxxx`).

**Example:**

```bash
markplane promote NOTE-vt3k8 --priority high --effort small
# Promoted NOTE-vt3k8 → TASK-hn5k3 — Evaluate caching strategies
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
| `--dev` | `false` | Dev mode: API only, no static file serving, permissive CORS (use with Next.js dev server) |

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
| `<ID>` | Item ID (e.g. `TASK-rm6d3`, `EPIC-xa7r2`, `PLAN-wk7n1`, `NOTE-dq6m1`) |

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
#  TASK-hn5k3 | Fix password ... | draft   | 2026-01-20   | 20
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
| `<ID>` | Item ID (e.g. `TASK-rm6d3`) |

**Options:**

| Option | Default | Description |
|--------|---------|-------------|
| `--user <USER>` | `$USER` env var (or `"me"`) | Assignee name |

For tasks, sets both status and assignee. For other item types, only updates status.

**Example:**

```bash
markplane start TASK-fq2x8
# TASK-fq2x8 → in-progress (assigned to daniel)

markplane start TASK-d4p7m --user alice
# TASK-d4p7m → in-progress (assigned to alice)
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
| `<ID>` | Item ID (e.g. `TASK-rm6d3`) |
| `<NEW_STATUS>` | New status value |

**Valid statuses by type:**

| Prefix | Valid statuses |
|--------|---------------|
| `TASK` | Configurable via `config.yaml` `workflows.task`. Defaults: `draft`, `backlog`, `planned`, `in-progress`, `done`, `cancelled` |
| `EPIC` | `later`, `next`, `now`, `done` |
| `PLAN` | `draft`, `approved`, `in-progress`, `done` |
| `NOTE` | `draft`, `active`, `archived` |

**Example:**

```bash
markplane status TASK-fq2x8 in-progress
# TASK-fq2x8 → in-progress

markplane status EPIC-xa7r2 now
# EPIC-xa7r2 → now
```

---

## sync

Regenerate INDEX.md files and `.context/` summaries.

```
markplane sync [OPTIONS]
```

**Options:**

| Option | Default | Description |
|--------|---------|-------------|
| `--normalize` | `false` | Also normalize task position keys (rewrites source files) |

Updates all INDEX.md routing files and regenerates AI context summaries in `.context/`. These are derived files, gitignored within `.markplane/`. Run this after making bulk changes or after a fresh clone.

Sync also runs automatically on `markplane init`, `markplane mcp` startup, and `markplane serve` startup.

The `--normalize` flag rewrites fractional position keys (generated by drag-and-drop reordering) into clean sequential ones (`a0`, `a1`, `a2`). This is cosmetic — fractional keys work correctly for ordering. Normalization modifies task frontmatter (source files), which is why it's opt-in rather than automatic.

**Example:**

```bash
markplane sync
# Syncing...
# ✓ All INDEX.md files and .context/ summaries regenerated.

markplane sync --normalize
# Normalizing positions...
# Syncing...
# ✓ All INDEX.md files and .context/ summaries regenerated.
```

---

## update

Update properties on any item. Replaces the old `assign` and `tag` commands.

```
markplane update <ID> [OPTIONS]
```

**Arguments:**

| Argument | Description |
|----------|-------------|
| `<ID>` | Item ID (e.g. `TASK-rm6d3`, `EPIC-xa7r2`, `PLAN-wk7n1`, `NOTE-dq6m1`) |

**Options:**

| Option | Applies to | Description |
|--------|-----------|-------------|
| `--title <TITLE>` | All | New title |
| `--status <STATUS>` | All | New status value |
| `--priority <PRIORITY>` | Tasks, Epics | New priority |
| `--effort <EFFORT>` | Tasks | Effort size: `xs`, `small`, `medium`, `large`, `xl` |
| `--type <TYPE>` | Tasks | Task type (configurable via `config.yaml`) |
| `--assignee <USER>` | Tasks | Set assignee (leading `@` stripped automatically) |
| `--clear-assignee` | Tasks | Clear assignee |
| `--position <KEY>` | Tasks | Set position key for manual ordering |
| `--clear-position` | Tasks | Clear position |
| `--add-tag <TAGS>` | Tasks, Epics, Notes | Comma-separated tags to add (duplicates ignored) |
| `--remove-tag <TAGS>` | Tasks, Epics, Notes | Comma-separated tags to remove |
| `--started <DATE>` | Epics | Set started date (YYYY-MM-DD) |
| `--clear-started` | Epics | Clear started date |
| `--target <DATE>` | Epics | Set target date (YYYY-MM-DD) |
| `--clear-target` | Epics | Clear target date |
| `--note-type <TYPE>` | Notes | Note type (configurable via `config.yaml`) |

Fields that don't apply to the item's entity type are rejected with an error. Multiple options can be combined in a single command.

**Examples:**

```bash
# Update effort and priority on a task
markplane update TASK-fq2x8 --effort large --priority high

# Assign and add tags
markplane update TASK-fq2x8 --assignee @daniel --add-tag "ui,frontend"

# Remove a tag and change type
markplane update TASK-fq2x8 --remove-tag wip --type bug

# Clear assignee
markplane update TASK-fq2x8 --clear-assignee

# Rename a task
markplane update TASK-fq2x8 --title "New title"

# Set epic dates
markplane update EPIC-xa7r2 --started 2026-02-20 --target 2026-06-01

# Change a note's type
markplane update NOTE-vt3k8 --note-type decision --add-tag arch
```
