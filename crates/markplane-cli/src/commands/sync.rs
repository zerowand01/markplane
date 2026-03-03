use colored::Colorize;
use markplane_core::Project;

pub fn run(normalize: bool) -> anyhow::Result<()> {
    let project = Project::from_current_dir()?;

    if normalize {
        println!("{}", "Normalizing positions...".dimmed());
        project.normalize_positions()?;
    }

    println!("{}", "Syncing...".dimmed());

    project.sync_all()?;

    println!(
        "{} All INDEX.md files and .context/ summaries regenerated.",
        "✓".green()
    );

    Ok(())
}
