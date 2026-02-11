# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [0.1.0] - Unreleased

### Added

#### Core Library (`markplane-core`)

- 4 entity types with YAML frontmatter: BacklogItem, Epic, Plan, Note
- CRUD operations: create, read, update, write for all entity types
- Status workflows: Backlog (draft/backlog/planned/in-progress/done/cancelled), Epic (planned/active/done), Plan (draft/approved/in-progress/done), Note (draft/active/archived)
- Classification enums: Priority (critical/high/medium/low/someday), ItemType (feature/bug/enhancement/chore/research/spike), Effort (xs/small/medium/large/xl), NoteType (research/analysis/idea/decision/meeting)
- ID system with `{PREFIX}-{NNN}` format (EPIC, BACK, PLAN, NOTE) and sequential counter management
- YAML frontmatter parsing via custom `---` delimiter splitter and serde_yaml
- Cross-reference extraction using `[[ID]]` wiki-style syntax with manual byte scanning
- Cross-reference validation reporting broken links with source file and target ID
- Reference graph builder for dependency visualization
- Dependency graph with `depends_on` and `blocks` relationships between backlog items
- Blocked item detection: finds items with unresolved (non-done) dependencies
- Context generation: summary.md, active-work.md, blocked-items.md, metrics.md
- INDEX.md sync: auto-generates table-of-contents indexes for all directories
- Full sync operation: regenerates all INDEX.md files and all .context/ files
- Stale item detection by configurable day threshold
- Archive system: move done/cancelled items to archive/ subdirectories
- Item path resolution checking both active and archive directories
- Project initialization scaffolding: directory structure, config.yaml, INDEX.md files, templates, special note files
- Template system with `{PLACEHOLDER}` token substitution via `render_template()`
- Query filtering by status, priority, epic, tags, assignee, and item type
- File locking via `fs2` on config.yaml during ID allocation to prevent concurrent conflicts
- YAML string sanitization: escapes quotes, backslashes, and newlines
- Title length validation (500 character maximum)
- Configurable context settings: token budget, recent days window, auto-generate flag
- Configurable archive settings: auto-archive threshold, keep-cancelled flag

#### CLI (`markplane-cli`)

- 22 subcommands: init, add, show, ls, status, sync, start, done, promote, plan, epic, note, assign, link, tag, check, stale, archive, context, metrics, graph, claude-md, dashboard
- `ls` sub-kinds: backlog (default), epics, plans, notes
- Filtering: `--status`, `--priority`, `--epic`, `--tags`, `--assignee`, `--type` flags on `ls`
- Colored terminal output with status and priority colorization
- Table formatting via `tabled` crate with column truncation for terminal width
- Dashboard view: project overview with status counts, in-progress items, blocked items, priority breakdown
- Dependency graph display with configurable `--depth`
- CLAUDE.md snippet generator (`claude-md` subcommand)
- Project metrics display (status distribution, priority distribution, epic progress)

#### MCP Server (`markplane-mcp`)

- JSON-RPC 2.0 protocol over stdio
- 15 tools: markplane_summary, markplane_query, markplane_show, markplane_add, markplane_update, markplane_start, markplane_done, markplane_sync, markplane_context, markplane_graph, markplane_promote, markplane_plan, markplane_link, markplane_check, markplane_stale
- 5 resources: 3 static (markplane://summary, markplane://active-work, markplane://blocked) + 2 dynamic templates (markplane://backlog/{id}, markplane://epic/{id})
- MCP protocol version 2024-11-05 with tools and resources capabilities
- Notification handling for `notifications/initialized` and `initialized` methods
- `--project` argument for explicit project path

#### Security

- YAML string sanitization to prevent injection via titles and tags
- Title length validation (500 character maximum) across all entity types
- File locking (`fs2`) on config.yaml during `next_id()` to prevent race conditions
- MCP stdin line length limit (1 MB) to prevent memory exhaustion
- Strong ID validation preventing path traversal (prefix must be EPIC/BACK/PLAN/NOTE, number must be valid u32)

#### Testing

- 207 tests total: 121 unit tests (markplane-core) + 51 integration tests (markplane-cli) + 35 integration tests (markplane-mcp)
- Integration tests using `assert_cmd` with `cargo_bin_cmd!` macro
- All tests passing, clippy clean
