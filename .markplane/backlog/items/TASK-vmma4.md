---
id: TASK-vmma4
title: Add GitHub repo scaffolding for open source readiness
status: backlog
priority: high
type: chore
effort: small
epic: EPIC-bb6pe
plan: null
depends_on: []
blocks: []
related: []
assignee: null
tags:
- release
- github
position: a0V
created: 2026-03-02
updated: 2026-03-02
---

# Add GitHub repo scaffolding for open source readiness

## Description

Add the standard GitHub community and automation files that make the repo ready for open source. Currently the repo has README, LICENSE, and CONTRIBUTING but is missing security reporting instructions, issue/PR templates, dependency automation, and issue labels. These files are low-effort boilerplate that set expectations for contributors and automate maintenance.

Also update CONTRIBUTING.md with an issue/support policy section covering what to file, bug report expectations, feature request policy, stale issue handling, and solo maintainer response time disclaimer.

## Acceptance Criteria

- [ ] `SECURITY.md` at repo root with private vulnerability reporting instructions
- [ ] `.github/ISSUE_TEMPLATE/bug_report.md` — version, OS, repro steps, expected/actual, logs
- [ ] `.github/ISSUE_TEMPLATE/feature_request.md` — problem statement, proposed solution, alternatives
- [ ] `.github/PULL_REQUEST_TEMPLATE.md` — what changed, how tested, breaking change checklist
- [ ] `.github/dependabot.yml` configured for both `cargo` and `npm` ecosystems
- [ ] GitHub issue labels created: `bug`, `enhancement`, `good first issue`, `help wanted`, `breaking`, `wontfix`, `duplicate`, `needs reproduction`
- [ ] CONTRIBUTING.md updated with "Issues & Support" section
- [ ] CONTRIBUTING.md updated with Conventional Commits guideline for PR titles
- [ ] README updated with solo-maintainer expectations ("solo-maintained, responses may be slow")

## Notes

- SECURITY.md: Use a simple template. Private reporting via GitHub's security advisory feature or email.
- Dependabot: Weekly schedule is fine for both ecosystems. Target `master` branch. Group minor/patch updates.
- Labels: Can be created via `gh label create` or manually in GitHub settings.
- CONTRIBUTING.md additions: Keep it solo-maintainer-friendly — "Issues and PRs are welcome, responses may be slow. Use reactions instead of +1 comments. Stale issues may be closed after 30 days."
- No CODE_OF_CONDUCT.md needed yet — add when the project has active contributors.
- No CHANGELOG.md — GitHub Release notes serve this purpose.
- Conventional Commits: Document as a PR title convention in CONTRIBUTING.md (`feat:`, `fix:`, `chore:`, `docs:`, `refactor:`, `!` for breaking). Not enforced by tooling — just a naming guideline. Enables future automated release notes/version bumping if needed.
- Consider enabling GitHub Discussions for "how do I..." questions to keep issues focused on bugs and actionable feature requests.

## References

- [[EPIC-bb6pe]]
- [[TASK-yzftd]]
