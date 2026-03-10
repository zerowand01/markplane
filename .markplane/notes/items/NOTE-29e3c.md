---
id: NOTE-29e3c
title: Homebrew tap infrastructure and token rotation
status: active
type: decision
related: []
tags:
- release
- homebrew
- infrastructure
created: 2026-03-10
updated: 2026-03-10
---

# Homebrew tap infrastructure and token rotation

## Overview

Markplane is distributed via a Homebrew tap at `zerowand01/homebrew-markplane`. The release workflow in the main repo automatically updates the tap formula when a new version is tagged.

## Components

| Component | Location | Purpose |
|-----------|----------|---------|
| Tap repo | `zerowand01/homebrew-markplane` | Hosts `Formula/markplane.rb` |
| Formula template | `.github/formula-template.rb` | Template with `%%VERSION%%` and `%%SHA_*%%` placeholders |
| Release workflow | `.github/workflows/release.yml` | `update-homebrew` job renders template and pushes to tap |
| PAT secret | `zerowand01/markplane` repo settings → Secrets → Actions | `TAP_GITHUB_TOKEN` — authenticates cross-repo push |

## How it works

1. You tag a release (e.g. `git tag v0.2.0 && git push --tags`)
2. Release workflow builds binaries for 3 unix targets + Windows
3. `update-homebrew` job downloads the 3 unix `.tar.gz` archives
4. Computes SHA256 for each, substitutes into the formula template via `sed`
5. Clones the tap repo using `TAP_GITHUB_TOKEN`, commits the updated formula, pushes

## Token: TAP_GITHUB_TOKEN

- **Type**: Fine-grained personal access token
- **Scope**: Only `zerowand01/homebrew-markplane`
- **Permission**: Contents → Read and write
- **Expiration**: None (low-risk — can only write to a single-file tap repo)

### Rotation procedure

1. Go to `github.com/settings/tokens?type=beta`
2. Delete the old token
3. Generate a new one with the same scope (repo: `zerowand01/homebrew-markplane`, permission: Contents read/write)
4. Go to `zerowand01/markplane` → Settings → Secrets and variables → Actions
5. Update `TAP_GITHUB_TOKEN` with the new value

### Why no expiration

The token can only write to a repo containing one Ruby file. A leak is low-impact (visible in git history, easily reverted) and detectable. Forced expiration adds operational risk (broken releases if forgotten) for minimal security benefit. Optional calendar-based rotation is recommended instead.

## Install command

```bash
brew install zerowand01/markplane/markplane
```
