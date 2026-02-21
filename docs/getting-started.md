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
  ├── roadmap/          (EPIC-NNN)
  ├── backlog/          (TASK-NNN)
  ├── plans/            (PLAN-NNN)
  ├── notes/            (NOTE-NNN)
  ├── templates/
  └── .context/

Get started:
  markplane add "My first task"
  markplane ls
```

This creates the `.markplane/` directory structure with config, templates, index files, and special note files (`ideas.md`, `decisions.md`).

## 2. Create an Epic

Epics represent strategic phases or major features. Create one to group related work:

```bash
markplane epic "User Authentication System" --priority high
```

Output:

```
Created EPIC-001 — User Authentication System
```

Epics start with status `planned`. You can list them with:

```bash
markplane ls epics
```

## 3. Add Tasks

Tasks are your primary work units. Create items linked to the epic:

```bash
markplane add "Implement login page" --type feature --priority high --effort medium --epic EPIC-001 --tags "auth,frontend"
```

Output:

```
Created TASK-001 — Implement login page
```

Add a few more items:

```bash
markplane add "Set up JWT token handling" --type feature --priority high --effort small --epic EPIC-001 --tags "auth,backend"
markplane add "Fix password reset flow" --type bug --priority critical --effort small --tags "auth"
markplane add "Add rate limiting to auth endpoints" --type enhancement --priority medium --effort medium --epic EPIC-001 --tags "auth,security"
```

### Available Options

- **Types**: `feature`, `bug`, `enhancement`, `chore`, `research`, `spike`
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
markplane ls --epic EPIC-001

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
markplane show TASK-001
```

Output:

```
TASK-001
Implement login page

  Status:   draft
  Priority: high
  Type:     feature
  Effort:   medium
  Tags:     auth, frontend
  Epic:     EPIC-001
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
markplane status TASK-001 backlog
markplane status TASK-001 planned
```

Output:

```
TASK-001 → planned
```

### Start working on an item

The `start` command sets the status to `in-progress` and assigns the item to you:

```bash
markplane start TASK-001
```

Output:

```
TASK-001 → in-progress (assigned to daniel)
```

You can specify a different user:

```bash
markplane start TASK-002 --user alice
```

### Mark an item as done

```bash
markplane done TASK-001
```

Output:

```
TASK-001 → done
```

### Update epic status

```bash
markplane status EPIC-001 active
```

## 7. Set Up Dependencies

Link items that depend on each other:

```bash
# TASK-004 depends on TASK-002 being completed first
markplane link TASK-004 TASK-002 -r depends-on

# Equivalently, TASK-002 blocks TASK-004
markplane link TASK-002 TASK-004 -r blocks
```

Both directions are automatically maintained — adding a `depends-on` link also adds the reverse `blocks` link on the target.

The `link` command supports 6 relation types: `blocks`, `depends-on`, `epic`, `plan`, `implements`, `related`. Use `--remove` to unlink:

```bash
# Remove a dependency
markplane link TASK-004 TASK-002 -r depends-on --remove
```

## 8. Update Item Properties

Use the `update` command to change any property on an item:

```bash
# Assign a task
markplane update TASK-003 --assignee @daniel

# Add tags
markplane update TASK-003 --add-tag "urgent,sprint-3"

# Change effort and priority
markplane update TASK-003 --effort large --priority high

# Remove a tag
markplane update TASK-003 --remove-tag urgent

# Clear assignee
markplane update TASK-003 --clear-assignee

# Rename a task and change its type
markplane update TASK-003 --title "New title" --type bug
```

Fields that don't apply to the item type are rejected (e.g. `--effort` on an epic).

## 9. Create Notes and Promote Them

Capture ideas, research, and decisions as notes:

```bash
markplane note "Evaluate OAuth providers" --type research --tags "auth,research"
```

Output:

```
Created NOTE-001 — Evaluate OAuth providers
```

Note types: `research`, `analysis`, `idea`, `decision`, `meeting`.

When a note becomes actionable, promote it to a task:

```bash
markplane promote NOTE-001 --priority high --effort medium
```

Output:

```
Promoted NOTE-001 → TASK-005 — Evaluate OAuth providers
```

The new task inherits the note's title and tags.

## 10. Create Implementation Plans

For complex items, create a linked implementation plan:

```bash
markplane plan TASK-001
```

Output:

```
Created PLAN-001 — Implementation plan for Implement login page
Linked to TASK-001
```

You can provide a custom title:

```bash
markplane plan TASK-002 --title "JWT auth architecture"
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

Or generate focused context for a specific item (includes linked epic, plan, and dependencies):

```bash
markplane context --item TASK-001
```

## 12. Validate Cross-References

Check for broken `[[ITEM-NNN]]` references across all files:

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

  .markplane/backlog/items/TASK-003.md references TASK-999 (not found)
  .markplane/plans/items/PLAN-001.md references TASK-050 (not found)
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
 TASK-003 | Fix password reset ... | draft    | 2026-01-20   | 20
 TASK-004 | Add rate limiting ...  | planned  | 2026-01-25   | 15
```

The default threshold is 30 days.

## 14. Archive Completed Work

Move completed items to archive subdirectories. You can archive a single item or batch-archive all completed items:

```bash
# Archive a single item
markplane archive TASK-001
```

Output:

```
✓ Archived TASK-001
```

For batch operations, preview first with `--dry-run`:

```bash
markplane archive --all-done --dry-run
```

Output:

```
→ Would archive 1 item(s):

  TASK-001 Implement login page (done)

Run without --dry-run to archive.
```

Then archive for real:

```bash
markplane archive --all-done
```

Items are moved to `{directory}/archive/` — they're preserved, not deleted, and can still be found by `markplane show`. To restore an archived item:

```bash
markplane unarchive TASK-001
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
  TASK-002 Set up JWT token handling (high @alice)

Blocked
  TASK-004 Add rate limiting — blocked by TASK-002

Active Epics
  EPIC-001 User Authentication System — 1/4 (25%)

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
markplane graph TASK-004 --depth 3
```

Output:

```
Dependency graph for TASK-004

TASK-004
  └─ TASK-002

Referenced by:
  (none)
```

## 18. AI Integration

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
