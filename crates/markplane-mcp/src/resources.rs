use std::fs;

use markplane_core::{BacklogStatus, Project, QueryFilter};
use serde_json::{json, Value};

use crate::protocol::{JsonRpcResponse, INTERNAL_ERROR, INVALID_PARAMS};

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
                "description": "Currently in-progress backlog items",
                "mimeType": "text/markdown"
            },
            {
                "uri": "markplane://blocked",
                "name": "Blocked Items",
                "description": "Items that have unresolved dependencies or need attention",
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
        _ => {
            return JsonRpcResponse::error(
                id,
                INVALID_PARAMS,
                format!("Unknown resource URI: {}", uri),
            );
        }
    };

    match result {
        Ok(content) => JsonRpcResponse::success(
            id,
            json!({
                "contents": [{
                    "uri": uri,
                    "mimeType": "text/markdown",
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
        .list_backlog_items(&QueryFilter::default())
        .map_err(|e| e.to_string())?;

    let mut in_progress = 0usize;
    let mut planned = 0usize;
    let mut done = 0usize;
    let total = items.len();

    for item in &items {
        match item.frontmatter.status {
            BacklogStatus::InProgress => in_progress += 1,
            BacklogStatus::Planned => planned += 1,
            BacklogStatus::Done => done += 1,
            _ => {}
        }
    }

    Ok(format!(
        "# {} Summary\n\n{}\n\n- Total items: {}\n- In progress: {}\n- Planned: {}\n- Done: {}\n",
        config.project.name, config.project.description, total, in_progress, planned, done,
    ))
}

fn read_active_work(project: &Project) -> Result<String, String> {
    let filter = QueryFilter {
        status: Some(vec!["in-progress".to_string()]),
        ..Default::default()
    };

    let items = project
        .list_backlog_items(&filter)
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
    let items = project
        .list_backlog_items(&QueryFilter::default())
        .map_err(|e| e.to_string())?;

    let blocked: Vec<_> = items
        .iter()
        .filter(|doc| !doc.frontmatter.depends_on.is_empty())
        .collect();

    if blocked.is_empty() {
        return Ok("# Blocked Items\n\nNo items with unresolved dependencies.\n".to_string());
    }

    let mut output = "# Blocked Items\n\n".to_string();
    for item in &blocked {
        let fm = &item.frontmatter;
        output.push_str(&format!(
            "- **{}** {} — blocked by: {}\n",
            fm.id,
            fm.title,
            fm.depends_on.join(", "),
        ));
    }

    Ok(output)
}
