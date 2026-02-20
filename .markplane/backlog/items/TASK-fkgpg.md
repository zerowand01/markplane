---
id: TASK-fkgpg
title: Swap serde_yaml to serde_yaml_ng
status: backlog
priority: low
type: chore
effort: small
tags:
- dependencies
epic: EPIC-6zdf4
plan: null
depends_on: []
blocks: []
assignee: null
position: a1
created: 2026-02-10
updated: 2026-02-10
---

# Swap serde_yaml to serde_yaml_ng

## Description

`serde_yaml` 0.9 was deprecated and archived by its maintainer in March 2024. While it works fine and has no known vulnerabilities, it will not receive security patches if an issue is found. `serde_yaml_ng` is the community-recommended maintained fork with an identical API, making it a drop-in replacement.

This is a low-urgency maintenance task — do it when convenient, not as a priority.

## Acceptance Criteria

- [ ] Replace `serde_yaml = "0.9"` with `serde_yaml_ng` in workspace `Cargo.toml`
- [ ] Update all `use serde_yaml::` to `use serde_yaml_ng::`
- [ ] Update `MarkplaneError::Yaml` from variant to use `serde_yaml_ng::Error`
- [ ] All tests pass
- [ ] Clippy clean

## Notes

The API is identical — this is a mechanical find-and-replace across the codebase. Avoid `serde_yml` (community warns against it). Do not use the lower-level `yaml-rust2` directly as it lacks serde integration.

## References

- Rust forum discussion: https://users.rust-lang.org/t/serde-yaml-deprecation-alternatives/108868
- Rust forum on current status: https://users.rust-lang.org/t/serde-and-yaml-support-status/125684
