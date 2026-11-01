---
id: TASK-5uyks
title: Remove npm overrides once upstream ships patched transitives
status: backlog
priority: low
type: chore
effort: medium
epic: null
plan: null
depends_on: []
blocks: []
related: []
assignee: null
tags:
- dependencies
- web-ui
position: aB
created: 2026-11-01
updated: 2026-11-01
---

# Remove npm overrides once upstream ships patched transitives

## Context

PR #101 (2026-11-01) added an `overrides` block to `crates/markplane-web/ui/package.json` to clear Dependabot alerts that upstream pins made unfixable:

| Override | Forces | Because |
|---|---|---|
| `@tailwindcss/typography` → `postcss-selector-parser` | `^6.1.5` | typography 0.5.20 exact-pins 6.0.10 (GHSA-rj75-hqrm-r3gf). Upstream fix: tailwindlabs/tailwindcss-typography#431 / #429, unmerged. |
| `@typescript-eslint/typescript-estree` → `minimatch` | `^11.0.3` | declares `^10.2.2`; 10.x contains the vulnerable `firstPhasePreProcess` path (GHSA-g3hv-75wg-m249) |
| `@ts-morph/common` → `minimatch` | `^11.0.3` | declares `^10.0.1`; same advisory |

An override keeps winning over whatever upstream declares, including future majors, and Dependabot version-update PRs never edit the `overrides` block. Each entry must be removed by hand once its parent declares a safe version, or it becomes a silent pin.

Do **not** replace these with a top-level `minimatch` override. v11's CommonJS build has no `default` export; `eslint-plugin-import` and two `jsx-a11y` utilities call `_interopRequireDefault(require("minimatch")).default(...)`, which would crash lint if those rules were ever enabled. Verified 2026-11-01.

## Acceptance Criteria

- [ ] `@tailwindcss/typography` override removed after a typography release that depends on `postcss-selector-parser >= 6.1.5`; `npm ls postcss-selector-parser` shows no copy below 6.1.5
- [ ] `@typescript-eslint/typescript-estree` override removed after it declares `minimatch >= 11.0.1`
- [ ] `@ts-morph/common` override removed after it declares `minimatch >= 11.0.1`
- [ ] After each removal: `npm install`, `npm run build`, `npm run lint` pass in `crates/markplane-web/ui`
- [ ] `npm audit` reports no root vulnerability other than `deepmerge` (dismissed, no fix upstream)
- [ ] `overrides` block deleted entirely once all three are gone

## Notes

Trigger to watch for: a Dependabot PR bumping `@tailwindcss/typography`, `typescript-eslint` or `shadcn` (which pulls ts-morph). Remove the matching override in that PR rather than merging it as-is.
