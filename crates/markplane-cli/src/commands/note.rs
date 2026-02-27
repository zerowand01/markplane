use markplane_core::Project;

use super::parse_comma_list;

pub fn run(
    title: String,
    note_type: Option<String>,
    tags: Option<String>,
    template: Option<String>,
) -> anyhow::Result<()> {
    let project = Project::from_current_dir()?;
    let config = project.load_config()?;
    let note_type = note_type.as_deref().unwrap_or(config.default_note_type());
    let tags = tags.map(|t| parse_comma_list(&t)).unwrap_or_default();
    let note = project.create_note(&title, note_type, tags, template.as_deref())?;
    println!("Created {} — {}", note.id, note.title);
    Ok(())
}
