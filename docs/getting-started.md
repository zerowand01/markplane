# Getting Started with Markplane

This guide walks you through a realistic workflow — from initializing a project to managing work items through their lifecycle.

## Prerequisites

Install Markplane (Rust 1.93.0+ required):

```bash
cargo install --path crates/markplane-cli
```

## 1. Initialize Your Project

Navigate to your repository and run:

```bash
markplane init --name "My App" --description "A web application"
```

Output:

```
Initialized Markplane project: My App

  .markplane/
  ├── config.yaml
  ├── INDEX.md
  ├── roadmap/          (EPIC-xxxxx)
  ├── backlog/          (TASK-xxxxx)
  ├── plans/            (PLAN-xxxxx)
  ├── notes/            (NOTE-xxxxx)
  ├── templates/
  └── .context/

Get started:
  markplane add "My first task"
  markplane ls
```

This creates the `.markplane/` directory structure with config, templates, index files, and special note files (`ideas.md`, `decisions.md`).

## How Items Relate

Markplane organizes work into four entity types, each with a distinct role:

| Entity | Role | Think of it as... |
|--------|------|-------------------|
| **Epic** | Strategic direction — *why* we're doing this | A goal or phase that groups related work |
| **Task** | The *what* — a concrete piece of work | A bug, feature, chore, or spike to complete |
| **Plan** | The *how* — implementation steps for one or more tasks | Phases, testing strategy, rollback steps, migration path |
| **Note** | Captured thinking — research, analysis, ideas, decisions | Structured knowledge that may or may not become actionable |

Items flow through a natural lifecycle:

```
Note (explore) → Task (commit) → Plan (design) → Done → Archive
```

Not every task needs a plan — only create one when the implementation approach isn't obvious. Notes can be promoted to tasks with `markplane promote` once they become actionable.

## 2. Create an Epic

Epics represent strategic phases or major features. Create one to group related work:

```bash
markplane epic "User Authentication System" --priority high
```

Output:

```
Created EPIC-xa7r2 — User Authentication System
```

Epics start with status `planned`. You can list them with:

```bash
markplane ls epics
```

## 3. Add Tasks

Tasks capture *what* needs to be done — a bug to fix, a feature to build, a chore to complete. They're your primary work units. Create items linked to the epic:

```bash
markplane add "Implement login page" --type feature --priority high --effort medium --epic EPIC-xa7r2 --tags "auth,frontend"
```

Output:

```
Created TASK-fq2x8 — Implement login page
```

Add a few more items:

```bash
markplane add "Set up JWT token handling" --type feature --priority high --effort small --epic EPIC-xa7r2 --tags "auth,backend"
markplane add "Fix password reset flow" --type bug --priority critical --effort small --tags "auth"
markplane add "Add rate limiting to auth endpoints" --type enhancement --priority medium --effort medium --epic EPIC-xa7r2 --tags "auth,security"
```

### Available Options

- **Types**: Configurable in `config.yaml` (`item_types`) or via the web UI Settings page. Defaults: `feature`, `bug`, `enhancement`, `chore`, `research`, `spike`
- **Priorities**: `critical`, `high`, `medium`, `low`, `someday`
- **Effort sizes**: `xs`, `small`, `medium`, `large`, `xl`

All new tasks start with status `draft`.

## 4. List and Filter Items

View all tasks:

```bash
markplane ls
```

This displays a table with ID, title, status, priority, effort, and epic columns.

Filter by status, priority, tags, or other criteria:

```bash
# Only high and critical priority items
markplane ls --priority high,critical

# Items tagged with "auth"
markplane ls --tags auth

# Items linked to a specific epic
markplane ls --epic EPIC-xa7r2

# Combine filters
markplane ls --status draft --priority high --tags auth
```

You can also filter by type and assignee:

```bash
markplane ls --type bug
markplane ls --assignee daniel
```

## 5. View Item Details

Inspect a specific item:

```bash
markplane show TASK-fq2x8
```

Output:

```
TASK-fq2x8
Implement login page

  Status:   draft
  Priority: high
  Type:     feature
  Effort:   medium
  Tags:     auth, frontend
  Epic:     EPIC-xa7r2
  Created:  2026-02-09
  Updated:  2026-02-09

────────────────────────────────────────────────────────────
# Implement login page

## Description
...
```

This works for any item type — tasks, epics, plans, and notes.

## 6. Move Items Through the Workflow

### Update status manually

```bash
markplane status TASK-fq2x8 backlog
markplane status TASK-fq2x8 planned
```

Output:

```
TASK-fq2x8 → planned
```

### Start working on an item

The `start` command sets the status to `in-progress` and assigns the item to you:

```bash
markplane start TASK-fq2x8
```

Output:

```
TASK-fq2x8 → in-progress (assigned to daniel)
```

You can specify a different user:

```bash
markplane start TASK-d4p7m --user alice
```

### Mark an item as done

```bash
markplane done TASK-fq2x8
```

Output:

```
TASK-fq2x8 → done
```

### Update epic status

```bash
markplane status EPIC-xa7r2 active
```

## 7. Set Up Dependencies

Link items that depend on each other:

```bash
# TASK-sv8r2 depends on TASK-d4p7m being completed first
markplane link TASK-sv8r2 TASK-d4p7m -r depends-on

# Equivalently, TASK-d4p7m blocks TASK-sv8r2
markplane link TASK-d4p7m TASK-sv8r2 -r blocks
```

Both directions are automatically maintained — adding a `depends-on` link also adds the reverse `blocks` link on the target.

The `link` command supports 6 relation types: `blocks`, `depends-on`, `epic`, `plan`, `implements`, `related`. Use `--remove` to unlink:

```bash
# Remove a dependency
markplane link TASK-sv8r2 TASK-d4p7m -r depends-on --remove
```

## 8. Update Item Properties

Use the `update` command to change any property on an item:

```bash
# Assign a task
markplane update TASK-hn5k3 --assignee @daniel

# Add tags
markplane update TASK-hn5k3 --add-tag "urgent,sprint-3"

# Change effort and priority
markplane update TASK-hn5k3 --effort large --priority high

# Remove a tag
markplane update TASK-hn5k3 --remove-tag urgent

# Clear assignee
markplane update TASK-hn5k3 --clear-assignee

# Rename a task and change its type
markplane update TASK-hn5k3 --title "New title" --type bug
```

Fields that don't apply to the item type are rejected (e.g. `--effort` on an epic).

## 9. Create Notes and Promote Them

Capture ideas, research, and decisions as notes:

```bash
markplane note "Evaluate OAuth providers" --type research --tags "auth,research"
```

Output:

```
Created NOTE-vt3k8 — Evaluate OAuth providers
```

Note types are configurable in `config.yaml` (`note_types`) or via the web UI Settings page. Defaults: `research`, `analysis`, `idea`, `decision`, `meeting`.

When a note becomes actionable, promote it to a task:

```bash
markplane promote NOTE-vt3k8 --priority high --effort medium
```

Output:

```
Promoted NOTE-vt3k8 → TASK-jt9w4 — Evaluate OAuth providers
```

The new task inherits the note's title and tags.

## 10. Create Implementation Plans

Tasks capture *what* needs to be done; plans capture *how* to do it — the implementation steps, phases, testing strategy, and rollback approach. A plan can implement one or more tasks. For complex work where the approach isn't obvious, create a linked implementation plan:

```bash
markplane plan TASK-fq2x8
```

Output:

```
Created PLAN-ya8v2 — Implementation plan for Implement login page
Linked to TASK-fq2x8
```

You can provide a custom title:

```bash
markplane plan TASK-d4p7m --title "JWT auth architecture"
```

The plan is automatically linked back to the task, and inherits the item's epic.

List all plans:

```bash
markplane ls plans
```

## 11. Sync Index Files and Context

After making changes, regenerate the INDEX.md files and AI context summaries:

```bash
markplane sync
```

Output:

```
Syncing...
✓ All INDEX.md files and .context/ summaries regenerated.
```

You can also regenerate just the context files:

```bash
markplane context
```

Or generate and print a specific context view to stdout:

```bash
markplane context --focus active-work
```

## 12. Validate Cross-References

Check for broken `[[ITEM-xxxxx]]` references across all files:

```bash
markplane check
```

Output (all good):

```
✓ No broken references found.
```

Output (with issues):

```
✗ 2 broken reference(s):

  .markplane/backlog/items/TASK-hn5k3.md references TASK-zz9x1 (not found)
  .markplane/plans/items/PLAN-ya8v2.md references TASK-dj7a4 (not found)
```

Include orphan detection to find items with no incoming references:

```bash
markplane check --orphans
```

## 13. Find Stale Items

Identify items that haven't been updated recently:

```bash
markplane stale --days 14
```

Output:

```
! 2 item(s) not updated in 14 days:

 ID       | Title                  | Status   | Last Updated | Days Stale
 TASK-hn5k3 | Fix password reset ... | draft    | 2026-01-20   | 20
 TASK-sv8r2 | Add rate limiting ...  | planned  | 2026-01-25   | 15
```

The default threshold is 30 days.

## 14. Archive Completed Work

Move completed items to archive subdirectories. You can archive a single item or batch-archive all completed items:

```bash
# Archive a single item
markplane archive TASK-fq2x8
```

Output:

```
✓ Archived TASK-fq2x8
```

For batch operations, preview first with `--dry-run`:

```bash
markplane archive --all-done --dry-run
```

Output:

```
→ Would archive 1 item(s):

  TASK-fq2x8 Implement login page (done)

Run without --dry-run to archive.
```

Then archive for real:

```bash
markplane archive --all-done
```

Items are moved to `{directory}/archive/` — they're preserved, not deleted, and can still be found by `markplane show`. To restore an archived item:

```bash
markplane unarchive TASK-fq2x8
```

To list archived items:

```bash
markplane ls --archived
```

## 15. View the Dashboard

Get a project overview showing in-progress work, blocked items, and epic progress:

```bash
markplane dashboard
```

Output:

```
✈  My App — Project Dashboard
══════════════════════════════════════════════════════

In Progress
  TASK-d4p7m Set up JWT token handling (high @alice)

Blocked
  TASK-sv8r2 Add rate limiting — blocked by TASK-d4p7m

Now
  EPIC-xa7r2 User Authentication System — 1/4 (25%)

3 open items | 1 in-progress | 1 blocked | 1 critical
```

## 16. View Metrics

Get detailed project statistics:

```bash
markplane metrics
```

This shows status distribution, priority breakdown, epic progress bars, and plan counts.

## 17. Visualize Dependencies

View the dependency graph for any item:

```bash
markplane graph TASK-sv8r2 --depth 3
```

Output:

```
Dependency graph for TASK-sv8r2

TASK-sv8r2
  └─ TASK-d4p7m

Referenced by:
  (none)
```

## 18. Connect Project Documentation

Markplane handles project management; your repo's `docs/` directory (or similar) handles technical documentation — architecture, API references, user guides. The `documentation_paths` config option bridges the two, so AI agents and INDEX navigation can discover both.

Add your docs directory to `.markplane/config.yaml`:

```yaml
documentation_paths:
  - docs
```

Paths are relative to the repo root. You can list multiple directories:

```yaml
documentation_paths:
  - docs
  - design
```

When `markplane sync` runs, it scans the configured paths for `*.md` files and:

- Adds a **"Project Documentation"** section to the root `INDEX.md` with links to each doc
- Adds a **"Key Documentation"** section to `.context/summary.md` so AI tools see your docs alongside project state

This keeps architecture docs, API specs, and other reference material in their conventional location while making them visible through Markplane's navigation and AI context layer.

## 19. AI Integration

### CLAUDE.md snippet

Generate a CLAUDE.md snippet for AI coding tools:

```bash
markplane claude-md
```

Add the output to your project's `CLAUDE.md` so AI assistants know where to find project context.

### MCP server

For deeper AI integration, the `markplane mcp` subcommand runs an MCP server with typed tools for reading and managing project data. See the [MCP setup guide](mcp-setup.md) for configuration details.

## Next Steps

- Read the [CLI Reference](cli-reference.md) for complete command documentation
- Explore the `.markplane/templates/` directory to see document templates
- Set up the MCP server for AI tool integration
- Add `.markplane/` to version control to share project state with your team
