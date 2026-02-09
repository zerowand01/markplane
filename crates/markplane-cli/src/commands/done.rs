use markplane_core::Project;

pub fn run(id: String) -> anyhow::Result<()> {
    let project = Project::from_current_dir()?;
    project.update_status(&id, "done")?;
    println!("{} → done", id);
    Ok(())
}
