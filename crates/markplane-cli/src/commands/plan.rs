use markplane_core::{parse_id, Task, IdPrefix, MarkplaneDocument, Project};
use chrono::Local;

pub fn run(id: String, title: Option<String>) -> anyhow::Result<()> {
    let project = Project::from_current_dir()?;
    let (prefix, _) = parse_id(&id)?;

    if prefix != IdPrefix::Task {
        anyhow::bail!("Can only create plans for tasks. Got: {}", id);
    }

    let mut doc: MarkplaneDocument<Task> = project.read_item(&id)?;
    let plan_title = title.unwrap_or_else(|| {
        format!("Implementation plan for {}", doc.frontmatter.title)
    });

    let plan = project.create_plan(
        &plan_title,
        vec![id.clone()],
        doc.frontmatter.epic.clone(),
    )?;

    // Link the plan back to the task
    doc.frontmatter.plan = Some(plan.id.clone());
    doc.frontmatter.updated = Local::now().date_naive();
    project.write_item(&id, &doc)?;

    println!("Created {} — {}", plan.id, plan.title);
    println!("Linked to {}", id);

    Ok(())
}
