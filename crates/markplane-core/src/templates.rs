//! Embedded templates for Markplane document types.
//!
//! Templates use placeholder tokens that are replaced at creation time:
//! - `{ID}` — item ID (e.g. TASK-042)
//! - `{TITLE}` — item title
//! - `{DATE}` — current date (YYYY-MM-DD)
//! - `{STATUS}` — initial status
//! - `{PRIORITY}` — priority level
//! - `{TYPE}` — item type or note type
//! - `{EFFORT}` — effort estimate
//! - `{EPIC}` — linked epic ID or "null"
//! - `{TAGS}` — YAML-formatted tags list
//! - `{IMPLEMENTS}` — YAML-formatted implements list (for plans)
//! - `{RELATED}` — YAML-formatted related list (for notes)
//! - `{PROJECT_NAME}` — project name (for init)

pub const TASK_TEMPLATE: &str = r#"---
id: {ID}
title: "{TITLE}"
status: {STATUS}
priority: {PRIORITY}
type: {TYPE}
effort: {EFFORT}
tags: {TAGS}
epic: {EPIC}
plan: null
depends_on: []
blocks: []
assignee: null
position: {POSITION}
created: {DATE}
updated: {DATE}
---

# {TITLE}

## Description

[What needs to be done and why — the problem, context, and key constraints.
An implementer reads this to understand the work. Focus on outcomes, not
implementation steps; a task defines the problem and success criteria,
not how to solve it.]

## Acceptance Criteria

[Observable outcomes that verify completeness — what you'd check in review.
Not an implementation checklist.]

- [ ] Criterion 1
- [ ] Criterion 2
- [ ] Criterion 3

## Notes

[Reference material, links, additional context.]

## References
"#;

pub const EPIC_TEMPLATE: &str = r#"---
id: {ID}
title: "{TITLE}"
status: planned
priority: {PRIORITY}
started: null
target: null
tags: []
depends_on: []
---

# {TITLE}

## Objective

[2-3 sentences: What does this epic achieve? Why does it matter?]

## Key Results

- [ ] KR1: [Measurable outcome]
- [ ] KR2: [Measurable outcome]
- [ ] KR3: [Measurable outcome]

## Notes

[Strategic context, dependencies on external work, risks.]
"#;

pub const PLAN_IMPLEMENTATION_TEMPLATE: &str = r#"---
id: {ID}
title: "{TITLE}"
status: draft
implements: {IMPLEMENTS}
epic: {EPIC}
created: {DATE}
updated: {DATE}
---

# {TITLE} Implementation Plan

<!-- PLAN AUTHORING GUIDANCE — delete this comment when filling in the template.
- Code in plans: contracts/interfaces, ONE pattern example, directory structures,
  critical algorithms. NOT full implementations or boilerplate.
- Use explicit file paths. Link to existing code rather than duplicating.
- Prefer bullets over prose. Target ~200 lines; if >300, split by concern. -->

## Overview

[What this plan accomplishes and the high-level approach.]

## Ground Truth

[Source files this plan's contracts, patterns, and conventions are derived from.
Every interface, struct, or convention cited must trace back here.
If a claim can't point to a source file, it's speculative — verify or remove.]

- `path/to/file.rs:L10-40` — What this establishes

## Approach

[Implementation approach — key design choices and how components fit together.]

## Non-Goals / Out of Scope

[What this plan does NOT address. Reference where deferred work is tracked.]

## Key Decisions

| Decision | Rationale |
|----------|-----------|
| [Choice made] | [Why this over alternatives] |

## Phases

### Phase 1: [Name]

- [ ] Step 1
- [ ] Step 2

**Checkpoint**: [What must be true when this phase is complete.]

### Phase 2: [Name]

- [ ] Step 1
- [ ] Step 2

**Checkpoint**: [What must be true when this phase is complete.]

## Testing Strategy

[How will this be tested?]

## Rollback Plan

[What happens if this needs to be reverted?]

## Pre-Approval Checklist

- [ ] Ground Truth refs verified against current codebase
- [ ] Cross-plan contracts are referenced, not redefined
- [ ] No speculative code — all patterns derived from existing source
- [ ] Plan is under ~200 lines

## References

<!-- CROSS-PLAN CONTRACTS: If this plan defines an interface consumed by other plans,
use a `## Cross-Plan Contract: [Name]` section as the canonical definition.
Other plans reference it: > **Contract source**: PLAN-xxxxx §Section Name -->
"#;

pub const PLAN_REFACTOR_TEMPLATE: &str = r#"---
id: {ID}
title: "{TITLE}"
status: draft
implements: {IMPLEMENTS}
epic: {EPIC}
created: {DATE}
updated: {DATE}
---

# {TITLE} Refactor Plan

<!-- PLAN AUTHORING GUIDANCE — delete this comment when filling in the template.
- Code in plans: contracts/interfaces, ONE pattern example, directory structures,
  critical algorithms. NOT full implementations or boilerplate.
- Use explicit file paths. Link to existing code rather than duplicating.
- Prefer bullets over prose. Target ~200 lines; if >300, split by concern. -->

## Motivation

[Why is this refactor needed? What problems does the current state cause?]

## Ground Truth

[Source files this plan's current-state description and target patterns are derived from.
The Current State section must match the actual codebase — verify, don't assume.
Every convention or pattern cited must trace back here.]

- `path/to/file.rs:L10-40` — What this establishes

## Current State

[Description of the current architecture/code. Must match Ground Truth refs above.]

## Target State

[Description of the desired architecture/code.]

## Non-Goals / Out of Scope

[What this refactor does NOT address. Reference where deferred work is tracked.]

## Key Decisions

| Decision | Rationale |
|----------|-----------|
| [Choice made] | [Why this over alternatives] |

## Migration Steps

### Step 1: [Name]

- [ ] Task 1
- [ ] Task 2

**Checkpoint**: [What must be true when this step is complete.]

### Step 2: [Name]

- [ ] Task 1
- [ ] Task 2

**Checkpoint**: [What must be true when this step is complete.]

## Testing Strategy

[How will correctness be verified during the refactor?]

## Risks

[What could go wrong?]

## Pre-Approval Checklist

- [ ] Ground Truth refs verified against current codebase
- [ ] Current State section matches actual code, not assumptions
- [ ] Cross-plan contracts are referenced, not redefined
- [ ] No speculative code — all patterns derived from existing source
- [ ] Plan is under ~200 lines

## References

<!-- CROSS-PLAN CONTRACTS: If this plan defines an interface consumed by other plans,
use a `## Cross-Plan Contract: [Name]` section as the canonical definition.
Other plans reference it: > **Contract source**: PLAN-xxxxx §Section Name -->
"#;

pub const NOTE_RESEARCH_TEMPLATE: &str = r#"---
id: {ID}
title: "{TITLE}"
type: research
status: draft
tags: {TAGS}
related: {RELATED}
created: {DATE}
updated: {DATE}
---

# {TITLE}

## Summary

[Brief summary of the research topic.]

## Findings

[Detailed research findings.]

## Recommendations

[Actionable recommendations based on findings.]

## References

[Sources, links, related items.]
"#;

pub const NOTE_ANALYSIS_TEMPLATE: &str = r#"---
id: {ID}
title: "{TITLE}"
type: analysis
status: draft
tags: {TAGS}
related: {RELATED}
created: {DATE}
updated: {DATE}
---

# {TITLE}

## Context

[What is being analyzed and why?]

## Analysis

[Detailed analysis.]

## Conclusions

[Key takeaways.]

## Next Steps

[Recommended actions.]
"#;

pub const NOTE_GENERIC_TEMPLATE: &str = r#"---
id: {ID}
title: "{TITLE}"
type: {TYPE}
status: draft
tags: {TAGS}
related: {RELATED}
created: {DATE}
updated: {DATE}
---

# {TITLE}

[Content goes here.]
"#;

pub const ROOT_INDEX_TEMPLATE: &str = r#"# Markplane Project Index
<!-- This file is the entry point for AI agents and human navigation -->

## Quick Navigation

| Module | Path | Purpose | Active Items |
|--------|------|---------|-------------|
| Roadmap | [roadmap/](roadmap/INDEX.md) | Strategic phases & epics | 0 active epics |
| Backlog | [backlog/](backlog/INDEX.md) | All work items | 0 open items |
| Plans | [plans/](plans/INDEX.md) | Implementation details | 0 active plans |
| Notes | [notes/](notes/INDEX.md) | Research & ideas | 0 active notes |
| AI Context | [.context/](.context/summary.md) | Generated summaries | Auto-updated |

## System Info
- Project: {PROJECT_NAME}
- ID counter: EPIC-000, TASK-000, PLAN-000, NOTE-000
- Last sync: {DATE}
- Config: [config.yaml](config.yaml)
"#;

pub const ROADMAP_INDEX_TEMPLATE: &str = r#"# Roadmap Index
<!-- Generated by markplane sync -->

## Active Epics

_No epics yet. Create one with `markplane epic "Epic title"`._

## Planned Epics
"#;

pub const TASK_INDEX_TEMPLATE: &str = r#"# Backlog Index
<!-- Generated by markplane sync -->

## In Progress (0)

## Blocked (0)

## Planned (0)

## Backlog (0)

_No tasks yet. Create one with `markplane add "Item title"`._

## Drafts (0)
"#;

pub const PLANS_INDEX_TEMPLATE: &str = r#"# Plans Index
<!-- Generated by markplane sync -->

## Active Plans

_No plans yet. Create one with `markplane plan TASK-xxx`._

## Completed Plans

_None._
"#;

pub const NOTES_INDEX_TEMPLATE: &str = r#"# Notes Index
<!-- Generated by markplane sync -->

## Active Notes

_No notes yet. Create one with `markplane note "Note title"`._

## Quick Capture

- [ideas.md](ideas.md) — Quick idea capture
- [decisions.md](decisions.md) — Decision log
"#;

pub const IDEAS_TEMPLATE: &str = r#"# Ideas

Quick capture for ideas that aren't tasks yet. Promote to backlog with `markplane promote NOTE-xxx`.

## Unsorted

-
"#;

pub const DECISIONS_TEMPLATE: &str = r#"# Decision Log

Lightweight decision log. Format: `## YYYY-MM-DD: Decision Title`

---
"#;

pub const GITIGNORE_TEMPLATE: &str = "\
# Derived files — regenerated by `markplane sync`
# Do not edit these manually; they are overwritten on every sync.
INDEX.md
.context/
";

/// Replace template placeholders with actual values.
pub fn render_template(template: &str, vars: &[(&str, &str)]) -> String {
    let mut result = template.to_string();
    for (key, value) in vars {
        result = result.replace(key, value);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_template_basic() {
        let result = render_template(
            "Hello {NAME}, today is {DATE}.",
            &[("{NAME}", "World"), ("{DATE}", "2026-02-09")],
        );
        assert_eq!(result, "Hello World, today is 2026-02-09.");
    }

    #[test]
    fn test_render_template_task() {
        let result = render_template(
            TASK_TEMPLATE,
            &[
                ("{ID}", "TASK-001"),
                ("{TITLE}", "Test item"),
                ("{STATUS}", "draft"),
                ("{PRIORITY}", "medium"),
                ("{TYPE}", "feature"),
                ("{EFFORT}", "medium"),
                ("{TAGS}", "[]"),
                ("{EPIC}", "null"),
                ("{POSITION}", "a0"),
                ("{DATE}", "2026-02-09"),
            ],
        );
        assert!(result.contains("id: TASK-001"));
        assert!(result.contains("title: \"Test item\""));
        assert!(result.contains("position: a0"));
        assert!(result.contains("# Test item"));
    }

    #[test]
    fn test_render_template_epic() {
        let result = render_template(
            EPIC_TEMPLATE,
            &[
                ("{ID}", "EPIC-001"),
                ("{TITLE}", "Phase 1"),
                ("{PRIORITY}", "high"),
            ],
        );
        assert!(result.contains("id: EPIC-001"));
        assert!(result.contains("priority: high"));
    }

    #[test]
    fn test_render_template_unreplaced_placeholders() {
        // If a placeholder isn't in the vars, it stays as-is
        let result = render_template("Hello {NAME}, {MISSING}.", &[("{NAME}", "World")]);
        assert_eq!(result, "Hello World, {MISSING}.");
    }

    #[test]
    fn test_render_template_no_vars() {
        let result = render_template("Just text.", &[]);
        assert_eq!(result, "Just text.");
    }

    #[test]
    fn test_render_template_special_chars_in_values() {
        let result = render_template(
            "Title: {TITLE}",
            &[("{TITLE}", "C++ & Rust <3 \"code\"")],
        );
        assert_eq!(result, "Title: C++ & Rust <3 \"code\"");
    }

    #[test]
    fn test_render_template_plan() {
        let result = render_template(
            PLAN_IMPLEMENTATION_TEMPLATE,
            &[
                ("{ID}", "PLAN-001"),
                ("{TITLE}", "Dark mode"),
                ("{IMPLEMENTS}", "[\"TASK-001\"]"),
                ("{EPIC}", "null"),
                ("{DATE}", "2026-02-09"),
            ],
        );
        assert!(result.contains("id: PLAN-001"));
        assert!(result.contains("implements: [\"TASK-001\"]"));
        assert!(result.contains("Implementation Plan"));
    }

    #[test]
    fn test_render_template_note() {
        let result = render_template(
            NOTE_GENERIC_TEMPLATE,
            &[
                ("{ID}", "NOTE-001"),
                ("{TITLE}", "My idea"),
                ("{TYPE}", "idea"),
                ("{TAGS}", "[]"),
                ("{RELATED}", "[]"),
                ("{DATE}", "2026-02-09"),
            ],
        );
        assert!(result.contains("id: NOTE-001"));
        assert!(result.contains("type: idea"));
    }
}
