---
id: TASK-3zks9
title: Add shell install script for curl-based installation
status: done
priority: high
type: feature
effort: medium
epic: EPIC-bb6pe
plan: null
depends_on: []
blocks: []
related: []
assignee: null
tags:
- release
- install
position: Zz
created: 2026-03-09
updated: 2026-03-10
---

# Add shell install script for curl-based installation

## Description

macOS Gatekeeper blocks unsigned binaries downloaded via browsers. Binaries downloaded via `curl` don't get the quarantine xattr, so a `curl | sh` install script sidesteps Gatekeeper entirely while providing a one-liner install experience on macOS and Linux. This is the standard pattern used by starship, deno, bun, and rustup.

Also update the README Installation section to reflect the new install methods: shell script as primary for macOS/Linux, manual binary download as secondary (with Gatekeeper workaround noted), build from source as tertiary. Stub out a Homebrew section placeholder for [[TASK-8xvxq]] to fill in later.

## Acceptance Criteria

- [x] `install.sh` in repo root detects OS (macOS, Linux) and arch (arm64, x86_64)
- [x] Downloads correct binary from GitHub Releases for the detected platform
- [x] Verifies SHA256 checksum against the checksums file in the release
- [x] Installs binary to `~/.local/bin/` (or configurable via env var)
- [x] Prints clear success message with next steps
- [x] Fails gracefully with helpful error on unsupported platforms (e.g., Windows, FreeBSD)
- [x] `curl -fsSL <raw-url>/install.sh | sh` works end-to-end on macOS and Linux
- [x] Script uploaded to existing v0.1.0 release via `gh release upload v0.1.0 install.sh`
- [x] Release workflow updated to upload `install.sh` automatically on future releases
- [x] README Installation section rewritten: Homebrew stub first, shell script second, manual download third, build from source fourth
- [x] macOS Gatekeeper workaround (`xattr -d`) documented for manual download path

## Decisions

- **No PowerShell installer for now** — Windows users can download the zip from GitHub Releases. Add a PowerShell script later if there's demand.
- **Checksum verification included** — Unlike starship/deno/bun which skip checksums, we verify against the SHA256 checksums file already published in each release. Near-zero cost, meaningful security improvement.
- **`~/.local/bin/` as default install location** — Follows XDG conventions, doesn't require sudo. Configurable via `INSTALL_DIR` env var.

## Notes

- `curl` and `wget` do NOT set the `com.apple.quarantine` xattr, so downloaded binaries run without Gatekeeper warnings
- The release workflow already generates SHA256 checksums — the script just needs to download and verify against them
- Reference implementations: starship (~550 lines), deno (~110 lines), bun (~400 lines) — aim for ~100-150 lines
- The release workflow needs a small addition to upload `install.sh` as a release asset
- README ordering changed from spec: Homebrew stub placed first (as the eventual recommended method), shell script second, to match the conventional priority order

## References

- [[EPIC-bb6pe]]
- [[TASK-gpxpw]]
- [[TASK-8xvxq]]
