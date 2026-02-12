use std::collections::HashMap;
use std::fs;

use chrono::Local;
use markplane_core::{
    Task, TaskStatus, Effort, Epic, IdPrefix, ItemType,
    MarkplaneDocument, Note, Priority, Project, QueryFilter,
    build_reference_graph, parse_id, validate_references,
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
                "description": "Query tasks with optional filters. Returns matching items.",
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
                            "description": "Item ID (e.g. TASK-042, EPIC-001, PLAN-003, NOTE-007)"
                        }
                    },
                    "required": ["id"]
                }
            },
            {
                "name": "markplane_add",
                "description": "Create a new task.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "title": {
                            "type": "string",
                            "description": "Title of the task"
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
                "name": "markplane_write",
                "description": "Write or replace the markdown body content of an item. Preserves frontmatter.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "id": {
                            "type": "string",
                            "description": "Item ID (e.g. TASK-001, EPIC-001, PLAN-001, NOTE-001)"
                        },
                        "body": {
                            "type": "string",
                            "description": "The full markdown body content (everything below the YAML frontmatter)"
                        }
                    },
                    "required": ["id", "body"]
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
                "description": "Set a task to in-progress status.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "id": {
                            "type": "string",
                            "description": "Task ID to start"
                        }
                    },
                    "required": ["id"]
                }
            },
            {
                "name": "markplane_done",
                "description": "Mark a task as done.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "id": {
                            "type": "string",
                            "description": "Task ID to complete"
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
            },
            {
                "name": "markplane_context",
                "description": "Generate a context summary for the project or a specific item.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "item": {
                            "type": "string",
                            "description": "Optional item ID to generate focused context for"
                        },
                        "focus": {
                            "type": "string",
                            "description": "Optional focus area (e.g. 'active-work', 'blocked', 'metrics')"
                        }
                    },
                    "required": []
                }
            },
            {
                "name": "markplane_graph",
                "description": "Build a reference graph showing how items relate to each other.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "id": {
                            "type": "string",
                            "description": "Item ID to build graph for"
                        },
                        "depth": {
                            "type": "number",
                            "description": "Max depth to traverse (default: 2)"
                        }
                    },
                    "required": ["id"]
                }
            },
            {
                "name": "markplane_promote",
                "description": "Promote a note to a task.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "note_id": {
                            "type": "string",
                            "description": "Note ID to promote (e.g. NOTE-007)"
                        },
                        "priority": {
                            "type": "string",
                            "description": "Priority for the new task (default: medium)"
                        },
                        "effort": {
                            "type": "string",
                            "description": "Effort size for the new task (default: medium)"
                        }
                    },
                    "required": ["note_id"]
                }
            },
            {
                "name": "markplane_plan",
                "description": "Create an implementation plan linked to a task.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "task_id": {
                            "type": "string",
                            "description": "Task ID to create plan for"
                        },
                        "title": {
                            "type": "string",
                            "description": "Optional plan title (defaults to 'Implementation plan for {task_id}')"
                        }
                    },
                    "required": ["task_id"]
                }
            },
            {
                "name": "markplane_link",
                "description": "Link two items with a blocks/depends_on relationship.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "from": {
                            "type": "string",
                            "description": "Source item ID"
                        },
                        "to": {
                            "type": "string",
                            "description": "Target item ID"
                        },
                        "relation": {
                            "type": "string",
                            "enum": ["blocks", "depends_on"],
                            "description": "Relationship type: 'blocks' or 'depends_on'"
                        }
                    },
                    "required": ["from", "to", "relation"]
                }
            },
            {
                "name": "markplane_check",
                "description": "Validate all cross-references in the project. Reports broken links.",
                "inputSchema": {
                    "type": "object",
                    "properties": {},
                    "required": []
                }
            },
            {
                "name": "markplane_stale",
                "description": "Find items that have not been updated recently.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "days": {
                            "type": "number",
                            "description": "Number of days to consider stale (default: 14)"
                        }
                    },
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
        "markplane_write" => handle_write(project, &args),
        "markplane_update" => handle_update(project, &args),
        "markplane_start" => handle_start(project, &args),
        "markplane_done" => handle_done(project, &args),
        "markplane_sync" => handle_sync(project),
        "markplane_context" => handle_context(project, &args),
        "markplane_graph" => handle_graph(project, &args),
        "markplane_promote" => handle_promote(project, &args),
        "markplane_plan" => handle_plan(project, &args),
        "markplane_link" => handle_link(project, &args),
        "markplane_check" => handle_check(project),
        "markplane_stale" => handle_stale(project, &args),
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
    let tasks = project
        .list_tasks(&QueryFilter::default())
        .map_err(|e| e.to_string())?;

    let mut in_progress = 0;
    let mut planned = 0;
    let mut draft = 0;
    let mut done = 0;
    let mut backlog = 0;
    for item in &tasks {
        match item.frontmatter.status {
            TaskStatus::InProgress => in_progress += 1,
            TaskStatus::Planned => planned += 1,
            TaskStatus::Draft => draft += 1,
            TaskStatus::Done => done += 1,
            TaskStatus::Backlog => backlog += 1,
            TaskStatus::Cancelled => {}
        }
    }

    let summary = format!(
        "# {} - Project Summary\n\n{}\n\n## Task Overview\n- Total items: {}\n- In progress: {}\n- Planned: {}\n- Backlog: {}\n- Draft: {}\n- Done: {}\n",
        config.project.name,
        config.project.description,
        tasks.len(),
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
        .list_tasks(&filter)
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

fn handle_write(project: &Project, args: &Value) -> Result<String, String> {
    let id = args
        .get("id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing required parameter: id".to_string())?;

    let body = args
        .get("body")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing required parameter: body".to_string())?;

    project
        .update_body(id, body)
        .map_err(|e| e.to_string())?;

    Ok(r#"{"success":true}"#.to_string())
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
        .create_task(title, item_type, priority, effort, epic, tags)
        .map_err(|e| e.to_string())?;

    let result = json!({ "id": item.id, "title": item.title });
    serde_json::to_string(&result).map_err(|e| e.to_string())
}

fn handle_update(project: &Project, args: &Value) -> Result<String, String> {
    let id = args
        .get("id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing required parameter: id".to_string())?;

    // Update status if provided (works for all item types via project.update_status)
    if let Some(status) = args.get("status").and_then(|v| v.as_str()) {
        project
            .update_status(id, status)
            .map_err(|e| e.to_string())?;
    }

    let has_priority = args.get("priority").and_then(|v| v.as_str()).is_some();
    let has_assignee = args.get("assignee").is_some();

    // Only update additional fields if requested
    if has_priority || has_assignee {
        let (prefix, _) = parse_id(id).map_err(|e| e.to_string())?;
        match prefix {
            IdPrefix::Task => {
                let mut doc: MarkplaneDocument<Task> =
                    project.read_item(id).map_err(|e| e.to_string())?;
                if let Some(priority_str) = args.get("priority").and_then(|v| v.as_str()) {
                    doc.frontmatter.priority = priority_str
                        .parse()
                        .map_err(|e: markplane_core::MarkplaneError| e.to_string())?;
                }
                if let Some(assignee_val) = args.get("assignee") {
                    doc.frontmatter.assignee = assignee_val.as_str().map(|s| s.to_string());
                }
                doc.frontmatter.updated = Local::now().date_naive();
                project.write_item(id, &doc).map_err(|e| e.to_string())?;
            }
            IdPrefix::Epic => {
                if has_priority {
                    let mut doc: MarkplaneDocument<Epic> =
                        project.read_item(id).map_err(|e| e.to_string())?;
                    if let Some(priority_str) = args.get("priority").and_then(|v| v.as_str()) {
                        doc.frontmatter.priority = priority_str
                            .parse()
                            .map_err(|e: markplane_core::MarkplaneError| e.to_string())?;
                    }
                    project.write_item(id, &doc).map_err(|e| e.to_string())?;
                }
            }
            IdPrefix::Plan | IdPrefix::Note => {
                // Plans and notes don't have priority or assignee fields
                if has_priority {
                    return Err(format!(
                        "{} items do not support the priority field",
                        prefix
                    ));
                }
                if has_assignee {
                    return Err(format!(
                        "{} items do not support the assignee field",
                        prefix
                    ));
                }
            }
        }
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
    project.sync_all().map_err(|e| e.to_string())?;
    Ok(r#"{"success":true}"#.to_string())
}

fn handle_context(project: &Project, args: &Value) -> Result<String, String> {
    let item_id = args.get("item").and_then(|v| v.as_str());
    let focus = args.get("focus").and_then(|v| v.as_str());

    if let Some(id) = item_id {
        // Return the raw content of a specific item as context
        let path = project.item_path(id).map_err(|e| e.to_string())?;
        return fs::read_to_string(&path).map_err(|e| e.to_string());
    }

    // Generate and return a context file based on focus
    match focus {
        Some("active-work") => {
            project
                .generate_context_active_work()
                .map_err(|e| e.to_string())?;
            let path = project.root().join(".context/active-work.md");
            fs::read_to_string(&path).map_err(|e| e.to_string())
        }
        Some("blocked") => {
            project
                .generate_context_blocked()
                .map_err(|e| e.to_string())?;
            let path = project.root().join(".context/blocked-items.md");
            fs::read_to_string(&path).map_err(|e| e.to_string())
        }
        Some("metrics") => {
            project
                .generate_context_metrics()
                .map_err(|e| e.to_string())?;
            let path = project.root().join(".context/metrics.md");
            fs::read_to_string(&path).map_err(|e| e.to_string())
        }
        _ => {
            // Default: generate full summary
            project
                .generate_context_summary()
                .map_err(|e| e.to_string())?;
            let path = project.root().join(".context/summary.md");
            fs::read_to_string(&path).map_err(|e| e.to_string())
        }
    }
}

fn handle_graph(project: &Project, args: &Value) -> Result<String, String> {
    let id = args
        .get("id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing required parameter: id".to_string())?;

    let max_depth = args
        .get("depth")
        .and_then(|v| v.as_u64())
        .unwrap_or(2) as usize;

    let graph = build_reference_graph(project).map_err(|e| e.to_string())?;

    // BFS from the given ID up to max_depth
    let mut visited: HashMap<&str, usize> = HashMap::new();
    let mut queue: std::collections::VecDeque<(&str, usize)> = std::collections::VecDeque::new();
    queue.push_back((id, 0));
    visited.insert(id, 0);

    while let Some((current, depth)) = queue.pop_front() {
        if depth >= max_depth {
            continue;
        }
        if let Some(refs) = graph.get(current) {
            for r in refs {
                if !visited.contains_key(r.as_str()) {
                    visited.insert(r.as_str(), depth + 1);
                    queue.push_back((r.as_str(), depth + 1));
                }
            }
        }
        // Also check reverse references (items that reference `current`)
        for (source, refs) in &graph {
            if refs.iter().any(|r| r == current) && !visited.contains_key(source.as_str()) {
                visited.insert(source.as_str(), depth + 1);
                queue.push_back((source.as_str(), depth + 1));
            }
        }
    }

    // Build output
    let mut output = format!("# Reference Graph for {}\n\n", id);
    output.push_str("## Outgoing References\n");
    if let Some(refs) = graph.get(id) {
        if refs.is_empty() {
            output.push_str("(none)\n");
        } else {
            for r in refs {
                output.push_str(&format!("- {} → {}\n", id, r));
            }
        }
    } else {
        output.push_str("(none)\n");
    }

    output.push_str("\n## Incoming References\n");
    let mut has_incoming = false;
    for (source, refs) in &graph {
        if refs.iter().any(|r| r == id) {
            output.push_str(&format!("- {} → {}\n", source, id));
            has_incoming = true;
        }
    }
    if !has_incoming {
        output.push_str("(none)\n");
    }

    output.push_str(&format!(
        "\n## Related Items (depth {})\n",
        max_depth
    ));
    let mut sorted_visited: Vec<_> = visited.iter().filter(|(k, _)| **k != id).collect();
    sorted_visited.sort_by_key(|(_, d)| *d);
    if sorted_visited.is_empty() {
        output.push_str("(none)\n");
    } else {
        for (item_id, depth) in sorted_visited {
            output.push_str(&format!("- {} (depth {})\n", item_id, depth));
        }
    }

    Ok(output)
}

fn handle_promote(project: &Project, args: &Value) -> Result<String, String> {
    let note_id = args
        .get("note_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing required parameter: note_id".to_string())?;

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

    // Read the note
    let note_doc: MarkplaneDocument<Note> =
        project.read_item(note_id).map_err(|e| e.to_string())?;

    // Create a task from the note's title and tags
    let item = project
        .create_task(
            &note_doc.frontmatter.title,
            ItemType::Feature,
            priority,
            effort,
            None,
            note_doc.frontmatter.tags.clone(),
        )
        .map_err(|e| e.to_string())?;

    let result = json!({
        "id": item.id,
        "title": item.title,
        "promoted_from": note_id
    });
    serde_json::to_string(&result).map_err(|e| e.to_string())
}

fn handle_plan(project: &Project, args: &Value) -> Result<String, String> {
    let task_id = args
        .get("task_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing required parameter: task_id".to_string())?;

    // Read the task to get its title and epic
    let task_doc: MarkplaneDocument<Task> =
        project.read_item(task_id).map_err(|e| e.to_string())?;

    let default_title = format!(
        "Implementation plan for {}",
        task_doc.frontmatter.title
    );
    let title = args
        .get("title")
        .and_then(|v| v.as_str())
        .unwrap_or(&default_title);

    let plan = project
        .create_plan(
            title,
            vec![task_id.to_string()],
            task_doc.frontmatter.epic.clone(),
        )
        .map_err(|e| e.to_string())?;

    // Link the plan back to the task
    let mut doc: MarkplaneDocument<Task> =
        project.read_item(task_id).map_err(|e| e.to_string())?;
    doc.frontmatter.plan = Some(plan.id.clone());
    doc.frontmatter.updated = Local::now().date_naive();
    project
        .write_item(task_id, &doc)
        .map_err(|e| e.to_string())?;

    let result = json!({
        "id": plan.id,
        "title": plan.title,
        "implements": task_id
    });
    serde_json::to_string(&result).map_err(|e| e.to_string())
}

fn handle_link(project: &Project, args: &Value) -> Result<String, String> {
    let from = args
        .get("from")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing required parameter: from".to_string())?;
    let to = args
        .get("to")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing required parameter: to".to_string())?;
    let relation = args
        .get("relation")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing required parameter: relation".to_string())?;

    // Verify both items exist
    project.item_path(from).map_err(|e| e.to_string())?;
    project.item_path(to).map_err(|e| e.to_string())?;

    // Both `from` and `to` must be TASK- items for blocks/depends_on
    let (from_prefix, _) = parse_id(from).map_err(|e| e.to_string())?;
    let (to_prefix, _) = parse_id(to).map_err(|e| e.to_string())?;

    if from_prefix != IdPrefix::Task || to_prefix != IdPrefix::Task {
        return Err(
            "blocks/depends_on linking is only supported between TASK- items".to_string(),
        );
    }

    match relation {
        "blocks" => {
            // `from` blocks `to`: add `to` to from.blocks, add `from` to to.depends_on
            let mut from_doc: MarkplaneDocument<Task> =
                project.read_item(from).map_err(|e| e.to_string())?;
            if !from_doc.frontmatter.blocks.contains(&to.to_string()) {
                from_doc.frontmatter.blocks.push(to.to_string());
            }
            from_doc.frontmatter.updated = Local::now().date_naive();
            project
                .write_item(from, &from_doc)
                .map_err(|e| e.to_string())?;

            let mut to_doc: MarkplaneDocument<Task> =
                project.read_item(to).map_err(|e| e.to_string())?;
            if !to_doc.frontmatter.depends_on.contains(&from.to_string()) {
                to_doc.frontmatter.depends_on.push(from.to_string());
            }
            to_doc.frontmatter.updated = Local::now().date_naive();
            project
                .write_item(to, &to_doc)
                .map_err(|e| e.to_string())?;
        }
        "depends_on" => {
            // `from` depends on `to`: add `to` to from.depends_on, add `from` to to.blocks
            let mut from_doc: MarkplaneDocument<Task> =
                project.read_item(from).map_err(|e| e.to_string())?;
            if !from_doc.frontmatter.depends_on.contains(&to.to_string()) {
                from_doc.frontmatter.depends_on.push(to.to_string());
            }
            from_doc.frontmatter.updated = Local::now().date_naive();
            project
                .write_item(from, &from_doc)
                .map_err(|e| e.to_string())?;

            let mut to_doc: MarkplaneDocument<Task> =
                project.read_item(to).map_err(|e| e.to_string())?;
            if !to_doc.frontmatter.blocks.contains(&from.to_string()) {
                to_doc.frontmatter.blocks.push(from.to_string());
            }
            to_doc.frontmatter.updated = Local::now().date_naive();
            project
                .write_item(to, &to_doc)
                .map_err(|e| e.to_string())?;
        }
        _ => {
            return Err(format!(
                "Unknown relation type: {}. Use 'blocks' or 'depends_on'.",
                relation
            ));
        }
    }

    Ok(r#"{"success":true}"#.to_string())
}

fn handle_check(project: &Project) -> Result<String, String> {
    let broken = validate_references(project).map_err(|e| e.to_string())?;

    if broken.is_empty() {
        return Ok("All cross-references are valid.".to_string());
    }

    let mut output = format!("{} broken reference(s) found:\n\n", broken.len());
    for br in &broken {
        output.push_str(&format!(
            "- {} references missing item {}\n",
            br.source_file, br.target_id
        ));
    }
    Ok(output)
}

fn handle_stale(project: &Project, args: &Value) -> Result<String, String> {
    let days = args
        .get("days")
        .and_then(|v| v.as_u64())
        .unwrap_or(14) as i64;

    let today = Local::now().date_naive();
    let cutoff = today - chrono::Duration::days(days);

    let items = project
        .list_tasks(&QueryFilter::default())
        .map_err(|e| e.to_string())?;

    let stale: Vec<_> = items
        .iter()
        .filter(|doc| {
            doc.frontmatter.status != TaskStatus::Done
                && doc.frontmatter.status != TaskStatus::Cancelled
                && doc.frontmatter.updated < cutoff
        })
        .collect();

    if stale.is_empty() {
        return Ok(format!(
            "No stale items found (threshold: {} days).",
            days
        ));
    }

    let mut output = format!(
        "{} stale item(s) (not updated in {} days):\n\n",
        stale.len(),
        days
    );
    for item in &stale {
        let fm = &item.frontmatter;
        output.push_str(&format!(
            "- {} {} (status: {}, last updated: {})\n",
            fm.id, fm.title, fm.status, fm.updated
        ));
    }
    Ok(output)
}
