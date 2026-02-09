use std::fs;

use markplane_core::{
    BacklogItem, BacklogStatus, Effort, ItemType, MarkplaneDocument, Priority, Project,
    QueryFilter,
};
use serde_json::{json, Value};

use crate::protocol::{JsonRpcResponse, INTERNAL_ERROR, INVALID_PARAMS};

/// Return the list of available tools with their JSON Schema input descriptions.
pub fn list_tools() -> Value {
    json!({
        "tools": [
            {
                "name": "markplane_summary",
                "description": "Get project state summary. Returns a text overview of the project.",
                "inputSchema": {
                    "type": "object",
                    "properties": {},
                    "required": []
                }
            },
            {
                "name": "markplane_query",
                "description": "Query backlog items with optional filters. Returns matching items.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "status": {
                            "type": "array",
                            "items": { "type": "string" },
                            "description": "Filter by status (draft, backlog, planned, in-progress, done, cancelled)"
                        },
                        "priority": {
                            "type": "array",
                            "items": { "type": "string" },
                            "description": "Filter by priority (critical, high, medium, low, someday)"
                        },
                        "epic": {
                            "type": "string",
                            "description": "Filter by epic ID (e.g. EPIC-001)"
                        },
                        "tags": {
                            "type": "array",
                            "items": { "type": "string" },
                            "description": "Filter by tags (matches any)"
                        },
                        "assignee": {
                            "type": "string",
                            "description": "Filter by assignee"
                        }
                    },
                    "required": []
                }
            },
            {
                "name": "markplane_show",
                "description": "Get full details of any item by ID. Returns frontmatter and body.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "id": {
                            "type": "string",
                            "description": "Item ID (e.g. BACK-042, EPIC-001, PLAN-003, NOTE-007)"
                        }
                    },
                    "required": ["id"]
                }
            },
            {
                "name": "markplane_add",
                "description": "Create a new backlog item.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "title": {
                            "type": "string",
                            "description": "Title of the backlog item"
                        },
                        "type": {
                            "type": "string",
                            "description": "Item type (feature, bug, enhancement, chore, research, spike). Default: feature"
                        },
                        "priority": {
                            "type": "string",
                            "description": "Priority (critical, high, medium, low, someday). Default: medium"
                        },
                        "effort": {
                            "type": "string",
                            "description": "Effort size (xs, small, medium, large, xl). Default: medium"
                        },
                        "epic": {
                            "type": "string",
                            "description": "Parent epic ID (e.g. EPIC-001)"
                        },
                        "tags": {
                            "type": "array",
                            "items": { "type": "string" },
                            "description": "Tags for the item"
                        }
                    },
                    "required": ["title"]
                }
            },
            {
                "name": "markplane_update",
                "description": "Update fields on an existing item.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "id": {
                            "type": "string",
                            "description": "Item ID to update"
                        },
                        "status": {
                            "type": "string",
                            "description": "New status value"
                        },
                        "priority": {
                            "type": "string",
                            "description": "New priority value"
                        },
                        "assignee": {
                            "type": "string",
                            "description": "New assignee"
                        }
                    },
                    "required": ["id"]
                }
            },
            {
                "name": "markplane_start",
                "description": "Set a backlog item to in-progress status.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "id": {
                            "type": "string",
                            "description": "Backlog item ID to start"
                        }
                    },
                    "required": ["id"]
                }
            },
            {
                "name": "markplane_done",
                "description": "Mark a backlog item as done.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "id": {
                            "type": "string",
                            "description": "Backlog item ID to complete"
                        }
                    },
                    "required": ["id"]
                }
            },
            {
                "name": "markplane_sync",
                "description": "Regenerate INDEX.md files and .context/ summaries.",
                "inputSchema": {
                    "type": "object",
                    "properties": {},
                    "required": []
                }
            }
        ]
    })
}

/// Dispatch a tool call by name and return a JSON-RPC response.
pub fn call_tool(id: Value, project: &Project, name: &str, args: Value) -> JsonRpcResponse {
    let result = match name {
        "markplane_summary" => handle_summary(project),
        "markplane_query" => handle_query(project, &args),
        "markplane_show" => handle_show(project, &args),
        "markplane_add" => handle_add(project, &args),
        "markplane_update" => handle_update(project, &args),
        "markplane_start" => handle_start(project, &args),
        "markplane_done" => handle_done(project, &args),
        "markplane_sync" => handle_sync(project),
        _ => {
            return JsonRpcResponse::error(
                id,
                INVALID_PARAMS,
                format!("Unknown tool: {}", name),
            );
        }
    };

    match result {
        Ok(content) => JsonRpcResponse::success(
            id,
            json!({
                "content": [{
                    "type": "text",
                    "text": content
                }]
            }),
        ),
        Err(e) => JsonRpcResponse::error(id, INTERNAL_ERROR, e),
    }
}

fn handle_summary(project: &Project) -> Result<String, String> {
    // Try to read .context/summary.md first
    let summary_path = project.root().join(".context/summary.md");
    if let Ok(content) = fs::read_to_string(&summary_path) {
        return Ok(content);
    }

    // Generate a basic summary from current state
    let config = project.load_config().map_err(|e| e.to_string())?;
    let backlog_items = project
        .list_backlog_items(&QueryFilter::default())
        .map_err(|e| e.to_string())?;

    let mut in_progress = 0;
    let mut planned = 0;
    let mut draft = 0;
    let mut done = 0;
    let mut backlog = 0;
    for item in &backlog_items {
        match item.frontmatter.status {
            BacklogStatus::InProgress => in_progress += 1,
            BacklogStatus::Planned => planned += 1,
            BacklogStatus::Draft => draft += 1,
            BacklogStatus::Done => done += 1,
            BacklogStatus::Backlog => backlog += 1,
            BacklogStatus::Cancelled => {}
        }
    }

    let summary = format!(
        "# {} - Project Summary\n\n{}\n\n## Backlog Overview\n- Total items: {}\n- In progress: {}\n- Planned: {}\n- Backlog: {}\n- Draft: {}\n- Done: {}\n",
        config.project.name,
        config.project.description,
        backlog_items.len(),
        in_progress,
        planned,
        backlog,
        draft,
        done,
    );

    Ok(summary)
}

fn handle_query(project: &Project, args: &Value) -> Result<String, String> {
    let filter = QueryFilter {
        status: args
            .get("status")
            .and_then(|v| serde_json::from_value(v.clone()).ok()),
        priority: args
            .get("priority")
            .and_then(|v| serde_json::from_value(v.clone()).ok()),
        epic: args
            .get("epic")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        tags: args
            .get("tags")
            .and_then(|v| serde_json::from_value(v.clone()).ok()),
        assignee: args
            .get("assignee")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        item_type: None,
    };

    let items = project
        .list_backlog_items(&filter)
        .map_err(|e| e.to_string())?;

    let results: Vec<Value> = items
        .iter()
        .map(|doc| {
            let fm = &doc.frontmatter;
            json!({
                "id": fm.id,
                "title": fm.title,
                "status": fm.status.to_string(),
                "priority": fm.priority.to_string(),
                "effort": fm.effort.to_string(),
            })
        })
        .collect();

    serde_json::to_string_pretty(&results).map_err(|e| e.to_string())
}

fn handle_show(project: &Project, args: &Value) -> Result<String, String> {
    let id = args
        .get("id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing required parameter: id".to_string())?;

    let path = project.item_path(id).map_err(|e| e.to_string())?;
    fs::read_to_string(&path).map_err(|e| e.to_string())
}

fn handle_add(project: &Project, args: &Value) -> Result<String, String> {
    let title = args
        .get("title")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing required parameter: title".to_string())?;

    let item_type: ItemType = args
        .get("type")
        .and_then(|v| v.as_str())
        .unwrap_or("feature")
        .parse()
        .map_err(|e: markplane_core::MarkplaneError| e.to_string())?;

    let priority: Priority = args
        .get("priority")
        .and_then(|v| v.as_str())
        .unwrap_or("medium")
        .parse()
        .map_err(|e: markplane_core::MarkplaneError| e.to_string())?;

    let effort: Effort = args
        .get("effort")
        .and_then(|v| v.as_str())
        .unwrap_or("medium")
        .parse()
        .map_err(|e: markplane_core::MarkplaneError| e.to_string())?;

    let epic = args
        .get("epic")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let tags: Vec<String> = args
        .get("tags")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or_default();

    let item = project
        .create_backlog_item(title, item_type, priority, effort, epic, tags)
        .map_err(|e| e.to_string())?;

    let result = json!({ "id": item.id, "title": item.title });
    serde_json::to_string(&result).map_err(|e| e.to_string())
}

fn handle_update(project: &Project, args: &Value) -> Result<String, String> {
    let id = args
        .get("id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing required parameter: id".to_string())?;

    // Update status if provided
    if let Some(status) = args.get("status").and_then(|v| v.as_str()) {
        project
            .update_status(id, status)
            .map_err(|e| e.to_string())?;
    }

    // Update priority and assignee require reading/writing the item
    let has_priority = args.get("priority").and_then(|v| v.as_str()).is_some();
    let has_assignee = args.get("assignee").is_some();

    if has_priority || has_assignee {
        let mut doc: MarkplaneDocument<BacklogItem> =
            project.read_item(id).map_err(|e| e.to_string())?;

        if let Some(priority_str) = args.get("priority").and_then(|v| v.as_str()) {
            doc.frontmatter.priority = priority_str
                .parse()
                .map_err(|e: markplane_core::MarkplaneError| e.to_string())?;
        }

        if let Some(assignee_val) = args.get("assignee") {
            doc.frontmatter.assignee = assignee_val.as_str().map(|s| s.to_string());
        }

        doc.frontmatter.updated = chrono::Local::now().date_naive();
        project.write_item(id, &doc).map_err(|e| e.to_string())?;
    }

    Ok(r#"{"success":true}"#.to_string())
}

fn handle_start(project: &Project, args: &Value) -> Result<String, String> {
    let id = args
        .get("id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing required parameter: id".to_string())?;

    project
        .update_status(id, "in-progress")
        .map_err(|e| e.to_string())?;

    Ok(r#"{"success":true}"#.to_string())
}

fn handle_done(project: &Project, args: &Value) -> Result<String, String> {
    let id = args
        .get("id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing required parameter: id".to_string())?;

    project
        .update_status(id, "done")
        .map_err(|e| e.to_string())?;

    Ok(r#"{"success":true}"#.to_string())
}

fn handle_sync(project: &Project) -> Result<String, String> {
    // Generate a basic summary into .context/summary.md
    let summary = handle_summary(project)?;
    let context_dir = project.root().join(".context");
    fs::create_dir_all(&context_dir).map_err(|e| e.to_string())?;
    fs::write(context_dir.join("summary.md"), &summary).map_err(|e| e.to_string())?;

    Ok(r#"{"success":true}"#.to_string())
}
