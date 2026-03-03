---
id: TASK-p7hs3
title: Input validation and path traversal prevention
status: done
priority: high
type: bug
effort: xs
epic: null
plan: null
depends_on: []
blocks: []
related: []
assignee: null
tags:
- security
- validation
- pre-release
position: a2
created: 2026-03-02
updated: 2026-03-02
---

# Input validation and path traversal prevention

## Description

Two path traversal vulnerabilities where unsanitized user input is used to construct file paths.

**Template path traversal via MCP** (High)
At `manifest.rs:52-58` and `project.rs:294-302`, `template_filename()` does not sanitize the `name` parameter. An MCP client sending `template: "x/../../README"` produces `task-x/../../README.md` and `PathBuf::join` resolves `../..`, allowing reads of `.md` files outside `templates/`. MCP is the primary AI agent interface, so this is a real attack surface. Reject template names containing `/`, `\`, or `..`. Allow only `[a-zA-Z0-9_-]`.

**`documentation_paths` allows traversal outside repo root** (Medium)
At `project.rs:1032-1050`, no validation that `documentation_paths` entries resolve within the repo root. Via `PATCH /api/config` (currently no auth + permissive CORS), an attacker could set `documentation_paths: ["../../etc"]` to scan files outside the project. Canonicalize and verify paths are within `repo_root`.

## Acceptance Criteria

- [ ] Template names validated to `[a-zA-Z0-9_-]` only — reject `/`, `\`, `..`
- [ ] `documentation_paths` entries canonicalized and verified within repo root
- [ ] Existing template resolution works for valid names
- [ ] Tests cover traversal attempts for both vectors

## References

- Source: Pre-release audit (2026-03-02)
