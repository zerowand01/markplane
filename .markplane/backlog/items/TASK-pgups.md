---
id: TASK-pgups
title: 'Data version gate: refuse to run when config.yaml version is newer than the binary'
status: done
priority: high
type: feature
effort: small
epic: EPIC-c5uem
plan: null
depends_on: []
blocks: []
related:
- TASK-4ed4i
assignee: null
tags:
- architecture
- migration
position: a8
created: 2026-10-22
updated: 2026-10-22
---

# Data version gate: refuse to run when config.yaml version is newer than the binary

## Description

`config.yaml` has carried a `version: 1` field since the first release, but nothing reads it. An old markplane binary pointed at a future v2 `.markplane/` directory proceeds as if nothing changed, and any config write (for example the web UI settings endpoint) round-trips the config through the struct and silently drops fields it does not know about.

A forward-compatibility gate only protects users if it is already in the binaries they have installed before the first breaking format change ships. That makes it the one part of the migration story that has to land early, independent of the full `markplane migrate` framework ([[TASK-4ed4i]]), which is deferred until there is a real migration to run.

The check belongs in one place in core, not in each of the 24 command entry points. Data newer than the binary means "upgrade markplane"; data older than the binary gets a distinct message (it cannot fire today, and [[TASK-4ed4i]] will point it at `markplane migrate` once that command exists).

## Acceptance Criteria

- [x] Core exposes a `DATA_VERSION` constant and a single `check_data_version()` that compares it against `config.yaml`
- [x] Every CLI command that operates on an existing project refuses with a clear message when the versions differ; `init` is unaffected
- [x] The message distinguishes newer data (upgrade markplane) from older data (this release cannot upgrade it)
- [x] MCP: every known tool call returns `isError: true` and every resource read returns an error on mismatch; unknown tools keep their METHOD_NOT_FOUND error; the check runs per call so a `git pull` mid-session is caught
- [x] MCP server still starts on mismatch so the AI client receives the message, but skips the startup sync
- [x] `markplane serve` refuses to start on mismatch, and a per-request middleware returns 503 `data_version_mismatch` if the version changes while it runs
- [x] Unit, CLI integration, and MCP integration tests cover both directions
- [x] Docs describe the `version` field and the gate behaviour

## Notes

Split out of [[TASK-4ed4i]] on 2026-10-22. The gate is implemented in `Project::from_current_dir()`; `Project::discover()` locates a project without the gate for the MCP server's startup path. `check_data_version()` parses only the `version` field, because a breaking config schema change would otherwise fail full `Config` parsing before the comparison runs.

## References

- [[TASK-4ed4i]] — the deferred migration framework this gate is a precondition for
