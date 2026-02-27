use markplane_core::{Effort, Priority, Project};

use super::parse_comma_list;

pub fn run(
    title: String,
    item_type: Option<String>,
    priority: String,
    effort: String,
    epic: Option<String>,
    tags: Option<String>,
    template: Option<String>,
) -> anyhow::Result<()> {
    let project = Project::from_current_dir()?;
    let config = project.load_config()?;
    let item_type = item_type.as_deref().unwrap_or(config.default_task_type());

    let priority: Priority = priority.parse()?;
    let effort: Effort = effort.parse()?;
    let tags = tags.map(|t| parse_comma_list(&t)).unwrap_or_default();

    let item = project.create_task(
        &title, item_type, priority, effort, epic, tags,
        template.as_deref(),
    )?;

    println!("Created {} — {}", item.id, item.title);

    Ok(())
}
