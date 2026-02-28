use std::collections::HashSet;
use std::fs;

use markplane_core::{StatusCategory, Project, QueryFilter, parse_id, IdPrefix};
use markplane_core::manifest;
use serde_json::{json, Value};

use super::protocol::{JsonRpcResponse, INTERNAL_ERROR, INVALID_PARAMS};

/// Return the list of available resources.
pub fn list_resources() -> Value {
    json!({
        "resources": [
            {
                "uri": "markplane://summary",
                "name": "Project Summary",
                "description": "Overview of the project state including item counts by status",
                "mimeType": "text/markdown"
            },
            {
                "uri": "markplane://active-work",
                "name": "Active Work",
                "description": "Currently in-progress tasks",
                "mimeType": "text/markdown"
            },
            {
                "uri": "markplane://blocked",
                "name": "Blocked Items",
                "description": "Items that have unresolved dependencies or need attention",
                "mimeType": "text/markdown"
            },
            {
                "uri": "markplane://templates",
                "name": "Template Manifest",
                "description": "Template configuration showing available templates for each item kind",
                "mimeType": "text/yaml"
            }
        ],
        "resourceTemplates": [
            {
                "uriTemplate": "markplane://task/{id}",
                "name": "Task",
                "description": "Full content of a task by ID",
                "mimeType": "text/markdown"
            },
            {
                "uriTemplate": "markplane://epic/{id}",
                "name": "Epic",
                "description": "Full content of an epic by ID",
                "mimeType": "text/markdown"
            },
            {
                "uriTemplate": "markplane://plan/{id}",
                "name": "Plan",
                "description": "Full content of an implementation plan by ID",
                "mimeType": "text/markdown"
            },
            {
                "uriTemplate": "markplane://note/{id}",
                "name": "Note",
                "description": "Full content of a note by ID",
                "mimeType": "text/markdown"
            }
        ]
    })
}

/// Read a resource by URI.
pub fn read_resource(id: Value, project: &Project, uri: &str) -> JsonRpcResponse {
    let result = match uri {
        "markplane://summary" => read_summary(project),
        "markplane://active-work" => read_active_work(project),
        "markplane://blocked" => read_blocked(project),
        "markplane://templates" => read_templates(project),
        _ if uri.starts_with("markplane://task/") => {
            let item_id = &uri["markplane://task/".len()..];
            read_task_item(project, item_id)
        }
        _ if uri.starts_with("markplane://epic/") => {
            let item_id = &uri["markplane://epic/".len()..];
            read_epic_item(project, item_id)
        }
        _ if uri.starts_with("markplane://plan/") => {
            let item_id = &uri["markplane://plan/".len()..];
            read_plan_item(project, item_id)
        }
        _ if uri.starts_with("markplane://note/") => {
            let item_id = &uri["markplane://note/".len()..];
            read_note_item(project, item_id)
        }
        _ => {
            return JsonRpcResponse::error(
                id,
                INVALID_PARAMS,
                format!("Unknown resource URI: {}", uri),
            );
        }
    };

    let mime_type = if uri == "markplane://templates" {
        "text/yaml"
    } else {
        "text/markdown"
    };

    match result {
        Ok(content) => JsonRpcResponse::success(
            id,
            json!({
                "contents": [{
                    "uri": uri,
                    "mimeType": mime_type,
                    "text": content,
                }]
            }),
        ),
        Err(e) => JsonRpcResponse::error(id, INTERNAL_ERROR, e),
    }
}

fn read_summary(project: &Project) -> Result<String, String> {
    // Try pre-generated summary first
    let summary_path = project.root().join(".context/summary.md");
    if let Ok(content) = fs::read_to_string(&summary_path) {
        return Ok(content);
    }

    // Generate inline
    let config = project.load_config().map_err(|e| e.to_string())?;
    let items = project
        .list_tasks(&QueryFilter::default())
        .map_err(|e| e.to_string())?;

    let workflow = &config.workflows.task;
    let mut in_progress = 0usize;
    let mut planned = 0usize;
    let mut done = 0usize;
    let total = items.len();

    for item in &items {
        match workflow.category_of(&item.frontmatter.status) {
            Some(StatusCategory::Active) => in_progress += 1,
            Some(StatusCategory::Planned) => planned += 1,
            Some(StatusCategory::Completed) => done += 1,
            _ => {}
        }
    }

    Ok(format!(
        "# {} Summary\n\n{}\n\n- Total items: {}\n- In progress: {}\n- Planned: {}\n- Done: {}\n",
        config.project.name, config.project.description, total, in_progress, planned, done,
    ))
}

fn read_active_work(project: &Project) -> Result<String, String> {
    let config = project.load_config().map_err(|e| e.to_string())?;
    let active_statuses: Vec<String> = config.workflows.task
        .statuses_in(StatusCategory::Active)
        .to_vec();
    let filter = QueryFilter {
        status: Some(if active_statuses.is_empty() {
            vec!["in-progress".to_string()]
        } else {
            active_statuses
        }),
        ..Default::default()
    };

    let items = project
        .list_tasks(&filter)
        .map_err(|e| e.to_string())?;

    if items.is_empty() {
        return Ok("# Active Work\n\nNo items currently in progress.\n".to_string());
    }

    let mut output = "# Active Work\n\n".to_string();
    for item in &items {
        let fm = &item.frontmatter;
        output.push_str(&format!(
            "- **{}** {} (priority: {}, effort: {})\n",
            fm.id, fm.title, fm.priority, fm.effort,
        ));
    }

    Ok(output)
}

fn read_blocked(project: &Project) -> Result<String, String> {
    let config = project.load_config().map_err(|e| e.to_string())?;
    let workflow = &config.workflows.task;
    let items = project
        .list_tasks(&QueryFilter::default())
        .map_err(|e| e.to_string())?;

    let blocked = markplane_core::find_blocked_items(&items, workflow);
    let completed_statuses: HashSet<&str> = workflow
        .statuses_in(StatusCategory::Completed)
        .iter().map(|s| s.as_str()).collect();
    let done_ids: HashSet<&str> = items
        .iter()
        .filter(|doc| completed_statuses.contains(doc.frontmatter.status.as_str()))
        .map(|doc| doc.frontmatter.id.as_str())
        .collect();

    if blocked.is_empty() {
        return Ok("# Blocked Items\n\nNo items with unresolved dependencies.\n".to_string());
    }

    let mut output = "# Blocked Items\n\n".to_string();
    for item in &blocked {
        let fm = &item.frontmatter;
        let unresolved: Vec<_> = fm
            .depends_on
            .iter()
            .filter(|dep| !done_ids.contains(dep.as_str()))
            .map(|s| s.as_str())
            .collect();
        output.push_str(&format!(
            "- **{}** {} — blocked by: {}\n",
            fm.id,
            fm.title,
            unresolved.join(", "),
        ));
    }

    Ok(output)
}

fn read_task_item(project: &Project, item_id: &str) -> Result<String, String> {
    // Validate the ID is a TASK- item
    let (prefix, _) = parse_id(item_id).map_err(|e| e.to_string())?;
    if prefix != IdPrefix::Task {
        return Err(format!("Expected TASK- ID, got: {}", item_id));
    }
    let path = project.item_path(item_id).map_err(|e| e.to_string())?;
    fs::read_to_string(&path).map_err(|e| e.to_string())
}

fn read_epic_item(project: &Project, item_id: &str) -> Result<String, String> {
    // Validate the ID is an EPIC- item
    let (prefix, _) = parse_id(item_id).map_err(|e| e.to_string())?;
    if prefix != IdPrefix::Epic {
        return Err(format!("Expected EPIC- ID, got: {}", item_id));
    }
    let path = project.item_path(item_id).map_err(|e| e.to_string())?;
    fs::read_to_string(&path).map_err(|e| e.to_string())
}

fn read_plan_item(project: &Project, item_id: &str) -> Result<String, String> {
    let (prefix, _) = parse_id(item_id).map_err(|e| e.to_string())?;
    if prefix != IdPrefix::Plan {
        return Err(format!("Expected PLAN- ID, got: {}", item_id));
    }
    let path = project.item_path(item_id).map_err(|e| e.to_string())?;
    fs::read_to_string(&path).map_err(|e| e.to_string())
}

fn read_note_item(project: &Project, item_id: &str) -> Result<String, String> {
    let (prefix, _) = parse_id(item_id).map_err(|e| e.to_string())?;
    if prefix != IdPrefix::Note {
        return Err(format!("Expected NOTE- ID, got: {}", item_id));
    }
    let path = project.item_path(item_id).map_err(|e| e.to_string())?;
    fs::read_to_string(&path).map_err(|e| e.to_string())
}

fn read_templates(project: &Project) -> Result<String, String> {
    // Try to read the manifest file first
    let manifest_path = project.root().join("templates/manifest.yaml");
    if let Ok(content) = fs::read_to_string(&manifest_path) {
        return Ok(content);
    }
    // Fall back to built-in default
    Ok(manifest::DEFAULT_MANIFEST.to_string())
}
