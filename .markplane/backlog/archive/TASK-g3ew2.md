---
id: TASK-g3ew2
title: Split frontmatter generation from body templates in create methods
status: done
priority: high
type: enhancement
effort: medium
epic: EPIC-6zdf4
plan: null
depends_on: []
blocks:
- TASK-45fmi
related: []
assignee: null
tags:
- core
- templates
position: a0f
created: 2026-02-22
updated: 2026-02-24
---

# Split frontmatter generation from body templates in create methods

## Description

Entity templates (`TASK_TEMPLATE`, `EPIC_TEMPLATE`, `PLAN_*_TEMPLATE`, `NOTE_*_TEMPLATE`) currently embed both YAML frontmatter and the markdown body scaffold in a single string. The `create_*()` methods render the entire template via `render_template()` with placeholder substitution, using `format_yaml_list()` for arrays and `sanitize_yaml_string()` for titles. This produces flow-style YAML arrays (`["a", "b"]`), but every subsequent update via `write_item()` round-trips through `serde_yaml::to_string()` which produces block-style arrays. This causes a cosmetic format inconsistency on first update.

The fix: `create_*()` methods should build the struct, serialize frontmatter via `write_frontmatter()` (same as the update path), and only use templates for the body scaffold. This gives one serialization path for all writes, consistent YAML formatting from creation, and cleanly separates frontmatter (programmatic) from body (template). This separation is a prerequisite for [[TASK-45fmi]] (customizable templates), which assumes body-only template files.

## Approach

For each `create_*()` method:
1. Build the typed struct (already done — each method builds the return struct)
2. Render the body-only template via `render_template()` (only `{TITLE}` substitution needed for body)
3. Create a `MarkplaneDocument { frontmatter, body }` and serialize via `write_frontmatter()`
4. Write to disk via `write_new_file()` (preserves `File::create_new()` ID collision protection — do NOT switch to `write_item()` which uses `fs::write`)

## Changes by file

### `templates.rs`
- Strip YAML frontmatter from 7 entity templates, leaving only the body markdown (everything below the second `---`)
- Keep `render_template()` — still used for body placeholder substitution and index templates
- Update template tests to match body-only content

### `project.rs` — 4 `create_*()` methods
- **`create_task()`**: Build `Task` struct + body from template → `MarkplaneDocument` → `write_frontmatter()` → `write_new_file()`. Remove `sanitize_yaml_string()` and `format_yaml_list()` calls.
- **`create_epic()`**: Same pattern. Note: `Epic` has no `created`/`updated` fields — serde_yaml handles this correctly.
- **`create_plan()`**: Same pattern. Note: `PLAN_REFACTOR_TEMPLATE` exists but `create_plan()` always uses the implementation template — keep this behavior for now (TASK-45fmi will make it selectable).
- **`create_note()`**: Same pattern. Still does `NoteType`-based template selection for body.

### `project.rs` — delete dead helpers
- Delete `format_yaml_list()` — no longer needed (serde_yaml serializes arrays)
- Delete `sanitize_yaml_string()` — no longer needed (serde_yaml handles YAML escaping)
- Delete their unit tests: `test_format_yaml_list`, `test_format_yaml_list_with_special_chars`, `test_sanitize_yaml_string`

### `lib.rs`
- No changes — `pub use templates::render_template` stays (used for index templates)

### Integration tests
- MCP test at `mcp_integration.rs:968` writes an epic file directly with flow-style YAML — this is a test fixture creating a file, not testing serialization. No change needed; both formats parse identically.

## Acceptance Criteria

- [ ] Entity templates (7) contain only body markdown — no YAML frontmatter
- [ ] All 4 `create_*()` methods serialize frontmatter via `write_frontmatter()`
- [ ] `write_new_file()` still used for creation (ID collision protection preserved)
- [ ] `format_yaml_list()` and `sanitize_yaml_string()` deleted
- [ ] YAML format consistent: block-style for non-empty arrays, `[]` for empty — from creation onward
- [ ] All existing tests pass (318 tests)
- [ ] Titles with special characters (quotes, emoji, backslashes, newlines) round-trip correctly through serde_yaml
- [ ] Created files are byte-identical to what an immediate read→write_item round-trip would produce

## Notes

- `validate_title_length()` stays — it's input validation, not serialization
- `render_template()` stays — used for body placeholders and index/init templates
- The `position` field uses `#[serde(skip_serializing_if = "Option::is_none")]` so it won't appear in files where it's None — matching current behavior
- Epic template currently hardcodes `started: null`, `target: null`, `tags: []`, `depends_on: []` — serde_yaml will produce the same from the struct defaults

## References

- [[TASK-45fmi]] — customizable templates (blocked by this task)
- [[TASK-m7i6q]] — update command expansion (established `write_frontmatter()` as the canonical write path)
