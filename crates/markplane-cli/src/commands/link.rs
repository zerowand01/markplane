use chrono::Local;
use markplane_core::{parse_id, Task, IdPrefix, MarkplaneDocument, Project};

pub fn run(id: String, blocks: Option<String>, depends_on: Option<String>) -> anyhow::Result<()> {
    if blocks.is_none() && depends_on.is_none() {
        anyhow::bail!("Specify --blocks or --depends-on (or both)");
    }

    let project = Project::from_current_dir()?;
    let (prefix, _) = parse_id(&id)?;

    if prefix != IdPrefix::Task {
        anyhow::bail!("Link is currently only supported for tasks. Got: {}", id);
    }

    let mut doc: MarkplaneDocument<Task> = project.read_item(&id)?;
    let today = Local::now().date_naive();

    if let Some(ref target) = blocks {
        // Validate target exists
        let _ = parse_id(target)?;
        if !doc.frontmatter.blocks.contains(target) {
            doc.frontmatter.blocks.push(target.clone());
        }

        // Add reverse link on target
        let (target_prefix, _) = parse_id(target)?;
        if target_prefix == IdPrefix::Task {
            let mut target_doc: MarkplaneDocument<Task> = project.read_item(target)?;
            if !target_doc.frontmatter.depends_on.contains(&id) {
                target_doc.frontmatter.depends_on.push(id.clone());
                target_doc.frontmatter.updated = today;
                project.write_item(target, &target_doc)?;
            }
        }

        println!("{} blocks {}", id, target);
    }

    if let Some(ref target) = depends_on {
        // Validate target exists
        let _ = parse_id(target)?;
        if !doc.frontmatter.depends_on.contains(target) {
            doc.frontmatter.depends_on.push(target.clone());
        }

        // Add reverse link on target
        let (target_prefix, _) = parse_id(target)?;
        if target_prefix == IdPrefix::Task {
            let mut target_doc: MarkplaneDocument<Task> = project.read_item(target)?;
            if !target_doc.frontmatter.blocks.contains(&id) {
                target_doc.frontmatter.blocks.push(id.clone());
                target_doc.frontmatter.updated = today;
                project.write_item(target, &target_doc)?;
            }
        }

        println!("{} depends on {}", id, target);
    }

    doc.frontmatter.updated = today;
    project.write_item(&id, &doc)?;

    Ok(())
}
