use colored::Colorize;
use markplane_core::{
    EpicStatus, NoteStatus, PlanStatus, Project, QueryFilter,
};

pub fn run(id: Option<String>, all_done: bool, dry_run: bool) -> anyhow::Result<()> {
    let project = Project::from_current_dir()?;

    if let Some(id) = id {
        // Single-item archive
        if dry_run {
            println!("{} Would archive: {}", "→".cyan(), id);
        } else {
            project.archive_item(&id)?;
            println!("{} Archived {}", "✓".green(), id);
        }
        return Ok(());
    }

    if !all_done {
        anyhow::bail!("Provide an item ID or use --all-done to archive all completed items");
    }

    // Batch archive all done/cancelled items across all entity types
    let mut to_archive: Vec<(String, String)> = Vec::new(); // (id, description)

    // Tasks: closed statuses (completed or cancelled categories)
    let config = project.load_config()?;
    let workflow = &config.workflows.task;
    let tasks = project.list_tasks(&QueryFilter::default())?;
    for doc in &tasks {
        let fm = &doc.frontmatter;
        if workflow.category_of(&fm.status).is_some_and(|c| c.is_closed()) {
            to_archive.push((fm.id.clone(), format!("{} ({})", fm.title, fm.status)));
        }
    }

    // Epics: done
    let epics = project.list_epics()?;
    for doc in &epics {
        let fm = &doc.frontmatter;
        if fm.status == EpicStatus::Done {
            to_archive.push((fm.id.clone(), format!("{} ({})", fm.title, fm.status)));
        }
    }

    // Plans: done
    let plans = project.list_plans()?;
    for doc in &plans {
        let fm = &doc.frontmatter;
        if fm.status == PlanStatus::Done {
            to_archive.push((fm.id.clone(), format!("{} ({})", fm.title, fm.status)));
        }
    }

    // Notes: archived status
    let notes = project.list_notes()?;
    for doc in &notes {
        let fm = &doc.frontmatter;
        if fm.status == NoteStatus::Archived {
            to_archive.push((fm.id.clone(), format!("{} ({})", fm.title, fm.status)));
        }
    }

    if to_archive.is_empty() {
        println!("{} No completed items to archive.", "✓".green());
        return Ok(());
    }

    if dry_run {
        println!(
            "{} Would archive {} item(s):\n",
            "→".cyan(),
            to_archive.len()
        );
        for (id, desc) in &to_archive {
            println!("  {} {}", id, desc);
        }
        println!("\nRun without --dry-run to archive.");
    } else {
        let count = to_archive.len();
        for (id, _) in &to_archive {
            project.archive_item(id)?;
            println!("  {} Archived {}", "✓".green(), id);
        }
        println!(
            "\n{} Archived {} item(s).",
            "✓".green(),
            count
        );
    }

    Ok(())
}
