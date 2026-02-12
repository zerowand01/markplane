use std::env;

use chrono::Local;
use markplane_core::{parse_id, Task, IdPrefix, MarkplaneDocument, Project};

pub fn run(id: String, user: Option<String>) -> anyhow::Result<()> {
    let project = Project::from_current_dir()?;
    let (prefix, _) = parse_id(&id)?;

    let assignee = user.unwrap_or_else(|| {
        env::var("USER")
            .or_else(|_| env::var("USERNAME"))
            .unwrap_or_else(|_| "me".to_string())
    });

    match prefix {
        IdPrefix::Task => {
            let mut doc: MarkplaneDocument<Task> = project.read_item(&id)?;
            doc.frontmatter.status = "in-progress".parse()?;
            doc.frontmatter.assignee = Some(assignee.clone());
            doc.frontmatter.updated = Local::now().date_naive();
            project.write_item(&id, &doc)?;
        }
        _ => {
            // For non-task items, just update status
            project.update_status(&id, "in-progress")?;
        }
    }

    println!("{} → in-progress (assigned to {})", id, assignee);
    Ok(())
}
