# Markplane File Format Reference

## Directory Structure

```
.markplane/
├── config.yaml              # Project configuration and ID counters
├── INDEX.md                 # Root navigation index (auto-generated)
├── roadmap/
│   ├── INDEX.md             # Epic listing (auto-generated)
│   ├── items/               # Epic files
│   │   ├── EPIC-001.md
│   │   └── EPIC-002.md
│   └── archive/             # Completed epics
├── backlog/
│   ├── INDEX.md             # Backlog listing by status/priority/epic (auto-generated)
│   ├── items/               # Task files
│   │   ├── TASK-001.md
│   │   └── TASK-002.md
│   └── archive/             # Done/cancelled items
├── plans/
│   ├── INDEX.md             # Plan listing (auto-generated)
│   ├── items/               # Plan files
│   │   └── PLAN-001.md
│   ├── templates/           # Plan-specific templates
│   └── archive/             # Completed plans
├── notes/
│   ├── INDEX.md             # Note listing (auto-generated)
│   ├── items/               # Note files
│   │   └── NOTE-001.md
│   ├── ideas.md             # Quick idea capture (special file)
│   ├── decisions.md         # Decision log (special file)
│   └── archive/             # Archived notes
├── templates/
│   ├── task.md              # Template for new tasks
│   ├── epic.md              # Template for new epics
│   ├── plan-implementation.md
│   ├── plan-refactor.md
│   ├── note-research.md
│   └── note-analysis.md
└── .context/
    ├── summary.md           # Project overview (~1000 tokens)
    ├── active-work.md       # Currently in-progress items
    ├── blocked-items.md     # Items with unresolved dependencies
    └── metrics.md           # Status/priority distribution, epic progress
```

## YAML Frontmatter Format

Every item file uses YAML frontmatter delimited by `---`:

```markdown
---
id: TASK-042
title: "Add dark mode"
status: in-progress
priority: high
type: feature
effort: medium
tags: ["ui", "theming"]
epic: EPIC-003
plan: null
depends_on: [TASK-038]
blocks: [TASK-045]
assignee: daniel
created: 2026-01-15
updated: 2026-02-09
---

# Add dark mode

Markdown body content here...
```

The frontmatter is parsed by splitting on `---` delimiters and deserializing the YAML section with `serde_yaml`. The body is everything after the closing `---`.

Titles are always double-quoted in the YAML to safely handle special characters. The system escapes `\`, `"`, `\n`, and `\r` in titles before writing.

## Field Reference

### Task (prefix: `TASK`)

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | string | yes | Unique identifier, e.g. `TASK-042` |
| `title` | string | yes | Item title (max 500 characters) |
| `status` | enum | yes | Current workflow status |
| `priority` | enum | yes | Priority level |
| `type` | enum | yes | Item classification |
| `effort` | enum | yes | Effort estimate |
| `tags` | string[] | no | Categorization tags (default: `[]`) |
| `epic` | string? | no | Parent epic ID, e.g. `EPIC-003` (default: `null`) |
| `plan` | string? | no | Linked plan ID, e.g. `PLAN-012` (default: `null`) |
| `depends_on` | string[] | no | IDs this item depends on (default: `[]`) |
| `blocks` | string[] | no | IDs this item blocks (default: `[]`) |
| `assignee` | string? | no | Assigned user (default: `null`) |
| `created` | date | yes | Creation date (`YYYY-MM-DD`) |
| `updated` | date | yes | Last modification date (`YYYY-MM-DD`) |

### Epic (prefix: `EPIC`)

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | string | yes | Unique identifier, e.g. `EPIC-003` |
| `title` | string | yes | Epic title (max 500 characters) |
| `status` | enum | yes | Current workflow status |
| `priority` | enum | yes | Priority level |
| `started` | date? | no | Date work began (default: `null`) |
| `target` | date? | no | Target completion date (default: `null`) |
| `tags` | string[] | no | Categorization tags (default: `[]`) |
| `depends_on` | string[] | no | Epic IDs this depends on (default: `[]`) |

### Plan (prefix: `PLAN`)

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | string | yes | Unique identifier, e.g. `PLAN-012` |
| `title` | string | yes | Plan title (max 500 characters) |
| `status` | enum | yes | Current workflow status |
| `implements` | string[] | no | Task IDs this plan implements (default: `[]`) |
| `epic` | string? | no | Parent epic ID (default: `null`) |
| `created` | date | yes | Creation date (`YYYY-MM-DD`) |
| `updated` | date | yes | Last modification date (`YYYY-MM-DD`) |

### Note (prefix: `NOTE`)

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | string | yes | Unique identifier, e.g. `NOTE-007` |
| `title` | string | yes | Note title (max 500 characters) |
| `type` | enum | yes | Note classification |
| `status` | enum | yes | Current workflow status |
| `tags` | string[] | no | Categorization tags (default: `[]`) |
| `related` | string[] | no | Related item IDs (default: `[]`) |
| `created` | date | yes | Creation date (`YYYY-MM-DD`) |
| `updated` | date | yes | Last modification date (`YYYY-MM-DD`) |

## Enum Values

### Status Workflows

**TaskStatus** — `draft` → `backlog` → `planned` → `in-progress` → `done`

Also: `cancelled` (terminal state, reachable from any non-done status)

```
draft → backlog → planned → in-progress → done
                                    ↘
                               cancelled
```

**EpicStatus** — `planned` → `active` → `done`

```
planned → active → done
```

**PlanStatus** — `draft` → `approved` → `in-progress` → `done`

```
draft → approved → in-progress → done
```

**NoteStatus** — `draft` → `active` → `archived`

```
draft → active → archived
```

### Priority

Values (highest to lowest): `critical`, `high`, `medium`, `low`, `someday`

### ItemType (Task only)

Values: `feature`, `bug`, `enhancement`, `chore`, `research`, `spike`

### Effort (Task only)

Values (smallest to largest): `xs`, `small`, `medium`, `large`, `xl`

### NoteType

Values: `research`, `analysis`, `idea`, `decision`, `meeting`

## ID System

IDs follow the format `PREFIX-NNN` where:
- **PREFIX** is one of: `EPIC`, `TASK`, `PLAN`, `NOTE`
- **NNN** is a zero-padded sequential number (e.g., `001`, `042`, `100`)

Rules:
- IDs are **permanent** — once assigned, never reused
- IDs are **sequential** — managed by counters in `config.yaml`
- The prefix determines the **directory**: `EPIC` → `roadmap/`, `TASK` → `backlog/`, `PLAN` → `plans/`, `NOTE` → `notes/`
- The filename is always `{ID}.md` (e.g., `TASK-042.md`)
- ID parsing is case-insensitive (`task-042` resolves to `TASK-042`)

Counter management uses file locking (`fs2`) on `config.yaml` to prevent duplicate IDs from concurrent processes.

## Cross-Reference Syntax

Use double-bracket wiki-link syntax to reference items:

```markdown
This feature depends on [[TASK-038]] and relates to [[EPIC-003]].
See [[PLAN-012]] for the implementation plan.
```

References work in both the markdown body and are also extracted from frontmatter fields (`epic`, `plan`, `depends_on`, `blocks`, `implements`, `related`).

Reference rules:
- Must contain a valid ID matching `PREFIX-NUMBER` format
- Cannot span newlines
- Only the four valid prefixes (`EPIC`, `TASK`, `PLAN`, `NOTE`) are recognized
- Invalid references (e.g., `[[INVALID-001]]`) are ignored during extraction

Use `markplane check` to validate all references and `markplane check --orphans` to find items with no incoming references.

## config.yaml Schema

```yaml
version: 1
project:
  name: "My Project"
  description: "Project description"
counters:
  EPIC: 3        # Last assigned EPIC number
  TASK: 42       # Last assigned TASK number
  PLAN: 12       # Last assigned PLAN number
  NOTE: 7        # Last assigned NOTE number
context:
  token_budget: 1000      # Target token budget for summary
  recent_days: 7          # Days to consider "recent" for completions
  auto_generate: true     # Auto-regenerate context on sync
archive:
  auto_archive_after_days: 30   # Days after done before auto-archive
  keep_cancelled: true          # Keep cancelled items (vs. archive them)
documentation_paths:            # Paths to project docs (relative to repo root)
  - docs                        # Scanned for *.md files; linked in INDEX and .context/
```

The `counters` map tracks the highest assigned number for each prefix. When a new item is created, the counter increments and the new value is used. Counters never decrease.

## Template System

Templates are embedded in the `markplane-core` binary as Rust string constants. They use `{PLACEHOLDER}` tokens that are replaced at creation time by `render_template()`.

### Available Placeholders

| Placeholder | Used In | Description |
|-------------|---------|-------------|
| `{ID}` | All | Item ID (e.g., `TASK-042`) |
| `{TITLE}` | All | Sanitized title string |
| `{DATE}` | All | Current date (`YYYY-MM-DD`) |
| `{STATUS}` | Backlog | Initial status value |
| `{PRIORITY}` | Backlog, Epic | Priority level |
| `{TYPE}` | Backlog, Note | Item/note type |
| `{EFFORT}` | Backlog | Effort estimate |
| `{EPIC}` | Backlog, Plan | Epic ID or `null` |
| `{TAGS}` | Backlog, Note | YAML-formatted tag list |
| `{IMPLEMENTS}` | Plan | YAML-formatted implements list |
| `{RELATED}` | Note | YAML-formatted related list |
| `{PROJECT_NAME}` | Root INDEX | Project name (init only) |

### Template Files

Templates are written to `.markplane/templates/` during `markplane init`:

- `task.md` — New tasks (description, acceptance criteria, notes, references sections)
- `epic.md` — New epics (objective, key results, notes)
- `plan-implementation.md` — Implementation plans (overview, approach, phases, testing, rollback)
- `plan-refactor.md` — Refactor plans (motivation, current/target state, migration steps, risks)
- `note-research.md` — Research notes (summary, findings, recommendations)
- `note-analysis.md` — Analysis notes (context, analysis, conclusions, next steps)

A generic note template is used for `idea`, `decision`, and `meeting` note types.

Unreplaced placeholders remain as-is in the output (no error is raised).

## INDEX.md Files

INDEX.md files are auto-generated by `markplane sync`. They contain a `<!-- Generated by markplane sync -->` header comment. Each directory has an index:

- **Root INDEX.md**: Quick navigation table with active item counts, ID counters, last sync date
- **backlog/INDEX.md**: Prioritized kanban view (In Progress, Blocked, Planned, Backlog, Drafts)
- **roadmap/INDEX.md**: Epics (active, planned) with nested task tables and progress
- **plans/INDEX.md**: Active and completed plans with their implements links
- **notes/INDEX.md**: Active notes table with type/status/tags, archived notes list, special files links

## .context/ Files

Context files are generated summaries optimized for AI consumption (~1000 token budget). Generated by `markplane sync` or `markplane context`.

- **summary.md**: Project overview — active epics with progress, in-progress work, blocked items, recent completions, priority queue, key metrics, key documentation (when `documentation_paths` is configured)
- **active-work.md**: Detailed view of all in-progress tasks and plans with full metadata
- **blocked-items.md**: Items with unresolved dependencies, showing which items block them
- **metrics.md**: Status distribution, priority distribution, epic progress percentages, plans summary

## Special Files

Two non-ID files live in `notes/`:

- **ideas.md**: Quick-capture scratch pad for ideas not yet promoted to tasks. Use `markplane promote NOTE-xxx` to convert a note to a task.
- **decisions.md**: Lightweight decision log. Format: `## YYYY-MM-DD: Decision Title` followed by context and rationale.

These files are excluded from directory scanning and reference extraction.
