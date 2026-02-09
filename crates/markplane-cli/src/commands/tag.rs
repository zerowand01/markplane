use chrono::Local;
use markplane_core::{parse_id, BacklogItem, IdPrefix, MarkplaneDocument, Note, Project};

use super::parse_comma_list;

pub fn run(id: String, tags: String) -> anyhow::Result<()> {
    let project = Project::from_current_dir()?;
    let (prefix, _) = parse_id(&id)?;
    let new_tags = parse_comma_list(&tags);
    let today = Local::now().date_naive();

    match prefix {
        IdPrefix::Back => {
            let mut doc: MarkplaneDocument<BacklogItem> = project.read_item(&id)?;
            for tag in &new_tags {
                if !doc.frontmatter.tags.contains(tag) {
                    doc.frontmatter.tags.push(tag.clone());
                }
            }
            doc.frontmatter.updated = today;
            project.write_item(&id, &doc)?;
        }
        IdPrefix::Note => {
            let mut doc: MarkplaneDocument<Note> = project.read_item(&id)?;
            for tag in &new_tags {
                if !doc.frontmatter.tags.contains(tag) {
                    doc.frontmatter.tags.push(tag.clone());
                }
            }
            doc.frontmatter.updated = today;
            project.write_item(&id, &doc)?;
        }
        _ => {
            anyhow::bail!("Tag is currently only supported for backlog items and notes. Got: {}", id);
        }
    }

    println!("{} tagged with: {}", id, new_tags.join(", "));
    Ok(())
}
