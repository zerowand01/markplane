use colored::Colorize;
use markplane_core::Project;
use std::fs;

pub fn run(item: Option<String>, focus: Option<String>) -> anyhow::Result<()> {
    let project = Project::from_current_dir()?;

    if let Some(ref id) = item {
        // Generate focused context for a specific item
        generate_item_context(&project, id)?;
    } else if let Some(ref _tag) = focus {
        // For now, just regenerate all context (focus filtering is a future enhancement)
        project.generate_all_context()?;
        println!(
            "{} Context files regenerated.",
            "✓".green()
        );
    } else {
        project.generate_all_context()?;
        println!(
            "{} Context files regenerated in .context/",
            "✓".green()
        );
    }

    Ok(())
}

/// Print focused context for a specific item by reading the item,
/// its linked plan, epic, and dependencies.
fn generate_item_context(project: &Project, id: &str) -> anyhow::Result<()> {
    let (prefix, _) = markplane_core::parse_id(id)?;

    match prefix {
        markplane_core::IdPrefix::Task => {
            let doc: markplane_core::MarkplaneDocument<markplane_core::Task> =
                project.read_item(id)?;
            let fm = &doc.frontmatter;

            println!("{}", format!("Context for {}", id).bold());
            println!("{}", "─".repeat(60).dimmed());
            println!();

            // Item itself
            println!("## {} — {}", fm.id, fm.title);
            println!("Status: {} | Priority: {} | Type: {} | Effort: {}", fm.status, fm.priority, fm.item_type, fm.effort);
            if let Some(ref epic) = fm.epic {
                println!("Epic: {}", epic);
            }
            if let Some(ref plan) = fm.plan {
                println!("Plan: {}", plan);
            }
            if !fm.depends_on.is_empty() {
                println!("Depends on: {}", fm.depends_on.join(", "));
            }
            if !fm.blocks.is_empty() {
                println!("Blocks: {}", fm.blocks.join(", "));
            }
            if !doc.body.trim().is_empty() {
                println!();
                println!("{}", doc.body.trim());
            }

            // Linked epic
            if let Some(ref epic_id) = fm.epic
                && let Ok(epic_doc) = project.read_item::<markplane_core::Epic>(epic_id) {
                    println!();
                    println!("{}", "─".repeat(60).dimmed());
                    println!("## Epic: {} — {}", epic_doc.frontmatter.id, epic_doc.frontmatter.title);
                    println!("Status: {}", epic_doc.frontmatter.status);
                }

            // Linked plan
            if let Some(ref plan_id) = fm.plan
                && let Ok(plan_doc) = project.read_item::<markplane_core::Plan>(plan_id) {
                    println!();
                    println!("{}", "─".repeat(60).dimmed());
                    println!("## Plan: {} — {}", plan_doc.frontmatter.id, plan_doc.frontmatter.title);
                    println!("Status: {}", plan_doc.frontmatter.status);
                    if !plan_doc.body.trim().is_empty() {
                        println!();
                        println!("{}", plan_doc.body.trim());
                    }
                }

            // Dependencies
            for dep_id in &fm.depends_on {
                if let Ok(dep_doc) = project.read_item::<markplane_core::Task>(dep_id) {
                    println!();
                    println!("{}", "─".repeat(60).dimmed());
                    println!("## Dependency: {} — {} ({})", dep_doc.frontmatter.id, dep_doc.frontmatter.title, dep_doc.frontmatter.status);
                }
            }
        }
        _ => {
            // For non-task items, just show the item content
            let path = project.item_path(id)?;
            let content = fs::read_to_string(path)?;
            println!("{}", content);
        }
    }

    Ok(())
}
