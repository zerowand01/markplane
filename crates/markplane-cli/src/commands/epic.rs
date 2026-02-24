use markplane_core::{Priority, Project};

pub fn run(title: String, priority: String) -> anyhow::Result<()> {
    let project = Project::from_current_dir()?;
    let priority: Priority = priority.parse()?;
    let epic = project.create_epic(&title, priority, None)?;
    println!("Created {} — {}", epic.id, epic.title);
    Ok(())
}
