# Releasing

Markplane uses a tag-triggered release workflow. Pushing a `v*` tag to GitHub builds platform binaries and creates a GitHub Release.

## Prerequisites

- All CI checks passing on `master`
- Version in `Cargo.toml` (workspace) updated to match the intended release

## Steps

1. **Bump the version** in `Cargo.toml` workspace:

   ```toml
   [workspace.package]
   version = "0.2.0"
   ```

   Commit and push to `master`:

   ```bash
   git add Cargo.toml
   git commit -m "chore: bump version to 0.2.0"
   git push
   ```

2. **Wait for CI** to pass on the version bump commit.

3. **Tag and push**:

   ```bash
   git tag v0.2.0
   git push origin v0.2.0
   ```

   This triggers the release workflow.

## What the Release Workflow Does

1. **Validates** the tag version matches `Cargo.toml` (e.g., tag `v0.2.0` must match `version = "0.2.0"`)
2. **Builds the frontend** once (`npm ci && npm run build` in `crates/markplane-web/ui/`)
3. **Builds platform binaries** with `--release --features embed-ui` (frontend embedded in the binary):
   - `aarch64-apple-darwin` (macOS Apple Silicon)
   - `x86_64-apple-darwin` (macOS Intel)
   - `x86_64-unknown-linux-musl` (Linux x86_64, statically linked)
   - `x86_64-pc-windows-msvc` (Windows x86_64)
4. **Smoke tests** each binary (`markplane --version`)
5. **Creates a GitHub Release** with:
   - Binary archives (e.g., `markplane-v0.2.0-aarch64-apple-darwin.tar.gz`)
   - SHA256 checksums
   - Auto-generated release notes
6. **Updates the Homebrew tap** — computes SHA256 checksums for the 3 unix archives, renders `.github/formula-template.rb` with the new version and checksums, and pushes the updated formula to `zerowand01/homebrew-markplane`

## Naming Convention

Binary archives use Rust target triples:

```
markplane-v{VERSION}-{TARGET}.tar.gz    (macOS, Linux)
markplane-v{VERSION}-{TARGET}.zip       (Windows)
```

## Secrets

| Secret | Purpose |
|--------|---------|
| `TAP_GITHUB_TOKEN` | Fine-grained PAT scoped to `zerowand01/homebrew-markplane` (Contents read/write). Used by the `update-homebrew` job to push formula updates. See [[NOTE-29e3c]] for rotation procedure. |

## Notes

- Regular pushes to `master` only trigger CI, not a release build
- You can bump the version without tagging — no release happens until a tag is pushed
- The tag must be on a commit where `Cargo.toml` has the matching version
- Rust toolchain is pinned to the `rust-version` in [`Cargo.toml`](../Cargo.toml) for reproducibility
