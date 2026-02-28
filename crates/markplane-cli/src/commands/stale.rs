use chrono::Local;
use colored::Colorize;
use markplane_core::{Project, QueryFilter};
use tabled::{Table, Tabled};

use super::formatting::truncate;

#[derive(Tabled)]
struct StaleRow {
    #[tabled(rename = "ID")]
    id: String,
    #[tabled(rename = "Title")]
    title: String,
    #[tabled(rename = "Status")]
    status: String,
    #[tabled(rename = "Last Updated")]
    updated: String,
    #[tabled(rename = "Days Stale")]
    days: String,
}

pub fn run(days: u32) -> anyhow::Result<()> {
    let project = Project::from_current_dir()?;
    let today = Local::now().date_naive();
    let cutoff = today - chrono::Duration::days(days as i64);

    let items = project.list_tasks(&QueryFilter::default())?;

    let config = project.load_config()?;
    let workflow = &config.workflows.task;
    let stale: Vec<StaleRow> = items
        .iter()
        .filter(|doc| {
            let fm = &doc.frontmatter;
            workflow.category_of(&fm.status).is_none_or(|c| c.is_open())
                && fm.updated < cutoff
        })
        .map(|doc| {
            let fm = &doc.frontmatter;
            let stale_days = (today - fm.updated).num_days();
            StaleRow {
                id: fm.id.clone(),
                title: truncate(&fm.title, 40),
                status: fm.status.to_string(),
                updated: fm.updated.to_string(),
                days: stale_days.to_string(),
            }
        })
        .collect();

    if stale.is_empty() {
        println!(
            "{} No items stale for more than {} days.",
            "✓".green(),
            days
        );
    } else {
        println!(
            "{} {} item(s) not updated in {} days:\n",
            "!".yellow(),
            stale.len(),
            days
        );
        println!("{}", Table::new(stale));
    }

    Ok(())
}
