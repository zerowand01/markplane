use colored::Colorize;
use markplane_core::{BacklogStatus, Priority, Project, QueryFilter};

pub fn run() -> anyhow::Result<()> {
    let project = Project::from_current_dir()?;
    let config = project.load_config()?;
    let items = project.list_backlog_items(&QueryFilter::default())?;
    let epics = project.list_epics()?;
    let plans = project.list_plans()?;

    println!("{}", format!("Markplane Metrics — {}", config.project.name).bold());
    println!("{}", "─".repeat(50).dimmed());
    println!();

    // Status distribution
    let total = items.len();
    let count_status = |s: &BacklogStatus| items.iter().filter(|i| i.frontmatter.status == *s).count();

    let in_progress = count_status(&BacklogStatus::InProgress);
    let planned = count_status(&BacklogStatus::Planned);
    let backlog = count_status(&BacklogStatus::Backlog);
    let draft = count_status(&BacklogStatus::Draft);
    let done = count_status(&BacklogStatus::Done);
    let cancelled = count_status(&BacklogStatus::Cancelled);

    println!("{}", "Backlog Status".bold());
    println!("  Total:       {}", total);
    println!("  In Progress: {}", in_progress.to_string().yellow());
    println!("  Planned:     {}", planned.to_string().cyan());
    println!("  Backlog:     {}", backlog.to_string().blue());
    println!("  Draft:       {}", draft.to_string().dimmed());
    println!("  Done:        {}", done.to_string().green());
    println!("  Cancelled:   {}", cancelled.to_string().dimmed());
    println!();

    // Priority distribution (open items only)
    let count_priority = |p: &Priority| {
        items
            .iter()
            .filter(|i| {
                i.frontmatter.priority == *p
                    && i.frontmatter.status != BacklogStatus::Done
                    && i.frontmatter.status != BacklogStatus::Cancelled
            })
            .count()
    };

    println!("{}", "Priority Distribution (open)".bold());
    println!(
        "  Critical: {}",
        count_priority(&Priority::Critical).to_string().red().bold()
    );
    println!(
        "  High:     {}",
        count_priority(&Priority::High).to_string().red()
    );
    println!(
        "  Medium:   {}",
        count_priority(&Priority::Medium).to_string().yellow()
    );
    println!(
        "  Low:      {}",
        count_priority(&Priority::Low).to_string().dimmed()
    );
    println!(
        "  Someday:  {}",
        count_priority(&Priority::Someday).to_string().dimmed()
    );
    println!();

    // Epic progress
    if !epics.is_empty() {
        println!("{}", "Epic Progress".bold());
        for epic in &epics {
            let epic_items: Vec<_> = items
                .iter()
                .filter(|i| i.frontmatter.epic.as_deref() == Some(&epic.frontmatter.id))
                .collect();
            let epic_total = epic_items.len();
            let epic_done = epic_items
                .iter()
                .filter(|i| i.frontmatter.status == BacklogStatus::Done)
                .count();
            let pct = if epic_total > 0 {
                (epic_done as f64 / epic_total as f64 * 100.0) as u32
            } else {
                0
            };
            let bar = progress_bar(pct, 20);
            println!(
                "  {} {} {}  {}/{} ({}%)",
                epic.frontmatter.id,
                epic.frontmatter.title,
                bar,
                epic_done,
                epic_total,
                pct
            );
        }
        println!();
    }

    // Plans
    if !plans.is_empty() {
        let active_plans = plans
            .iter()
            .filter(|p| p.frontmatter.status != markplane_core::PlanStatus::Done)
            .count();
        let done_plans = plans
            .iter()
            .filter(|p| p.frontmatter.status == markplane_core::PlanStatus::Done)
            .count();
        println!("{}", "Plans".bold());
        println!("  Active:    {}", active_plans);
        println!("  Completed: {}", done_plans);
    }

    Ok(())
}

fn progress_bar(pct: u32, width: usize) -> String {
    let filled = (pct as usize * width) / 100;
    let empty = width - filled;
    format!(
        "[{}{}]",
        "█".repeat(filled).green(),
        "░".repeat(empty).dimmed()
    )
}
