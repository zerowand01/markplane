use colored::Colorize;
use markplane_core::{Priority, Project, QueryFilter, ScanScope, StatusCategory};

pub fn run() -> anyhow::Result<()> {
    let project = Project::from_current_dir()?;
    let config = project.load_config()?;
    let workflow = &config.workflows.task;
    let items = project.list_tasks(&QueryFilter::default())?;
    let epics = project.list_epics()?;
    let plans = project.list_plans()?;

    println!(
        "{}",
        format!("Markplane Metrics — {}", config.project.name).bold()
    );
    println!("{}", "─".repeat(50).dimmed());
    println!();

    // Status distribution (by category)
    let total = items.len();
    let count_category = |cat: StatusCategory| -> usize {
        let statuses: std::collections::HashSet<&str> = workflow
            .statuses_in(cat)
            .iter()
            .map(|s| s.as_str())
            .collect();
        items
            .iter()
            .filter(|i| statuses.contains(i.frontmatter.status.as_str()))
            .count()
    };

    let in_progress = count_category(StatusCategory::Active);
    let planned = count_category(StatusCategory::Planned);
    let backlog = count_category(StatusCategory::Backlog);
    let draft = count_category(StatusCategory::Draft);
    let done = count_category(StatusCategory::Completed);
    let cancelled = count_category(StatusCategory::Cancelled);

    println!("{}", "Task Status".bold());
    println!("  Total:       {}", total);
    println!("  In Progress: {}", in_progress.to_string().yellow());
    println!("  Planned:     {}", planned.to_string().cyan());
    println!("  Backlog:     {}", backlog.to_string().blue());
    println!("  Draft:       {}", draft.to_string().dimmed());
    println!("  Done:        {}", done.to_string().green());
    println!("  Cancelled:   {}", cancelled.to_string().dimmed());
    println!();

    // Priority distribution (open items only)
    let closed_statuses: std::collections::HashSet<&str> = workflow
        .statuses_in(StatusCategory::Completed)
        .iter()
        .chain(workflow.statuses_in(StatusCategory::Cancelled).iter())
        .map(|s| s.as_str())
        .collect();

    let count_priority = |p: &Priority| {
        items
            .iter()
            .filter(|i| {
                i.frontmatter.priority == *p
                    && !closed_statuses.contains(i.frontmatter.status.as_str())
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

    // Epic progress (uses all tasks including archived for accurate counts)
    if !epics.is_empty() {
        let all_tasks = project.list_tasks(&QueryFilter {
            scope: ScanScope::All,
            ..Default::default()
        })?;
        let cancelled_set: std::collections::HashSet<&str> = workflow
            .statuses_in(StatusCategory::Cancelled)
            .iter()
            .map(|s| s.as_str())
            .collect();
        let completed_set: std::collections::HashSet<&str> = workflow
            .statuses_in(StatusCategory::Completed)
            .iter()
            .map(|s| s.as_str())
            .collect();
        println!("{}", "Epic Progress".bold());
        for epic in &epics {
            let epic_items: Vec<_> = all_tasks
                .iter()
                .filter(|i| {
                    i.frontmatter.epic.as_deref() == Some(&epic.frontmatter.id)
                        && !cancelled_set.contains(i.frontmatter.status.as_str())
                })
                .collect();
            let epic_total = epic_items.len();
            let epic_done = epic_items
                .iter()
                .filter(|i| completed_set.contains(i.frontmatter.status.as_str()))
                .count();
            let pct = if epic_total > 0 {
                (epic_done as f64 / epic_total as f64 * 100.0) as u32
            } else {
                0
            };
            let bar = progress_bar(pct, 20);
            println!(
                "  {} {} {}  {}/{} ({}%)",
                epic.frontmatter.id, epic.frontmatter.title, bar, epic_done, epic_total, pct
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
