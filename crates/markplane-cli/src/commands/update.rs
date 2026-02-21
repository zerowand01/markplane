use chrono::NaiveDate;
use markplane_core::{Patch, Project, UpdateFields};

use super::parse_comma_list;

#[allow(clippy::too_many_arguments)]
pub fn run(
    id: String,
    title: Option<String>,
    status: Option<String>,
    priority: Option<String>,
    effort: Option<String>,
    item_type: Option<String>,
    assignee: Option<String>,
    clear_assignee: bool,
    position: Option<String>,
    clear_position: bool,
    add_tags: Option<String>,
    remove_tags: Option<String>,
    started: Option<String>,
    clear_started: bool,
    target: Option<String>,
    clear_target: bool,
    note_type: Option<String>,
) -> anyhow::Result<()> {
    let project = Project::from_current_dir()?;

    let assignee_patch = if clear_assignee {
        Patch::Clear
    } else if let Some(val) = assignee {
        // Strip leading @ if present
        let val = val.strip_prefix('@').unwrap_or(&val).to_string();
        Patch::Set(val)
    } else {
        Patch::Unchanged
    };

    let position_patch = if clear_position {
        Patch::Clear
    } else if let Some(val) = position {
        Patch::Set(val)
    } else {
        Patch::Unchanged
    };

    let started_patch = if clear_started {
        Patch::Clear
    } else if let Some(ref val) = started {
        Patch::Set(val.parse::<NaiveDate>().map_err(|e| {
            anyhow::anyhow!("Invalid date for --started: {} (expected YYYY-MM-DD)", e)
        })?)
    } else {
        Patch::Unchanged
    };

    let target_patch = if clear_target {
        Patch::Clear
    } else if let Some(ref val) = target {
        Patch::Set(val.parse::<NaiveDate>().map_err(|e| {
            anyhow::anyhow!("Invalid date for --target: {} (expected YYYY-MM-DD)", e)
        })?)
    } else {
        Patch::Unchanged
    };

    let fields = UpdateFields {
        title,
        status,
        priority,
        effort,
        item_type,
        assignee: assignee_patch,
        position: position_patch,
        add_tags: add_tags.map(|s| parse_comma_list(&s)).unwrap_or_default(),
        remove_tags: remove_tags.map(|s| parse_comma_list(&s)).unwrap_or_default(),
        started: started_patch,
        target: target_patch,
        note_type,
    };

    project.update_item(&id, fields)?;
    println!("Updated {}", id);
    Ok(())
}
