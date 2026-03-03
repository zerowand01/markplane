use colored::Colorize;
use markplane_core::{
    Priority, Project, QueryFilter, ScanScope, StatusCategory, find_blocked_items,
};

pub fn run() -> anyhow::Result<()> {
    let project = Project::from_current_dir()?;
    let config = project.load_config()?;
    let workflow = &config.workflows.task;
    let items = project.list_tasks(&QueryFilter::default())?;
    let epics = project.list_epics()?;

    // Build category sets for filtering
    let active_statuses: std::collections::HashSet<&str> = workflow
        .statuses_in(StatusCategory::Active)
        .iter()
        .map(|s| s.as_str())
        .collect();
    let completed_statuses: std::collections::HashSet<&str> = workflow
        .statuses_in(StatusCategory::Completed)
        .iter()
        .map(|s| s.as_str())
        .collect();
    let cancelled_statuses: std::collections::HashSet<&str> = workflow
        .statuses_in(StatusCategory::Cancelled)
        .iter()
        .map(|s| s.as_str())
        .collect();
    let closed_statuses: std::collections::HashSet<&str> = completed_statuses
        .iter()
        .chain(cancelled_statuses.iter())
        .copied()
        .collect();

    println!(
        "{}",
        format!("✈  {} — Project Dashboard", config.project.name).bold()
    );
    println!("{}", "═".repeat(50));
    println!();

    // In-progress work
    let in_progress: Vec<_> = items
        .iter()
        .filter(|i| active_statuses.contains(i.frontmatter.status.as_str()))
        .collect();
    if !in_progress.is_empty() {
        println!("{}", "In Progress".bold().yellow());
        for item in &in_progress {
            let fm = &item.frontmatter;
            let assignee = fm
                .assignee
                .as_ref()
                .map(|a| format!(" @{}", a))
                .unwrap_or_default();
            println!("  {} {} ({}{})", fm.id, fm.title, fm.priority, assignee);
        }
        println!();
    }

    // Blocked items
    let blocked = find_blocked_items(&items, workflow);
    if !blocked.is_empty() {
        let done_ids: std::collections::HashSet<&str> = items
            .iter()
            .filter(|i| completed_statuses.contains(i.frontmatter.status.as_str()))
            .map(|i| i.frontmatter.id.as_str())
            .collect();
        println!("{}", "Blocked".bold().red());
        for item in &blocked {
            let fm = &item.frontmatter;
            let blockers: Vec<&str> = fm
                .depends_on
                .iter()
                .filter(|dep| !done_ids.contains(dep.as_str()))
                .map(|s| s.as_str())
                .collect();
            println!(
                "  {} {} — blocked by {}",
                fm.id,
                fm.title,
                blockers.join(", ")
            );
        }
        println!();
    }

    // Now epics summary (uses all tasks including archived for accurate progress)
    let now_epics: Vec<_> = epics
        .iter()
        .filter(|e| e.frontmatter.status == markplane_core::EpicStatus::Now)
        .collect();
    if !now_epics.is_empty() {
        let all_tasks = project.list_tasks(&QueryFilter {
            scope: ScanScope::All,
            ..Default::default()
        })?;
        println!("{}", "Now".bold().cyan());
        for epic in &now_epics {
            let epic_items: Vec<_> = all_tasks
                .iter()
                .filter(|i| {
                    i.frontmatter.epic.as_deref() == Some(&epic.frontmatter.id)
                        && !cancelled_statuses.contains(i.frontmatter.status.as_str())
                })
                .collect();
            let total = epic_items.len();
            let done_count = epic_items
                .iter()
                .filter(|i| completed_statuses.contains(i.frontmatter.status.as_str()))
                .count();
            let pct = if total > 0 {
                (done_count as f64 / total as f64 * 100.0) as u32
            } else {
                0
            };
            println!(
                "  {} {} — {}/{} ({}%)",
                epic.frontmatter.id, epic.frontmatter.title, done_count, total, pct
            );
        }
        println!();
    }

    // Quick counts
    let open = items
        .iter()
        .filter(|i| !closed_statuses.contains(i.frontmatter.status.as_str()))
        .count();
    let critical = items
        .iter()
        .filter(|i| {
            i.frontmatter.priority == Priority::Critical
                && !closed_statuses.contains(i.frontmatter.status.as_str())
        })
        .count();

    println!(
        "{} open items | {} in-progress | {} blocked | {} critical",
        open.to_string().bold(),
        in_progress.len().to_string().yellow(),
        blocked.len().to_string().red(),
        critical.to_string().red().bold()
    );

    Ok(())
}
