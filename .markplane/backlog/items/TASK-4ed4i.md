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
related:
- TASK-pgups
assignee: null
tags:
- architecture
- migration
position: a4
created: 2026-02-20
updated: 2026-10-22
---

# Migration framework for markplane version upgrades

## Description

When markplane's data format changes in a breaking way between releases (renamed or removed fields, restructured directories, changed semantics of an existing field), user repos with existing `.markplane/` data need a reliable upgrade path. Without one, breaking changes force users to transform their data by hand or start fresh.

This task is **deferred until the first breaking format change is designed**. It should be built alongside that migration so the framework is validated against a real case rather than an empty migration list.

**What already exists and does not need a migration**

- *Version detection*: [[TASK-pgups]] added the `DATA_VERSION` constant and the gate that refuses to run when `config.yaml` is newer or older than the binary. When `migrate` ships, update the "older" branch of the message in `error.rs` to point users at it.
- *New config fields*: every field added since v0.1.0 uses a serde default. Adding a field is not a breaking change and must not bump `DATA_VERSION`.
- *New directories*: `ensure_layout()` creates missing layout directories on demand.
- *New built-in templates*: template resolution falls back to the built-in when no file exists in `templates/`.
- *Data repair*: malformed data is handled by lazy self-heal plus `markplane check --fix` (see the position-key fix, #92), not by migrations.

**What the framework must provide when needed**

- *Explicit migration via* `markplane migrate`: runs all pending migrations sequentially (v1→v2, v2→v3, never skipping). Each migration is a function built into the binary. After completing, it bumps `version` in config.yaml.
- *Git as the safety net*: `.markplane/` is tracked by git, so migrations are reversible via `git checkout .markplane/`. No down-migrations. `migrate` refuses to run when `.markplane/` has uncommitted changes so rollback is always clean.
- *Post-migration pipeline*: migrate → sync → check, so derived views are regenerated and cross-references validated before the user commits.
- *Version mismatch across teams*: handled by the existing gate. A teammate who pulls migrated data is told to upgrade markplane.

## Acceptance Criteria

- [ ] `markplane migrate` runs pending migrations sequentially from the data version to `DATA_VERSION`
- [ ] Each migration is a self-contained function built into the binary with signature `fn(&Project) -> Result<()>`; adding one means adding a function and bumping `DATA_VERSION`
- [ ] `version` in config.yaml is updated after each successful migration step
- [ ] Migrations modify files but do not commit; the user controls the commit
- [ ] `markplane migrate` refuses to run when `.markplane/` has uncommitted changes (overridable with `--force`)
- [ ] `markplane migrate` runs `sync` then `check` after migrating and reports any integrity issues
- [ ] Migrations never overwrite files under `templates/`; a template missing a newly required placeholder produces a warning instead
- [ ] MCP server exposes no migrate tool; the per-call gate from [[TASK-pgups]] tells the AI to have the user run the CLI

## Notes

**Changes from the original 2026-02-20 design** (reviewed 2026-10-22):

- The only concrete migration in the original design, sequential→random IDs, was dropped. Random IDs, the `items/` subdirectory, and the counters removal all landed before v0.1.0 (2026-03-09), so no released binary ever produced the old format and there is no user data to convert.
- "Migrations overwrite templates" was dropped. The customizable template system has shipped, so `templates/` files are user-owned.
- "Migrations add config fields with defaults" and "migrations scaffold directories" were dropped as already handled (see above).
- `--dry-run` was dropped. It doubles the work of every migration (a plan mode and an apply mode) for little gain when git provides diff and rollback. The clean-working-tree precondition gives the same safety more cheaply.
- Version detection was split out as [[TASK-pgups]] because a forward-compatibility gate only works if it is in binaries in the wild before the first breaking change.

**Framework shape**: a `&[fn(&Project) -> Result<()>]` slice indexed by source version. No registry, no plugin system, no external migration files.

## References

- [[TASK-pgups]] — data version gate (done)
- [[TASK-skq58]] — the pre-release ID change this task was originally written for
