use colored::Colorize;
use markplane_core::{Epic, IdPrefix, MarkplaneDocument, Note, Plan, Project, Task, parse_id};

use super::formatting::{colorize_priority, colorize_status};

pub fn run(id: String) -> anyhow::Result<()> {
    let project = Project::from_current_dir()?;
    let (prefix, _) = parse_id(&id)?;

    match prefix {
        IdPrefix::Task => show_task(&project, &id)?,
        IdPrefix::Epic => show_epic(&project, &id)?,
        IdPrefix::Plan => show_plan(&project, &id)?,
        IdPrefix::Note => show_note(&project, &id)?,
    }

    Ok(())
}

fn show_task(project: &Project, id: &str) -> anyhow::Result<()> {
    let config = project.load_config()?;
    let doc: MarkplaneDocument<Task> = project.read_item(id)?;
    let fm = &doc.frontmatter;

    println!("{}", fm.id.bold());
    println!("{}", fm.title.bold().white());
    println!();
    println!(
        "  Status:   {}",
        colorize_status(&fm.status, config.task_category(&fm.status))
    );
    println!(
        "  Priority: {}",
        colorize_priority(&fm.priority.to_string())
    );
    println!("  Type:     {}", fm.item_type);
    println!("  Effort:   {}", fm.effort);

    if !fm.tags.is_empty() {
        println!("  Tags:     {}", fm.tags.join(", "));
    }
    if let Some(ref epic) = fm.epic {
        println!("  Epic:     {}", epic);
    }
    if let Some(ref plan) = fm.plan {
        println!("  Plan:     {}", plan);
    }
    if let Some(ref assignee) = fm.assignee {
        println!("  Assignee: {}", assignee);
    }
    if !fm.depends_on.is_empty() {
        println!("  Depends:  {}", fm.depends_on.join(", "));
    }
    if !fm.blocks.is_empty() {
        println!("  Blocks:   {}", fm.blocks.join(", "));
    }
    if !fm.related.is_empty() {
        println!("  Related:  {}", fm.related.join(", "));
    }
    println!("  Created:  {}", fm.created);
    println!("  Updated:  {}", fm.updated);

    if !doc.body.trim().is_empty() {
        println!();
        println!("{}", "─".repeat(60).dimmed());
        println!("{}", doc.body.trim());
    }

    Ok(())
}

fn show_epic(project: &Project, id: &str) -> anyhow::Result<()> {
    let doc: MarkplaneDocument<Epic> = project.read_item(id)?;
    let fm = &doc.frontmatter;

    println!("{}", fm.id.bold());
    println!("{}", fm.title.bold().white());
    println!();
    println!(
        "  Status:   {}",
        colorize_status(&fm.status.to_string(), None)
    );
    println!(
        "  Priority: {}",
        colorize_priority(&fm.priority.to_string())
    );

    if let Some(ref started) = fm.started {
        println!("  Started:  {}", started);
    }
    if let Some(ref target) = fm.target {
        println!("  Target:   {}", target);
    }
    if !fm.tags.is_empty() {
        println!("  Tags:     {}", fm.tags.join(", "));
    }
    if !fm.related.is_empty() {
        println!("  Related:  {}", fm.related.join(", "));
    }
    println!("  Created:  {}", fm.created);
    println!("  Updated:  {}", fm.updated);

    if !doc.body.trim().is_empty() {
        println!();
        println!("{}", "─".repeat(60).dimmed());
        println!("{}", doc.body.trim());
    }

    Ok(())
}

fn show_plan(project: &Project, id: &str) -> anyhow::Result<()> {
    let doc: MarkplaneDocument<Plan> = project.read_item(id)?;
    let fm = &doc.frontmatter;

    println!("{}", fm.id.bold());
    println!("{}", fm.title.bold().white());
    println!();
    println!(
        "  Status:     {}",
        colorize_status(&fm.status.to_string(), None)
    );

    if !fm.implements.is_empty() {
        println!("  Implements: {}", fm.implements.join(", "));
    }
    if !fm.related.is_empty() {
        println!("  Related:    {}", fm.related.join(", "));
    }
    println!("  Created:    {}", fm.created);
    println!("  Updated:    {}", fm.updated);

    if !doc.body.trim().is_empty() {
        println!();
        println!("{}", "─".repeat(60).dimmed());
        println!("{}", doc.body.trim());
    }

    Ok(())
}

fn show_note(project: &Project, id: &str) -> anyhow::Result<()> {
    let doc: MarkplaneDocument<Note> = project.read_item(id)?;
    let fm = &doc.frontmatter;

    println!("{}", fm.id.bold());
    println!("{}", fm.title.bold().white());
    println!();
    println!(
        "  Status:  {}",
        colorize_status(&fm.status.to_string(), None)
    );
    println!("  Type:    {}", fm.note_type);

    if !fm.tags.is_empty() {
        println!("  Tags:    {}", fm.tags.join(", "));
    }
    if !fm.related.is_empty() {
        println!("  Related: {}", fm.related.join(", "));
    }
    println!("  Created: {}", fm.created);
    println!("  Updated: {}", fm.updated);

    if !doc.body.trim().is_empty() {
        println!();
        println!("{}", "─".repeat(60).dimmed());
        println!("{}", doc.body.trim());
    }

    Ok(())
}
