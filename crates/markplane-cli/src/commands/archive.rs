use chrono::Local;
use colored::Colorize;
use markplane_core::{BacklogStatus, Project, QueryFilter};

pub fn run(dry_run: bool) -> anyhow::Result<()> {
    let project = Project::from_current_dir()?;
    let config = project.load_config()?;
    let today = Local::now().date_naive();
    let cutoff = today - chrono::Duration::days(config.archive.auto_archive_after_days as i64);

    let items = project.list_backlog_items(&QueryFilter::default())?;

    let archivable: Vec<_> = items
        .iter()
        .filter(|doc| {
            let fm = &doc.frontmatter;
            let is_done = fm.status == BacklogStatus::Done;
            let is_cancelled = fm.status == BacklogStatus::Cancelled && config.archive.keep_cancelled;
            (is_done || is_cancelled) && fm.updated <= cutoff
        })
        .collect();

    if archivable.is_empty() {
        println!(
            "{} No items eligible for archiving (done/cancelled for {}+ days).",
            "✓".green(),
            config.archive.auto_archive_after_days
        );
        return Ok(());
    }

    if dry_run {
        println!(
            "{} Would archive {} item(s):\n",
            "→".cyan(),
            archivable.len()
        );
        for doc in &archivable {
            let fm = &doc.frontmatter;
            println!("  {} {} ({})", fm.id, fm.title, fm.status);
        }
        println!("\nRun without --dry-run to archive.");
    } else {
        let count = archivable.len();
        for doc in &archivable {
            project.archive_item(&doc.frontmatter.id)?;
            println!("  {} Archived {}", "✓".green(), doc.frontmatter.id);
        }
        println!(
            "\n{} Archived {} item(s).",
            "✓".green(),
            count
        );
    }

    Ok(())
}
