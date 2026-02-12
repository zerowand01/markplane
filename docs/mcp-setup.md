# MCP Server Setup Guide

MCP (Model Context Protocol) is the standard protocol for connecting AI coding tools to external systems, and Markplane's MCP server provides structured, typed access to your project management data over JSON-RPC 2.0 via stdio.

## Installation

The MCP server is a separate binary from the CLI. Install it with:

```bash
cargo install --path crates/markplane-mcp
```

This puts `markplane-mcp` in your Cargo bin directory (typically `~/.cargo/bin/`). Verify it's available:

```bash
markplane-mcp --help
```

## Configuration

### Claude Code

Add to `~/.claude/mcp.json`:

```json
{
  "mcpServers": {
    "markplane": {
      "command": "markplane-mcp",
      "args": [],
      "env": {}
    }
  }
}
```

To specify an explicit project path:

```json
{
  "mcpServers": {
    "markplane": {
      "command": "markplane-mcp",
      "args": ["--project", "/path/to/repo"],
      "env": {}
    }
  }
}
```

### Cursor

Add to `.cursor/mcp.json` in your project root (same format):

```json
{
  "mcpServers": {
    "markplane": {
      "command": "markplane-mcp",
      "args": [],
      "env": {}
    }
  }
}
```

## How It Works

The MCP server runs as a stdio process. It reads JSON-RPC 2.0 requests (one per line) from stdin and writes responses to stdout. Diagnostic messages go to stderr. The server automatically locates the `.markplane/` directory by walking up from the current working directory, or uses the `--project` argument if provided.

**Protocol version**: `2025-11-25`

**Security limits**: Input lines are capped at 1 MB to prevent memory exhaustion.

## Tool Catalog

The server exposes 16 tools via the `tools/list` method.

### Context & Navigation

| Tool | Description | Required Params | Optional Params |
|------|-------------|-----------------|-----------------|
| `markplane_summary` | Get project state summary. Returns a text overview of the project. | *(none)* | *(none)* |
| `markplane_context` | Generate a context summary for the project or a specific item. | *(none)* | `item` (string): item ID for focused context; `focus` (string): area like `active-work`, `blocked`, `metrics` |

### Query & Search

| Tool | Description | Required Params | Optional Params |
|------|-------------|-----------------|-----------------|
| `markplane_query` | Query backlog items with optional filters. Returns matching items. | *(none)* | `status` (string[]): filter by status; `priority` (string[]): filter by priority; `epic` (string): filter by epic ID; `tags` (string[]): filter by tags; `assignee` (string): filter by assignee |
| `markplane_show` | Get full details of any item by ID. Returns frontmatter and body. | `id` (string) | *(none)* |
| `markplane_graph` | Build a reference graph showing how items relate to each other. | `id` (string) | `depth` (number): max traversal depth, default 2 |

### Create & Update

| Tool | Description | Required Params | Optional Params |
|------|-------------|-----------------|-----------------|
| `markplane_add` | Create a new backlog item. | `title` (string) | `type` (string): feature/bug/enhancement/chore/research/spike, default feature; `priority` (string): critical/high/medium/low/someday, default medium; `effort` (string): xs/small/medium/large/xl, default medium; `epic` (string): parent epic ID; `tags` (string[]): tags for the item |
| `markplane_write` | Write or replace the markdown body content of an item. Preserves frontmatter. | `id` (string), `body` (string) | *(none)* |
| `markplane_update` | Update fields on an existing item. | `id` (string) | `status` (string): new status; `priority` (string): new priority; `assignee` (string): new assignee |
| `markplane_start` | Set a backlog item to in-progress status. | `id` (string) | *(none)* |
| `markplane_done` | Mark a backlog item as done. | `id` (string) | *(none)* |

### Workflow

| Tool | Description | Required Params | Optional Params |
|------|-------------|-----------------|-----------------|
| `markplane_promote` | Promote a note to a backlog item. | `note_id` (string) | `priority` (string): default medium; `effort` (string): default medium |
| `markplane_plan` | Create an implementation plan linked to a backlog item. | `backlog_id` (string) | `title` (string): defaults to "Implementation plan for {item title}" |
| `markplane_link` | Link two items with a blocks/depends_on relationship. | `from` (string), `to` (string), `relation` (string): `blocks` or `depends_on` | *(none)* |

### Maintenance

| Tool | Description | Required Params | Optional Params |
|------|-------------|-----------------|-----------------|
| `markplane_sync` | Regenerate INDEX.md files and .context/ summaries. | *(none)* | *(none)* |
| `markplane_check` | Validate all cross-references in the project. Reports broken links. | *(none)* | *(none)* |
| `markplane_stale` | Find items that have not been updated recently. | *(none)* | `days` (number): threshold in days, default 14 |

## Resource Catalog

The server exposes 7 resources via the `resources/list` method. All resources return `text/markdown` content.

### Static Resources

| URI | Name | Description |
|-----|------|-------------|
| `markplane://summary` | Project Summary | Overview of the project state including item counts by status |
| `markplane://active-work` | Active Work | Currently in-progress backlog items |
| `markplane://blocked` | Blocked Items | Items that have unresolved dependencies or need attention |

### Dynamic Resource Templates

| URI Template | Name | Description |
|--------------|------|-------------|
| `markplane://backlog/{id}` | Backlog Item | Full content of a backlog item by ID (e.g. `markplane://backlog/BACK-042`) |
| `markplane://epic/{id}` | Epic | Full content of an epic by ID (e.g. `markplane://epic/EPIC-001`) |
| `markplane://plan/{id}` | Plan | Full content of an implementation plan by ID (e.g. `markplane://plan/PLAN-001`) |
| `markplane://note/{id}` | Note | Full content of a note by ID (e.g. `markplane://note/NOTE-001`) |

## JSON-RPC Examples

All communication uses [JSON-RPC 2.0](https://www.jsonrpc.org/specification). Each request is a single JSON object on one line.

### Initialize

**Request:**
```json
{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "protocolVersion": "2025-11-25",
    "capabilities": {
      "tools": {},
      "resources": {}
    },
    "serverInfo": {
      "name": "markplane",
      "version": "0.1.0",
      "description": "AI-native, markdown-first project management. Files are the source of truth, git is the changelog."
    },
    "instructions": "Markplane is an AI-native, markdown-first project management system for the project \"My Project\". ..."
  }
}
```

The `instructions` field contains dynamic guidance built from the project's `config.yaml`, including entity types, status workflows, the create-then-edit workflow, and cross-reference syntax. The full text is typically ~1500 characters.

### markplane_add

**Request:**
```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "tools/call",
  "params": {
    "name": "markplane_add",
    "arguments": {
      "title": "Add dark mode support",
      "type": "feature",
      "priority": "high",
      "effort": "medium",
      "epic": "EPIC-001",
      "tags": ["ui", "theming"]
    }
  }
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "result": {
    "content": [
      {
        "type": "text",
        "text": "{\"id\":\"BACK-001\",\"title\":\"Add dark mode support\"}"
      }
    ]
  }
}
```

### markplane_ls (query)

**Request:**
```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "method": "tools/call",
  "params": {
    "name": "markplane_query",
    "arguments": {
      "status": ["in-progress"],
      "priority": ["critical", "high"]
    }
  }
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "result": {
    "content": [
      {
        "type": "text",
        "text": "[\n  {\n    \"id\": \"BACK-001\",\n    \"title\": \"Add dark mode support\",\n    \"status\": \"in-progress\",\n    \"priority\": \"high\",\n    \"effort\": \"medium\"\n  }\n]"
      }
    ]
  }
}
```

### markplane_show

**Request:**
```json
{
  "jsonrpc": "2.0",
  "id": 4,
  "method": "tools/call",
  "params": {
    "name": "markplane_show",
    "arguments": {
      "id": "BACK-001"
    }
  }
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": 4,
  "result": {
    "content": [
      {
        "type": "text",
        "text": "---\nid: BACK-001\ntitle: \"Add dark mode support\"\nstatus: in-progress\npriority: high\ntype: feature\n...\n---\n\n# Add dark mode support\n\n## Description\n..."
      }
    ]
  }
}
```

## Error Handling

The server uses standard JSON-RPC error codes:

| Code | Meaning |
|------|---------|
| -32700 | Parse error (malformed JSON) |
| -32600 | Invalid request |
| -32601 | Method not found |
| -32602 | Invalid params (unknown tool, missing URI) |
| -32603 | Internal error (file I/O, validation failure) |

**Example error response:**
```json
{
  "jsonrpc": "2.0",
  "id": 5,
  "error": {
    "code": -32603,
    "message": "Item BACK-999 not found in backlog or its archive"
  }
}
```

## Notifications

The server recognizes the `notifications/initialized` and `initialized` methods as client acknowledgment notifications. These do not produce a response.

## Architecture

The MCP server wraps the same `markplane-core` library used by the CLI:

```
CLI binary ──> Core Library (Rust) <── MCP Server (stdio)
                     |
              .markplane/ (markdown files)
```

Both the CLI and MCP server share identical file parsing, YAML handling, cross-reference validation, and context generation logic.
