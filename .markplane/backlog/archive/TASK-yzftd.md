---
id: TASK-yzftd
title: Set up GitHub Actions CI pipeline
status: done
priority: high
type: chore
effort: medium
epic: EPIC-bb6pe
plan: null
depends_on: []
blocks: []
related: []
assignee: null
tags:
- ci
- release
position: a0
created: 2026-02-13
updated: 2026-03-10
---

# Set up GitHub Actions CI pipeline

## Description

Create a GitHub Actions CI workflow that runs on every push and PR. Should validate the full stack: both Rust workspace crates (`markplane-core`, `markplane-cli`) and the Next.js frontend (`crates/markplane-web/ui/`). This is the foundation for the release workflow — we need confidence that the build is clean before cutting releases.

## Acceptance Criteria

- [ ] Workflow triggers on push to all branches (`'**'`, no tags) and on PRs targeting `master`
- [ ] Runs on ubuntu-latest (Linux only — release workflow covers macOS/Windows)
- [ ] Runs `cargo fmt --check` (formatting gate)
- [ ] Runs `cargo clippy --locked --workspace --all-targets -- -D warnings` (default features only)
- [ ] Runs `cargo test --locked --workspace`
- [ ] Runs `npm ci && npm run lint && npm run build` in `crates/markplane-web/ui/`
- [ ] Uses `dtolnay/rust-toolchain` with explicit `components: clippy, rustfmt` (minimal profile omits them)
- [ ] Uses Rust toolchain caching (`Swatinem/rust-cache@v2`, placed after toolchain setup)
- [ ] Uses Node.js with npm caching (`actions/setup-node` with `cache: 'npm'`)
- [ ] Add `.nvmrc` file in `crates/markplane-web/ui/` pinning Node.js 24
- [ ] Add `concurrency` group with conditional `cancel-in-progress` (cancel on branches, not on `master`)

## Decisions

- **Linux-only CI**: macOS runners are 10x more expensive and slower to provision. Platform-specific compile/link issues are caught by the release workflow which builds on macOS and Windows. Add macOS to CI only if platform-specific test failures emerge.
- **Default features only for clippy/test**: Dropping `--all-features` avoids coupling the frontend build to every Rust lint/test run. The `embed-ui` feature is structurally simple (rust-embed + mime_guess) and gets validated in the release workflow. Keeps CI fast and decoupled.
- **`npm ci` over `npm install`**: Deterministic installs from lockfile. Fails if lockfile is out of sync with package.json. Requires `package-lock.json` to be committed.
- **ESLint step added**: Already configured in the frontend (`eslint.config.mjs`). Gating it in CI prevents lint drift at ~5s cost.
- **Push on all branches, not just `master`**: Gives developers immediate CI feedback on feature branches without waiting to open a PR. Uses `branches: ['**']` (not bare `push:`) to avoid triggering on tag pushes, which are handled by the release workflow ([[TASK-gpxpw]]).
- **`--locked` on cargo commands**: Ensures `Cargo.lock` is in sync — same rationale as `npm ci`. Applies to `clippy` and `test` but not `fmt` (which doesn't resolve dependencies).
- **Concurrency control**: Cancels redundant in-progress CI runs when a branch is pushed multiple times. Uses conditional `cancel-in-progress: ${{ github.ref != 'refs/heads/master' }}` so master runs always complete (important once release/deploy workflows exist).

## Notes

- Rust stable toolchain pinned to 1.93.0 (edition 2024, matches `rust-version` in Cargo.toml)
- No `rust-toolchain.toml` exists — pin version via `dtolnay/rust-toolchain` action config with `components: clippy, rustfmt` (minimal profile excludes them)
- Node.js 24 LTS (Active) — Next.js 16 requires Node 18+
- No frontend test framework configured (ESLint only for static analysis) — no `npm test` step needed
- `notify` crate uses macOS fsevent backend but compiles conditionally; Linux uses inotify — tests should pass on Ubuntu
- Step ordering for fast failure: fmt → clippy → test → frontend (lint + build)
- Ensure `package-lock.json` is committed in `crates/markplane-web/ui/` — `npm ci` fails without it
- CI security: Don't expose repo secrets to PRs from forks. GitHub Actions restricts secrets on `pull_request` from forks by default — don't override this with `pull_request_target` unless necessary.

## References

- [[EPIC-bb6pe]]
- [[TASK-gpxpw]]
