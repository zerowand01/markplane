# Markplane Architecture

## System Overview

```
┌──────────────────────────────────┐
│        markplane CLI binary       │
│  (clap, colored, tabled, anyhow)  │
│                                   │
│  ├── CLI subcommands              │
│  └── MCP server (markplane mcp)   │
│       (JSON-RPC 2.0, stdio)       │
└────────────────┬──────────────────┘
                 │
          ┌──────▼──────┐
          │markplane-core│
          │  (lib crate) │
          └──────┬───────┘
                 │
          ┌──────▼──────┐
          │ .markplane/  │
          │ (filesystem) │
          └──────────────┘
```

Both the CLI commands and MCP server are thin wrappers around `markplane-core`. All business logic — data models, CRUD, querying, sync, reference validation, context generation — lives in the core library.

## Crate Responsibilities

### markplane-core (library)

The core library contains all domain logic. It exposes a `Project` struct that represents a `.markplane/` directory and provides methods for every operation.

**Modules:**

| Module | Purpose |
|--------|---------|
| `models` | Entity structs (`Task`, `Epic`, `Plan`, `Note`), enums (`TaskStatus`, `Priority`, `Effort`, etc.), `Config` (including configurable `task_types`/`note_types`), `MarkplaneDocument<T>` wrapper, ID parsing/formatting |
| `project` | `Project` struct — init, config, ID management, CRUD operations, archiving, `find_blocked_items()` |
| `frontmatter` | Parse and serialize `---\nyaml\n---\nbody` format |
| `query` | `QueryFilter` struct, `list_tasks()`, `list_epics()`, `list_plans()`, `list_notes()` with filtering and sorting |
| `references` | `extract_references()` (wiki-link `[[ID]]` scanning), `validate_references()`, `find_orphans()`, `build_reference_graph()` |
| `links` | `LinkRelation` enum (blocks, depends-on, epic, plan, implements, related), `LinkAction` enum, `Project::link_items()` — centralized cross-entity linking with type validation and reciprocal management |
| `index` | INDEX.md generation for all directories (root, backlog, roadmap, plans, notes) |
| `context` | `.context/` file generation — summary, active-work, blocked-items, metrics |
| `templates` | Embedded template constants and `render_template()` placeholder replacement |
| `error` | `MarkplaneError` enum (via `thiserror`) and `Result<T>` type alias |

### markplane-cli (binary: `markplane`)

The CLI crate provides the user-facing terminal interface and the integrated MCP server.

- **Argument parsing**: `clap` with derive macros
- **Commands**: 25 subcommands — `init`, `add`, `show`, `ls`, `status`, `sync`, `start`, `done`, `promote`, `plan`, `epic`, `note`, `update`, `link`, `check`, `stale`, `archive`, `unarchive`, `context`, `metrics`, `graph`, `claude-md`, `dashboard`, `serve`, `mcp`
- **Formatting**: `commands/formatting.rs` — shared helpers for truncation, status/priority colorization (via `colored`), table output (via `tabled`)
- **MCP module** (`src/mcp/`): The `markplane mcp` subcommand runs the MCP server enabling AI tools (Claude, Cursor, etc.) to interact with the project
  - **Protocol**: JSON-RPC 2.0 over stdio (one JSON object per line)
  - **Tools**: 17 tools — `markplane_summary`, `markplane_context`, `markplane_query`, `markplane_show`, `markplane_graph`, `markplane_add`, `markplane_update`, `markplane_start`, `markplane_move`, `markplane_done`, `markplane_promote`, `markplane_plan`, `markplane_link`, `markplane_archive`, `markplane_unarchive`, `markplane_sync`, `markplane_check`
  - **Resources**: 3 static resources (`markplane://summary`, `markplane://active-work`, `markplane://blocked`) + 4 dynamic templates (`markplane://task/{id}`, `markplane://epic/{id}`, `markplane://plan/{id}`, `markplane://note/{id}`)
  - **Error handling**: Tool handlers return `Result<String, String>`; errors map to JSON-RPC error codes
- **Error handling**: `anyhow::Result` at the top level

## Data Model

### MarkplaneDocument\<T\>

All items are wrapped in a generic document type that separates YAML frontmatter from the markdown body:

```rust
pub struct MarkplaneDocument<T> {
    pub frontmatter: T,   // Deserialized YAML (Task, Epic, etc.)
    pub body: String,      // Markdown content after the closing ---
}
```

### Entity Types

```
                    ┌──────────────────────┐
                    │       Config         │
                    │ version, project,    │
                    │ context, archive     │
                    │                      │
                    └──────────────────────┘

  ┌───────────────┐  ┌───────────────┐  ┌───────────────┐  ┌───────────────┐
  │     Task      │  │     Epic      │  │     Plan      │  │     Note      │
  │ TASK-xxxxx    │  │ EPIC-xxxxx    │  │ PLAN-xxxxx    │  │ NOTE-xxxxx    │
  │               │  │               │  │               │  │               │
  │ id, title,    │  │ id, title,    │  │ id, title,    │  │ id, title,    │
  │ status,       │  │ status,       │  │ status,       │  │ type,         │
  │ priority,     │  │ priority,     │  │ implements[], │  │ status,       │
  │ type, effort, │  │ started?,     │  │ related[],    │  │ tags[],       │
  │ tags[],       │  │ target?,      │  │ created,      │  │ related[],    │
  │ epic?,        │  │ tags[],       │  │ updated       │  │ created,      │
  │ plan?,        │  │ related[],    │  │               │  │ updated       │
  │ depends_on[], │  │               │  │               │  │               │
  │ blocks[],     │  │               │  │               │  │               │
  │ related[],    │  │               │  │               │  │               │
  │ assignee?,    │  │               │  │               │  │               │
  │ created,      │  │               │  │               │  │               │
  │ updated       │  │               │  │               │  │               │
  └───────────────┘  └───────────────┘  └───────────────┘  └───────────────┘
```

Relationships between entities:
- `Task.epic` → links to an `Epic`
- `Task.plan` → links to a `Plan`
- `Task.depends_on[]` / `blocks[]` → links to other `Task`s
- `Plan.implements[]` → links to `Task`s it implements
- `*.related[]` → bidirectional links to any item type (all four entity types)

## Data Flow

### Adding an Item

```
CLI: markplane add "Fix login bug" --type bug --priority high
  │
  ├─ Parse args via clap
  ├─ Project::from_current_dir()  → find .markplane/
  ├─ validate_title_length(title) → reject if > 500 chars
  ├─ project.next_id(&IdPrefix::Task)
  │    ├─ Generate random 5-char alphanumeric suffix
  │    └─ Check for collision against existing items
  ├─ sanitize_yaml_string(title)
  ├─ render_template(TASK_TEMPLATE, vars)
  ├─ Write .markplane/backlog/items/TASK-fq2x8.md
  └─ Return Task struct
```

### Sync

```
CLI: markplane sync [--normalize]
  │
  ├─ Project::from_current_dir()
  ├─ (if --normalize) normalize_positions()
  │    └─ Rewrite fractional position keys to clean sequential ones
  ├─ sync_all_indexes()
  │    ├─ generate_root_index()      → INDEX.md
  │    ├─ generate_backlog_index()   → backlog/INDEX.md
  │    ├─ generate_roadmap_index()   → roadmap/INDEX.md
  │    ├─ generate_plans_index()     → plans/INDEX.md
  │    └─ generate_notes_index()     → notes/INDEX.md
  └─ generate_all_context()
       ├─ generate_context_summary()     → .context/summary.md
       ├─ generate_context_active_work() → .context/active-work.md
       ├─ generate_context_blocked()     → .context/blocked-items.md
       └─ generate_context_metrics()     → .context/metrics.md
```

INDEX.md files and `.context/` are **derived files** — fully regenerated from source files. They are gitignored within `.markplane/` to prevent merge conflicts on derived data. Sync also runs automatically on `markplane init`, `markplane mcp` startup, and `markplane serve` startup.

Position normalization (`--normalize`) is separate from sync because it modifies source files (task frontmatter position keys). The fractional keys generated by drag-and-drop work correctly for ordering — normalization to clean sequential keys is cosmetic.

### Reference Validation

```
CLI: markplane check --orphans
  │
  ├─ validate_references(project)
  │    ├─ For each .md file in backlog/, roadmap/, plans/, notes/:
  │    │    ├─ extract_references(content)  → [[ID]] wiki-links
  │    │    └─ Check each ID resolves via project.item_path()
  │    └─ Return list of BrokenReference { source_file, target_id }
  │
  └─ find_orphans(project)
       ├─ Collect all item IDs from filenames
       ├─ Collect all referenced IDs (body [[refs]] + frontmatter fields)
       └─ Return IDs with no incoming references
```

## Error Handling Architecture

```
markplane-core                    markplane-cli
┌──────────────────┐              ┌───────────────────────────────────┐
│  MarkplaneError  │              │ CLI commands:                     │
│  ├─ Io(io::Error)│─────?──────► │   anyhow::Result → colored output│
│  ├─ Yaml(..)     │              │                                   │
│  ├─ NotFound(..) │              │ MCP server (markplane mcp):       │
│  ├─ InvalidId    │─────?──────► │   Result<String, String>          │
│  ├─ InvalidTrans │              │   → JSON-RPC Error (code+message) │
│  ├─ InvalidStatus│              └───────────────────────────────────┘
│  ├─ DuplicateId  │
│  ├─ InvalidLink  │
│  ├─ BrokenRef    │
│  ├─ NotInit      │
│  ├─ Config       │
│  └─ Frontmatter  │
└──────────────────┘
```

Core errors are typed and specific. CLI commands convert them to user-friendly messages via `anyhow`. The MCP module converts them to JSON-RPC error responses with standard error codes.

## Security Model

### Input Validation
- **Title length**: Capped at 500 characters to prevent resource exhaustion
- **YAML sanitization**: `sanitize_yaml_string()` escapes `\`, `"`, `\n`, `\r` before embedding in YAML templates, preventing YAML injection
- **Tag quoting**: `format_yaml_list()` quotes each tag value and escapes inner quotes
- **ID validation**: `parse_id()` enforces strict `PREFIX-RANDOM` format (5-char alphanumeric suffix) — only `EPIC`, `TASK`, `PLAN`, `NOTE` prefixes accepted. This prevents path traversal since IDs determine file paths.

### Concurrency Safety
- **Random IDs**: `next_id()` generates random 5-character alphanumeric IDs — no shared counter, safe for parallel git branches.
- **Atomic file creation**: All item creation uses `File::create_new()` (`O_CREAT | O_EXCL`), which atomically fails if the file already exists. This prevents TOCTOU races where two concurrent processes generate the same random ID.

### MCP-Specific
- **Stdin line limit**: 1 MB maximum per line — oversized inputs are rejected with a parse error, preventing memory exhaustion from malformed requests.

## Key Design Decisions

### Filesystem as Database
Files are the source of truth. Each item is a standalone markdown file with YAML frontmatter. No SQL database, no binary format. This makes the data:
- Version-controllable with git
- Human-readable and editable
- Portable (no external dependencies)
- Accessible to AI tools as plain text

### INDEX.md Router Pattern
Every directory has an INDEX.md that summarizes its contents. AI agents read the index (~200 tokens) to discover what's available, then load only the files they need. This keeps AI context budgets low while maintaining full navigability.

### Custom Frontmatter Parser
Rather than depending on a full markdown parser, Markplane uses a simple `---\nyaml\n---\nbody` splitter. This is faster, has zero dependencies beyond `serde_yaml`, and is sufficient since the frontmatter format is fully controlled.

### Manual Reference Extraction
Cross-references (`[[TASK-rm6d3]]`) are extracted via byte scanning rather than regex. This avoids a regex dependency and gives precise control over what constitutes a valid reference (must pass `parse_id()` validation).

### Generic Document Wrapper
`MarkplaneDocument<T>` separates the typed frontmatter from the freeform markdown body. This allows a single `read_item<T>()` / `write_item<T>()` path for all entity types while preserving type safety.

### serde_yaml 0.9
Despite being deprecated, `serde_yaml 0.9` is used over the newer `serde_yml` because the replacement crate is at version 0.0.x and not yet mature enough for production use.
