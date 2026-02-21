use markplane_core::{parse_id, Task, IdPrefix, MarkplaneDocument, Project, LinkRelation, LinkAction};

pub fn run(id: String, title: Option<String>) -> anyhow::Result<()> {
    let project = Project::from_current_dir()?;
    let (prefix, _) = parse_id(&id)?;

    if prefix != IdPrefix::Task {
        anyhow::bail!("Can only create plans for tasks. Got: {}", id);
    }

    let doc: MarkplaneDocument<Task> = project.read_item(&id)?;
    let plan_title = title.unwrap_or_else(|| {
        format!("Implementation plan for {}", doc.frontmatter.title)
    });

    let plan = project.create_plan(
        &plan_title,
        vec![],
        doc.frontmatter.epic.clone(),
    )?;

    // Link the plan to the task via the centralized link system
    project.link_items(&id, &plan.id, LinkRelation::Plan, LinkAction::Add)?;

    println!("Created {} — {}", plan.id, plan.title);
    println!("Linked to {}", id);

    Ok(())
}
