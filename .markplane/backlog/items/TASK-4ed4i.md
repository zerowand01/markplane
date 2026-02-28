---
id: TASK-4ed4i
title: Migration framework for markplane version upgrades
status: backlog
priority: high
type: feature
effort: medium
epic: EPIC-c5uem
plan: null
depends_on: []
blocks: []
related: []
assignee: null
tags:
- architecture
- migration
position: a3
created: 2026-02-20
updated: 2026-02-25
---

# Migration framework for markplane version upgrades

## Description

When markplane's data format changes between versions (new fields, renamed fields, restructured directories, changed ID formats), user repos with existing `.markplane/` data need a reliable upgrade path. Without a migration framework, breaking changes force users to manually transform their data or start fresh.

`config.yaml` already has a `version: 1` field. Build a migration framework around this:

**Version detection**: every markplane command reads `config.yaml` and compares the `version` field against the binary's expected version. If the data version is older, markplane refuses to proceed and tells the user to run `markplane migrate`. If the data version is newer than the binary, markplane tells the user to upgrade the binary.

**Explicit migration via** `markplane migrate`: runs all pending migrations sequentially (v1→v2, v2→v3, never skipping). Each migration is a function built into the binary — no external scripts or migration files. After completing, it updates the `version` field in config.yaml. Migrations modify files but do not commit — the user reviews changes with `git diff` and commits when satisfied.

**Git as the safety net**: since `.markplane/` is tracked by git, migrations are inherently reversible via `git checkout .markplane/`. Down-migrations are unnecessary. This is a key advantage of the file-based architecture — the version control system provides atomicity (commit) and rollback (checkout) for free.

**Version mismatch across teams**: when a team member pulls migrated data, their older binary detects the version mismatch and prompts them to upgrade markplane. Same pattern as any shared tooling version constraint.

The framework should be built to make adding future migrations straightforward — each migration is a function with a clear signature that transforms `.markplane/` data from version N to N+1.

## Acceptance Criteria

- [ ] Every markplane command checks `config.yaml` version before executing and refuses with a clear message if the data version doesn't match
- [ ] `markplane migrate` runs pending migrations sequentially from current version to target version
- [ ] Each migration is a self-contained function built into the binary
- [ ] `version` field in config.yaml is updated after successful migration
- [ ] Migration modifies files but does not commit — user controls the commit
- [ ] Binary detects data versions newer than itself and prompts user to upgrade markplane
- [ ] v1→v2 migration implements the sequential→random ID conversion
- [ ] `markplane migrate --dry-run` previews changes without modifying files
- [ ] `markplane migrate` runs `sync` automatically after successful migration so INDEX.md and `.context/` reflect the new format
- [ ] `markplane migrate` runs `check` (validation) after migration and reports any broken references or integrity issues
- [ ] Migrations can scaffold new directories and files, not just transform existing data
- [ ] Migrations update templates (overwrite strategy)
- [ ] Migrations add new `config.yaml` fields with sensible defaults when the schema evolves
- [ ] MCP server: every tool checks version and returns a clear error on mismatch (same gate as CLI)

## Notes

The framework should be simple — a vector of migration functions indexed by version number, each taking a `&Project` (or `&Path`) and returning `Result<()>`. No migration registry, no plugin system, no down-migrations. Adding a new migration means adding one function and incrementing the expected version constant.

**MCP version gate**: refuse all tools on version mismatch — no degraded read-only mode. A single version check at the top of tool dispatch keeps it simple and consistent with CLI behavior. The error message tells the AI exactly what to do: `"Data version mismatch (v1, expected v2). Run 'markplane migrate' first."`

**Template strategy**: overwrite templates during migration for now. When a future customizable-templates feature is designed, it will introduce merge logic that respects user modifications. Until then, templates are managed by markplane and safe to replace.

**AI context**: no special migration-context files are needed. CLI/MCP error messages on version mismatch provide sufficient context for AI assistants to guide the user. AI sessions start fresh, so historical "what changed" files have low value.

**Post-migration pipeline**: migrate → sync → check. This ensures derived views are regenerated and cross-references are validated before the user commits.

## References
