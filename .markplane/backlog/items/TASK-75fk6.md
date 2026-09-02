---
id: TASK-75fk6
title: Upgrade TypeScript past 5.x when toolchain allows
status: backlog
priority: someday
type: chore
effort: small
epic: null
plan: null
depends_on: []
blocks: []
related: []
assignee: null
tags:
- web-ui
- dependencies
position: a6
created: 2026-09-02
updated: 2026-09-02
---

# Upgrade TypeScript past 5.x when toolchain allows

## Description

`crates/markplane-web/ui/package.json` pins `typescript: "^5"` (currently
5.9.3) while TypeScript 7.x is released. Dependabot repeatedly opens upgrade
PRs that fail CI, most recently #30 (closed).

The blocker is upstream and specific:

```
eslint-config-next@16.3.3  ->  typescript-eslint: ^8.46.0
@typescript-eslint/typescript-estree  peer:  typescript >=4.8.4 <6.1.0
```

`typescript-eslint` is transitive via `eslint-config-next`, so the supported
TypeScript ceiling is not ours to set. TS 7 fails with
`typescript-eslint does not support TS 7.0` during `npm run lint`, which is
the ubuntu-only leg of CI (the frontend build does not run on Windows).

TS 6.0.x *is* inside the supported range and would merge today. It is
deliberately not being taken: the ceiling is `<6.1.0`, so the next TypeScript
minor re-blocks it, and dependabot will keep proposing 7.x regardless. The
intermediate hop buys weeks and stops no noise.

Not urgent. TypeScript is a devDependency used only to type-check the
frontend build; nothing in the shipped binary depends on it, and 5.9.3 is
supported with no known vulnerabilities.

## Acceptance Criteria

- [ ] `typescript` upgraded to the current major, or a deliberate decision
      recorded to stay on 5.x
- [ ] `npm run lint` and `npm run build` pass on the ubuntu CI leg
- [ ] No pinning or override added that suppresses the peer-range check

## Decisions

- **Skip TS 6.0.x, wait for a direct 5 -> 7 jump**: the `<6.1.0` ceiling makes
  6.0.x a short-lived resting place. Revisit when `eslint-config-next` ships
  a `typescript-eslint` whose peer range admits TS 7.
- **Do not override the peer range**: forcing the install past a declared
  incompatibility trades a red CI check for silent lint misbehavior.

## Notes

- Watch `eslint-config-next` releases; the constraint moves when Next bumps
  its bundled `typescript-eslint`.
- Recheck with:
  `npm view @typescript-eslint/typescript-estree@latest peerDependencies`
- Same shape as #29 (eslint 9 -> 10, also closed): both are Next's bundled
  lint stack lagging a major release, not a defect here.

## References
