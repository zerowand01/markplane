---
id: TASK-hv5xv
title: Web server security hardening
status: backlog
priority: critical
type: enhancement
effort: medium
epic: null
plan: null
depends_on: []
blocks:
- TASK-r4mdt
related:
- TASK-hdhaz
assignee: null
tags:
- security
- web-server
- pre-release
position: a1
created: 2026-03-02
updated: 2026-03-02
---

# Web server security hardening

## Description

The web server in `serve.rs` has several security issues identified in the pre-release audit. All are in `crates/markplane-cli/src/commands/serve.rs` and should be addressed in a single pass.

**Permissive CORS in production** (Critical)
`CorsLayer::permissive()` is applied in all modes including production (lines 107-127). Any website can make cross-origin requests to the local markplane server and perform full CRUD. Restrict CORS to `http://localhost:{port}` in non-dev modes.

**Link API maps all errors to 400** (High)
`POST /api/link` (line 1972) uses hardcoded `StatusCode::BAD_REQUEST` for all errors including I/O failures. Every other handler uses `map_core_error()`. Replace with `.map_err(map_core_error)?`.

**No request body size limit** (Medium)
No `DefaultBodyLimit` layer configured (lines 81-127). Add explicit `axum::extract::DefaultBodyLimit::max(2_097_152)` layer.

**Search query no max length** (Medium)
`GET /api/search` (lines 2011-2016) validates `query.len() < 2` but not max length. Add `|| query.len() > 500` check.

**Error messages expose filesystem paths** (Low)
`map_core_error()` (lines 418-435) passes full I/O error strings including absolute paths to HTTP clients. Return generic messages for `Io` and `Yaml` variants.

**No HTTP response compression** (Low)
Add `tower_http::compression::CompressionLayer`.

**No WebSocket origin validation or connection limit** (Low)
WebSocket endpoint (lines 307-346) has no `Origin` header validation and no connection cap. Validate origin against `localhost:{port}`.

## Acceptance Criteria

- [ ] CORS restricted to `http://localhost:{port}` in non-dev mode; `--dev` retains permissive
- [ ] `POST /api/link` uses `map_core_error()` like all other handlers
- [ ] Explicit body size limit configured
- [ ] Search query max length enforced
- [ ] `Io` and `Yaml` errors return generic messages to HTTP clients (full errors logged server-side)
- [ ] Response compression enabled
- [ ] WebSocket validates `Origin` header
- [ ] All existing tests pass

## References

- Source: Pre-release audit (2026-03-02)
