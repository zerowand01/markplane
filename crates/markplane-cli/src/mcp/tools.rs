use std::collections::HashMap;
use std::fs;

use markplane_core::{
    Task, TaskStatus, Effort, ItemType, LinkAction, LinkRelation,
    MarkplaneDocument, MoveDirective, Note, NoteType, Patch, Priority, Project, QueryFilter,
    UpdateFields, build_reference_graph, validate_references,
};
use serde_json::{json, Value};

use super::protocol::{JsonRpcResponse, INTERNAL_ERROR, INVALID_PARAMS};

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
                "description": "Query items with optional filters. Returns matching items.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "kind": {
                            "type": "string",
                            "description": "Item kind to query: tasks (default), epics, plans, or notes"
                        },
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
                        },
                        "archived": {
                            "type": "boolean",
                            "description": "If true, query archived items instead of active ones"
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
                "description": "Create a new item.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "title": {
                            "type": "string",
                            "description": "Title of the item"
                        },
                        "kind": {
                            "type": "string",
                            "description": "Item kind: task (default), epic, or note"
                        },
                        "type": {
                            "type": "string",
                            "description": "Item type (feature, bug, enhancement, chore, research, spike). Tasks only. Default: feature"
                        },
                        "priority": {
                            "type": "string",
                            "description": "Priority (critical, high, medium, low, someday). Tasks and epics. Default: medium"
                        },
                        "effort": {
                            "type": "string",
                            "description": "Effort size (xs, small, medium, large, xl). Tasks only. Default: medium"
                        },
                        "epic": {
                            "type": "string",
                            "description": "Parent epic ID (e.g. EPIC-001). Tasks only."
                        },
                        "note_type": {
                            "type": "string",
                            "description": "Note type (research, analysis, idea, decision, meeting). Notes only. Default: research"
                        },
                        "tags": {
                            "type": "array",
                            "items": { "type": "string" },
                            "description": "Tags for the item. Tasks and notes."
                        },
                        "template": {
                            "type": "string",
                            "description": "Template name override (e.g. 'bug', 'refactor', 'research'). Uses type-based or kind defaults if omitted."
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
                        "title": {
                            "type": "string",
                            "description": "New title"
                        },
                        "status": {
                            "type": "string",
                            "description": "New status value. Task: draft/backlog/planned/in-progress/done/cancelled. Epic: now/next/later/done. Plan: draft/approved/in-progress/done. Note: draft/active/archived."
                        },
                        "priority": {
                            "type": "string",
                            "description": "New priority value"
                        },
                        "effort": {
                            "type": "string",
                            "description": "Effort size (xs, small, medium, large, xl). Tasks only."
                        },
                        "type": {
                            "type": "string",
                            "description": "Item type (feature, bug, enhancement, chore, research, spike). Tasks only."
                        },
                        "assignee": {
                            "type": "string",
                            "description": "New assignee. Set to null to clear. Tasks only."
                        },
                        "position": {
                            "type": "string",
                            "description": "Position key for manual ordering within priority group. Set to null to clear. Tasks only."
                        },
                        "add_tags": {
                            "type": "array",
                            "items": { "type": "string" },
                            "description": "Tags to add"
                        },
                        "remove_tags": {
                            "type": "array",
                            "items": { "type": "string" },
                            "description": "Tags to remove"
                        },
                        "started": {
                            "type": "string",
                            "description": "Started date (YYYY-MM-DD). Epics only. Set to null to clear."
                        },
                        "target": {
                            "type": "string",
                            "description": "Target date (YYYY-MM-DD). Epics only. Set to null to clear."
                        },
                        "note_type": {
                            "type": "string",
                            "description": "Note type (research, analysis, idea, decision, meeting). Notes only."
                        }
                    },
                    "required": ["id"]
                }
            },
            {
                "name": "markplane_start",
                "description": "Set a task to in-progress status. For epics, use markplane_update with status: now/next/later/done instead.",
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
                "name": "markplane_move",
                "description": "Move a task to a new position within its priority group. Handles fractional-indexing math automatically.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "id": {
                            "type": "string",
                            "description": "Task ID to move (e.g. TASK-042)"
                        },
                        "to": {
                            "type": "string",
                            "enum": ["top", "bottom"],
                            "description": "Move to top or bottom of the priority group"
                        },
                        "before": {
                            "type": "string",
                            "description": "Task ID to position before"
                        },
                        "after": {
                            "type": "string",
                            "description": "Task ID to position after"
                        }
                    },
                    "required": ["id"]
                }
            },
            {
                "name": "markplane_done",
                "description": "Mark a task as done. Also works for epics, plans, and notes.",
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
                "description": "Generate a context summary for the project.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
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
                            "description": "Optional plan title (defaults to 'Implementation plan for <task title>')"
                        },
                        "template": {
                            "type": "string",
                            "description": "Template name override (e.g. 'refactor', 'implementation'). Defaults to 'implementation'."
                        }
                    },
                    "required": ["task_id"]
                }
            },
            {
                "name": "markplane_link",
                "description": "Link two items with a typed relationship (blocks, depends_on, epic, plan, implements, related).",
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
                            "enum": ["blocks", "depends_on", "epic", "plan", "implements", "related"],
                            "description": "Relationship type: blocks, depends_on, epic, plan, implements, or related"
                        },
                        "remove": {
                            "type": "boolean",
                            "description": "If true, remove the link instead of adding it. Default: false"
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
                "name": "markplane_archive",
                "description": "Archive an item (move from items/ to archive/). Works for any entity type.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "id": {
                            "type": "string",
                            "description": "Item ID to archive (e.g. TASK-042, EPIC-001)"
                        }
                    },
                    "required": ["id"]
                }
            },
            {
                "name": "markplane_unarchive",
                "description": "Restore an archived item (move from archive/ back to items/).",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "id": {
                            "type": "string",
                            "description": "Item ID to restore (e.g. TASK-042, EPIC-001)"
                        }
                    },
                    "required": ["id"]
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
        "markplane_move" => handle_move(project, &args),
        "markplane_done" => handle_done(project, &args),
        "markplane_sync" => handle_sync(project),
        "markplane_context" => handle_context(project, &args),
        "markplane_graph" => handle_graph(project, &args),
        "markplane_promote" => handle_promote(project, &args),
        "markplane_plan" => handle_plan(project, &args),
        "markplane_link" => handle_link(project, &args),
        "markplane_check" => handle_check(project),
        "markplane_archive" => handle_archive(project, &args),
        "markplane_unarchive" => handle_unarchive(project, &args),
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
    let kind = args
        .get("kind")
        .and_then(|v| v.as_str())
        .unwrap_or("tasks");

    let archived = args
        .get("archived")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let results: Vec<Value> = match kind {
        "tasks" => {
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
                archived,
            };

            let items = project.list_tasks(&filter).map_err(|e| e.to_string())?;
            items
                .iter()
                .map(|doc| {
                    let fm = &doc.frontmatter;
                    json!({
                        "id": fm.id,
                        "title": fm.title,
                        "status": fm.status.to_string(),
                        "priority": fm.priority.to_string(),
                        "effort": fm.effort.to_string(),
                        "updated": fm.updated.to_string(),
                    })
                })
                .collect()
        }
        "epics" => {
            let items = project.list_epics_filtered(archived).map_err(|e| e.to_string())?;
            items
                .iter()
                .map(|doc| {
                    let fm = &doc.frontmatter;
                    json!({
                        "id": fm.id,
                        "title": fm.title,
                        "status": fm.status.to_string(),
                        "priority": fm.priority.to_string(),
                        "created": fm.created.to_string(),
                        "updated": fm.updated.to_string(),
                    })
                })
                .collect()
        }
        "plans" => {
            let items = project.list_plans_filtered(archived).map_err(|e| e.to_string())?;
            items
                .iter()
                .map(|doc| {
                    let fm = &doc.frontmatter;
                    json!({
                        "id": fm.id,
                        "title": fm.title,
                        "status": fm.status.to_string(),
                        "updated": fm.updated.to_string(),
                    })
                })
                .collect()
        }
        "notes" => {
            let items = project.list_notes_filtered(archived).map_err(|e| e.to_string())?;
            items
                .iter()
                .map(|doc| {
                    let fm = &doc.frontmatter;
                    json!({
                        "id": fm.id,
                        "title": fm.title,
                        "status": fm.status.to_string(),
                        "type": fm.note_type.to_string(),
                        "updated": fm.updated.to_string(),
                    })
                })
                .collect()
        }
        _ => return Err(format!("Unknown kind: {}. Expected tasks, epics, plans, or notes", kind)),
    };

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

    let kind = args
        .get("kind")
        .and_then(|v| v.as_str())
        .unwrap_or("task");

    let template = args
        .get("template")
        .and_then(|v| v.as_str());

    match kind {
        "task" => {
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
                .create_task(title, item_type, priority, effort, epic, tags, template)
                .map_err(|e| e.to_string())?;

            let result = json!({ "id": item.id, "title": item.title });
            serde_json::to_string(&result).map_err(|e| e.to_string())
        }
        "epic" => {
            let priority: Priority = args
                .get("priority")
                .and_then(|v| v.as_str())
                .unwrap_or("medium")
                .parse()
                .map_err(|e: markplane_core::MarkplaneError| e.to_string())?;

            let epic = project
                .create_epic(title, priority, template)
                .map_err(|e| e.to_string())?;

            let result = json!({ "id": epic.id, "title": epic.title });
            serde_json::to_string(&result).map_err(|e| e.to_string())
        }
        "note" => {
            let note_type: NoteType = args
                .get("note_type")
                .and_then(|v| v.as_str())
                .unwrap_or("research")
                .parse()
                .map_err(|e: markplane_core::MarkplaneError| e.to_string())?;

            let tags: Vec<String> = args
                .get("tags")
                .and_then(|v| serde_json::from_value(v.clone()).ok())
                .unwrap_or_default();

            let note = project
                .create_note(title, note_type, tags, template)
                .map_err(|e| e.to_string())?;

            let result = json!({ "id": note.id, "title": note.title });
            serde_json::to_string(&result).map_err(|e| e.to_string())
        }
        _ => Err(format!("Unknown kind: {}. Expected task, epic, or note", kind)),
    }
}

fn handle_update(project: &Project, args: &Value) -> Result<String, String> {
    let id = args
        .get("id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing required parameter: id".to_string())?;

    // Build Patch for assignee: explicit null → Clear, string → Set, absent → Unchanged
    let assignee = if let Some(val) = args.get("assignee") {
        if val.is_null() {
            Patch::Clear
        } else if let Some(s) = val.as_str() {
            Patch::Set(s.to_string())
        } else {
            Patch::Unchanged
        }
    } else {
        Patch::Unchanged
    };

    // Build Patch for position
    let position = if let Some(val) = args.get("position") {
        if val.is_null() {
            Patch::Clear
        } else if let Some(s) = val.as_str() {
            Patch::Set(s.to_string())
        } else {
            Patch::Unchanged
        }
    } else {
        Patch::Unchanged
    };

    // Build Patch for started date
    let started = if let Some(val) = args.get("started") {
        if val.is_null() {
            Patch::Clear
        } else if let Some(s) = val.as_str() {
            let date = s.parse().map_err(|_| format!("Invalid date for started: {} (expected YYYY-MM-DD)", s))?;
            Patch::Set(date)
        } else {
            Patch::Unchanged
        }
    } else {
        Patch::Unchanged
    };

    // Build Patch for target date
    let target = if let Some(val) = args.get("target") {
        if val.is_null() {
            Patch::Clear
        } else if let Some(s) = val.as_str() {
            let date = s.parse().map_err(|_| format!("Invalid date for target: {} (expected YYYY-MM-DD)", s))?;
            Patch::Set(date)
        } else {
            Patch::Unchanged
        }
    } else {
        Patch::Unchanged
    };

    let fields = UpdateFields {
        title: args.get("title").and_then(|v| v.as_str()).map(|s| s.to_string()),
        status: args.get("status").and_then(|v| v.as_str()).map(|s| s.to_string()),
        priority: args.get("priority").and_then(|v| v.as_str()).map(|s| s.to_string()),
        effort: args.get("effort").and_then(|v| v.as_str()).map(|s| s.to_string()),
        item_type: args.get("type").and_then(|v| v.as_str()).map(|s| s.to_string()),
        assignee,
        position,
        add_tags: args.get("add_tags")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_default(),
        remove_tags: args.get("remove_tags")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_default(),
        started,
        target,
        note_type: args.get("note_type").and_then(|v| v.as_str()).map(|s| s.to_string()),
    };

    project.update_item(id, fields).map_err(|e| e.to_string())?;

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

fn handle_move(project: &Project, args: &Value) -> Result<String, String> {
    let id = args
        .get("id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing required parameter: id".to_string())?;

    let to = args.get("to").and_then(|v| v.as_str());
    let before = args.get("before").and_then(|v| v.as_str());
    let after = args.get("after").and_then(|v| v.as_str());

    let directive = match (to, before, after) {
        (Some("top"), None, None) => MoveDirective::Top,
        (Some("bottom"), None, None) => MoveDirective::Bottom,
        (None, Some(target), None) => MoveDirective::Before(target.to_string()),
        (None, None, Some(target)) => MoveDirective::After(target.to_string()),
        (Some(v), None, None) => {
            return Err(format!("Invalid 'to' value: {}. Expected 'top' or 'bottom'", v));
        }
        _ => {
            return Err(
                "Provide exactly one of: to (top/bottom), before, or after".to_string(),
            );
        }
    };

    project
        .move_item(id, directive)
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
    let focus = args.get("focus").and_then(|v| v.as_str());

    let (generate, filename): (fn(&Project) -> markplane_core::Result<()>, &str) = match focus {
        Some("active-work") => (Project::generate_context_active_work, "active-work.md"),
        Some("blocked") => (Project::generate_context_blocked, "blocked-items.md"),
        Some("metrics") => (Project::generate_context_metrics, "metrics.md"),
        None | Some("summary") => (Project::generate_context_summary, "summary.md"),
        Some(other) => {
            return Err(format!(
                "Unknown focus area: {}. Expected active-work, blocked, metrics, or summary",
                other
            ));
        }
    };
    generate(project).map_err(|e| e.to_string())?;
    let path = project.root().join(format!(".context/{}", filename));
    fs::read_to_string(&path).map_err(|e| e.to_string())
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
            None,
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

    let template = args
        .get("template")
        .and_then(|v| v.as_str());

    let plan = project
        .create_plan(
            title,
            vec![],
            task_doc.frontmatter.epic.clone(),
            template,
        )
        .map_err(|e| e.to_string())?;

    // Link the plan to the task via the centralized link system
    project
        .link_items(task_id, &plan.id, LinkRelation::Plan, LinkAction::Add)
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
    let relation_str = args
        .get("relation")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing required parameter: relation".to_string())?;
    let remove = args
        .get("remove")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let relation: LinkRelation = relation_str.parse().map_err(|e: markplane_core::MarkplaneError| e.to_string())?;
    let action = if remove { LinkAction::Remove } else { LinkAction::Add };

    project
        .link_items(from, to, relation, action)
        .map_err(|e| e.to_string())?;

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

fn handle_archive(project: &Project, args: &Value) -> Result<String, String> {
    let id = args
        .get("id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing required parameter: id".to_string())?;

    project
        .archive_item(id)
        .map_err(|e| e.to_string())?;

    Ok(format!("{{\"success\":true,\"archived\":\"{}\"}}", id))
}

fn handle_unarchive(project: &Project, args: &Value) -> Result<String, String> {
    let id = args
        .get("id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing required parameter: id".to_string())?;

    project
        .unarchive_item(id)
        .map_err(|e| e.to_string())?;

    Ok(format!("{{\"success\":true,\"restored\":\"{}\"}}", id))
}
