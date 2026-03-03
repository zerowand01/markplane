use colored::Colorize;
use markplane_core::Project;
use std::fs;

pub fn run(focus: Option<String>) -> anyhow::Result<()> {
    let project = Project::from_current_dir()?;

    if let Some(ref tag) = focus {
        // Generate and print the requested context view to stdout
        let (generate, filename): (fn(&Project) -> markplane_core::Result<()>, &str) = match tag
            .as_str()
        {
            "active-work" => (Project::generate_context_active_work, "active-work.md"),
            "blocked" => (Project::generate_context_blocked, "blocked-items.md"),
            "metrics" => (Project::generate_context_metrics, "metrics.md"),
            "summary" => (Project::generate_context_summary, "summary.md"),
            other => anyhow::bail!(
                "Unknown focus area '{}'. Valid options: active-work, blocked, metrics, summary",
                other
            ),
        };
        generate(&project)?;
        let path = project.root().join(format!(".context/{}", filename));
        let content = fs::read_to_string(&path)?;
        print!("{}", content);
    } else {
        project.generate_all_context()?;
        println!("{} Context files regenerated in .context/", "✓".green());
    }

    Ok(())
}
