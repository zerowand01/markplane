---
id: NOTE-v3uet
title: 'Now/Next/Later roadmap pattern: research and architectural decision'
status: active
type: decision
related:
- TASK-s6vjt
tags:
- roadmap
- architecture
created: 2026-02-22
updated: 2026-02-22
---

# Now/Next/Later roadmap pattern: research and architectural decision

## Context

Markplane's roadmap view organizes epics into time horizons. The web UI already renders a Now/Next/Later layout, but the underlying data model uses `status: planned | active | done` with priority as a proxy for time horizon. This research evaluates the Now/Next/Later pattern and informs the decision to adopt it as the native epic lifecycle model.

## The Now/Next/Later Roadmap Pattern

### Origin

Invented by Janna Bastow (co-founder of ProdPad and Mind the Product) in 2012. Originally called "Current/Near Term/Future" before a customer suggested the Now/Next/Later terminology. Created as an explicit rejection of timeline-based roadmaps.

### Core Insight

Replace calendar deadlines with **confidence/commitment horizons**. Only "Now" carries a delivery commitment. "Next" and "Later" represent direction and priority, not promises.

| Horizon | Commitment | Detail Level | Confidence |
|---------|-----------|--------------|------------|
| **Now** | High — actively in progress | Fully scoped, broken into tasks | High |
| **Next** | Directional — intended next | Partially defined, in discovery | Medium |
| **Later** | Aspirational — strategic direction | Problem statements, big boulders | Low |

### Comparison with Alternatives

| Approach | Organized By | Flexibility | Discovery-Friendly |
|----------|-------------|-------------|-------------------|
| Timeline | Calendar dates | Low | No |
| Theme-based | Strategic themes | Medium | Somewhat |
| Outcome/OKR | Measurable results | High | Yes |
| **Now/Next/Later** | **Commitment horizon** | **High** | **Yes** |

NNL and outcome-based approaches are complementary, not competing. NNL provides temporal structure; outcomes provide the content within each horizon.

### Why NNL Fits Markplane

- **AI-native alignment**: Confidence gradient maps naturally to how AI assistants reason about work — what's actionable now vs. what needs more definition.
- **File-based simplicity**: A single field value (now/next/later/done) is trivial to store in YAML frontmatter and query.
- **No false precision**: Markdown-first projects don't benefit from Gantt charts. NNL embraces the inherent uncertainty of software planning.
- **Graduated density**: The web UI can show richer detail for "Now" epics and compact views for "Later" — matching the confidence gradient with visual weight.

## Architectural Decision: NNL as the Epic Lifecycle

### Decision

Replace the epic `status` values `planned | active | done` with `now | next | later | done`, making the Now/Next/Later pattern a first-class concept in the data model rather than a derived computation in the UI.

### Rationale

**Priority and time horizon are orthogonal dimensions.** The previous model used priority (critical/high vs. medium/low) as a proxy for Next vs. Later. This conflation meant:

- A medium-priority epic could never appear in "Next" (even if it's a quick win that unblocks other work)
- A high-priority epic could never appear in "Later" (even if prerequisites aren't met or discovery is incomplete)

The correct model treats them independently:

- **Priority** = how important is this? (intrinsic to the epic)
- **Time horizon** = when do we intend to work on this? (planning decision based on readiness, dependencies, capacity)

Priority determines rank *within* a horizon column. Horizon determines *which column*.

### Key Best Practices

1. **Only "Now" carries commitment.** Next and Later are directional, not promises.
2. **Items graduate leftward with increasing detail.** Later → Next → Now should correlate with increasing specificity and task breakdown.
3. **Actively prune "Later."** Without curation it becomes an infinite dumping ground. Regular review cadence is essential.
4. **Different review cadences per horizon.** Now at sprint level, Next monthly, Later quarterly.
5. **Priority ranks within a column, not across columns.** A critical item in "Later" signals unresolved prerequisites, not misclassification.

## References

- Janna Bastow, "Why I Invented the Now-Next-Later Roadmap" (ProdPad, 2012)
- Product School, "Curves Ahead: Navigating Change with Now-Next-Later Roadmap"
- Marty Cagan, "Product Roadmaps" (SVPG) — critique of feature-oriented roadmaps
- Teresa Torres, Opportunity Solution Trees — complementary discovery framework
- Aha!, ProdPad, airfocus — tools with first-class NNL support
