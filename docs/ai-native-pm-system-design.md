> **Note**: This is the original design rationale document, written before implementation began. It captures the vision, design thinking, and decisions that shaped the project. For current implementation details, see [Architecture](architecture.md). For file format specifications, see [File Format Reference](file-format.md).

# Markplane: AI-Native Project Management System Design

**Date**: 2026-02-09
**Status**: Design Proposal

---

## 1. Name & Concept

### Name: **Markplane**

*Mark* (markdown) + *plane* (control plane). In infrastructure, the control plane is the layer that manages and orchestrates a system — it doesn't do the work itself, it directs it. Markplane is the control plane for your project: the management layer that AI agents and humans consume to understand state, make decisions, and direct work.

**Tagline**: "Your repo is your project manager."

**Visual Identity**: The airplane serves as a playful visual motif — logo, loading animations, CLI spinners — while the name's meaning stays rooted in the control plane concept.

**Philosophy**: Markplane treats your repository's docs directory as a first-class project management system. No database. No SaaS. No vendor lock-in. Files are the source of truth, git is the changelog, and AI reads them natively.

### Design Principles

1. **Markdown-first**: Works with any text editor. No special tooling required to read or edit.
2. **AI-optimized**: Structured for LLM context windows. Individual files stay under 2,000 tokens. Index files serve as routers.
3. **Git-native**: Every change is a commit. Diffs are meaningful. PRs can include PM state changes alongside code.
4. **Human-readable**: Makes complete sense without tooling. A new developer can navigate the system by browsing files.
5. **Lightweight**: Minimal overhead. Creating a work item is creating a markdown file.
6. **Composable**: Use only the modules you need. Start with just a backlog. Add roadmap later. Add AI context when ready.
7. **Progressive**: Conventions work without tooling. CLI enhances the experience. Web UI is optional.

---

## 2. Directory Structure

```
.markplane/                          # Root (inside repo, version controlled)
├── config.yaml                      # System configuration
├── INDEX.md                         # AI Router: master index of the entire system
│
├── roadmap/                         # Epics & phases
│   ├── INDEX.md                     # Roadmap overview + phase status summary
│   ├── EPIC-001-foundation.md       # Individual epic files
│   ├── EPIC-002-auth-services.md
│   └── archive/                     # Completed epics
│
├── backlog/                         # Work items
│   ├── INDEX.md                     # Backlog overview: counts by status, priority
│   ├── BACK-001.md                  # Individual backlog items
│   ├── BACK-002.md
│   ├── BACK-003.md
│   └── archive/                     # Completed/cancelled items
│       └── BACK-000.md
│
├── plans/                           # Implementation plans
│   ├── INDEX.md                     # Active plans summary
│   ├── PLAN-001-feature-name.md     # Individual plan files
│   ├── PLAN-002-refactor-name.md
│   ├── archive/                     # Completed plans
│   │   ├── INDEX.md                 # Archive catalog
│   │   └── PLAN-000-old-plan.md
│   └── templates/                   # Plan templates
│       ├── implementation.md
│       └── refactor.md
│
├── notes/                           # Research, ideas, analysis, drafts
│   ├── INDEX.md                     # Notes overview
│   ├── NOTE-001-research-topic.md   # Individual notes
│   ├── NOTE-002-analysis.md
│   ├── ideas.md                     # Quick capture (no ID needed)
│   ├── decisions.md                 # Lightweight decision log
│   └── archive/                     # Archived notes
│
├── kb/                              # Knowledge base (architectural docs)
│   ├── INDEX.md                     # KB overview & navigation
│   ├── architecture.md              # System architecture
│   ├── api-contracts.md             # API specifications
│   ├── features/                    # Feature documentation
│   │   ├── feature-a.md
│   │   └── feature-b.md
│   └── guides/                      # How-to guides
│       ├── development-setup.md
│       └── deployment.md
│
├── templates/                       # Document templates
│   ├── backlog-item.md
│   ├── epic.md
│   ├── plan-implementation.md
│   ├── plan-refactor.md
│   ├── note-research.md
│   └── note-analysis.md
│
└── .context/                        # AI context layer (generated, gitignored optional)
    ├── summary.md                   # Full project summary (~1000 tokens)
    ├── active-work.md               # Currently in-progress items
    ├── blocked-items.md             # Items needing attention
    └── metrics.md                   # Project health metrics
```

### Why This Structure

- **Flat files with IDs**: Each work item is its own file. This keeps files small (AI-friendly), allows git to track individual items, and makes cross-referencing precise.
- **INDEX.md files**: Every directory has an INDEX.md that serves as a router for AI agents. An AI reads the index, determines which files are relevant, and loads only those files.
- **Archive subdirectories**: Completed work moves to `archive/` directories. This keeps active directories clean while preserving history.
- **Separation of concerns**: Roadmap (strategic), backlog (tactical), plans (implementation), notes (exploratory), kb (reference) each have clear purposes.
- **`.context/` directory**: Generated summaries that compress the project state for AI consumption. Can be gitignored (regenerated from source files) or committed (for AI tools that can't run scripts).

### Relationship to Existing `docs/`

Markplane can coexist with or replace an existing `docs/` directory:

- **Coexist**: `.markplane/` handles PM concerns, `docs/` keeps external documentation (READMEs, API docs, user guides)
- **Replace**: For projects where `docs/` already contains PM content, migrate the PM-relevant files into `.markplane/` and keep `docs/` for pure documentation
- **Hybrid**: `.markplane/kb/` can symlink to or reference existing architectural docs

---

## 3. ID System & Cross-Referencing

### ID Format

```
{PREFIX}-{NUMBER}
```

| Prefix | Entity | Example |
|--------|--------|---------|
| `EPIC` | Roadmap epic/phase | `EPIC-001` |
| `BACK` | Backlog item | `BACK-042` |
| `PLAN` | Implementation plan | `PLAN-015` |
| `NOTE` | Note/research/analysis | `NOTE-007` |

### Rules

1. **IDs are permanent**: Once assigned, an ID never changes or gets reused.
2. **IDs are sequential**: New items get the next available number. Gaps are OK (deleted items leave gaps).
3. **IDs map to filenames**: `BACK-042` lives at `.markplane/backlog/BACK-042.md` (or `.markplane/backlog/archive/BACK-042.md` if archived).
4. **Cross-reference syntax**: Use `[[BACK-042]]` to reference any item from any document. This works because:
   - The prefix tells you the type (and therefore the directory)
   - The number tells you the file
   - Tools can resolve these to paths; humans can navigate manually

### Cross-Reference Examples

In a backlog item:
```markdown
## Related
- Epic: [[EPIC-003]]
- Plan: [[PLAN-012]]
- See also: [[NOTE-005]], [[BACK-039]]
```

In a plan:
```markdown
## Overview
This plan implements [[BACK-042]] and [[BACK-043]], part of [[EPIC-003]].
```

In a note:
```markdown
Research findings relevant to [[BACK-042]]. May also impact [[EPIC-005]].
```

### Referential Integrity

The CLI can validate references:
```bash
markplane check         # Find broken references
markplane orphans       # Find items with no incoming references
markplane graph BACK-042  # Show reference graph for an item
```

---

## 4. Backlog Item Format

Each backlog item is a standalone markdown file with YAML frontmatter.

### Template: `.markplane/templates/backlog-item.md`

```markdown
---
id: BACK-{NUMBER}
title: "{Title}"
status: draft                    # draft | backlog | planned | in-progress | done | cancelled
priority: medium                 # critical | high | medium | low | someday
type: feature                    # feature | bug | enhancement | chore | research | spike
effort: medium                   # xs (< 1h) | small (< 4h) | medium (< 1d) | large (< 1w) | xl (1w+)
tags: []                         # freeform tags for filtering
epic: null                       # e.g., EPIC-003
plan: null                       # e.g., PLAN-012
depends_on: []                   # e.g., [BACK-039, BACK-040]
blocks: []                       # e.g., [BACK-045]
assignee: null                   # person or team
created: 2026-02-09
updated: 2026-02-09
---

# {Title}

## Description

[What needs to be done and why. 2-5 sentences.]

## Acceptance Criteria

- [ ] Criterion 1
- [ ] Criterion 2
- [ ] Criterion 3

## Notes

[Additional context, research links, implementation hints, related discussions.]

## References

- Related: [[NOTE-005]]
- Depends on: [[BACK-039]]
```

### Example: `BACK-042.md`

```markdown
---
id: BACK-042
title: "Add dark mode support to dashboard"
status: backlog
priority: high
type: feature
effort: medium
tags: [ui, theming, accessibility]
epic: EPIC-003
plan: null
depends_on: [BACK-038]
blocks: [BACK-045]
assignee: null
created: 2026-01-15
updated: 2026-02-09
---

# Add dark mode support to dashboard

## Description

Implement system-aware dark mode for the main dashboard. Should respect OS preference by default with manual toggle. Theme variables are already defined in the design system but not wired up to component library.

## Acceptance Criteria

- [ ] Dashboard respects OS dark mode preference on load
- [ ] Manual toggle in settings persists across sessions
- [ ] All core components render correctly in both modes
- [ ] Contrast ratios meet WCAG 2.2 AA standards

## Notes

Design system already has CSS custom properties for both themes. Main work is wiring the toggle and auditing component-level overrides.

## References

- Epic: [[EPIC-003]]
- Depends on: [[BACK-038]] (design system token finalization)
- Blocks: [[BACK-045]] (themed email templates)
```

### Status Workflow

```
draft ──→ backlog ──→ planned ──→ in-progress ──→ done ──→ (archive)
  │                       │                          │
  └──→ cancelled          └──→ cancelled             └──→ cancelled
```

- **draft**: Idea captured but not yet vetted. May be incomplete.
- **backlog**: Vetted and accepted as real work. Ready to be prioritized.
- **planned**: Has an associated implementation plan ([[PLAN-xxx]]). Ready to start.
- **in-progress**: Actively being worked on.
- **done**: Completed. Will be moved to `archive/` during cleanup.
- **cancelled**: Will not be done. Moved to `archive/` with reason in notes.

---

## 5. Epic / Roadmap Format

Epics represent high-level phases or themes of work. They group backlog items and provide strategic tracking.

### Template: `.markplane/templates/epic.md`

```markdown
---
id: EPIC-{NUMBER}
title: "{Phase/Epic Name}"
status: active                   # planned | active | done | paused
priority: high                   # critical | high | medium | low
started: null                    # date work began
target: null                     # target completion date
tags: []
depends_on: []                   # e.g., [EPIC-001]
---

# {Phase/Epic Name}

## Objective

[2-3 sentences: What does this epic achieve? Why does it matter?]

## Key Results

- [ ] KR1: [Measurable outcome]
- [ ] KR2: [Measurable outcome]
- [ ] KR3: [Measurable outcome]

## Backlog Items

| ID | Title | Status | Priority | Effort |
|----|-------|--------|----------|--------|
| [[BACK-042]] | Event-Based Cache Invalidation | backlog | high | medium |
| [[BACK-043]] | Multi-org Support | draft | medium | large |

## Progress

<!-- Updated by CLI or manually -->
- Total items: 12
- Done: 5 (42%)
- In progress: 2
- Remaining: 5

## Notes

[Strategic context, dependencies on external work, risks.]
```

### Progress Auto-Calculation

The CLI can regenerate the Progress section by scanning backlog items that reference this epic:

```bash
markplane sync EPIC-003   # Update progress from linked backlog items
markplane sync --all      # Update all epics
```

---

## 6. Plan Format

Plans are detailed implementation documents linked to one or more backlog items.

### Markplane preserves the existing plan templates

Markplane's plan format builds on common implementation plan templates with one key addition: YAML frontmatter for structured metadata that enables cross-referencing and lifecycle tracking.

```markdown
---
id: PLAN-{NUMBER}
title: "{Feature/Refactor Name}"
status: draft                    # draft | approved | in-progress | done
implements: [BACK-042, BACK-043] # backlog items this plan addresses
epic: EPIC-003
created: 2026-02-09
updated: 2026-02-09
---

# {Feature Name} Implementation Plan

[Rest of plan follows existing template structure...]
```

### Plan Lifecycle

```
draft ──→ approved ──→ in-progress ──→ done ──→ (archive)
```

- **draft**: Plan is being designed. May be reviewed in PR.
- **approved**: Plan reviewed and accepted. Backlog items move to `planned`.
- **in-progress**: Implementation underway. Backlog items move to `in-progress`.
- **done**: Implementation complete. Plan moves to `archive/`.

---

## 7. Notes Format

Notes are lightweight documents for ideas, research, analysis, and meeting notes. They have a minimal frontmatter and flexible structure.

```markdown
---
id: NOTE-{NUMBER}
title: "{Topic}"
type: research                   # research | analysis | idea | decision | meeting
status: active                   # draft | active | archived
tags: [topic-a, topic-b]
related: [BACK-042, PLAN-012]   # links to relevant items
created: 2026-02-09
updated: 2026-02-09
---

# {Topic}

[Freeform content. No required structure beyond frontmatter.]
```

### Special Note Files (No ID Required)

Some notes are ongoing, living documents that don't need individual IDs:

- **`ideas.md`**: Quick capture file. Bullet points for things that aren't backlog items yet.
- **`decisions.md`**: Lightweight decision log. `## YYYY-MM-DD: Decision Title` format.

### Promotion Workflow

Notes can be promoted to backlog items:

```bash
markplane promote NOTE-007     # Creates BACK-XXX from NOTE-007, links them
```

This creates a new backlog item pre-populated from the note, and adds a reference back.

---

## 8. AI Context Layer

The `.context/` directory contains generated summaries optimized for AI consumption. These files compress the project state into token-efficient formats.

### `summary.md` - The AI Entry Point (~1000 tokens)

```markdown
# Project: My SaaS Platform
<!-- Generated by markplane context --all | 2026-02-09 -->

## Active Epics
- EPIC-003: User Dashboard & Theming (67% complete, 4 items remaining)
- EPIC-005: API & Performance (30% complete, 7 items remaining)

## In-Progress Work
- BACK-042: Add dark mode support (high, @daniel)
- BACK-055: API response caching (medium, @daniel)
- PLAN-012: Dark mode implementation (Phase 2 of 3)

## Blocked Items
- BACK-045: Themed email templates (blocked by BACK-042)

## Recent Completions (last 7 days)
- BACK-041: Fix pagination on user list (done 2026-02-08)
- PLAN-011: Pagination refactor (done 2026-02-08)

## Priority Queue (next up)
1. BACK-043: Add search to dashboard (high, planned)
2. BACK-044: Export reports to CSV (high, backlog)
3. BACK-046: Optimize database queries (medium, backlog)

## Key Metrics
- Backlog: 47 items (8 critical, 15 high, 18 medium, 6 low)
- Velocity: ~5 items/week (last 4 weeks)
- Oldest open item: BACK-003 (45 days, low priority)
```

### `active-work.md` - Current Sprint Focus (~500 tokens)

Details on what's actively being worked on. Links to plans and relevant context files for each item.

### Router Pattern

When an AI agent needs context about the project:

1. **Read `.markplane/INDEX.md`** - Understand the system structure
2. **Read `.markplane/.context/summary.md`** - Get project state overview
3. **Based on task, read specific files** - e.g., if working on auth, read `EPIC-003.md`, `BACK-042.md`, `PLAN-012.md`

This three-step pattern keeps total context under ~4,000 tokens for most tasks, well within the effective window identified by Stanford research.

### Context Generation

```bash
markplane context              # Regenerate all .context/ files
markplane context --focus auth  # Generate focused context for auth-related work
markplane context --item BACK-042  # Generate deep context for a specific item
```

The `--item` variant pulls in: the item itself, its epic, its plan, its dependencies, its blockers, and relevant notes. This is the "everything an AI needs to work on this item" bundle.

---

## 9. INDEX.md Files

Every directory has an INDEX.md that serves as a table of contents and routing guide.

### Root INDEX.md

```markdown
# Markplane Project Index
<!-- This file is the entry point for AI agents and human navigation -->

## Quick Navigation

| Module | Path | Purpose | Active Items |
|--------|------|---------|-------------|
| Roadmap | [roadmap/](roadmap/INDEX.md) | Strategic phases & epics | 3 active epics |
| Backlog | [backlog/](backlog/INDEX.md) | All work items | 47 open items |
| Plans | [plans/](plans/INDEX.md) | Implementation details | 4 active plans |
| Notes | [notes/](notes/INDEX.md) | Research & ideas | 12 active notes |
| Knowledge Base | [kb/](kb/INDEX.md) | Architecture & guides | 15 documents |
| AI Context | [.context/](.context/summary.md) | Generated summaries | Auto-updated |

## System Info
- ID counter: EPIC-008, BACK-063, PLAN-019, NOTE-023
- Last sync: 2026-02-09
- Config: [config.yaml](config.yaml)
```

### Backlog INDEX.md

```markdown
# Backlog Index
<!-- Generated by markplane sync -->

## By Status

### In Progress (2)
- [[BACK-042]] Add dark mode support (high, @daniel)
- [[BACK-055]] API response caching (medium, @daniel)

### Planned (3)
- [[BACK-043]] Add search to dashboard (high, PLAN-013)
- [[BACK-044]] Export reports to CSV (high, PLAN-014)
- [[BACK-048]] Form validation enhancements (medium, PLAN-015)

### Backlog (15)
- [[BACK-046]] Optimize database queries (medium)
- [[BACK-047]] User profile customization (low)
[... truncated for brevity ...]

### Draft (8)
- [[BACK-060]] Custom avatar file upload (medium)
[... truncated for brevity ...]

## By Priority
- Critical: 2 items
- High: 12 items
- Medium: 22 items
- Low: 8 items
- Someday: 3 items

## By Epic
- [[EPIC-003]]: 8 items (5 done, 2 in-progress, 1 planned)
- [[EPIC-005]]: 12 items (3 done, 1 in-progress, 8 backlog)
[...]
```

---

## 10. Configuration

### `.markplane/config.yaml`

```yaml
# Markplane Configuration
version: 1

project:
  name: "My Project"
  description: "Project description"

# ID counters (managed by CLI, can be edited manually)
counters:
  EPIC: 8
  BACK: 63
  PLAN: 19
  NOTE: 23

# Status workflows (customizable)
workflows:
  backlog:
    statuses: [draft, backlog, planned, in-progress, done, cancelled]
    default: draft
    archive_on: [done, cancelled]
  epic:
    statuses: [planned, active, done, paused]
    default: planned
    archive_on: [done]
  plan:
    statuses: [draft, approved, in-progress, done]
    default: draft
    archive_on: [done]
  note:
    statuses: [draft, active, archived]
    default: draft
    archive_on: [archived]

# Priority levels (customizable)
priorities: [critical, high, medium, low, someday]

# Effort sizes
effort_sizes: [xs, small, medium, large, xl]

# Context generation settings
context:
  token_budget: 1000           # Target tokens for summary.md
  recent_days: 7               # "Recent completions" window
  auto_generate: true          # Regenerate .context/ on sync

# Archive settings
archive:
  auto_archive_after_days: 30  # Auto-archive done items after N days
  keep_cancelled: true         # Archive cancelled items (vs delete)
```

---

## 11. Lifecycle Management

### The Idea-to-Archive Flow

```
 ┌─────────┐     ┌──────────┐     ┌───────────┐     ┌────────────┐
 │  Ideas   │────▶│  Backlog  │────▶│   Plans    │────▶│   Archive  │
 │ (notes)  │     │  (items)  │     │  (detail)  │     │ (history)  │
 └─────────┘     └──────────┘     └───────────┘     └────────────┘
   ideas.md        BACK-xxx.md      PLAN-xxx.md       archive/
   NOTE-xxx.md     status: draft→   status: draft→
                   backlog→planned   approved→done
                   →in-progress→done
```

### Concrete Workflow

1. **Capture**: Developer has an idea → adds bullet to `notes/ideas.md` or creates `NOTE-xxx.md`
2. **Promote**: Idea is worth pursuing → `markplane promote NOTE-007` → creates `BACK-063`
3. **Triage**: Backlog item gets priority and effort estimates → status moves to `backlog`
4. **Plan**: Complex item needs a plan → `markplane plan BACK-063` → creates `PLAN-019`, links them, status moves to `planned`
5. **Execute**: Implementation begins → status moves to `in-progress`
6. **Complete**: Work done, acceptance criteria met → status moves to `done`
7. **Archive**: After configured period → item moves to `archive/` directory

### Staleness Prevention

```bash
markplane stale              # List items not updated in 30+ days
markplane stale --days 14    # Custom threshold
```

Items flagged as stale appear in `.context/summary.md` as needing attention.

---

## 12. CLI Tool Design

### Installation

```bash
# As a standalone tool
brew install markplane       # macOS
cargo install markplane      # From source (Rust for speed)

# Or as a project dependency
pnpm add -D markplane        # Node projects
pip install markplane         # Python projects
```

### Core Commands

```bash
# Initialize
markplane init                    # Create .markplane/ structure in current repo

# Create items
markplane add                     # Interactive: create backlog item
markplane add "Fix login bug" --type bug --priority high
markplane epic "Phase 5: Observability"
markplane plan BACK-042           # Create plan linked to backlog item
markplane note "Research: caching strategies" --type research

# View & navigate
markplane ls                      # List active backlog items
markplane ls --status in-progress # Filter by status
markplane ls --epic EPIC-003      # Filter by epic
markplane ls --priority critical,high  # Filter by priority
markplane show BACK-042           # Display item details
markplane graph BACK-042          # Show dependency graph

# Update
markplane status BACK-042 in-progress  # Change status
markplane assign BACK-042 @daniel      # Assign item
markplane tag BACK-042 cache,permissions  # Add tags
markplane link BACK-042 --blocks BACK-045  # Add dependency

# Workflow shortcuts
markplane start BACK-042          # Set to in-progress + assign to current user
markplane done BACK-042           # Set to done
markplane promote NOTE-007        # Note → backlog item

# Project health
markplane status                  # Dashboard: summary of project state
markplane stale                   # Items needing attention
markplane check                   # Validate references, find broken links
markplane metrics                 # Velocity, burndown, age distribution

# AI context
markplane context                 # Regenerate .context/ files
markplane context --focus auth    # Focused context for a domain
markplane context --item BACK-042 # Deep context for specific item
markplane context --clipboard     # Copy context to clipboard for AI chat

# Sync & maintenance
markplane sync                    # Update all INDEX.md files, epic progress
markplane archive                 # Move done/cancelled items to archive
markplane archive --dry-run       # Preview what would be archived
```

### AI Agent Integration

The CLI is designed so AI coding agents can use it directly:

```bash
# AI agent reads project context
markplane context --item BACK-042 --format markdown

# AI agent creates a plan
markplane plan BACK-042 --title "Cache invalidation implementation"

# AI agent updates status
markplane done BACK-042

# AI agent creates follow-up items
markplane add "Update documentation for cache changes" --type chore --epic EPIC-003
```

### CLAUDE.md / AGENTS.md Integration

Markplane generates a snippet for your `CLAUDE.md`:

```bash
markplane claude-md     # Output snippet to add to CLAUDE.md
```

Output:
```markdown
## Project Management
This project uses Markplane for project management. Key files:
- `.markplane/INDEX.md` - Navigation entry point
- `.markplane/.context/summary.md` - Current project state
- `.markplane/backlog/INDEX.md` - All work items
- `.markplane/plans/INDEX.md` - Implementation plans
When working on a task, read the relevant backlog item and its linked plan first.
```

---

## 13. Web UI Concept (Optional Enhancement)

A lightweight web UI that reads/writes the same markdown files. Not a replacement for the CLI—an additional interface.

### Key Features

- **Kanban board**: Drag items between status columns
- **Roadmap timeline**: Visual epic/phase progress
- **Dependency graph**: Interactive visualization of item relationships
- **Search & filter**: Full-text search with faceted filtering
- **Real-time sync**: Watches filesystem for changes (like a local dev server)

### Implementation Approach

```bash
markplane serve              # Start local web UI on http://localhost:4200
markplane serve --port 8080  # Custom port
```

- **Read**: Parse markdown + YAML frontmatter from `.markplane/` directory
- **Write**: Generate markdown files with proper frontmatter
- **No database**: The filesystem IS the database
- **Git integration**: Show git blame for item history, create commits for changes

### Technical Stack (if built as software)

- **Backend**: Rust (fast file I/O, markdown parsing)
- **Frontend**: React + Tailwind (consistent with modern dev tools)
- **File watching**: `notify` crate for filesystem events
- **Markdown parsing**: `pulldown-cmark` or `comrak`
- **YAML**: `serde_yaml`

---

## 14. Migration Guide

Most projects already have some form of documentation — a monolithic TASK.md, a flat BACKLOG.md, a `docs/plans/` directory, etc. Markplane provides migration patterns for common starting points.

### Common Migration Patterns

| Current Pattern | Markplane Migration | Approach |
|-----------------|-------------------|----------|
| Monolithic roadmap file (TASK.md, ROADMAP.md) | `roadmap/EPIC-xxx.md` files | Split by phase/epic, add frontmatter, archive completed phases |
| Flat backlog file (BACKLOG.md, TODO.md) | `backlog/BACK-xxx.md` files | Extract each item, assign IDs, add structured frontmatter |
| Plans directory (`docs/plans/`) | `plans/PLAN-xxx.md` files | Add frontmatter with `implements` links to backlog items |
| Notes / drafts / research | `notes/NOTE-xxx.md` files | Add IDs and frontmatter with type field |
| Architecture docs (`docs/blueprint/`, `docs/architecture/`) | `kb/` knowledge base | Move or symlink, add INDEX.md for navigation |
| Ideas / brainstorming files | `notes/ideas.md` | Keep as quick-capture, promote items to backlog with `markplane promote` |
| Plan templates | `templates/` | Expand with backlog, epic, and note templates |

### Migration CLI

```bash
markplane migrate --source docs/ --analyze  # Dry run: show what would be migrated
markplane migrate --source docs/            # Execute migration
```

### Phased Migration Approach

1. **Phase 1**: Create `.markplane/` structure, config, templates, INDEX.md files. Keep existing docs in place.
2. **Phase 2**: Migrate backlog items to individual files with frontmatter. Highest-value change.
3. **Phase 3**: Split monolithic roadmap into epic files. Archive completed phases.
4. **Phase 4**: Add frontmatter to existing plans, link to backlog items. Migrate notes with IDs.
5. **Phase 5**: Generate `.context/` summaries. Update CLAUDE.md/AGENTS.md with Markplane integration.

---

## 15. Comparison: Before & After

### Before (Typical Docs-Based PM)

**Finding a backlog item:**
1. Open a single large backlog file (could be 100+ items)
2. Ctrl+F to search
3. No structured metadata, no status tracking, no IDs
4. Cross-references are informal text mentions

**AI context for a task:**
1. AI reads full roadmap file (5,000-20,000+ tokens, most irrelevant)
2. AI reads full backlog file (another 5,000-10,000+ tokens, most irrelevant)
3. AI guesses which plan files might be relevant
4. Total context: 10,000+ tokens, low signal-to-noise

**Tracking progress:**
1. Manual checkbox counting in roadmap file
2. No per-item status tracking in backlog
3. No velocity metrics, no staleness detection

### After (Markplane)

**Finding a backlog item:**
1. `markplane show BACK-042` or open `.markplane/backlog/BACK-042.md`
2. Full metadata in frontmatter: status, priority, epic, plan, dependencies
3. Cross-references via `[[BACK-042]]` syntax

**AI context for a task:**
1. AI reads `.markplane/.context/summary.md` (~1,000 tokens)
2. AI reads `BACK-042.md` (~300 tokens) and `PLAN-012.md` (~800 tokens)
3. Total context: ~2,100 tokens, high signal-to-noise
4. Or: `markplane context --item BACK-042` generates a focused bundle

**Tracking progress:**
1. `markplane status` shows dashboard
2. Epic progress auto-calculated from child items
3. Velocity tracked from `done` date timestamps
4. Stale items flagged automatically

---

## 16. Design Decisions & Rationale

| Decision | Rationale |
|----------|-----------|
| Individual files per backlog item | AI context windows degrade with large files (Stanford: >32k tokens). One item = one focused file (~300 tokens). |
| YAML frontmatter over inline metadata | Structured, parseable, consistent. YAML is the standard for markdown metadata. |
| `[[ID]]` cross-reference syntax | Familiar (wiki-style), unambiguous, tooling-friendly. Prefix determines location. |
| `.markplane/` directory name | Dot-prefix keeps it out of the way. Clear branding. Version controlled by default. |
| Flat file structure (not nested by epic) | Simpler navigation, no moving files when epic changes, IDs are globally unique. |
| Archive directories (not deletion) | Preserves history. Git tracks moves. AI can reference completed work for context. |
| INDEX.md as router pattern | AI reads index (~200 tokens), decides what else to load. Prevents loading entire directory. |
| Config in YAML (not JSON) | Human-readable, supports comments, standard for config files. |
| Rust for CLI (proposed) | Fast startup, single binary, excellent file I/O. CLI tools need to be instant. |
| Optional `.context/` generation | Works without it (humans read files directly). AI gets value from compressed summaries. |
| Progressive adoption | Start with just `backlog/`. Add `roadmap/` when you have phases. Add `.context/` when AI usage is heavy. |

---

## 17. Token Budget Analysis

Based on the Stanford "lost-in-the-middle" research and Manus context engineering patterns:

| Document Type | Target Size | Rationale |
|--------------|-------------|-----------|
| Backlog item | ~200-400 tokens | One focused concern. Read several without budget pressure. |
| Epic | ~400-600 tokens | Strategic overview + item table. |
| Plan | ~800-2,000 tokens | Detailed but phased. AI reads relevant phase only. |
| Note | ~200-1,000 tokens | Varies. Research can be longer. |
| INDEX.md | ~200-500 tokens | Routing info only. Minimal prose. |
| `.context/summary.md` | ~1,000 tokens | The "RAM" for project state. |
| Full task context | ~2,000-3,000 tokens | Item + plan + epic + dependencies. Sweet spot for AI. |

**Comparison with typical monolithic approach:**
- Single roadmap file: ~5,000-20,000 tokens (all phases mixed, completed work included)
- Single backlog file: ~5,000-10,000+ tokens (all items mixed, no filtering)
- Total for "understand project state": ~10,000-30,000 tokens
- **Markplane equivalent**: ~1,000 tokens (summary.md) + ~300 tokens per relevant item

---

## 18. MCP Server Integration

The MCP (Model Context Protocol) server is the native integration layer for AI coding tools. Instead of AI reading raw markdown files, it gets structured, typed access to the project management system through purpose-built tools.

### Why MCP

MCP is the emerging standard for how AI tools connect to external systems (adopted by Claude Code, Cursor, Windsurf, and others). An MCP server turns Markplane from "files the AI can read" into "a live project management API the AI uses natively."

| Integration Level | How AI Interacts | Fidelity |
|-------------------|-----------------|----------|
| Conventions only | AI reads markdown via Glob/Read, parses manually | Low — fragile, no structured queries |
| CLI installed | AI calls CLI via Bash (`markplane ls --status in-progress`) | Medium — text output, requires parsing |
| **MCP server** | AI has native typed tools with structured input/output | High — first-class integration, auto-sync |

### MCP Tool Definitions

```typescript
// Context & Navigation
markplane_summary        // Get project state summary (~1000 tokens)
markplane_context        // Get focused context for a specific item or domain
  { item?: string, focus?: string, include_plan?: boolean }

// Query & Search
markplane_query          // Query backlog items with filters
  { status?: string[], priority?: string[], epic?: string, tags?: string[], assignee?: string }
markplane_show           // Get full details of any item
  { id: string }         // e.g., "BACK-042", "EPIC-003", "PLAN-012"
markplane_graph          // Get dependency graph for an item
  { id: string, depth?: number }

// Create & Update
markplane_add            // Create a new backlog item
  { title: string, type?: string, priority?: string, epic?: string, tags?: string[], depends_on?: string[] }
markplane_update         // Update item fields
  { id: string, status?: string, priority?: string, assignee?: string, tags?: string[] }
markplane_start          // Set item to in-progress + assign
  { id: string }
markplane_done           // Mark item complete
  { id: string }

// Workflow
markplane_promote        // Promote a note to a backlog item
  { note_id: string }
markplane_plan           // Create a plan linked to a backlog item
  { backlog_id: string, title: string }
markplane_link           // Link items together
  { from: string, to: string, relation: "blocks" | "depends_on" | "related" }

// Maintenance
markplane_sync           // Update all INDEX.md files and .context/ summaries
markplane_check          // Validate cross-references, find broken links
markplane_stale          // List items not updated in N days
  { days?: number }
```

### Example: AI Autonomous Workflow via MCP

```
1. AI calls markplane_summary → understands project state
2. AI calls markplane_query { status: ["backlog"], priority: ["critical", "high"] }
     → gets prioritized list of available work
3. AI calls markplane_start { id: "BACK-042" }
     → claims the item, status moves to in-progress
4. AI calls markplane_context { item: "BACK-042", include_plan: true }
     → gets item + plan + dependencies as focused context
5. AI implements the feature using code tools
6. AI calls markplane_done { id: "BACK-042" }
     → marks complete, INDEX.md and .context/ auto-update
7. AI calls markplane_add { title: "Update docs for cache changes", epic: "EPIC-003" }
     → creates follow-up item discovered during implementation
```

No file parsing. No convention knowledge required. The AI interacts with Markplane as a structured API.

### MCP Resources (Read-Only Context)

MCP also supports resources — structured data the AI can pull into context:

```
markplane://summary              # Project state summary
markplane://backlog/BACK-042     # Specific item details
markplane://epic/EPIC-003        # Epic with progress
markplane://active-work          # Currently in-progress items
markplane://blocked              # Items needing attention
```

### Implementation

The MCP server wraps the same core library as the CLI:

```
┌──────────────┐     ┌──────────────┐     ┌──────────────┐
│  CLI binary   │────▶│  Core Library │◀────│  MCP Server   │
│  (user runs)  │     │  (Rust)      │     │  (stdio/SSE)  │
└──────────────┘     └──────────────┘     └──────────────┘
                            │
                     ┌──────┴──────┐
                     │ .markplane/  │
                     │  (markdown)  │
                     └─────────────┘
```

- CLI and MCP server share the same Rust core (file parsing, YAML handling, index generation)
- MCP server runs as a stdio or SSE process, configured in the AI tool's MCP settings
- File watching enables real-time updates — if a human edits a file, the MCP server reflects changes immediately

### Configuration in AI Tools

**Claude Code** (`~/.claude/mcp.json`):
```json
{
  "mcpServers": {
    "markplane": {
      "command": "markplane",
      "args": ["mcp", "--project", "/path/to/repo"]
    }
  }
}
```

**Cursor** (`.cursor/mcp.json`):
```json
{
  "mcpServers": {
    "markplane": {
      "command": "markplane",
      "args": ["mcp", "--project", "."]
    }
  }
}
```

---

## 19. Future Possibilities

### Team Collaboration

For multi-developer teams:
- **Git branches**: Each developer works on items in branches. PR includes PM state changes.
- **Assignee tracking**: `assignee` field in frontmatter
- **Conflict resolution**: YAML frontmatter has clean git diffs

### Plugin System

```bash
markplane plugin install github-issues   # Sync with GitHub Issues
markplane plugin install linear          # Sync with Linear
markplane plugin install slack           # Post updates to Slack
```

Markplane as the local source of truth, with plugins for external sync.

---

## 20. Summary

Markplane is a conventions-first, tools-second approach to project management that:

1. **Uses the filesystem as the database**: Every work item is a markdown file. Git is the changelog.
2. **Optimizes for AI consumption**: Small focused files, index-based routing, generated context summaries.
3. **Works without tooling**: Any text editor can create and manage items. The CLI enhances but doesn't gate.
4. **Scales from solo to team**: Start with a backlog directory. Add epics, plans, and context generation as complexity grows.
5. **Complements existing tools**: Not a Jira replacement. The AI-facing interface for your project's state.

The name "Markplane" captures the essence: markdown as your project's control plane — the management layer that orchestrates work, consumed natively by both humans and AI.
