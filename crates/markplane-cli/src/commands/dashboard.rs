use colored::Colorize;
use markplane_core::{BacklogStatus, Priority, Project, QueryFilter};

pub fn run() -> anyhow::Result<()> {
    let project = Project::from_current_dir()?;
    let config = project.load_config()?;
    let items = project.list_backlog_items(&QueryFilter::default())?;
    let epics = project.list_epics()?;

    println!(
        "{}",
        format!("✈  {} — Project Dashboard", config.project.name).bold()
    );
    println!("{}", "═".repeat(50));
    println!();

    // In-progress work
    let in_progress: Vec<_> = items
        .iter()
        .filter(|i| i.frontmatter.status == BacklogStatus::InProgress)
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
    let done_ids: std::collections::HashSet<&str> = items
        .iter()
        .filter(|i| i.frontmatter.status == BacklogStatus::Done)
        .map(|i| i.frontmatter.id.as_str())
        .collect();
    let blocked: Vec<_> = items
        .iter()
        .filter(|i| {
            i.frontmatter.status != BacklogStatus::Done
                && i.frontmatter.status != BacklogStatus::Cancelled
                && !i.frontmatter.depends_on.is_empty()
                && i.frontmatter
                    .depends_on
                    .iter()
                    .any(|dep| !done_ids.contains(dep.as_str()))
        })
        .collect();
    if !blocked.is_empty() {
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

    // Active epics summary
    let active_epics: Vec<_> = epics
        .iter()
        .filter(|e| e.frontmatter.status == markplane_core::EpicStatus::Active)
        .collect();
    if !active_epics.is_empty() {
        println!("{}", "Active Epics".bold().cyan());
        for epic in &active_epics {
            let epic_items: Vec<_> = items
                .iter()
                .filter(|i| i.frontmatter.epic.as_deref() == Some(&epic.frontmatter.id))
                .collect();
            let total = epic_items.len();
            let done_count = epic_items
                .iter()
                .filter(|i| i.frontmatter.status == BacklogStatus::Done)
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
        .filter(|i| {
            i.frontmatter.status != BacklogStatus::Done
                && i.frontmatter.status != BacklogStatus::Cancelled
        })
        .count();
    let critical = items
        .iter()
        .filter(|i| {
            i.frontmatter.priority == Priority::Critical
                && i.frontmatter.status != BacklogStatus::Done
                && i.frontmatter.status != BacklogStatus::Cancelled
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
