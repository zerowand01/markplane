use colored::Colorize;
use markplane_core::Project;

pub fn run(id: String) -> anyhow::Result<()> {
    let project = Project::from_current_dir()?;
    project.unarchive_item(&id)?;
    println!("{} Restored {} from archive", "✓".green(), id);
    Ok(())
}
