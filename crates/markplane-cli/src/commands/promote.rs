use markplane_core::{
    parse_id, Effort, IdPrefix, ItemType, MarkplaneDocument, Note, Priority, Project,
};

pub fn run(id: String, priority: String, effort: String) -> anyhow::Result<()> {
    let project = Project::from_current_dir()?;
    let (prefix, _) = parse_id(&id)?;

    if prefix != IdPrefix::Note {
        anyhow::bail!("Can only promote notes. Got: {}", id);
    }

    let doc: MarkplaneDocument<Note> = project.read_item(&id)?;
    let note = &doc.frontmatter;

    let priority: Priority = priority.parse()?;
    let effort: Effort = effort.parse()?;

    let item = project.create_backlog_item(
        &note.title,
        ItemType::Feature,
        priority,
        effort,
        None,
        note.tags.clone(),
    )?;

    println!(
        "Promoted {} → {} — {}",
        id, item.id, item.title
    );

    Ok(())
}
