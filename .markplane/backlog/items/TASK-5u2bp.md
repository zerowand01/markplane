---
id: TASK-5u2bp
title: Improve MCP guidance so agents fill in template placeholders
status: draft
priority: high
type: enhancement
effort: small
epic: null
plan: null
depends_on: []
blocks: []
related: []
assignee: null
tags:
- mcp
- dx
position: a6
created: 2026-04-01
updated: 2026-04-01
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

## Acceptance Criteria

- [ ] `markplane_add` return message includes file path and edit reminder
- [ ] MCP instructions steps 3-4 clarify placeholder obligation
- [ ] File Editing section reinforces the fill-in expectation
- [ ] Existing tests pass, no behavioral change to item creation logic

## Notes

- Deliberately not prescribing create-then-edit vs interleaved workflow — both are valid
- No new tools or parameters needed — this is purely about guidance
- Reported by a user who likes markplane but hit this friction repeatedly

## References
