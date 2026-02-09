use markplane_core::{Effort, ItemType, Priority, Project};

use super::parse_comma_list;

pub fn run(
    title: String,
    item_type: String,
    priority: String,
    effort: String,
    epic: Option<String>,
    tags: Option<String>,
) -> anyhow::Result<()> {
    let project = Project::from_current_dir()?;

    let item_type: ItemType = item_type.parse()?;
    let priority: Priority = priority.parse()?;
    let effort: Effort = effort.parse()?;
    let tags = tags.map(|t| parse_comma_list(&t)).unwrap_or_default();

    let item = project.create_backlog_item(&title, item_type, priority, effort, epic, tags)?;

    println!("Created {} — {}", item.id, item.title);

    Ok(())
}
