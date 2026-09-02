---
id: TASK-sk3g3
title: Document cross-platform CI decision
status: done
priority: low
type: chore
effort: xs
epic: null
plan: null
depends_on: []
blocks: []
related: []
assignee: null
tags:
- ci
position: a7
created: 2026-09-02
updated: 2026-09-02
---

# Document cross-platform CI decision

## Description

CI ran on `ubuntu-latest` only from its creation ([[TASK-yzftd]]) until
2026-09-02, when `windows-latest` was added in #48. That original decision
carried an explicit escape clause:

> **Linux-only CI**: macOS runners are 10x more expensive and slower to
> provision. Platform-specific compile/link issues are caught by the release
> workflow which builds on macOS and Windows. Add macOS to CI only if
> platform-specific test failures emerge.

Platform-specific test failures emerged. #35 reported that every mutating
operation failed on Windows with `os error 33`, and the first Windows CI run
confirmed it: 27 of 75 CLI integration tests failed, and the core and MCP
suites never ran at all. The clause fired exactly as written, so #48 honors
the original decision rather than reversing it.

That reasoning currently lives only in a PR body. This repo keeps CI
decisions in task files, so it is recorded here.

## Acceptance Criteria

- [x] The rationale for adding `windows-latest` is recorded alongside the
      original Linux-only decision
- [x] CONTRIBUTING.md tells contributors which platforms CI runs on
- [x] The PR template checklist matches the commands CI actually runs

## Decisions

- **Windows, not macOS**: the cost argument that justified Linux-only does
  not extend equally. GitHub bills Windows runners at 2x and macOS at 10x, so
  Windows is an order of magnitude cheaper to add. macOS stays out of CI and
  remains covered by the release workflow's build matrix.
- **Clippy and tests on both platforms, rustfmt and frontend on Linux only**:
  formatting is textual and platform-independent, and the Next.js build does
  not vary by runner OS. Duplicating either adds minutes for no signal.
  Clippy runs on both because lints can be `cfg`-gated.
- **`fail-fast: false`**: one platform failing must not cancel the other, or a
  Windows failure would hide the Linux result and vice versa.
- **Timeout raised 20 -> 30 minutes**: Windows Rust builds are materially
  slower on a cold cache.
- **Required status checks updated in the `master-protection` ruleset**: the
  job rename from `Check` to `Check (${{ matrix.os }})` orphaned the required
  context, blocking every PR to master until both new contexts were
  registered. Renaming a CI job that a ruleset requires is a breaking change
  to the merge gate — check `/rulesets`, not just
  `/branches/master/protection`, which does not report rulesets.

## Notes

- The class of bug this catches was invisible for months: `release.yml` builds
  a Windows binary but only smoke-tests `--version`, so it never exercised the
  code paths that were broken. Expanding those smoke tests is [[TASK-t672g]].
- Windows is now a required check, so this class of regression cannot merge
  past CI again.

## References

- [[TASK-yzftd]]
- [[TASK-t672g]]
- [[EPIC-bb6pe]]
