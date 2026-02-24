use markplane_core::{NoteType, Project};

use super::parse_comma_list;

pub fn run(
    title: String,
    note_type: String,
    tags: Option<String>,
    template: Option<String>,
) -> anyhow::Result<()> {
    let project = Project::from_current_dir()?;
    let note_type: NoteType = note_type.parse()?;
    let tags = tags.map(|t| parse_comma_list(&t)).unwrap_or_default();
    let note = project.create_note(&title, note_type, tags, template.as_deref())?;
    println!("Created {} — {}", note.id, note.title);
    Ok(())
}
