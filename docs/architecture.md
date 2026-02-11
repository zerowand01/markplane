# Markplane Architecture

## System Overview

```
┌─────────────────┐     ┌─────────────────┐
│  markplane CLI   │     │  markplane-mcp   │
│  (clap, colored, │     │  (JSON-RPC 2.0,  │
│   tabled, anyhow)│     │   stdio, serde)  │
└────────┬─────────┘     └────────┬─────────┘
         │                        │
         └───────────┬────────────┘
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

Both the CLI and MCP server are thin wrappers around `markplane-core`. All business logic — data models, CRUD, querying, sync, reference validation, context generation — lives in the core library.

## Crate Responsibilities

### markplane-core (library)

The core library contains all domain logic. It exposes a `Project` struct that represents a `.markplane/` directory and provides methods for every operation.

**Modules:**

| Module | Purpose |
|--------|---------|
| `models` | Entity structs (`BacklogItem`, `Epic`, `Plan`, `Note`), enums (`BacklogStatus`, `Priority`, etc.), `Config`, `MarkplaneDocument<T>` wrapper, ID parsing/formatting |
| `project` | `Project` struct — init, config, ID management, CRUD operations, archiving, `find_blocked_items()` |
| `frontmatter` | Parse and serialize `---\nyaml\n---\nbody` format |
| `query` | `QueryFilter` struct, `list_backlog_items()`, `list_epics()`, `list_plans()`, `list_notes()` with filtering and sorting |
| `references` | `extract_references()` (wiki-link `[[ID]]` scanning), `validate_references()`, `find_orphans()`, `build_reference_graph()` |
| `index` | INDEX.md generation for all directories (root, backlog, roadmap, plans, notes) |
| `context` | `.context/` file generation — summary, active-work, blocked-items, metrics |
| `templates` | Embedded template constants and `render_template()` placeholder replacement |
| `error` | `MarkplaneError` enum (via `thiserror`) and `Result<T>` type alias |

### markplane-cli (binary: `markplane`)

The CLI crate provides the user-facing terminal interface.

- **Argument parsing**: `clap` with derive macros
- **Commands**: 22 subcommands — `init`, `add`, `show`, `ls`, `status`, `sync`, `start`, `done`, `promote`, `plan`, `epic`, `note`, `assign`, `link`, `tag`, `check`, `stale`, `archive`, `context`, `metrics`, `graph`, `claude-md`, `dashboard`
- **Formatting**: `commands/formatting.rs` — shared helpers for truncation, status/priority colorization (via `colored`), table output (via `tabled`)
- **Error handling**: `anyhow::Result` at the top level

### markplane-mcp (binary: `markplane-mcp`)

The MCP server enables AI tools (Claude, Cursor, etc.) to interact with the project.

- **Protocol**: JSON-RPC 2.0 over stdio (one JSON object per line)
- **Tools**: 15 tools — `markplane_summary`, `markplane_query`, `markplane_show`, `markplane_add`, `markplane_update`, `markplane_start`, `markplane_done`, `markplane_sync`, `markplane_epic`, `markplane_note`, `markplane_plan`, `markplane_check`, `markplane_link`, `markplane_graph`, `markplane_archive`
- **Resources**: 3 static resources (`markplane://summary`, `markplane://active-work`, `markplane://blocked`) + 2 dynamic templates (`markplane://backlog/{id}`, `markplane://epic/{id}`)
- **Error handling**: Tool handlers return `Result<String, String>`; errors map to JSON-RPC error codes

## Data Model

### MarkplaneDocument\<T\>

All items are wrapped in a generic document type that separates YAML frontmatter from the markdown body:

```rust
pub struct MarkplaneDocument<T> {
    pub frontmatter: T,   // Deserialized YAML (BacklogItem, Epic, etc.)
    pub body: String,      // Markdown content after the closing ---
}
```

### Entity Types

```
                    ┌──────────────────────┐
                    │       Config         │
                    │ version, project,    │
                    │ counters, context,   │
                    │ archive              │
                    └──────────────────────┘

  ┌───────────────┐  ┌───────────────┐  ┌───────────────┐  ┌───────────────┐
  │   BacklogItem │  │     Epic      │  │     Plan      │  │     Note      │
  │ BACK-NNN      │  │ EPIC-NNN      │  │ PLAN-NNN      │  │ NOTE-NNN      │
  │               │  │               │  │               │  │               │
  │ id, title,    │  │ id, title,    │  │ id, title,    │  │ id, title,    │
  │ status,       │  │ status,       │  │ status,       │  │ type,         │
  │ priority,     │  │ priority,     │  │ implements[], │  │ status,       │
  │ type, effort, │  │ started?,     │  │ epic?,        │  │ tags[],       │
  │ tags[],       │  │ target?,      │  │ created,      │  │ related[],    │
  │ epic?,        │  │ tags[],       │  │ updated       │  │ created,      │
  │ plan?,        │  │ depends_on[]  │  │               │  │ updated       │
  │ depends_on[], │  │               │  │               │  │               │
  │ blocks[],     │  │               │  │               │  │               │
  │ assignee?,    │  │               │  │               │  │               │
  │ created,      │  │               │  │               │  │               │
  │ updated       │  │               │  │               │  │               │
  └───────────────┘  └───────────────┘  └───────────────┘  └───────────────┘
```

Relationships between entities:
- `BacklogItem.epic` → links to an `Epic`
- `BacklogItem.plan` → links to a `Plan`
- `BacklogItem.depends_on[]` / `blocks[]` → links to other `BacklogItem`s
- `Plan.implements[]` → links to `BacklogItem`s it implements
- `Plan.epic` → links to an `Epic`
- `Note.related[]` → links to any item type
- `Epic.depends_on[]` → links to other `Epic`s

## Data Flow

### Adding an Item

```
CLI: markplane add "Fix login bug" --type bug --priority high
  │
  ├─ Parse args via clap
  ├─ Project::from_current_dir()  → find .markplane/
  ├─ validate_title_length(title) → reject if > 500 chars
  ├─ project.next_id(&IdPrefix::Back)
  │    ├─ Lock config.yaml (fs2 advisory lock)
  │    ├─ Read counter, increment, write back
  │    └─ Unlock
  ├─ sanitize_yaml_string(title)
  ├─ render_template(BACKLOG_TEMPLATE, vars)
  ├─ Write .markplane/backlog/items/BACK-001.md
  └─ Return BacklogItem struct
```

### Sync

```
CLI: markplane sync
  │
  ├─ Project::from_current_dir()
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
markplane-core                    markplane-cli           markplane-mcp
┌──────────────────┐
│  MarkplaneError  │              anyhow::Result          Result<String, String>
│  ├─ Io(io::Error)│─────────?───────────┐
│  ├─ Yaml(..)     │                     │                ┌──────────────────┐
│  ├─ NotFound(..) │                     ▼                │  JSON-RPC Error  │
│  ├─ InvalidId    │              Display for user ──►    │  code + message  │
│  ├─ InvalidTrans │              colored output          └──────────────────┘
│  ├─ InvalidStatus│
│  ├─ DuplicateId  │
│  ├─ BrokenRef    │
│  ├─ NotInit      │
│  ├─ Config       │
│  └─ Frontmatter  │
└──────────────────┘
```

Core errors are typed and specific. The CLI converts them to user-friendly messages via `anyhow`. The MCP server converts them to JSON-RPC error responses with standard error codes.

## Security Model

### Input Validation
- **Title length**: Capped at 500 characters to prevent resource exhaustion
- **YAML sanitization**: `sanitize_yaml_string()` escapes `\`, `"`, `\n`, `\r` before embedding in YAML templates, preventing YAML injection
- **Tag quoting**: `format_yaml_list()` quotes each tag value and escapes inner quotes
- **ID validation**: `parse_id()` enforces strict `PREFIX-NUMBER` format — only `EPIC`, `BACK`, `PLAN`, `NOTE` prefixes accepted. This prevents path traversal since IDs determine file paths.

### Concurrency Safety
- **File locking**: `next_id()` acquires an exclusive `fs2` advisory lock on `config.yaml` before reading/incrementing counters, preventing duplicate IDs from concurrent processes.

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
Cross-references (`[[BACK-042]]`) are extracted via byte scanning rather than regex. This avoids a regex dependency and gives precise control over what constitutes a valid reference (must pass `parse_id()` validation).

### Generic Document Wrapper
`MarkplaneDocument<T>` separates the typed frontmatter from the freeform markdown body. This allows a single `read_item<T>()` / `write_item<T>()` path for all entity types while preserving type safety.

### serde_yaml 0.9
Despite being deprecated, `serde_yaml 0.9` is used over the newer `serde_yml` because the replacement crate is at version 0.0.x and not yet mature enough for production use.
