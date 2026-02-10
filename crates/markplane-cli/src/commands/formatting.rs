use colored::Colorize;

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

pub(crate) fn colorize_status(status: &str) -> String {
    match status {
        "done" => status.green().to_string(),
        "in-progress" | "active" => status.yellow().to_string(),
        "cancelled" => status.red().dimmed().to_string(),
        "draft" => status.dimmed().to_string(),
        "backlog" => status.blue().to_string(),
        "planned" | "approved" => status.cyan().to_string(),
        "paused" => status.magenta().to_string(),
        "archived" => status.dimmed().to_string(),
        _ => status.to_string(),
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
