---
id: TASK-5wph3
title: Enhance plan and task templates with AI authoring guidance
status: done
priority: high
type: enhancement
effort: medium
tags:
- templates
- ai-guidance
epic: EPIC-a5vs9
plan: null
depends_on: []
blocks: []
assignee: null
position: a0V
created: 2026-02-20
updated: 2026-02-24
---

# Enhance plan and task templates with AI authoring guidance

## Description

Plan and task templates are too thin for AI-assisted authoring. The structural metadata (frontmatter, wiki-links, dependencies) works well, but the free-form body provides no guardrails. This leads to seven recurring problems observed in real-world usage:

1. **Speculative code** — AI writes interfaces, structs, and config values without verifying against the actual codebase. Plans look authoritative but contain fabricated details.
2. **Duplicated contracts** — When multiple plans share an interface (e.g., a message schema used by both producer and consumer), each plan independently defines it. The definitions diverge with no canonical source.
3. **Oversized plans** — Without size guidance, plans grow to 300-400+ lines covering too many concerns. This wastes AI context window and increases inconsistency.
4. **Task/plan content overlap** — Tasks include implementation-level detail (file paths, config values, constructor signatures) that the linked plan re-specifies. When they diverge, neither is authoritative.
5. **Missing scope boundaries** — Plans lack explicit non-goals, leading to scope creep and ambiguity about what's covered vs. deferred.
6. **Lost decisions** — Architectural decisions (e.g., "use X over Y because Z") are made in ephemeral chat but not recorded. Future implementers lack rationale.
7. **No verification checkpoint** — Plans move from draft to approved with no structured check that content matches the codebase.

### Plan template enhancements

Add HTML guidance comments (deleted when filling in the template) that instruct AI agents on content policy:
- Code in plans should be limited to: contracts/interfaces, ONE pattern example to replicate, directory structures, critical algorithms where precision matters
- Code should NOT be: full copy-paste implementations, every file that needs changing, boilerplate the AI can generate
- Use explicit file paths, not "update the relevant files"
- Prefer bullets over prose; keep sections concise and scannable
- Link to existing code/docs for context rather than duplicating content into the plan
- Size target: ~200 lines. If exceeding 300, split by concern (phase boundaries are natural split points)

Add new sections to the plan body:

- **Non-Goals / Out of Scope** — Explicitly lists what the plan does NOT address, so scope boundaries are visible and scope creep is preventable. Should reference where deferred work will be tracked (e.g., a future task).
- **Key Decisions** — `| Decision | Rationale |` table capturing architectural choices made during planning. Future implementers understand WHY the chosen approach was selected over alternatives. These are plan-scoped decisions, not project-wide ADRs.
- **Ground Truth** — Lists source files (`path/to/file.rs:line-range — Description`) that the plan's contracts, patterns, and conventions are derived from. Every interface, struct, config pattern, or convention cited in the plan must trace back to an entry here. If a claim can't point to a source file, it's speculative and should be verified or removed. This is the primary defense against AI-generated fiction in plans.
- **Pre-Approval Checklist** — Verification gate before `draft → approved`. Checks: (1) ground truth refs verified against current codebase, (2) cross-plan contracts are referenced not redefined, (3) no speculative code — all patterns derived from existing source, (4) plan is under ~200 lines.

Enhance existing sections:

- **Phases** — Each phase ends with a checkpoint statement defining what must be true when the phase is complete (e.g., "service starts, health checks pass, API responds with empty list").
- **Cross-plan contract convention** — If a plan defines an interface consumed by other plans, mark it as `## Cross-Plan Contract: [Name]` — this is the canonical definition. Other plans consuming that contract reference it via wiki-link (`> **Contract source**: [[PLAN-xxxxx]] §Section Name`), never redefine it.

The refactor template gets the same enhancements adapted for refactor context. In particular, Ground Truth verifies that the Current State section actually matches the codebase rather than being written from assumption.

### Task template enhancement

Add a guidance comment clarifying two boundaries:

1. **WHAT vs HOW** — Acceptance criteria define outcomes and constraints (e.g., "circuit breaker prevents cascading failures with configurable threshold"), not implementation details (e.g., "circuit breaker in `internal/connectors/executor.go` with threshold: 5 using `sony/gobreaker`"). Implementation detail belongs in the linked PLAN. Exception: when a task defines an interface contract that other tasks depend on, the contract shape belongs in the description.
2. **Description vs Acceptance Criteria** — The Description carries the full context: what's being built, why, the problems it solves, and key constraints. It's what an implementer reads to understand the work. Acceptance criteria are concise, verifiable checkboxes that confirm completeness — not a spec.

## Acceptance Criteria

- [ ] Plan-implementation template includes AI guidance comments covering code policy, link-don't-duplicate, and size guidance
- [ ] Plan-implementation template includes Non-Goals, Key Decisions, Ground Truth, and Pre-Approval Checklist sections
- [ ] Phase sections include checkpoint guidance
- [ ] Cross-plan contract convention documented in plan templates
- [ ] Plan-refactor template has same enhancements adapted for refactor context
- [ ] Task template includes guidance comment on WHAT vs HOW and Description vs Acceptance Criteria boundaries
- [ ] Templates updated in both `crates/markplane-core/src/templates.rs` and `.markplane/templates/`
- [ ] `cargo test` passes

## Notes

Files to modify:
- `crates/markplane-core/src/templates.rs` — canonical template source (compiled into binary)
- `.markplane/templates/plan-implementation.md` — user-facing copy
- `.markplane/templates/plan-refactor.md` — user-facing copy
- `.markplane/templates/task.md` — user-facing copy

Design principles:
- Guidance goes in HTML comments that get deleted when filling in the template
- Sections are suggested structure, not mandatory bureaucracy
- Target ~80-100 total lines for enhanced plan template (currently ~42)
- Frontmatter stays structural — all new content is body-only

## References
