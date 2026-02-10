use markplane_core::{Project, QueryFilter};
use tabled::{Table, Tabled};

use super::{parse_comma_list, LsKind};
use super::formatting::{truncate, colorize_status, colorize_priority};

#[derive(Tabled)]
struct BacklogRow {
    #[tabled(rename = "ID")]
    id: String,
    #[tabled(rename = "Title")]
    title: String,
    #[tabled(rename = "Status")]
    status: String,
    #[tabled(rename = "Priority")]
    priority: String,
    #[tabled(rename = "Effort")]
    effort: String,
    #[tabled(rename = "Epic")]
    epic: String,
}

#[derive(Tabled)]
struct EpicRow {
    #[tabled(rename = "ID")]
    id: String,
    #[tabled(rename = "Title")]
    title: String,
    #[tabled(rename = "Status")]
    status: String,
    #[tabled(rename = "Priority")]
    priority: String,
}

#[derive(Tabled)]
struct PlanRow {
    #[tabled(rename = "ID")]
    id: String,
    #[tabled(rename = "Title")]
    title: String,
    #[tabled(rename = "Status")]
    status: String,
    #[tabled(rename = "Implements")]
    implements: String,
}

#[derive(Tabled)]
struct NoteRow {
    #[tabled(rename = "ID")]
    id: String,
    #[tabled(rename = "Title")]
    title: String,
    #[tabled(rename = "Type")]
    note_type: String,
    #[tabled(rename = "Status")]
    status: String,
}

pub fn run(
    kind: Option<LsKind>,
    status: Option<String>,
    priority: Option<String>,
    epic: Option<String>,
    tags: Option<String>,
    assignee: Option<String>,
    item_type: Option<String>,
) -> anyhow::Result<()> {
    let project = Project::from_current_dir()?;

    match kind {
        Some(LsKind::Epics) => list_epics(&project),
        Some(LsKind::Plans) => list_plans(&project),
        Some(LsKind::Notes) => list_notes(&project),
        None => list_backlog(&project, status, priority, epic, tags, assignee, item_type),
    }
}

fn list_backlog(
    project: &Project,
    status: Option<String>,
    priority: Option<String>,
    epic: Option<String>,
    tags: Option<String>,
    assignee: Option<String>,
    item_type: Option<String>,
) -> anyhow::Result<()> {
    let filter = QueryFilter {
        status: status.map(|s| parse_comma_list(&s)),
        priority: priority.map(|s| parse_comma_list(&s)),
        epic,
        tags: tags.map(|s| parse_comma_list(&s)),
        assignee,
        item_type: item_type.map(|s| parse_comma_list(&s)),
    };

    let items = project.list_backlog_items(&filter)?;

    if items.is_empty() {
        println!("No backlog items found.");
        return Ok(());
    }

    let rows: Vec<BacklogRow> = items
        .iter()
        .map(|doc| {
            let fm = &doc.frontmatter;
            BacklogRow {
                id: fm.id.clone(),
                title: truncate(&fm.title, 40),
                status: colorize_status(&fm.status.to_string()),
                priority: colorize_priority(&fm.priority.to_string()),
                effort: fm.effort.to_string(),
                epic: fm.epic.as_deref().unwrap_or("—").to_string(),
            }
        })
        .collect();

    println!("{}", Table::new(rows));

    Ok(())
}

fn list_epics(project: &Project) -> anyhow::Result<()> {
    let items = project.list_epics()?;

    if items.is_empty() {
        println!("No epics found.");
        return Ok(());
    }

    let rows: Vec<EpicRow> = items
        .iter()
        .map(|doc| {
            let fm = &doc.frontmatter;
            EpicRow {
                id: fm.id.clone(),
                title: truncate(&fm.title, 40),
                status: colorize_status(&fm.status.to_string()),
                priority: colorize_priority(&fm.priority.to_string()),
            }
        })
        .collect();

    println!("{}", Table::new(rows));

    Ok(())
}

fn list_plans(project: &Project) -> anyhow::Result<()> {
    let items = project.list_plans()?;

    if items.is_empty() {
        println!("No plans found.");
        return Ok(());
    }

    let rows: Vec<PlanRow> = items
        .iter()
        .map(|doc| {
            let fm = &doc.frontmatter;
            PlanRow {
                id: fm.id.clone(),
                title: truncate(&fm.title, 40),
                status: colorize_status(&fm.status.to_string()),
                implements: if fm.implements.is_empty() {
                    "—".to_string()
                } else {
                    fm.implements.join(", ")
                },
            }
        })
        .collect();

    println!("{}", Table::new(rows));

    Ok(())
}

fn list_notes(project: &Project) -> anyhow::Result<()> {
    let items = project.list_notes()?;

    if items.is_empty() {
        println!("No notes found.");
        return Ok(());
    }

    let rows: Vec<NoteRow> = items
        .iter()
        .map(|doc| {
            let fm = &doc.frontmatter;
            NoteRow {
                id: fm.id.clone(),
                title: truncate(&fm.title, 40),
                note_type: fm.note_type.to_string(),
                status: colorize_status(&fm.status.to_string()),
            }
        })
        .collect();

    println!("{}", Table::new(rows));

    Ok(())
}
