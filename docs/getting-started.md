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
  ├── backlog/          (BACK-NNN)
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

## 3. Add Backlog Items

Backlog items are your primary work units. Create items linked to the epic:

```bash
markplane add "Implement login page" --type feature --priority high --effort medium --epic EPIC-001 --tags "auth,frontend"
```

Output:

```
Created BACK-001 — Implement login page
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

All new backlog items start with status `draft`.

## 4. List and Filter Items

View all backlog items:

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
markplane show BACK-001
```

Output:

```
BACK-001
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

This works for any item type — backlog items, epics, plans, and notes.

## 6. Move Items Through the Workflow

### Update status manually

```bash
markplane status BACK-001 backlog
markplane status BACK-001 planned
```

Output:

```
BACK-001 → planned
```

### Start working on an item

The `start` command sets the status to `in-progress` and assigns the item to you:

```bash
markplane start BACK-001
```

Output:

```
BACK-001 → in-progress (assigned to daniel)
```

You can specify a different user:

```bash
markplane start BACK-002 --user alice
```

### Mark an item as done

```bash
markplane done BACK-001
```

Output:

```
BACK-001 → done
```

### Update epic status

```bash
markplane status EPIC-001 active
```

## 7. Set Up Dependencies

Link items that depend on each other:

```bash
# BACK-004 depends on BACK-002 being completed first
markplane link BACK-004 --depends-on BACK-002

# Equivalently, BACK-002 blocks BACK-004
markplane link BACK-002 --blocks BACK-004
```

Both directions are automatically maintained — adding a `depends-on` link also adds the reverse `blocks` link on the target.

## 8. Assign and Tag Items

Assign a backlog item to someone:

```bash
markplane assign BACK-003 @daniel
```

Add tags to an existing item:

```bash
markplane tag BACK-003 "urgent,sprint-3"
```

Tags are additive — existing tags are preserved.

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

When a note becomes actionable, promote it to a backlog item:

```bash
markplane promote NOTE-001 --priority high --effort medium
```

Output:

```
Promoted NOTE-001 → BACK-005 — Evaluate OAuth providers
```

The new backlog item inherits the note's title and tags.

## 10. Create Implementation Plans

For complex items, create a linked implementation plan:

```bash
markplane plan BACK-001
```

Output:

```
Created PLAN-001 — Implementation plan for Implement login page
Linked to BACK-001
```

You can provide a custom title:

```bash
markplane plan BACK-002 --title "JWT auth architecture"
```

The plan is automatically linked back to the backlog item, and inherits the item's epic.

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
markplane context --item BACK-001
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

  .markplane/backlog/items/BACK-003.md references BACK-999 (not found)
  .markplane/plans/items/PLAN-001.md references BACK-050 (not found)
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
 BACK-003 | Fix password reset ... | draft    | 2026-01-20   | 20
 BACK-004 | Add rate limiting ...  | planned  | 2026-01-25   | 15
```

The default threshold is 30 days.

## 14. Archive Completed Work

Move done items to archive subdirectories. Preview first with `--dry-run`:

```bash
markplane archive --dry-run
```

Output:

```
→ Would archive 1 item(s):

  BACK-001 Implement login page (done)

Run without --dry-run to archive.
```

Then archive for real:

```bash
markplane archive
```

Items are moved to `{directory}/archive/` — they're preserved, not deleted, and can still be found by `markplane show`.

The archive threshold is configured in `.markplane/config.yaml` (default: 30 days after completion).

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
  BACK-002 Set up JWT token handling (high @alice)

Blocked
  BACK-004 Add rate limiting — blocked by BACK-002

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
markplane graph BACK-004 --depth 3
```

Output:

```
Dependency graph for BACK-004

BACK-004
  └─ BACK-002

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

For deeper AI integration, the `markplane-mcp` binary provides an MCP server with typed tools for reading and managing project data. See the [MCP setup guide](mcp-setup.md) for configuration details.

## Next Steps

- Read the [CLI Reference](cli-reference.md) for complete command documentation
- Explore the `.markplane/templates/` directory to see document templates
- Set up the MCP server for AI tool integration
- Add `.markplane/` to version control to share project state with your team
