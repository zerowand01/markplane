---
id: TASK-zesq4
title: Upgrade ESLint past 9.x when eslint-config-next supports it
status: backlog
priority: someday
type: chore
effort: small
epic: null
plan: null
depends_on: []
blocks: []
related:
- TASK-75fk6
assignee: null
tags:
- web-ui
- dependencies
position: a7
created: 2026-09-30
updated: 2026-09-30
---

# Upgrade ESLint past 9.x when eslint-config-next supports it

## Description

`crates/markplane-web/ui/package.json` pins `eslint: "^9"` while ESLint 10 is
released. Dependabot PR #58 (ESLint 9.39.2 -> 10.9.1, closed) failed the
ubuntu CI leg at `npm run lint`:

```
TypeError: Error while loading rule 'react/display-name':
  contextOrFilename.getFilename is not a function
    at .../eslint-config-next/node_modules/eslint-plugin-react/lib/util/version.js
```

ESLint 10 removed the deprecated `context.getFilename()` API, and the
`eslint-plugin-react` bundled with `eslint-config-next` still calls it. Its
peer range also stops at `^9`, as do `eslint-plugin-import` and
`eslint-plugin-jsx-a11y`. As with TypeScript ([[TASK-75fk6]]), the ceiling
comes from `eslint-config-next`, so it is not ours to lift.

ESLint major updates are ignored in `.github/dependabot.yml` until this is
unblocked, so this task is the reminder to revisit.

Not urgent. ESLint is a devDependency used only for linting; nothing in the
shipped binary depends on it.

## Acceptance Criteria

- [ ] `eslint` is on 10.x and `npm run lint` passes in CI
- [ ] The `eslint` major-version ignore rule is removed from `.github/dependabot.yml`

## Notes

Check whether a newer `eslint-config-next` ships an `eslint-plugin-react`
that supports ESLint 10 before attempting the upgrade.

## References

- Dependabot PR #58 (closed)
