use colored::Colorize;
use markplane_core::Project;

pub fn run() -> anyhow::Result<()> {
    let project = Project::from_current_dir()?;

    println!("{}", "Syncing...".dimmed());

    project.sync_all()?;

    println!("{} All INDEX.md files and .context/ summaries regenerated.", "✓".green());

    Ok(())
}
