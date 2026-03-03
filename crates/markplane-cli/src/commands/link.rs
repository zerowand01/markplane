use markplane_core::{LinkAction, LinkRelation, Project};

pub fn run(from: String, to: String, relation: String, remove: bool) -> anyhow::Result<()> {
    let project = Project::from_current_dir()?;
    let relation: LinkRelation = relation.parse()?;
    let action = if remove {
        LinkAction::Remove
    } else {
        LinkAction::Add
    };

    project.link_items(&from, &to, relation, action)?;

    let verb = if remove { "Unlinked" } else { "Linked" };
    println!("{} {} {} {}", verb, from, relation, to);

    Ok(())
}
