//! Embedded templates for Markplane document types.
//!
//! Entity templates contain only the markdown **body** scaffold — no YAML
//! frontmatter. The `create_*()` methods build the typed struct, serialize
//! frontmatter via `write_frontmatter()`, and combine it with the body.
//!
//! Body templates use placeholder tokens replaced at creation time:
//! - `{TITLE}` — item title (used in headings)
//!
//! Index/init templates may use additional tokens:
//! - `{PROJECT_NAME}` — project name (for init)
//! - `{DATE}` — current date (for init)

pub const TASK_TEMPLATE: &str = r#"# {TITLE}

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

pub const TASK_BUG_TEMPLATE: &str = r#"# {TITLE}

## Description

[What is the bug? Include error messages if applicable.]

## Steps to Reproduce

1. Step 1
2. Step 2
3. Step 3

## Expected Behavior

[What should happen?]

## Actual Behavior

[What happens instead?]

## Notes

[Environment details, workarounds, screenshots, or other context.]

## References
"#;

pub const EPIC_TEMPLATE: &str = r#"# {TITLE}

## Objective

[2-3 sentences: What does this epic achieve? Why does it matter?]

## Key Results

- [ ] KR1: [Measurable outcome]
- [ ] KR2: [Measurable outcome]
- [ ] KR3: [Measurable outcome]

## Notes

[Strategic context, dependencies on external work, risks.]
"#;

pub const PLAN_IMPLEMENTATION_TEMPLATE: &str = r#"# {TITLE} Implementation Plan

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

pub const PLAN_REFACTOR_TEMPLATE: &str = r#"# {TITLE} Refactor Plan

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

pub const NOTE_RESEARCH_TEMPLATE: &str = r#"# {TITLE}

## Summary

[Brief summary of the research topic.]

## Findings

[Detailed research findings.]

## Recommendations

[Actionable recommendations based on findings.]

## References

[Sources, links, related items.]
"#;

pub const NOTE_ANALYSIS_TEMPLATE: &str = r#"# {TITLE}

## Context

[What is being analyzed and why?]

## Analysis

[Detailed analysis.]

## Conclusions

[Key takeaways.]

## Next Steps

[Recommended actions.]
"#;

pub const NOTE_GENERIC_TEMPLATE: &str = r#"# {TITLE}

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

// ── Starter content body templates ───────────────────────────────────────
// Used by `seed_starter_content()` during `markplane init`. These are internal
// templates (not exposed to users). Placeholders are substituted with generated
// IDs via `render_template()`.

pub const STARTER_EPIC_BODY: &str = "\
## Objective

Get your markplane project set up and ready for use. This epic tracks the \
initial onboarding steps to help you learn the system by doing.

## Key Results

- [ ] Project configuration reviewed and customized ([[{SETUP_TASK_ID}]])
- [ ] Existing work imported or captured ([[{IMPORT_TASK_ID}]])
- [ ] Team is using markplane for day-to-day project tracking

## Notes

Delete this epic once onboarding is complete, or keep it as a reference.
";

pub const STARTER_SETUP_TASK_BODY: &str = "\
## Description

Walk through the markplane configuration and customize it for your project.

## Steps

1. **Review config.yaml** — Open `.markplane/config.yaml` and update the project \
name and description. Check that the task types, note types, and workflow statuses \
fit your team's process.

2. **Customize templates** — Browse `.markplane/templates/` and edit the markdown \
templates to match your preferred format. Templates use `{TITLE}` placeholders.

3. **Set up AI integration** — If you use Claude Code or another AI tool, run \
`markplane claude-md` to generate the integration snippet for your `CLAUDE.md`.

4. **Configure documentation paths** — Add `documentation_paths` to `config.yaml` \
to link your project's existing docs (e.g., `docs/`, `README.md`) into the \
markplane index.

## Acceptance Criteria

- [ ] Config reflects actual project name and workflow
- [ ] Templates customized or defaults confirmed
- [ ] AI integration configured (if applicable)
";

pub const STARTER_IMPORT_TASK_BODY: &str = "\
## Description

Migrate existing tasks, issues, or TODOs into markplane so everything lives in \
one place.

See [[{PLAN_ID}]] for a step-by-step migration guide covering GitHub Issues, \
Jira, and inline TODOs.

## Acceptance Criteria

- [ ] All active work items captured in markplane
- [ ] Old tracking system archived or redirected
";

pub const STARTER_PLAN_BODY: &str = "\
## Context

This plan covers how to bring existing work items into markplane for \
[[{IMPORT_TASK_ID}]].

## Quick Reference

```bash
# Create a task
markplane add \"Task title\" --type feature --priority high

# Create an epic and link tasks
markplane epic \"Epic title\"
markplane link <task-id> <epic-id> -r epic

# Create a plan for a task
markplane plan <task-id>

# View your backlog
markplane ls
```

## Migration Steps

### From GitHub Issues

1. Export issues (or review them manually)
2. Create a markplane task for each active issue
3. Group related tasks under epics
4. Close the GitHub issues with a note pointing to markplane

### From Jira

1. Export active items from your Jira board
2. Map Jira statuses to markplane statuses (backlog, planned, in-progress, done)
3. Create tasks with appropriate types and priorities
4. Link related items using `markplane link`

### From Inline TODOs

1. Search your codebase: `grep -r \"TODO\\|FIXME\\|HACK\" src/`
2. Create a task for each actionable TODO
3. Replace the inline comment with a wiki-link reference to the task
4. Use tags to categorize (e.g., `--tags tech-debt`)
";

pub const STARTER_NOTE_BODY: &str = "\
## Purpose

A running log of key project decisions. Each entry captures the context, \
options considered, and rationale — so future-you (or teammates) can understand \
*why* things are the way they are.

## Template

When adding a new decision, copy this template:

```markdown
### Decision Title

**Date:** YYYY-MM-DD
**Status:** Proposed | Accepted | Superseded
**Context:** What prompted this decision?
**Options considered:**
1. Option A — pros/cons
2. Option B — pros/cons

**Decision:** What was decided and why.
**Consequences:** What follows from this decision.
```

---

### Use markplane for project management

**Date:** {TODAY}
**Status:** Accepted
**Context:** Needed a lightweight, AI-friendly project tracking system that \
lives in the repo alongside the code. Related to [[{EPIC_ID}]].
**Options considered:**
1. GitHub Issues — good integration but separate from codebase context
2. Jira — powerful but heavyweight, poor AI integration
3. Markplane — markdown-first, lives in repo, AI-native

**Decision:** Adopted markplane. Files are the source of truth, git is the \
changelog. AI tools can read and update items directly.
**Consequences:** Team needs to learn markplane CLI. All project tracking \
happens through `.markplane/` directory.
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
        let result = render_template(TASK_TEMPLATE, &[("{TITLE}", "Test item")]);
        assert!(result.starts_with("# Test item"));
        assert!(result.contains("## Description"));
    }

    #[test]
    fn test_render_template_epic() {
        let result = render_template(EPIC_TEMPLATE, &[("{TITLE}", "Phase 1")]);
        assert!(result.starts_with("# Phase 1"));
        assert!(result.contains("## Objective"));
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
        let result = render_template("Title: {TITLE}", &[("{TITLE}", "C++ & Rust <3 \"code\"")]);
        assert_eq!(result, "Title: C++ & Rust <3 \"code\"");
    }

    #[test]
    fn test_render_template_plan() {
        let result = render_template(PLAN_IMPLEMENTATION_TEMPLATE, &[("{TITLE}", "Dark mode")]);
        assert!(result.starts_with("# Dark mode Implementation Plan"));
        assert!(result.contains("## Overview"));
    }

    #[test]
    fn test_render_template_note() {
        let result = render_template(NOTE_GENERIC_TEMPLATE, &[("{TITLE}", "My idea")]);
        assert!(result.starts_with("# My idea"));
        assert!(result.contains("[Content goes here.]"));
    }
}
