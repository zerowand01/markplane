pub mod protocol;
pub mod resources;
pub mod tools;

use std::io::{self, BufRead, Write};
use std::path::PathBuf;

use markplane_core::Project;
use serde_json::{json, Value};

use protocol::{JsonRpcRequest, JsonRpcResponse, INVALID_REQUEST, METHOD_NOT_FOUND, PARSE_ERROR};

/// Run the MCP server over stdin/stdout.
pub fn run(project_path: Option<PathBuf>) -> anyhow::Result<()> {
    eprintln!("markplane-mcp: starting server");

    let project = resolve_project(project_path)?;

    const MAX_LINE_LENGTH: usize = 1_048_576; // 1 MB

    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut stdout = stdout.lock();
    let mut reader = stdin.lock();
    let mut line = String::new();

    loop {
        line.clear();
        match reader.read_line(&mut line) {
            Ok(0) => break, // EOF
            Ok(_) => {}
            Err(e) => {
                eprintln!("markplane-mcp: read error: {}", e);
                break;
            }
        }

        if line.len() > MAX_LINE_LENGTH {
            eprintln!(
                "markplane-mcp: input line exceeds {} bytes, skipping",
                MAX_LINE_LENGTH
            );
            let response = JsonRpcResponse::error(
                Value::Null,
                PARSE_ERROR,
                format!("Input line exceeds maximum length of {} bytes", MAX_LINE_LENGTH),
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
                let response = JsonRpcResponse::error(
                    Value::Null,
                    PARSE_ERROR,
                    format!("Parse error: {}", e),
                );
                write_response(&mut stdout, &response);
                continue;
            }
        };

        // Notifications (no id) don't get a response
        let is_notification = request.id.is_none()
            || request.id.as_ref().is_some_and(|v| v.is_null());

        if request.method == "notifications/initialized" || request.method == "initialized" {
            // Client acknowledgement, no response needed
            eprintln!("markplane-mcp: client initialized");
            continue;
        }

        let response = handle_request(&project, &request);

        if !is_notification
            && let Some(resp) = response {
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
        "tools/list" => Some(handle_tools_list(id)),
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
    let project_name = project
        .load_config()
        .map(|c| c.project.name)
        .unwrap_or_else(|_| "Unknown".to_string());

    format!(
        "Markplane is an AI-native, markdown-first project management system for the project \"{project_name}\". \
Files are the source of truth, git is the changelog.\n\
\n\
## Entity Types\n\
- TASK-NNN: Tasks (bugs, features, chores). Statuses: draft → backlog → planned → in-progress → done (also cancelled)\n\
- EPIC-NNN: Strategic epics grouping related tasks. Statuses: planned → active → done\n\
- PLAN-NNN: Implementation plans linked to tasks. Statuses: draft → approved → in-progress → done\n\
- NOTE-NNN: Research notes, ideas, and decisions. Statuses: draft → active → archived\n\
\n\
## Recommended Workflow\n\
1. Use markplane_summary or markplane_query to understand current project state\n\
2. Use markplane_show to read full details of any item by ID\n\
3. Use markplane_add to create new tasks (creates template with placeholder content)\n\
4. Edit the markdown file directly to fill in the body content\n\
5. Use markplane_update/markplane_start/markplane_done to track progress\n\
6. Use markplane_archive/markplane_unarchive to manage completed items\n\
7. Use markplane_sync to regenerate indexes and context summaries\n\
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

fn handle_tools_list(id: Value) -> JsonRpcResponse {
    JsonRpcResponse::success(id, tools::list_tools())
}

fn handle_tools_call(
    id: Value,
    project: &Project,
    params: &Option<Value>,
) -> JsonRpcResponse {
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

fn handle_resources_read(
    id: Value,
    project: &Project,
    params: &Option<Value>,
) -> JsonRpcResponse {
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
