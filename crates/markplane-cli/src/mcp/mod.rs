pub mod protocol;
pub mod resources;
pub mod tools;

use std::io::{self, BufRead, Read, Write};
use std::path::PathBuf;

use markplane_core::Project;
use serde_json::{Value, json};

use protocol::{INVALID_REQUEST, JsonRpcRequest, JsonRpcResponse, METHOD_NOT_FOUND, PARSE_ERROR};

/// Run the MCP server over stdin/stdout.
pub fn run(project_path: Option<PathBuf>) -> anyhow::Result<()> {
    eprintln!("markplane-mcp: starting server");

    let project = resolve_project(project_path)?;

    // Sync derived files on startup to ensure they're up-to-date
    if let Err(e) = project.sync_all() {
        eprintln!("markplane-mcp: sync warning: {}", e);
    }

    const MAX_LINE_LENGTH: usize = 1_048_576; // 1 MB

    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut stdout = stdout.lock();
    let mut reader = stdin.lock();
    let mut line = String::new();

    loop {
        line.clear();
        match (&mut reader)
            .take(MAX_LINE_LENGTH as u64)
            .read_line(&mut line)
        {
            Ok(0) => break, // EOF
            Ok(_) => {}
            Err(e) => {
                eprintln!("markplane-mcp: read error: {}", e);
                break;
            }
        }

        // If we hit the take limit without finding a newline, the line is oversized
        if line.len() >= MAX_LINE_LENGTH && !line.ends_with('\n') {
            eprintln!(
                "markplane-mcp: input line exceeds {} bytes, skipping",
                MAX_LINE_LENGTH
            );
            // Drain remaining bytes until newline or EOF without allocating
            while let Ok(buf) = reader.fill_buf() {
                if buf.is_empty() {
                    break;
                }
                if let Some(pos) = buf.iter().position(|&b| b == b'\n') {
                    reader.consume(pos + 1);
                    break;
                }
                let len = buf.len();
                reader.consume(len);
            }
            let response = JsonRpcResponse::error(
                Value::Null,
                PARSE_ERROR,
                format!(
                    "Input line exceeds maximum length of {} bytes",
                    MAX_LINE_LENGTH
                ),
            );
            write_response(&mut stdout, &response);
            continue;
        }

        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let request: JsonRpcRequest = match serde_json::from_str(trimmed) {
            Ok(req) => req,
            Err(e) => {
                let response =
                    JsonRpcResponse::error(Value::Null, PARSE_ERROR, format!("Parse error: {}", e));
                write_response(&mut stdout, &response);
                continue;
            }
        };

        // Per JSON-RPC 2.0, only requests without an `id` member are notifications.
        // `"id": null` is a valid ID that MUST receive a response.
        let is_notification = request.id.is_none();

        if request.method == "notifications/initialized" || request.method == "initialized" {
            // Client acknowledgement, no response needed
            eprintln!("markplane-mcp: client initialized");
            continue;
        }

        let response = handle_request(&project, &request);

        if !is_notification && let Some(resp) = response {
            write_response(&mut stdout, &resp);
        }
    }

    eprintln!("markplane-mcp: server shutting down");
    Ok(())
}

fn resolve_project(path: Option<PathBuf>) -> anyhow::Result<Project> {
    match path {
        Some(p) => {
            let markplane_dir = if p.ends_with(".markplane") {
                p
            } else {
                p.join(".markplane")
            };
            if !markplane_dir.is_dir() {
                anyhow::bail!(
                    "No .markplane/ directory found at {}",
                    markplane_dir.display()
                );
            }
            Ok(Project::new(markplane_dir))
        }
        None => Project::from_current_dir().map_err(|e| e.into()),
    }
}

fn handle_request(project: &Project, request: &JsonRpcRequest) -> Option<JsonRpcResponse> {
    let id = request.id.clone().unwrap_or(Value::Null);

    match request.method.as_str() {
        "initialize" => Some(handle_initialize(id, project)),
        "tools/list" => Some(handle_tools_list(id, project)),
        "tools/call" => Some(handle_tools_call(id, project, &request.params)),
        "resources/list" => Some(handle_resources_list(id)),
        "resources/read" => Some(handle_resources_read(id, project, &request.params)),
        "ping" => Some(JsonRpcResponse::success(id, json!({}))),
        _ => {
            eprintln!("markplane-mcp: unknown method: {}", request.method);
            Some(JsonRpcResponse::error(
                id,
                METHOD_NOT_FOUND,
                format!("Method not found: {}", request.method),
            ))
        }
    }
}

fn handle_initialize(id: Value, project: &Project) -> JsonRpcResponse {
    let instructions = build_instructions(project);

    JsonRpcResponse::success(
        id,
        json!({
            "protocolVersion": "2025-11-25",
            "capabilities": {
                "tools": {},
                "resources": {}
            },
            "serverInfo": {
                "name": "markplane",
                "version": env!("CARGO_PKG_VERSION"),
                "description": "AI-native, markdown-first project management. Files are the source of truth, git is the changelog."
            },
            "instructions": instructions
        }),
    )
}

fn build_instructions(project: &Project) -> String {
    let config = project.load_config().ok();
    let project_name = config
        .as_ref()
        .map(|c| c.project.name.clone())
        .unwrap_or_else(|| "Unknown".to_string());

    let task_types = config
        .as_ref()
        .map(|c| c.task_types.join(", "))
        .unwrap_or_else(|| "feature, bug, enhancement, chore, research, spike".to_string());
    let note_types = config
        .as_ref()
        .map(|c| c.note_types.join(", "))
        .unwrap_or_else(|| "research, analysis, idea, decision, meeting".to_string());
    let task_statuses = config
        .as_ref()
        .map(|c| {
            use markplane_core::StatusCategory;
            StatusCategory::ALL
                .iter()
                .filter_map(|cat| {
                    let statuses = c.workflows.task.statuses_in(*cat);
                    if statuses.is_empty() {
                        None
                    } else {
                        Some(statuses.join(", "))
                    }
                })
                .collect::<Vec<_>>()
                .join(" → ")
        })
        .unwrap_or_else(|| {
            "draft → backlog → planned → in-progress → done (also cancelled)".to_string()
        });

    format!(
        "Markplane is an AI-native, markdown-first project management system for the project \"{project_name}\". \
Files are the source of truth, git is the changelog.\n\
\n\
## Entity Types\n\
- TASK-NNN: Tasks (bugs, features, chores). Statuses: {task_statuses}\n\
- EPIC-NNN: Strategic epics grouping related tasks. Statuses: later → next → now → done\n\
- PLAN-NNN: Implementation plans linked to tasks. Statuses: draft → approved → in-progress → done\n\
- NOTE-NNN: Research notes, ideas, and decisions. Statuses: draft → active → archived\n\
\n\
## Configured Types\n\
- Task types: {task_types}\n\
- Note types: {note_types}\n\
\n\
## Recommended Workflow\n\
1. Use markplane_summary or markplane_query to understand current project state\n\
2. Use markplane_show to read full details of any item by ID\n\
3. Use markplane_add to create new items (creates template with placeholder content)\n\
4. Edit the markdown file directly to fill in the body content\n\
5. Use markplane_update to track progress\n\
6. Use markplane_move to reorder tasks within a priority group (top, bottom, before/after another task)\n\
7. Use markplane_archive/markplane_unarchive to manage completed items\n\
8. Use markplane_sync to regenerate indexes and context summaries\n\
\n\
## File Editing\n\
Items are markdown files in .markplane/ — edit them directly using your file editing tools:\n\
- Tasks: .markplane/backlog/items/TASK-NNN.md\n\
- Epics: .markplane/roadmap/items/EPIC-NNN.md\n\
- Plans: .markplane/plans/items/PLAN-NNN.md\n\
- Notes: .markplane/notes/items/NOTE-NNN.md\n\
The body content (below the YAML frontmatter) is free-form markdown. \
Use markplane tools for structural operations (create, status, linking, sync) \
and edit files directly for content changes (descriptions, acceptance criteria, notes).\n\
\n\
## Cross-References\n\
Use [[TASK-042]] wiki-link syntax to reference other items in markdown body content. \
The prefix determines the entity type and location."
    )
}

fn handle_tools_list(id: Value, project: &Project) -> JsonRpcResponse {
    JsonRpcResponse::success(id, tools::list_tools(project))
}

fn handle_tools_call(id: Value, project: &Project, params: &Option<Value>) -> JsonRpcResponse {
    let params = match params {
        Some(p) => p,
        None => {
            return JsonRpcResponse::error(
                id,
                INVALID_REQUEST,
                "Missing params for tools/call".to_string(),
            );
        }
    };

    let name = match params.get("name").and_then(|v| v.as_str()) {
        Some(n) => n,
        None => {
            return JsonRpcResponse::error(
                id,
                INVALID_REQUEST,
                "Missing 'name' in tools/call params".to_string(),
            );
        }
    };

    let arguments = params
        .get("arguments")
        .cloned()
        .unwrap_or_else(|| json!({}));

    tools::call_tool(id, project, name, arguments)
}

fn handle_resources_list(id: Value) -> JsonRpcResponse {
    JsonRpcResponse::success(id, resources::list_resources())
}

fn handle_resources_read(id: Value, project: &Project, params: &Option<Value>) -> JsonRpcResponse {
    let params = match params {
        Some(p) => p,
        None => {
            return JsonRpcResponse::error(
                id,
                INVALID_REQUEST,
                "Missing params for resources/read".to_string(),
            );
        }
    };

    let uri = match params.get("uri").and_then(|v| v.as_str()) {
        Some(u) => u,
        None => {
            return JsonRpcResponse::error(
                id,
                INVALID_REQUEST,
                "Missing 'uri' in resources/read params".to_string(),
            );
        }
    };

    resources::read_resource(id, project, uri)
}

fn write_response(out: &mut impl Write, response: &JsonRpcResponse) {
    match serde_json::to_string(response) {
        Ok(json) => {
            if let Err(e) = writeln!(out, "{}", json) {
                eprintln!("markplane-mcp: write error: {}", e);
            }
            if let Err(e) = out.flush() {
                eprintln!("markplane-mcp: flush error: {}", e);
            }
        }
        Err(e) => {
            eprintln!("markplane-mcp: serialize error: {}", e);
        }
    }
}
