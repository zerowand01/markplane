# Decision Log

Lightweight decision log. Format: `## YYYY-MM-DD: Decision Title`

---

## 2026-02-22: Now/Next/Later roadmap pattern

Replace epic statuses `planned | active | done` with `now | next | later | done`, making the Now/Next/Later pattern a first-class concept in the data model. Priority and time horizon are orthogonal dimensions that should not be conflated.

See [[NOTE-v3uet]] for full research and rationale.

---

## 2026-02-23: Do not migrate from npm to pnpm

The web UI package manager stays as npm. The cost-benefit ratio is unfavorable — npm is a build-time detail invisible to end users, there's no CI/CD pipeline where speed matters, and pnpm's strict hoisting risks breaking TipTap/Radix dependencies. Revisit if CI/CD is added or the project adopts a workspace structure.

See [[NOTE-b3wiq]] for full analysis.

---
