---
id: TASK-gpxpw
title: Build GitHub Actions release workflow with embedded web UI
status: backlog
priority: high
type: chore
effort: large
epic: EPIC-bb6pe
plan: null
depends_on:
- TASK-yzftd
blocks:
- TASK-8xvxq
related: []
assignee: null
tags:
- ci
- release
position: a1
created: 2026-02-13
updated: 2026-03-02
---

# Build GitHub Actions release workflow with embedded web UI

## Description

Create a GitHub Actions release workflow triggered by version tags (e.g. `v0.1.0`). The workflow builds the Next.js frontend, then compiles the Rust binary with `--features embed-ui` for each target platform. Uploads the resulting binaries to a GitHub Release with checksums.

## Acceptance Criteria

- [ ] Triggered by pushing a `v*` tag
- [ ] Validates tag version matches `Cargo.toml` version (e.g. tag `v0.1.0` must match `version = "0.1.0"`)
- [ ] Builds frontend once: `npm ci && npm run build` in `crates/markplane-web/ui/`
- [ ] Shares frontend `out/` artifact across platform build jobs via `actions/upload-artifact`
- [ ] Builds Rust binary with `--release --features embed-ui` for:
  - macOS arm64 (Apple Silicon) — `aarch64-apple-darwin` on `macos-latest`
  - macOS x86_64 — `x86_64-apple-darwin` on `macos-latest`
  - Linux x86_64 — `x86_64-unknown-linux-musl` on `ubuntu-latest` (via `cross` or musl target)
  - Windows x86_64 — `x86_64-pc-windows-msvc` on `windows-latest`
- [ ] Smoke test each binary after build (`./markplane --version`)
- [ ] Creates GitHub Release with binary assets and SHA256 checksums
- [ ] Release notes auto-generated via `generate_release_notes: true`
- [ ] Binaries named consistently (e.g. `markplane-v0.1.0-darwin-arm64.tar.gz`)

## Decisions

- **Separate per-architecture binaries (no universal binary)**: Follows Rust ecosystem convention (ripgrep, fd, bat all ship per-arch). Half the download size per binary. Simpler build — no `lipo` step. Package managers (Homebrew, cargo-binstall) handle arch selection. Can add a universal binary as an additional artifact later if requested.
- **`npm ci` over `npm install`**: Deterministic installs from lockfile in CI.
- **Smoke test each binary**: Near-zero cost (`--version` takes <1s). Catches dynamic linking failures, broken musl static linking, rust-embed initialization issues, and cross-compilation architecture mismatches before publishing.
- **Tag-version validation**: Prevents publishing a binary where `markplane --version` and the GitHub Release version disagree. Simple 5-line shell check at workflow start.

## Notes

- No OpenSSL or system library dependencies in the crate — simplifies cross-compilation
- Frontend static export outputs to `out/` (configured via `output: "export"` in `next.config.ts`) — this is what `rust-embed` picks up
- Next.js 16.1.6, React 19 — versions pinned in `package-lock.json`
- Use `cross` or `cargo-zigbuild` for Linux musl target if native musl toolchain isn't available on Ubuntu runner
- Native runners for macOS (arm64 + x86_64 both build on `macos-latest` which is arm64) and Windows
- Job structure: `build-frontend` → `build-binaries` (matrix, `needs: build-frontend`) → `create-release` (needs: `build-binaries`)
- Use `softprops/action-gh-release@v2` for release creation

## References

- [[EPIC-bb6pe]]
- [[TASK-yzftd]]
- [[TASK-8xvxq]]
