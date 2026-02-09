use markplane_core::Project;

pub fn run(id: String, new_status: String) -> anyhow::Result<()> {
    let project = Project::from_current_dir()?;
    project.update_status(&id, &new_status)?;
    println!("{} → {}", id, new_status);
    Ok(())
}
