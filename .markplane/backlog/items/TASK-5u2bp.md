---
id: TASK-5u2bp
title: Improve MCP guidance so agents fill in template placeholders
status: backlog
priority: high
type: enhancement
effort: medium
epic: null
plan: null
depends_on: []
blocks: []
related:
- TASK-4peyk
assignee: null
tags:
- mcp
- dx
position: a6
created: 2026-04-01
updated: 2026-09-30
---

# Improve MCP guidance so agents fill in template placeholders

## Description

When agents use `markplane_add` to create items, they frequently leave the
`[bracketed placeholder]` template content unfilled. This is especially common
during batch creation (e.g. "create tasks for these 10 features"). The agent
treats the tool call as the completed action and moves on without editing the
created files.

User-reported issue — agents need stronger signals at two points:
1. **In the MCP instructions** — make clear that created items contain placeholders
   that must be replaced with real content.
2. **In the `markplane_add` return message** — include the file path and a reminder
   to edit the file, so the nudge appears right where the agent's attention is.

## Changes

### 1. Improve `markplane_add` tool response (`mcp/tools.rs`)

Change the return from bare JSON (`{"id":"TASK-xxx","title":"..."}`) to include
the file path and a next-step reminder:

```
Created TASK-xxx: "Title"
File: .markplane/backlog/items/TASK-xxx.md

Next step: Edit the file above to replace [placeholder] sections with actual content.
```

### 2. Strengthen MCP instructions (`mcp/mod.rs` `build_instructions()`)

Update workflow steps 3-4 from:

```
3. Use markplane_add to create new items (creates template with placeholder content)
4. Edit the markdown file directly to fill in the body content
```

To:

```
3. Use markplane_add to create new items — this creates a file from a template
   with [bracketed placeholder] sections that MUST be replaced with real content
4. Edit each created markdown file to replace ALL [bracketed placeholders] with
   actual content. Items are not complete until placeholders are filled.
```

Add a reinforcing line to the File Editing section:

```
After creating an item, its body contains [bracketed placeholder] text from the
template — always replace these with real content before moving on.
```

### 3. State the frontmatter boundary (`mcp/mod.rs` `build_instructions()`)

The File Editing section currently says "edit them directly", which agents read
as licence to hand-edit frontmatter (e.g. changing `title:` in the file). Add:

```
Edit only the body below the closing `---`. Change anything in the YAML
frontmatter (title, status, priority, links, etc.) through the markplane tools.
```

### 4. Add `status` and `body` parameters to `markplane_add` (`mcp/tools.rs`)

- `status` (optional): create the item directly in a given status, validated
  against the configured workflow. Saves an `add` + `update` round trip for
  projects that work from `backlog`. CLI equivalent: [[TASK-4peyk]].
- `body` (optional): markdown body to use instead of the template. An agent that
  supplies the body in the same call never leaves placeholders behind, which
  addresses the core problem of this task directly. Core's update path already
  accepts a body; `create_task` / `create_*` need to accept one too.

When `body` is supplied, the return message should not include the
"replace placeholders" reminder from change 1.

## Acceptance Criteria

- [ ] `markplane_add` return message includes file path and edit reminder
- [ ] MCP instructions steps 3-4 clarify placeholder obligation
- [ ] File Editing section reinforces the fill-in expectation
- [ ] MCP instructions state that frontmatter is changed through tools, body is edited directly
- [ ] `markplane_add` accepts optional `status` (validated) and `body` parameters
- [ ] Omitting `status` / `body` keeps current behavior
- [ ] Integration tests for the new parameters; MCP docs (`docs/mcp-setup.md`) updated

## Notes

- Deliberately not prescribing create-then-edit vs interleaved workflow — both are valid
- Changes 1–3 are purely guidance; change 4 adds optional parameters (added
  2026-09-30 from external agent feedback on markplane friction)
- Reported by a user who likes markplane but hit this friction repeatedly

## References

- [[TASK-4peyk]]
