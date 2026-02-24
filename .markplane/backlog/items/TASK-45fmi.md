---
id: TASK-45fmi
title: Customizable templates with manifest and MCP integration
status: backlog
priority: high
type: feature
effort: large
tags:
- templates
- mcp
epic: EPIC-a5vs9
plan: null
depends_on:
- TASK-xvx5f
- TASK-g3ew2
blocks: []
assignee: null
position: a0h
created: 2026-02-22
updated: 2026-02-22
---

# Customizable templates with manifest and MCP integration

## Description

Make templates customizable by users. Currently templates are hardcoded Rust constants in `templates.rs`. Users should be able to customize the body scaffold for any entity type, create named variants, and have the AI discover available templates via MCP.

### Template files

Templates are **body-only** markdown files in `.markplane/templates/`. The system continues to generate frontmatter programmatically — templates only control the markdown body content below the `---` frontmatter block. Templates use `{PLACEHOLDER}` tokens for system-injected values (`{ID}`, `{TITLE}`, `{DATE}`, etc.).

### File naming convention

- `{kind}.md` — default template for that entity kind (e.g. `task.md`, `epic.md`)
- `{kind}-{name}.md` — named variant (e.g. `task-bug.md`, `plan-refactor.md`)

### Manifest

`.markplane/templates/manifest.yaml` is the single source of truth for template metadata:

```yaml
task:
  default: default                # fallback when no type or template specified
  type_defaults:                  # type → default template mapping
    bug: bug
    # types without entries fall through to "default"
  templates:
    default:
      description: "Standard task with description and acceptance criteria"
    bug:
      description: "Bug report with steps to reproduce and expected/actual behavior"

plan:
  default: implementation
  templates:
    implementation:
      description: "Phased implementation plan with testing strategy"
    refactor:
      description: "Refactor plan with current/target state analysis"

note:
  default: default
  type_defaults:
    research: research
    analysis: analysis
  templates:
    default:
      description: "General purpose note for ideas, decisions, meetings"
    research:
      description: "Research note with findings and recommendations"
    analysis:
      description: "Analysis note with conclusions and next steps"

epic:
  templates:
    default:
      description: "Epic with objective and key results"
```

### Template resolution chain

When creating an item, the body template is selected by:

1. **Explicit `--template` / `template` param** → look for `{kind}-{name}.md`
2. **`type_defaults[type]`** in manifest → use mapped template
3. **`default`** for the kind in manifest → use that
4. **Built-in compiled constant** → final fallback (ensures things work even if files are missing)

### `markplane init` changes

Init generates the full `.markplane/templates/` directory:
- `manifest.yaml` with all built-in templates registered
- All built-in template body files (`task.md`, `task-bug.md`, `epic.md`, `plan-implementation.md`, `plan-refactor.md`, `note.md`, `note-research.md`, `note-analysis.md`)

This serves as working defaults, customization starting point, and examples for creating new variants. Replaces the current informational-only template files that are written but never read.

### Core changes

- `render_template()` and creation methods (`create_task`, `create_epic`, `create_plan`, `create_note`) gain template resolution logic
- New `template` parameter on creation methods (optional, explicit override)
- Manifest parsing (new struct/module for reading `manifest.yaml`)
- Frontmatter/body split already done by [[TASK-g3ew2]] — `create_*()` methods already use `write_frontmatter()` and body-only template constants
- `PLAN_REFACTOR_TEMPLATE` finally becomes usable (currently dead code — `create_plan` always uses implementation template)

### CLI changes

- `markplane add` gains `--template` flag (optional)
- `markplane plan` gains `--template` flag (optional, e.g. `--template refactor`)
- `markplane note` already uses type-driven selection — now also supports `--template` override

### MCP changes

- New resource `markplane://templates` — returns manifest contents so the AI can discover available templates and their descriptions
- `markplane_add` tool gains `template` parameter (optional, works alongside `kind` and `type` from [[TASK-xvx5f]])
- `markplane_plan` tool gains `template` parameter (optional)
- No dedicated template creation/update/delete tools — direct file editing is sufficient for the rare template authoring operation

### What is NOT included

- No custom variables / custom placeholders (just edit the template directly)
- No custom frontmatter fields (use free-form body sections instead)
- No `template` field stored in item frontmatter (template is creation-time guidance only, irrelevant after item is filled out)
- No `markplane_template_create` MCP tool (rare operation, direct file editing is fine, avoids burning context)

## Acceptance Criteria

- [ ] `.markplane/templates/` directory with body-only markdown files
- [ ] `manifest.yaml` with template metadata, descriptions, and type defaults
- [ ] Template resolution chain: explicit → type default → kind default → built-in
- [ ] `markplane init` generates manifest and all built-in template files
- [ ] `--template` flag on `add`, `plan`, and `note` CLI commands
- [ ] `template` param on `markplane_add` and `markplane_plan` MCP tools
- [ ] MCP resource `markplane://templates` returns available templates
- [ ] Plan refactor template is selectable (no longer dead code)
- [ ] Existing behavior preserved when no custom templates exist (built-in fallback)
- [ ] Tests for template resolution chain, manifest parsing, and creation with templates

## Notes

- This builds on [[TASK-xvx5f]] (polymorphic `markplane_add` with `kind` param) — the `template` param is additive
- [[TASK-g3ew2]] splits frontmatter from templates — after that task, template constants are already body-only and `create_*()` methods use `write_frontmatter()`. This task adds resolution logic (manifest, file lookup, fallback chain) on top of that foundation.
- The built-in template constants in `templates.rs` remain as compiled-in fallbacks, ensuring the system works even without `.markplane/templates/`
- Template body files should use the same `{PLACEHOLDER}` tokens as today — no new placeholder system needed

## References
