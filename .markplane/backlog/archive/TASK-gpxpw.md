---
id: TASK-gpxpw
title: Build GitHub Actions release workflow with embedded web UI
status: done
priority: high
type: chore
effort: large
epic: EPIC-bb6pe
plan: null
depends_on: []
blocks: []
related: []
assignee: null
tags:
- ci
- release
position: a1
created: 2026-02-13
updated: 2026-03-10
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
  - Linux x86_64 — `x86_64-unknown-linux-musl` on `ubuntu-latest` (via `musl-tools`)
  - Windows x86_64 — `x86_64-pc-windows-msvc` on `windows-latest`
- [ ] Smoke test each binary after build (`./markplane --version`)
- [ ] Creates GitHub Release with binary assets and SHA256 checksums
- [ ] Release notes auto-generated via `generate_release_notes: true`
- [ ] Binaries named with Rust target triples (e.g. `markplane-v0.1.0-aarch64-apple-darwin.tar.gz`, `.zip` for Windows)

## Decisions

- **Separate per-architecture binaries (no universal binary)**: Follows Rust ecosystem convention (ripgrep, fd, bat all ship per-arch). Half the download size per binary. Simpler build — no `lipo` step. Package managers (Homebrew, cargo-binstall) handle arch selection. Can add a universal binary as an additional artifact later if requested.
- **macOS x86_64 via cross-compile on arm64 runner**: `macos-latest` is arm64; add `x86_64-apple-darwin` target via `rustup target add`. Avoids needing a separate `macos-13` (Intel) runner.
- **`musl-tools` apt package over `cross`**: `apt-get install musl-tools` + `rustup target add x86_64-unknown-linux-musl` on the Ubuntu runner. Avoids Docker overhead of `cross` and keeps the build straightforward.
- **`npm ci` over `npm install`**: Deterministic installs from lockfile in CI.
- **Smoke test each binary**: Near-zero cost (`--version` takes <1s). Catches dynamic linking failures, broken musl static linking, rust-embed initialization issues, and cross-compilation architecture mismatches before publishing.
- **Tag-version validation**: Prevents publishing a binary where `markplane --version` and the GitHub Release version disagree. Simple 5-line shell check at workflow start.

## Notes

- Pin Rust toolchain to 1.93.0 (matching CI) for reproducibility
- Use `Swatinem/rust-cache@v2` — matrix builds are expensive without caching
- Use `node-version-file: crates/markplane-web/ui/.nvmrc` (matching CI) instead of hardcoding Node version
- Set `permissions: contents: write` at workflow level (required for release creation)
- No OpenSSL or system library dependencies in the crate — simplifies cross-compilation
- Frontend static export outputs to `out/` (configured via `output: "export"` in `next.config.ts`) — this is what `rust-embed` picks up
- Next.js 16.1.6, React 19 — versions pinned in `package-lock.json`
- Job structure: `validate` → `build-frontend` → `build-binaries` (matrix) → `create-release`
- Use `softprops/action-gh-release@v2` for release creation
- **v0.1.0 released successfully on 2026-03-09** — all 4 platform builds passed, macOS arm64 binary tested locally (init, add, ls, serve with embedded web UI all working). macOS Gatekeeper blocked the downloaded binary; resolved with `xattr -d com.apple.quarantine` — led to [[TASK-3zks9]] (install script) to avoid this for users.

## References

- [[EPIC-bb6pe]]
- [[TASK-yzftd]]
- [[TASK-8xvxq]]
- [Release process](../../../docs/releasing.md)
