use markplane_core::{parse_id, IdPrefix, Project, StatusCategory};

pub fn run(id: String) -> anyhow::Result<()> {
    let project = Project::from_current_dir()?;
    let (prefix, _) = parse_id(&id)?;

    let done_status = if prefix == IdPrefix::Task {
        let config = project.load_config()?;
        config.workflows.task
            .statuses_in(StatusCategory::Completed)
            .first()
            .cloned()
            .unwrap_or_else(|| "done".to_string())
    } else {
        "done".to_string()
    };

    project.update_status(&id, &done_status)?;
    println!("{} → {}", id, done_status);
    Ok(())
}
