---
id: TASK-hcbud
title: Cleanup and documentation fixes
status: backlog
priority: medium
type: chore
effort: xs
epic: null
plan: null
depends_on: []
blocks: []
related: []
assignee: null
tags:
- cleanup
- docs
- pre-release
position: a1V
created: 2026-03-02
updated: 2026-03-02
---

# Cleanup and documentation fixes

## Description

Small cleanup items and documentation fixes identified in the pre-release audit. All low-risk, quick wins.

**CONTRIBUTING.md and architecture.md reference nonexistent functions** (Low)
`CONTRIBUTING.md` and `docs/architecture.md` reference `sanitize_yaml_string()` and `format_yaml_list()` which no longer exist in the codebase (likely removed during refactoring). Remove or update the stale references.

**`manifest.rs` returns `Result<_, String>` instead of `MarkplaneError`** (Low)
At `manifest.rs:44-47`, `load_manifest()` breaks the error handling convention. All other core functions return `Result<_, MarkplaneError>`. Callers must `.ok()` or remap. Change to `MarkplaneError`.

**Remove unused `anyhow` dependency from `markplane-core`** (Low)
`anyhow` appears in `markplane-core/Cargo.toml` but is never imported. Core uses `thiserror`/`MarkplaneError`. Run `cargo remove anyhow` from markplane-core.

**`docs/` watched non-recursively by file watcher** (Low)
At `serve.rs:226-228`, `notify::RecursiveMode::NonRecursive` means changes in subdirectories like `docs/web-ui/` are not detected. Change to `RecursiveMode::Recursive`.

## Acceptance Criteria

- [ ] No references to nonexistent functions in CONTRIBUTING.md or architecture.md
- [ ] `load_manifest()` returns `Result<_, MarkplaneError>`
- [ ] `anyhow` removed from markplane-core dependencies
- [ ] `docs/` directory watched recursively
- [ ] `cargo build --workspace` succeeds
- [ ] All existing tests pass

## References

- Source: Pre-release audit (2026-03-02)
