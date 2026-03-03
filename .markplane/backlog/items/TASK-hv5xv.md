---
id: TASK-hv5xv
title: Web server security hardening
status: done
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
`CorsLayer::permissive()` is applied in all three branches (dev, embed-ui, non-embed-ui). Any website can make cross-origin requests to the local markplane server and perform full CRUD. Restrict CORS to `http://localhost:{port}` and `http://127.0.0.1:{port}` in non-dev modes (both origins needed since the server binds to `127.0.0.1` but users access via `localhost`).

**Link API maps all errors to 400** (High)
`POST /api/link` uses hardcoded `StatusCode::BAD_REQUEST` for all `link_items()` errors including I/O failures. Every other handler uses `map_core_error()`. Replace with `.map_err(map_core_error)?`.

**No request body size limit** (Medium)
No `DefaultBodyLimit` layer configured on the router. Add explicit `axum::extract::DefaultBodyLimit::max(2_097_152)` layer.

**Search query no max length** (Medium)
`GET /api/search` validates `query.len() < 2` but has no upper bound. Add `|| query.len() > 500` check. Note: `.len()` is byte length, which is fine as a rough guard.

**Error messages expose filesystem paths** (Low)
`map_core_error()` passes full `std::io::Error` and `serde_yaml::Error` strings — including absolute filesystem paths — to HTTP clients. Return generic messages for `Io` and `Yaml` variants and log the full error server-side with `eprintln!`.

**No HTTP response compression** (Low)
Add `tower_http::compression::CompressionLayer`. Requires adding `"compression-gzip"` (or `"compression-full"`) to the `tower-http` features in the workspace `Cargo.toml`.

**No WebSocket origin validation or connection limit** (Low)
`ws_handler` accepts all upgrade requests with no `Origin` check and no connection cap. The broadcast channel buffer (256) provides some backpressure but doesn't limit concurrent connections. Validate origin against `localhost:{port}` and `127.0.0.1:{port}`.

## Acceptance Criteria

- [ ] CORS restricted to `http://localhost:{port}` and `http://127.0.0.1:{port}` in non-dev mode; `--dev` retains permissive
- [ ] `POST /api/link` uses `map_core_error()` like all other handlers
- [ ] Explicit body size limit configured
- [ ] Search query max length enforced
- [ ] `Io` and `Yaml` errors return generic messages to HTTP clients (full errors logged server-side)
- [ ] Response compression enabled
- [ ] WebSocket validates `Origin` header against allowed origins
- [ ] `tower-http` features updated in workspace `Cargo.toml` (add `compression-gzip`)
- [ ] All existing tests pass

## References

- Source: Pre-release audit (2026-03-02)
