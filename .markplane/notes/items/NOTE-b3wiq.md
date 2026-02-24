---
id: NOTE-b3wiq
title: "npm to pnpm migration analysis"
type: decision
status: draft
tags: ["web-ui", "tooling"]
related: []
created: 2026-02-23
updated: 2026-02-23
---

# npm to pnpm migration analysis

## Context

Explored whether the web UI (`crates/markplane-web/ui/`) should migrate from npm to pnpm as its package manager.

### Current Setup

- **35 total deps** (27 prod + 8 dev), ~2,761 transitive packages
- **631 MB** `node_modules/`, 514 KB `package-lock.json` (lockfile v3)
- No `.npmrc`, no `engines` field, no CI/CD pipeline
- Standard `npm install && npm run build` documented in ~8 files
- Static export (`output: "export"`) — zero Node.js at runtime
- Optional `rust-embed` bakes `out/` into the Rust binary

### Arguments For pnpm

- **Disk savings**: Content-addressable store + symlinked `node_modules` reduces the 631 MB footprint, especially across multiple projects
- **Strict resolution**: Catches phantom dependency bugs (importing undeclared transitive deps)
- **Faster repeat installs**: Global store cache benefits CI pipelines
- **Readable lockfile**: `pnpm-lock.yaml` is easier to review than `package-lock.json`
- **Industry momentum**: pnpm is the modern standard for TS/React projects

### Arguments Against pnpm

- **Low impact**: Web UI is build-time only with static output — npm works fine
- **Extra tool requirement**: npm ships with Node.js; pnpm requires separate install or Corepack
- **Strict hoisting risks**: TipTap (many `@tiptap/*` packages) and Radix UI can break with pnpm's non-hoisted `node_modules` — may need `shamefully-hoist=true` workarounds
- **Documentation churn**: ~8 files reference `npm` — changing them is busywork with no functional gain
- **Single package**: pnpm's workspace advantages don't apply to this one-package setup
- **No CI/CD yet**: Install speed benefits are irrelevant without automated builds

## Decision

**Do not migrate.** The cost-benefit ratio is unfavorable for this project.

### Rationale

1. npm is a build-time detail invisible to end users (static files baked into a Rust binary)
2. No CI/CD pipeline exists where install speed would matter
3. Adding pnpm increases contributor friction (extra prerequisite)
4. TipTap/Radix peer dependency trees risk compatibility issues with strict hoisting
5. Higher-value improvements exist (CI/CD automation, build.rs integration) that would deliver more than a package manager swap

### Revisit Conditions

Reconsider if:
- A CI/CD pipeline is added (install speed becomes measurable)
- The project adopts a monorepo/workspace structure (pnpm workspaces shine here)
- Multiple frontend packages are introduced
- npm introduces a regression or security concern
