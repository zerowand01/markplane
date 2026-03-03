---
id: TASK-r4mdt
title: Add authentication to web server
status: backlog
priority: someday
type: feature
effort: large
epic: null
plan: null
depends_on:
- TASK-hv5xv
blocks: []
related: []
assignee: null
tags:
- security
- web-server
- pre-release
position: Zz
created: 2026-03-02
updated: 2026-03-02
---

# Add authentication to web server

## Description

**No authentication on web server** (Feature Gap)

The web server exposes 22 REST endpoints and a WebSocket with zero authentication — no API key, no session, no token. Combined with the CORS fix in [[TASK-hv5xv]], the attack surface is reduced (only same-origin requests allowed), but any local process can still access the full API.

Even a simple static token (generated at `markplane init`, stored in `config.yaml`, required as `X-Markplane-Token` header) would significantly raise the bar. The web UI would need to read the token from a known location or receive it at startup.

This is an architectural decision — the implementation approach (static token, session cookie, etc.) and how the web UI authenticates need to be designed before implementation.

## Acceptance Criteria

- [ ] Web server requires authentication for all mutating endpoints
- [ ] Token or credential generated automatically and stored securely
- [ ] Web UI can authenticate without manual user configuration
- [ ] `--dev` mode behavior defined (skip auth or auto-inject)
- [ ] MCP server unaffected (separate stdio transport)
- [ ] Documentation updated

## References

- Source: Pre-release audit (2026-03-02)
- Related: [[TASK-hv5xv]] (CORS fix provides the immediate security boundary)
