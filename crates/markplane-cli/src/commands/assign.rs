use chrono::Local;
use markplane_core::{parse_id, Task, IdPrefix, MarkplaneDocument, Project};

pub fn run(id: String, user: String) -> anyhow::Result<()> {
    let project = Project::from_current_dir()?;
    let (prefix, _) = parse_id(&id)?;

    // Strip leading @ if present
    let user = user.strip_prefix('@').unwrap_or(&user).to_string();

    match prefix {
        IdPrefix::Task => {
            let mut doc: MarkplaneDocument<Task> = project.read_item(&id)?;
            doc.frontmatter.assignee = Some(user.clone());
            doc.frontmatter.updated = Local::now().date_naive();
            project.write_item(&id, &doc)?;
        }
        _ => {
            anyhow::bail!("Assign is currently only supported for tasks. Got: {}", id);
        }
    }

    println!("{} assigned to {}", id, user);
    Ok(())
}
