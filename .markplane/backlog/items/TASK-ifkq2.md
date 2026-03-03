---
id: TASK-ifkq2
title: Frontmatter and file format robustness
status: planned
priority: high
type: bug
effort: medium
epic: null
plan: null
depends_on: []
blocks: []
related: []
assignee: null
tags:
- parsing
- robustness
- pre-release
position: a3V
created: 2026-03-02
updated: 2026-03-03
---

# Frontmatter and file format robustness

## Description

Several issues with how markplane handles files that aren't exactly what it expects — CRLF endings, corrupted files, malformed data, edge-case paths.

**CRLF line endings break frontmatter parsing** (High)
At `frontmatter.rs:79-91`, `find_closing_delimiter()` uses `line.len() + 1` for byte offsets, but `.lines()` strips `\r` from CRLF. Each CRLF line shifts the offset by -1. On Windows or repos with CRLF files, the closing `---` position points to the wrong byte and YAML extraction fails silently.

**Archive detection uses fragile path string matching** (Low)
At `project.rs:986, 1006, 1023`, `source.to_string_lossy().contains("/archive/")` breaks if the project path contains `/archive/`. Also uses Unix `/` separator. Check parent directory components instead.

**Silent error swallowing in `scan_dir_entries`** (Low)
At `query.rs:167-176`, file read errors and YAML parse errors are silently skipped with `continue`. Corrupted files invisibly disappear from query results. Log warnings to stderr.

**Template manifest parse failures silently fall back** (Low)
At `project.rs:272`, malformed `manifest.yaml` is silently ignored and built-in templates are used. Log a warning.

**`file_name().unwrap()` in `list_documentation_files()`** (Low)
At `project.rs:1058`, `.file_name().unwrap()` can panic on edge-case paths. Replace with `.ok_or_else()?` or `.unwrap_or_default()`. Note: risk is theoretical since glob results for `*.md` always have a file component.

## Acceptance Criteria

- [ ] Frontmatter parsing handles CRLF (`\r\n`) line endings correctly
- [ ] Archive detection checks parent directory components, not path string matching
- [ ] `scan_dir_entries` logs warnings on file read errors and parse failures
- [ ] Malformed `manifest.yaml` produces a stderr warning
- [ ] No `.unwrap()` on `file_name()` in production code
- [ ] All existing tests pass
- [ ] New test for CRLF frontmatter parsing

## References

- Source: Pre-release audit (2026-03-02)
