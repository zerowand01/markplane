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
