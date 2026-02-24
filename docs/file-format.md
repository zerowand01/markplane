# Markplane File Format Reference

## Directory Structure

```
.markplane/
├── config.yaml              # Project configuration
├── INDEX.md                 # Root navigation index (auto-generated)
├── roadmap/
│   ├── INDEX.md             # Epic listing (auto-generated)
│   ├── items/               # Epic files
│   │   ├── EPIC-xa7r2.md
│   │   └── EPIC-kb4n9.md
│   └── archive/             # Completed epics
├── backlog/
│   ├── INDEX.md             # Backlog listing by status/priority/epic (auto-generated)
│   ├── items/               # Task files
│   │   ├── TASK-fq2x8.md
│   │   └── TASK-d4p7m.md
│   └── archive/             # Done/cancelled items
├── plans/
│   ├── INDEX.md             # Plan listing (auto-generated)
│   ├── items/               # Plan files
│   │   └── PLAN-ya8v2.md
│   ├── templates/           # Plan-specific templates
│   └── archive/             # Completed plans
├── notes/
│   ├── INDEX.md             # Note listing (auto-generated)
│   ├── items/               # Note files
│   │   └── NOTE-vt3k8.md
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
id: TASK-rm6d3
title: "Add dark mode"
status: in-progress
priority: high
type: feature
effort: medium
tags: ["ui", "theming"]
epic: EPIC-gc8t5
plan: null
depends_on: [TASK-wp7v2]
blocks: [TASK-bg8t1]
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
| `id` | string | yes | Unique identifier, e.g. `TASK-rm6d3` |
| `title` | string | yes | Item title (max 500 characters) |
| `status` | enum | yes | Current workflow status |
| `priority` | enum | yes | Priority level |
| `type` | enum | yes | Item classification |
| `effort` | enum | yes | Effort estimate |
| `tags` | string[] | no | Categorization tags (default: `[]`) |
| `epic` | string? | no | Parent epic ID, e.g. `EPIC-gc8t5` (default: `null`) |
| `plan` | string? | no | Linked plan ID, e.g. `PLAN-rj9d4` (default: `null`) |
| `depends_on` | string[] | no | IDs this item depends on (default: `[]`) |
| `blocks` | string[] | no | IDs this item blocks (default: `[]`) |
| `assignee` | string? | no | Assigned user (default: `null`) |
| `created` | date | yes | Creation date (`YYYY-MM-DD`) |
| `updated` | date | yes | Last modification date (`YYYY-MM-DD`) |

### Epic (prefix: `EPIC`)

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | string | yes | Unique identifier, e.g. `EPIC-gc8t5` |
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
| `id` | string | yes | Unique identifier, e.g. `PLAN-rj9d4` |
| `title` | string | yes | Plan title (max 500 characters) |
| `status` | enum | yes | Current workflow status |
| `implements` | string[] | no | Task IDs this plan implements (default: `[]`) |
| `epic` | string? | no | Parent epic ID (default: `null`) |
| `created` | date | yes | Creation date (`YYYY-MM-DD`) |
| `updated` | date | yes | Last modification date (`YYYY-MM-DD`) |

### Note (prefix: `NOTE`)

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | string | yes | Unique identifier, e.g. `NOTE-dq6m1` |
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

IDs follow the format `PREFIX-RANDOM` where:
- **PREFIX** is one of: `EPIC`, `TASK`, `PLAN`, `NOTE`
- **RANDOM** is a 5-character random alphanumeric suffix (e.g., `4ed4i`, `k7x9m`, `rm6d3`)

The random suffix uses a reduced 32-character alphabet (a-z, 0-9 minus ambiguous chars `0`, `o`, `1`, `l`), giving ~33 million combinations per prefix.

Rules:
- IDs are **permanent** — once assigned, never reused
- IDs are **randomly generated** — no shared counter, safe for concurrent processes and parallel git branches
- The prefix determines the **directory**: `EPIC` → `roadmap/`, `TASK` → `backlog/`, `PLAN` → `plans/`, `NOTE` → `notes/`
- The filename is always `{ID}.md` (e.g., `TASK-rm6d3.md`)
- ID parsing is case-insensitive (`task-rm6d3` resolves to `TASK-rm6d3`)
- New IDs are collision-checked against existing items before assignment

## Cross-Reference Syntax

Use double-bracket wiki-link syntax to reference items:

```markdown
This feature depends on [[TASK-wp7v2]] and relates to [[EPIC-gc8t5]].
See [[PLAN-rj9d4]] for the implementation plan.
```

References work in both the markdown body and are also extracted from frontmatter fields (`epic`, `plan`, `depends_on`, `blocks`, `implements`, `related`).

Reference rules:
- Must contain a valid ID matching `PREFIX-RANDOM` format (e.g. `TASK-4ed4i`)
- Cannot span newlines
- Only the four valid prefixes (`EPIC`, `TASK`, `PLAN`, `NOTE`) are recognized
- Invalid references (e.g., `[[INVALID-x7k2f]]`) are ignored during extraction

Use `markplane check` to validate all references and `markplane check --orphans` to find items with no incoming references.

## config.yaml Schema

```yaml
version: 1
project:
  name: "My Project"
  description: "Project description"
context:
  token_budget: 1000      # Target token budget for summary
  recent_days: 7          # Days to consider "recent" for completions
  auto_generate: true     # Auto-regenerate context on sync
documentation_paths:            # Paths to project docs (relative to repo root)
  - docs                        # Scanned for *.md files; linked in INDEX and .context/
```

IDs are randomly generated (no counter in config). See [ID System](#id-system) for details.

### documentation_paths

Bridges Markplane's navigation and AI context layer to your repo's existing documentation (architecture docs, API references, user guides, etc.).

```yaml
documentation_paths:
  - docs
  - design/specs
```

Paths are relative to the repo root. Each path is scanned for `*.md` files during `markplane sync`, which then:

- Adds a **"Project Documentation"** section to the root `INDEX.md` with links to each discovered file
- Adds a **"Key Documentation"** section to `.context/summary.md` so AI coding tools see your docs alongside project state

This keeps technical documentation in its conventional repo location (`docs/`, etc.) while making it discoverable through Markplane. Markplane handles PM concerns (epics, tasks, plans, notes); `documentation_paths` connects everything else without moving files around.

## Template System

Templates are embedded in the `markplane-core` binary as Rust string constants. They use `{PLACEHOLDER}` tokens that are replaced at creation time by `render_template()`.

### Available Placeholders

| Placeholder | Used In | Description |
|-------------|---------|-------------|
| `{ID}` | All | Item ID (e.g., `TASK-rm6d3`) |
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

INDEX.md files are auto-generated by `markplane sync`. They contain a `<!-- Generated by markplane sync -->` header comment. They are **gitignored** within `.markplane/` (along with `.context/`) because they are fully derived from source files — regenerating them after a merge or clone is instant via `markplane sync`. This prevents merge conflicts on derived data.

Sync runs automatically on `markplane init`, `markplane mcp` startup, and `markplane serve` startup, so these files are always available when needed.

Each directory has an index:

- **Root INDEX.md**: Quick navigation table with active item counts, last sync date
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
