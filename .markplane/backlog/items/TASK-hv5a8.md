---
id: TASK-hv5a8
title: 'Web UI: show an upgrade banner on data version mismatch'
status: done
priority: medium
type: feature
effort: small
epic: EPIC-c5uem
plan: null
depends_on: []
blocks: []
related:
- TASK-pgups
- TASK-4ed4i
assignee: null
tags:
- web-ui
- migration
position: aB
created: 2026-10-22
updated: 2026-10-22
---

# Web UI: show an upgrade banner on data version mismatch

## Description

Since [[TASK-pgups]], `markplane serve` answers every API request with HTTP 503 and the error code `data_version_mismatch` when the `version` in `.markplane/config.yaml` is newer than the running binary supports. The response body carries the message that tells the user to upgrade markplane. The web UI does not recognise that code, so a user who pulls migrated data while the dashboard is open sees generic error toasts or empty pages with no explanation and no action.

The CLI and MCP server already deliver the message directly. The web UI is the only surface where the user cannot see it, and it has the same deadline as the gate itself: the banner must be in installed binaries before the first breaking format change ships, because the old binary is the one that displays it.

## Acceptance Criteria

- [x] When any API request returns 503 with code `data_version_mismatch`, the UI shows a persistent banner at the top of every page with the message from the response body
- [x] The banner is not dismissable and does not depend on the WebSocket connection, which is behind the same gate
- [x] Queries stop retrying while the mismatch is present, so the console and network tab are not flooded
- [x] Other error codes keep their existing handling (toasts, inline errors)
- [x] The banner clears without a manual reload once requests succeed again, for example after the user restarts an upgraded `markplane serve`

## Notes

Implemented 2026-10-22. All API calls go through one `request()` helper in `lib/api.ts` that throws an `ApiError` carrying the server's error code; a mismatch sets a module-level store (`lib/data-version.ts`) that the banner reads with `useSyncExternalStore`. The query client's retry predicate skips retries for that code. Recovery: while a mismatch is recorded, any WebSocket event and any reconnect invalidates every query, and the banner component refetches everything the moment the store flips back to null, so pages that errored during the mismatch recover without a reload. The `/ws` route is behind the same gate, so a page loaded during the mismatch has no socket and recovers through the reconnect loop (5s backoff while mismatched) once the server accepts connections again. Smoke-tested in headless Chrome over DevTools: banner appears on watcher-triggered refetch and on navigation; clears after a server restart with fixed data and after an in-place fix, with errored pages re-rendering.

The error envelope is `{ "error": { "code": "data_version_mismatch", "message": "..." } }`. The check runs in a per-request middleware in `serve.rs`, so the first failing request can be any query. Hooking the shared fetch helper or the TanStack Query cache's global error handler is simpler than handling it per hook.

## References

- [[TASK-pgups]] — the server-side gate this surfaces
- [[TASK-4ed4i]] — the migration framework whose first migration will make this visible
