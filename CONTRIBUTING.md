# Contributing to Markplane

Thank you for your interest in contributing to Markplane! This guide covers everything you need to get started.

## Prerequisites

- **Rust 1.93.0+** (edition 2024) — install via [rustup](https://rustup.rs/)
- **Git** for version control

Verify your setup:

```bash
rustc --version   # 1.93.0 or later
cargo --version
```

## Clone and Build

```bash
git clone https://github.com/your-org/markplane.git
cd markplane
cargo build --workspace
```

## Workspace Structure

Markplane is a Cargo workspace with two crates:

| Crate | Type | Binary | Role |
|-------|------|--------|------|
| `markplane-core` | Library | — | Data models, CRUD, sync, references, context generation |
| `markplane-cli` | Binary | `markplane` | CLI interface (clap), MCP server (`markplane mcp` subcommand), web UI server |

The CLI binary depends on `markplane-core` for all business logic. It provides terminal commands, an integrated MCP server (JSON-RPC 2.0 over stdio), and a web UI server.

## Running Tests

```bash
# All tests
cargo test --workspace

# By crate
cargo test -p markplane-core     # Unit tests
cargo test -p markplane-cli      # CLI, MCP, and serve integration tests
```

Integration tests use `assert_cmd` with the `cargo_bin_cmd!()` macro (not the deprecated `Command::cargo_bin()`). MCP integration tests live in `crates/markplane-cli/tests/mcp_integration.rs`.

## Linting

```bash
cargo clippy --workspace
```

All code must be clippy-clean with no warnings.

## Error Handling Conventions

Each crate uses a different error strategy appropriate to its role:

- **`markplane-core`**: Uses `thiserror` to define `MarkplaneError` — a typed enum with variants like `Io`, `Yaml`, `NotFound`, `InvalidId`, `InvalidTransition`, `InvalidStatus`, `DuplicateId`, `BrokenReference`, `NotInitialized`, `Config`, and `Frontmatter`. Functions return `markplane_core::Result<T>`.

- **`markplane-cli`**: Uses `anyhow::Result` for top-level error handling. Core errors propagate naturally through `?` since `MarkplaneError` implements `std::error::Error`. The MCP module (`src/mcp/`) uses `Result<String, String>` for tool handlers, mapping errors to JSON-RPC error responses with appropriate error codes (`PARSE_ERROR`, `INVALID_REQUEST`, `METHOD_NOT_FOUND`, `INVALID_PARAMS`, `INTERNAL_ERROR`).

## Code Style

- **Rust edition 2024** — use modern syntax features (e.g., `let` chains in `if let`).
- **Elide explicit lifetimes** when Rust can infer them (`clippy::needless_lifetimes`).
- **No external regex dependency** — reference extraction uses manual byte scanning.
- **serde conventions**: Use `#[serde(rename_all = "kebab-case")]` for status enums, `#[serde(rename = "type")]` for reserved-word fields.
- Keep individual `.markplane/` files under ~2,000 tokens for AI readability.

## Key Patterns

- **Templates**: Document templates use `{PLACEHOLDER}` tokens (e.g., `{ID}`, `{TITLE}`, `{DATE}`) replaced by `render_template()` at creation time.
- **YAML safety**: Titles are sanitized via `sanitize_yaml_string()` (escapes `\`, `"`, `\n`, `\r`) before template substitution. Tags are quoted in YAML lists via `format_yaml_list()`. Title length is capped at 500 characters.
- **Random IDs**: `next_id()` generates random 5-char alphanumeric IDs and checks for collisions — no shared counter or file locking needed.
- **Frontmatter parsing**: Custom `---\nyaml\n---\nbody` splitter (not a full markdown parser) plus `serde_yaml` for deserialization.

## Adding a New CLI Command

1. Create a new module in `crates/markplane-cli/src/commands/` (e.g., `mycommand.rs`).
2. Add a variant to the `Commands` enum in `commands/mod.rs`.
3. Add the match arm in `execute()`.
4. Write integration tests using `assert_cmd`.

## Adding a New MCP Tool

1. Add the tool schema to `list_tools()` in `crates/markplane-cli/src/mcp/tools.rs`.
2. Add the match arm in `call_tool()`.
3. Implement the handler function returning `Result<String, String>`.
4. Write integration tests in `crates/markplane-cli/tests/mcp_integration.rs` that send JSON-RPC requests to the binary.

## Commit Guidelines

- Write clear commit messages summarizing the "why", not just the "what".
- Keep commits focused — one logical change per commit.
- Ensure `cargo test --workspace` and `cargo clippy --workspace` pass before committing.

## Questions?

Open an issue on the repository for questions, bugs, or feature proposals.
