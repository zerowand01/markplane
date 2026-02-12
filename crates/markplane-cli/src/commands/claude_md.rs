use std::env;
use std::fs;
use std::path::PathBuf;

use colored::Colorize;

const SNIPPET: &str = r#"## Project Management
This project uses Markplane for project management. Key files:
- `.markplane/INDEX.md` - Navigation entry point
- `.markplane/.context/summary.md` - Current project state
- `.markplane/backlog/INDEX.md` - All work items
- `.markplane/plans/INDEX.md` - Implementation plans
When working on a task, read the relevant task item and its linked plan first."#;

pub fn run() -> anyhow::Result<()> {
    let claude_md = find_claude_md();

    if let Ok(existing) = fs::read_to_string(&claude_md) {
        if existing.contains("## Project Management") {
            println!(
                "{} CLAUDE.md already contains the Project Management section.",
                "✓".green()
            );
            return Ok(());
        }
        // Append to existing file
        let mut content = existing;
        if !content.ends_with('\n') {
            content.push('\n');
        }
        content.push('\n');
        content.push_str(SNIPPET);
        content.push('\n');
        fs::write(&claude_md, content)?;
        println!(
            "{} Appended Project Management section to {}",
            "✓".green(),
            claude_md.display()
        );
    } else {
        // Create new file
        let content = format!("{}\n", SNIPPET);
        fs::write(&claude_md, content)?;
        println!(
            "{} Created {} with Project Management section",
            "✓".green(),
            claude_md.display()
        );
    }

    Ok(())
}

fn find_claude_md() -> PathBuf {
    // Walk up from current directory looking for an existing CLAUDE.md
    if let Ok(cwd) = env::current_dir() {
        let mut dir = cwd.as_path();
        loop {
            let candidate = dir.join("CLAUDE.md");
            if candidate.exists() {
                return candidate;
            }
            match dir.parent() {
                Some(parent) => dir = parent,
                None => break,
            }
        }
        // No existing CLAUDE.md found — create in cwd
        cwd.join("CLAUDE.md")
    } else {
        PathBuf::from("CLAUDE.md")
    }
}
