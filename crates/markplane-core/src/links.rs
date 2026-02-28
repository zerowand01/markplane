use std::fmt;
use std::str::FromStr;

use chrono::{Local, NaiveDate};

use crate::error::{MarkplaneError, Result};
use crate::models::*;
use crate::project::Project;

// ── Types ─────────────────────────────────────────────────────────────────

/// The kind of relationship between two items.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LinkRelation {
    /// Task blocks Task (reciprocal: depends_on)
    Blocks,
    /// Task depends on Task (reciprocal: blocks)
    DependsOn,
    /// Task belongs to Epic (no reciprocal)
    Epic,
    /// Task -> Plan (reciprocal: plan.implements)
    Plan,
    /// Plan implements Task (reciprocal: task.plan)
    Implements,
    /// Any item can be related to any other item (bidirectional)
    Related,
}

impl FromStr for LinkRelation {
    type Err = MarkplaneError;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "blocks" => Ok(LinkRelation::Blocks),
            "depends_on" | "depends-on" => Ok(LinkRelation::DependsOn),
            "epic" => Ok(LinkRelation::Epic),
            "plan" => Ok(LinkRelation::Plan),
            "implements" => Ok(LinkRelation::Implements),
            "related" => Ok(LinkRelation::Related),
            _ => Err(MarkplaneError::InvalidLink(format!(
                "Unknown link relation: {}. Use blocks, depends_on, epic, plan, implements, or related.",
                s
            ))),
        }
    }
}

impl fmt::Display for LinkRelation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LinkRelation::Blocks => write!(f, "blocks"),
            LinkRelation::DependsOn => write!(f, "depends-on"),
            LinkRelation::Epic => write!(f, "epic"),
            LinkRelation::Plan => write!(f, "plan"),
            LinkRelation::Implements => write!(f, "implements"),
            LinkRelation::Related => write!(f, "related"),
        }
    }
}

/// Whether to add or remove a link.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LinkAction {
    Add,
    Remove,
}

impl fmt::Display for LinkAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LinkAction::Add => write!(f, "added"),
            LinkAction::Remove => write!(f, "removed"),
        }
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────

fn push_unique(vec: &mut Vec<String>, val: &str) {
    if !vec.iter().any(|v| v == val) {
        vec.push(val.to_string());
    }
}

fn remove_value(vec: &mut Vec<String>, val: &str) {
    vec.retain(|v| v != val);
}

fn require_prefix(id: &str, actual: &IdPrefix, expected: &[IdPrefix]) -> Result<()> {
    if !expected.contains(actual) {
        let names: Vec<&str> = expected.iter().map(|p| p.as_str()).collect();
        return Err(MarkplaneError::InvalidLink(format!(
            "{} must be a {} item, got {}",
            id,
            names.join(" or "),
            actual.as_str()
        )));
    }
    Ok(())
}

/// Update the `related` vec on a single item (any entity type).
fn update_related(
    project: &Project,
    id: &str,
    target: &str,
    prefix: &IdPrefix,
    action: LinkAction,
    today: NaiveDate,
) -> Result<()> {
    match prefix {
        IdPrefix::Task => {
            let mut doc: MarkplaneDocument<Task> = project.read_item(id)?;
            match action {
                LinkAction::Add => push_unique(&mut doc.frontmatter.related, target),
                LinkAction::Remove => remove_value(&mut doc.frontmatter.related, target),
            }
            doc.frontmatter.updated = today;
            project.write_item(id, &doc)?;
        }
        IdPrefix::Epic => {
            let mut doc: MarkplaneDocument<Epic> = project.read_item(id)?;
            match action {
                LinkAction::Add => push_unique(&mut doc.frontmatter.related, target),
                LinkAction::Remove => remove_value(&mut doc.frontmatter.related, target),
            }
            doc.frontmatter.updated = today;
            project.write_item(id, &doc)?;
        }
        IdPrefix::Plan => {
            let mut doc: MarkplaneDocument<Plan> = project.read_item(id)?;
            match action {
                LinkAction::Add => push_unique(&mut doc.frontmatter.related, target),
                LinkAction::Remove => remove_value(&mut doc.frontmatter.related, target),
            }
            doc.frontmatter.updated = today;
            project.write_item(id, &doc)?;
        }
        IdPrefix::Note => {
            let mut doc: MarkplaneDocument<Note> = project.read_item(id)?;
            match action {
                LinkAction::Add => push_unique(&mut doc.frontmatter.related, target),
                LinkAction::Remove => remove_value(&mut doc.frontmatter.related, target),
            }
            doc.frontmatter.updated = today;
            project.write_item(id, &doc)?;
        }
    }
    Ok(())
}

// ── Implementation ────────────────────────────────────────────────────────

impl Project {
    /// Add or remove a link between two items.
    pub fn link_items(
        &self,
        from: &str,
        to: &str,
        relation: LinkRelation,
        action: LinkAction,
    ) -> Result<()> {
        // Self-link check
        if from == to {
            return Err(MarkplaneError::InvalidLink(format!(
                "Cannot link an item to itself: {}",
                from
            )));
        }

        // Validate both items exist
        self.item_path(from)?;
        self.item_path(to)?;

        let (from_prefix, _) = parse_id(from)?;
        let (to_prefix, _) = parse_id(to)?;
        let today = Local::now().date_naive();

        match relation {
            LinkRelation::Blocks => {
                require_prefix(from, &from_prefix, &[IdPrefix::Task])?;
                require_prefix(to, &to_prefix, &[IdPrefix::Task])?;

                let mut from_doc: MarkplaneDocument<Task> = self.read_item(from)?;
                let mut to_doc: MarkplaneDocument<Task> = self.read_item(to)?;

                match action {
                    LinkAction::Add => {
                        push_unique(&mut from_doc.frontmatter.blocks, to);
                        push_unique(&mut to_doc.frontmatter.depends_on, from);
                    }
                    LinkAction::Remove => {
                        remove_value(&mut from_doc.frontmatter.blocks, to);
                        remove_value(&mut to_doc.frontmatter.depends_on, from);
                    }
                }

                from_doc.frontmatter.updated = today;
                to_doc.frontmatter.updated = today;
                self.write_item(from, &from_doc)?;
                self.write_item(to, &to_doc)?;
            }

            LinkRelation::DependsOn => {
                require_prefix(from, &from_prefix, &[IdPrefix::Task])?;
                require_prefix(to, &to_prefix, &[IdPrefix::Task])?;

                let mut from_doc: MarkplaneDocument<Task> = self.read_item(from)?;
                let mut to_doc: MarkplaneDocument<Task> = self.read_item(to)?;

                match action {
                    LinkAction::Add => {
                        push_unique(&mut from_doc.frontmatter.depends_on, to);
                        push_unique(&mut to_doc.frontmatter.blocks, from);
                    }
                    LinkAction::Remove => {
                        remove_value(&mut from_doc.frontmatter.depends_on, to);
                        remove_value(&mut to_doc.frontmatter.blocks, from);
                    }
                }

                from_doc.frontmatter.updated = today;
                to_doc.frontmatter.updated = today;
                self.write_item(from, &from_doc)?;
                self.write_item(to, &to_doc)?;
            }

            LinkRelation::Epic => {
                require_prefix(from, &from_prefix, &[IdPrefix::Task])?;
                require_prefix(to, &to_prefix, &[IdPrefix::Epic])?;

                let mut doc: MarkplaneDocument<Task> = self.read_item(from)?;
                match action {
                    LinkAction::Add => {
                        doc.frontmatter.epic = Some(to.to_string());
                    }
                    LinkAction::Remove => {
                        if doc.frontmatter.epic.as_deref() == Some(to) {
                            doc.frontmatter.epic = None;
                        }
                    }
                }
                doc.frontmatter.updated = today;
                self.write_item(from, &doc)?;
            }

            LinkRelation::Plan => {
                require_prefix(from, &from_prefix, &[IdPrefix::Task])?;
                require_prefix(to, &to_prefix, &[IdPrefix::Plan])?;

                let mut task_doc: MarkplaneDocument<Task> = self.read_item(from)?;
                let mut plan_doc: MarkplaneDocument<Plan> = self.read_item(to)?;

                match action {
                    LinkAction::Add => {
                        // If task already has a different plan, clean up old plan
                        if let Some(ref old_plan_id) = task_doc.frontmatter.plan
                            && old_plan_id != to
                        {
                            let mut old_plan: MarkplaneDocument<Plan> =
                                self.read_item(old_plan_id)?;
                            remove_value(&mut old_plan.frontmatter.implements, from);
                            old_plan.frontmatter.updated = today;
                            self.write_item(old_plan_id, &old_plan)?;
                        }
                        task_doc.frontmatter.plan = Some(to.to_string());
                        push_unique(&mut plan_doc.frontmatter.implements, from);
                    }
                    LinkAction::Remove => {
                        if task_doc.frontmatter.plan.as_deref() == Some(to) {
                            task_doc.frontmatter.plan = None;
                        }
                        remove_value(&mut plan_doc.frontmatter.implements, from);
                    }
                }

                task_doc.frontmatter.updated = today;
                plan_doc.frontmatter.updated = today;
                self.write_item(from, &task_doc)?;
                self.write_item(to, &plan_doc)?;
            }

            LinkRelation::Implements => {
                require_prefix(from, &from_prefix, &[IdPrefix::Plan])?;
                require_prefix(to, &to_prefix, &[IdPrefix::Task])?;

                let mut plan_doc: MarkplaneDocument<Plan> = self.read_item(from)?;
                let mut task_doc: MarkplaneDocument<Task> = self.read_item(to)?;

                match action {
                    LinkAction::Add => {
                        // If task already has a different plan, clean up old plan
                        if let Some(ref old_plan_id) = task_doc.frontmatter.plan
                            && old_plan_id != from
                        {
                            let mut old_plan: MarkplaneDocument<Plan> =
                                self.read_item(old_plan_id)?;
                            remove_value(&mut old_plan.frontmatter.implements, to);
                            old_plan.frontmatter.updated = today;
                            self.write_item(old_plan_id, &old_plan)?;
                        }
                        push_unique(&mut plan_doc.frontmatter.implements, to);
                        task_doc.frontmatter.plan = Some(from.to_string());
                    }
                    LinkAction::Remove => {
                        remove_value(&mut plan_doc.frontmatter.implements, to);
                        if task_doc.frontmatter.plan.as_deref() == Some(from) {
                            task_doc.frontmatter.plan = None;
                        }
                    }
                }

                plan_doc.frontmatter.updated = today;
                task_doc.frontmatter.updated = today;
                self.write_item(from, &plan_doc)?;
                self.write_item(to, &task_doc)?;
            }

            LinkRelation::Related => {
                // Any item can be related to any other — bidirectional
                update_related(self, from, to, &from_prefix, action, today)?;
                update_related(self, to, from, &to_prefix, action, today)?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_project() -> (TempDir, Project) {
        let tmp = TempDir::new().unwrap();
        let project = Project::init(tmp.path().to_path_buf(), "test-project", "Test").unwrap();
        (tmp, project)
    }

    fn create_task(project: &Project, title: &str) -> String {
        let task = project
            .create_task(title, "feature", Priority::Medium, Effort::Medium, None, vec![], None)
            .unwrap();
        task.id
    }

    fn create_epic(project: &Project, title: &str) -> String {
        let epic = project.create_epic(title, Priority::Medium, None).unwrap();
        epic.id
    }

    fn create_plan(project: &Project, title: &str, task_id: &str) -> String {
        let plan = project.create_plan(title, vec![], None).unwrap();
        // Don't auto-link — we'll use link_items to test
        let _ = task_id;
        plan.id
    }

    fn create_note(project: &Project, title: &str) -> String {
        let note = project
            .create_note(title, "idea", vec![], None)
            .unwrap();
        note.id
    }

    #[test]
    fn test_link_blocks() {
        let (_tmp, project) = setup_project();
        let t1 = create_task(&project, "Blocker");
        let t2 = create_task(&project, "Blocked");

        project
            .link_items(&t1, &t2, LinkRelation::Blocks, LinkAction::Add)
            .unwrap();

        let doc1: MarkplaneDocument<Task> = project.read_item(&t1).unwrap();
        let doc2: MarkplaneDocument<Task> = project.read_item(&t2).unwrap();
        assert!(doc1.frontmatter.blocks.contains(&t2));
        assert!(doc2.frontmatter.depends_on.contains(&t1));
    }

    #[test]
    fn test_link_depends_on_task_task() {
        let (_tmp, project) = setup_project();
        let t1 = create_task(&project, "A");
        let t2 = create_task(&project, "B");

        project
            .link_items(&t1, &t2, LinkRelation::DependsOn, LinkAction::Add)
            .unwrap();

        let doc1: MarkplaneDocument<Task> = project.read_item(&t1).unwrap();
        let doc2: MarkplaneDocument<Task> = project.read_item(&t2).unwrap();
        assert!(doc1.frontmatter.depends_on.contains(&t2));
        assert!(doc2.frontmatter.blocks.contains(&t1));
    }

    #[test]
    fn test_depends_on_rejects_epic_as_source() {
        let (_tmp, project) = setup_project();
        let e1 = create_epic(&project, "Epic");
        let t1 = create_task(&project, "Task");

        let result = project.link_items(&e1, &t1, LinkRelation::DependsOn, LinkAction::Add);
        assert!(result.is_err());
    }

    #[test]
    fn test_depends_on_rejects_epic_as_target() {
        let (_tmp, project) = setup_project();
        let t1 = create_task(&project, "Task");
        let e1 = create_epic(&project, "Epic");

        let result = project.link_items(&t1, &e1, LinkRelation::DependsOn, LinkAction::Add);
        assert!(result.is_err());
    }

    #[test]
    fn test_depends_on_rejects_epic_epic() {
        let (_tmp, project) = setup_project();
        let e1 = create_epic(&project, "First");
        let e2 = create_epic(&project, "Second");

        let result = project.link_items(&e1, &e2, LinkRelation::DependsOn, LinkAction::Add);
        assert!(result.is_err());
    }

    #[test]
    fn test_link_epic_on_task() {
        let (_tmp, project) = setup_project();
        let t1 = create_task(&project, "A task");
        let e1 = create_epic(&project, "An epic");

        project
            .link_items(&t1, &e1, LinkRelation::Epic, LinkAction::Add)
            .unwrap();

        let doc: MarkplaneDocument<Task> = project.read_item(&t1).unwrap();
        assert_eq!(doc.frontmatter.epic, Some(e1));
    }

    #[test]
    fn test_link_plan() {
        let (_tmp, project) = setup_project();
        let t1 = create_task(&project, "Task");
        let p1 = create_plan(&project, "Plan", &t1);

        project
            .link_items(&t1, &p1, LinkRelation::Plan, LinkAction::Add)
            .unwrap();

        let task_doc: MarkplaneDocument<Task> = project.read_item(&t1).unwrap();
        let plan_doc: MarkplaneDocument<Plan> = project.read_item(&p1).unwrap();
        assert_eq!(task_doc.frontmatter.plan, Some(p1.clone()));
        assert!(plan_doc.frontmatter.implements.contains(&t1));
    }

    #[test]
    fn test_link_plan_replacement() {
        let (_tmp, project) = setup_project();
        let t1 = create_task(&project, "Task");
        let p1 = create_plan(&project, "Old plan", &t1);
        let p2 = create_plan(&project, "New plan", &t1);

        // Link to first plan
        project
            .link_items(&t1, &p1, LinkRelation::Plan, LinkAction::Add)
            .unwrap();
        // Switch to second plan
        project
            .link_items(&t1, &p2, LinkRelation::Plan, LinkAction::Add)
            .unwrap();

        let task_doc: MarkplaneDocument<Task> = project.read_item(&t1).unwrap();
        let old_plan: MarkplaneDocument<Plan> = project.read_item(&p1).unwrap();
        let new_plan: MarkplaneDocument<Plan> = project.read_item(&p2).unwrap();

        assert_eq!(task_doc.frontmatter.plan, Some(p2.clone()));
        assert!(!old_plan.frontmatter.implements.contains(&t1));
        assert!(new_plan.frontmatter.implements.contains(&t1));
    }

    #[test]
    fn test_link_implements() {
        let (_tmp, project) = setup_project();
        let t1 = create_task(&project, "Task");
        let p1 = create_plan(&project, "Plan", &t1);

        project
            .link_items(&p1, &t1, LinkRelation::Implements, LinkAction::Add)
            .unwrap();

        let plan_doc: MarkplaneDocument<Plan> = project.read_item(&p1).unwrap();
        let task_doc: MarkplaneDocument<Task> = project.read_item(&t1).unwrap();
        assert!(plan_doc.frontmatter.implements.contains(&t1));
        assert_eq!(task_doc.frontmatter.plan, Some(p1.clone()));
    }

    #[test]
    fn test_link_related() {
        let (_tmp, project) = setup_project();
        let n1 = create_note(&project, "Research");
        let t1 = create_task(&project, "Task");

        project
            .link_items(&n1, &t1, LinkRelation::Related, LinkAction::Add)
            .unwrap();

        // Bidirectional: both sides should have the link
        let note_doc: MarkplaneDocument<Note> = project.read_item(&n1).unwrap();
        let task_doc: MarkplaneDocument<Task> = project.read_item(&t1).unwrap();
        assert!(note_doc.frontmatter.related.contains(&t1));
        assert!(task_doc.frontmatter.related.contains(&n1));
    }

    #[test]
    fn test_unlink_blocks() {
        let (_tmp, project) = setup_project();
        let t1 = create_task(&project, "A");
        let t2 = create_task(&project, "B");

        project
            .link_items(&t1, &t2, LinkRelation::Blocks, LinkAction::Add)
            .unwrap();
        project
            .link_items(&t1, &t2, LinkRelation::Blocks, LinkAction::Remove)
            .unwrap();

        let doc1: MarkplaneDocument<Task> = project.read_item(&t1).unwrap();
        let doc2: MarkplaneDocument<Task> = project.read_item(&t2).unwrap();
        assert!(!doc1.frontmatter.blocks.contains(&t2));
        assert!(!doc2.frontmatter.depends_on.contains(&t1));
    }

    #[test]
    fn test_unlink_plan() {
        let (_tmp, project) = setup_project();
        let t1 = create_task(&project, "Task");
        let p1 = create_plan(&project, "Plan", &t1);

        project
            .link_items(&t1, &p1, LinkRelation::Plan, LinkAction::Add)
            .unwrap();
        project
            .link_items(&t1, &p1, LinkRelation::Plan, LinkAction::Remove)
            .unwrap();

        let task_doc: MarkplaneDocument<Task> = project.read_item(&t1).unwrap();
        let plan_doc: MarkplaneDocument<Plan> = project.read_item(&p1).unwrap();
        assert_eq!(task_doc.frontmatter.plan, None);
        assert!(!plan_doc.frontmatter.implements.contains(&t1));
    }

    #[test]
    fn test_link_self_reference_fails() {
        let (_tmp, project) = setup_project();
        let t1 = create_task(&project, "Solo");

        let result = project.link_items(&t1, &t1, LinkRelation::Blocks, LinkAction::Add);
        assert!(result.is_err());
    }

    #[test]
    fn test_link_invalid_source_type() {
        let (_tmp, project) = setup_project();
        let e1 = create_epic(&project, "Epic");
        let t1 = create_task(&project, "Task");

        // Epic cannot be source of Blocks
        let result = project.link_items(&e1, &t1, LinkRelation::Blocks, LinkAction::Add);
        assert!(result.is_err());
    }

    #[test]
    fn test_link_idempotent() {
        let (_tmp, project) = setup_project();
        let t1 = create_task(&project, "A");
        let t2 = create_task(&project, "B");

        project
            .link_items(&t1, &t2, LinkRelation::Blocks, LinkAction::Add)
            .unwrap();
        // Adding again should be idempotent
        project
            .link_items(&t1, &t2, LinkRelation::Blocks, LinkAction::Add)
            .unwrap();

        let doc1: MarkplaneDocument<Task> = project.read_item(&t1).unwrap();
        assert_eq!(
            doc1.frontmatter.blocks.iter().filter(|b| *b == &t2).count(),
            1
        );
    }

    #[test]
    fn test_unlink_depends_on_task_task() {
        let (_tmp, project) = setup_project();
        let t1 = create_task(&project, "A");
        let t2 = create_task(&project, "B");

        project
            .link_items(&t1, &t2, LinkRelation::DependsOn, LinkAction::Add)
            .unwrap();
        project
            .link_items(&t1, &t2, LinkRelation::DependsOn, LinkAction::Remove)
            .unwrap();

        let doc1: MarkplaneDocument<Task> = project.read_item(&t1).unwrap();
        let doc2: MarkplaneDocument<Task> = project.read_item(&t2).unwrap();
        assert!(!doc1.frontmatter.depends_on.contains(&t2));
        assert!(!doc2.frontmatter.blocks.contains(&t1));
    }

    #[test]
    fn test_depends_on_task_task_has_reciprocal() {
        let (_tmp, project) = setup_project();
        let t1 = create_task(&project, "Dependent");
        let t2 = create_task(&project, "Dependency");

        project
            .link_items(&t1, &t2, LinkRelation::DependsOn, LinkAction::Add)
            .unwrap();

        let doc1: MarkplaneDocument<Task> = project.read_item(&t1).unwrap();
        let doc2: MarkplaneDocument<Task> = project.read_item(&t2).unwrap();
        assert!(doc1.frontmatter.depends_on.contains(&t2));
        assert!(doc2.frontmatter.blocks.contains(&t1));
    }

    #[test]
    fn test_unlink_epic_on_task() {
        let (_tmp, project) = setup_project();
        let t1 = create_task(&project, "A task");
        let e1 = create_epic(&project, "An epic");

        project
            .link_items(&t1, &e1, LinkRelation::Epic, LinkAction::Add)
            .unwrap();
        project
            .link_items(&t1, &e1, LinkRelation::Epic, LinkAction::Remove)
            .unwrap();

        let doc: MarkplaneDocument<Task> = project.read_item(&t1).unwrap();
        assert_eq!(doc.frontmatter.epic, None);
    }

    #[test]
    fn test_unlink_implements() {
        let (_tmp, project) = setup_project();
        let t1 = create_task(&project, "Task");
        let p1 = create_plan(&project, "Plan", &t1);

        project
            .link_items(&p1, &t1, LinkRelation::Implements, LinkAction::Add)
            .unwrap();
        project
            .link_items(&p1, &t1, LinkRelation::Implements, LinkAction::Remove)
            .unwrap();

        let plan_doc: MarkplaneDocument<Plan> = project.read_item(&p1).unwrap();
        let task_doc: MarkplaneDocument<Task> = project.read_item(&t1).unwrap();
        assert!(!plan_doc.frontmatter.implements.contains(&t1));
        assert_eq!(task_doc.frontmatter.plan, None);
    }

    #[test]
    fn test_unlink_related() {
        let (_tmp, project) = setup_project();
        let n1 = create_note(&project, "Research");
        let t1 = create_task(&project, "Task");

        project
            .link_items(&n1, &t1, LinkRelation::Related, LinkAction::Add)
            .unwrap();
        project
            .link_items(&n1, &t1, LinkRelation::Related, LinkAction::Remove)
            .unwrap();

        // Bidirectional: both sides should be cleared
        let note_doc: MarkplaneDocument<Note> = project.read_item(&n1).unwrap();
        let task_doc: MarkplaneDocument<Task> = project.read_item(&t1).unwrap();
        assert!(!note_doc.frontmatter.related.contains(&t1));
        assert!(!task_doc.frontmatter.related.contains(&n1));
    }

    #[test]
    fn test_unlink_nonexistent_is_noop() {
        let (_tmp, project) = setup_project();
        let t1 = create_task(&project, "A");
        let t2 = create_task(&project, "B");

        // Removing a link that doesn't exist should be a silent no-op
        project
            .link_items(&t1, &t2, LinkRelation::Blocks, LinkAction::Remove)
            .unwrap();

        let doc1: MarkplaneDocument<Task> = project.read_item(&t1).unwrap();
        assert!(doc1.frontmatter.blocks.is_empty());
    }

    #[test]
    fn test_link_invalid_target_type() {
        let (_tmp, project) = setup_project();
        let t1 = create_task(&project, "Task");
        let e1 = create_epic(&project, "Epic");

        // Task cannot block an Epic (Blocks requires Task→Task)
        let result = project.link_items(&t1, &e1, LinkRelation::Blocks, LinkAction::Add);
        assert!(result.is_err());
    }

    #[test]
    fn test_implements_plan_replacement() {
        let (_tmp, project) = setup_project();
        let t1 = create_task(&project, "Task");
        let p1 = create_plan(&project, "Old plan", &t1);
        let p2 = create_plan(&project, "New plan", &t1);

        // Link via first plan
        project
            .link_items(&p1, &t1, LinkRelation::Implements, LinkAction::Add)
            .unwrap();
        // Switch via second plan — should clean up first
        project
            .link_items(&p2, &t1, LinkRelation::Implements, LinkAction::Add)
            .unwrap();

        let task_doc: MarkplaneDocument<Task> = project.read_item(&t1).unwrap();
        let old_plan: MarkplaneDocument<Plan> = project.read_item(&p1).unwrap();
        let new_plan: MarkplaneDocument<Plan> = project.read_item(&p2).unwrap();

        assert_eq!(task_doc.frontmatter.plan, Some(p2.clone()));
        assert!(!old_plan.frontmatter.implements.contains(&t1));
        assert!(new_plan.frontmatter.implements.contains(&t1));
    }

    #[test]
    fn test_link_related_task_task() {
        let (_tmp, project) = setup_project();
        let t1 = create_task(&project, "Task A");
        let t2 = create_task(&project, "Task B");

        project
            .link_items(&t1, &t2, LinkRelation::Related, LinkAction::Add)
            .unwrap();

        let doc1: MarkplaneDocument<Task> = project.read_item(&t1).unwrap();
        let doc2: MarkplaneDocument<Task> = project.read_item(&t2).unwrap();
        assert!(doc1.frontmatter.related.contains(&t2));
        assert!(doc2.frontmatter.related.contains(&t1));
    }

    #[test]
    fn test_link_related_task_epic() {
        let (_tmp, project) = setup_project();
        let t1 = create_task(&project, "Task");
        let e1 = create_epic(&project, "Epic");

        project
            .link_items(&t1, &e1, LinkRelation::Related, LinkAction::Add)
            .unwrap();

        let task_doc: MarkplaneDocument<Task> = project.read_item(&t1).unwrap();
        let epic_doc: MarkplaneDocument<Epic> = project.read_item(&e1).unwrap();
        assert!(task_doc.frontmatter.related.contains(&e1));
        assert!(epic_doc.frontmatter.related.contains(&t1));
    }

    #[test]
    fn test_link_related_epic_plan() {
        let (_tmp, project) = setup_project();
        let e1 = create_epic(&project, "Epic");
        let t1 = create_task(&project, "Task");
        let p1 = create_plan(&project, "Plan", &t1);

        project
            .link_items(&e1, &p1, LinkRelation::Related, LinkAction::Add)
            .unwrap();

        let epic_doc: MarkplaneDocument<Epic> = project.read_item(&e1).unwrap();
        let plan_doc: MarkplaneDocument<Plan> = project.read_item(&p1).unwrap();
        assert!(epic_doc.frontmatter.related.contains(&p1));
        assert!(plan_doc.frontmatter.related.contains(&e1));
    }

    #[test]
    fn test_link_related_idempotent() {
        let (_tmp, project) = setup_project();
        let t1 = create_task(&project, "Task");
        let e1 = create_epic(&project, "Epic");

        project
            .link_items(&t1, &e1, LinkRelation::Related, LinkAction::Add)
            .unwrap();
        // Adding again should be idempotent
        project
            .link_items(&t1, &e1, LinkRelation::Related, LinkAction::Add)
            .unwrap();

        let task_doc: MarkplaneDocument<Task> = project.read_item(&t1).unwrap();
        let epic_doc: MarkplaneDocument<Epic> = project.read_item(&e1).unwrap();
        assert_eq!(
            task_doc.frontmatter.related.iter().filter(|r| *r == &e1).count(),
            1
        );
        assert_eq!(
            epic_doc.frontmatter.related.iter().filter(|r| *r == &t1).count(),
            1
        );
    }
}
