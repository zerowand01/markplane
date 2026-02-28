use colored::Colorize;
use markplane_core::StatusCategory;

pub(crate) fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        let mut end = max - 1;
        while !s.is_char_boundary(end) && end > 0 {
            end -= 1;
        }
        format!("{}…", &s[..end])
    }
}

pub(crate) fn colorize_status(status: &str, category: Option<StatusCategory>) -> String {
    // Use category for color if available, fall back to string matching for non-task statuses
    match category {
        Some(StatusCategory::Completed) => status.green().to_string(),
        Some(StatusCategory::Active) => status.blue().to_string(),
        Some(StatusCategory::Cancelled) => status.red().dimmed().to_string(),
        Some(StatusCategory::Draft) => status.dimmed().to_string(),
        Some(StatusCategory::Backlog) => status.blue().to_string(),
        Some(StatusCategory::Planned) => status.cyan().to_string(),
        None => {
            // Fallback for non-task statuses (epics, plans, notes)
            match status {
                "done" => status.green().to_string(),
                "in-progress" | "active" | "now" => status.yellow().to_string(),
                "cancelled" => status.red().dimmed().to_string(),
                "draft" => status.dimmed().to_string(),
                "backlog" => status.blue().to_string(),
                "planned" | "approved" | "next" => status.cyan().to_string(),
                "archived" | "later" => status.dimmed().to_string(),
                _ => status.to_string(),
            }
        }
    }
}

pub(crate) fn colorize_priority(priority: &str) -> String {
    match priority {
        "critical" => priority.red().bold().to_string(),
        "high" => priority.red().to_string(),
        "medium" => priority.yellow().to_string(),
        "low" | "someday" => priority.dimmed().to_string(),
        _ => priority.to_string(),
    }
}
