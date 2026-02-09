use markplane_core::{parse_id, BacklogItem, IdPrefix, MarkplaneDocument, Project};
use chrono::Local;

pub fn run(id: String, title: Option<String>) -> anyhow::Result<()> {
    let project = Project::from_current_dir()?;
    let (prefix, _) = parse_id(&id)?;

    if prefix != IdPrefix::Back {
        anyhow::bail!("Can only create plans for backlog items. Got: {}", id);
    }

    let mut doc: MarkplaneDocument<BacklogItem> = project.read_item(&id)?;
    let plan_title = title.unwrap_or_else(|| {
        format!("Implementation plan for {}", doc.frontmatter.title)
    });

    let plan = project.create_plan(
        &plan_title,
        vec![id.clone()],
        doc.frontmatter.epic.clone(),
    )?;

    // Link the plan back to the backlog item
    doc.frontmatter.plan = Some(plan.id.clone());
    doc.frontmatter.updated = Local::now().date_naive();
    project.write_item(&id, &doc)?;

    println!("Created {} — {}", plan.id, plan.title);
    println!("Linked to {}", id);

    Ok(())
}
