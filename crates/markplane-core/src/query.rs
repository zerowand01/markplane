use std::fs;

use crate::error::Result;
use crate::frontmatter::parse_frontmatter;
use crate::models::*;
use crate::project::Project;

/// Controls which directories are scanned for items.
#[derive(Debug, Clone, Copy, Default)]
pub enum ScanScope {
    /// Scan items/ only (current default behavior).
    #[default]
    Active,
    /// Scan archive/ only.
    Archived,
    /// Scan both items/ and archive/.
    All,
}

/// Filter criteria for querying items.
#[derive(Debug, Default)]
pub struct QueryFilter {
    pub status: Option<Vec<String>>,
    pub priority: Option<Vec<String>>,
    pub epic: Option<String>,
    pub tags: Option<Vec<String>>,
    pub assignee: Option<String>,
    pub item_type: Option<Vec<String>>,
    /// Which directories to scan: Active (items/), Archived (archive/), or All (both).
    pub scope: ScanScope,
}

impl Project {
    /// List tasks, optionally filtered.
    /// Results are sorted by priority (critical first), then by ID.
    pub fn list_tasks(
        &self,
        filter: &QueryFilter,
    ) -> Result<Vec<MarkplaneDocument<Task>>> {
        let dir = self.item_dir(&IdPrefix::Task);
        let mut items = scan_directory::<Task>(&dir, filter.scope)?;

        items.retain(|doc| {
            let fm = &doc.frontmatter;

            if let Some(ref statuses) = filter.status
                && !statuses.contains(&fm.status.to_string()) {
                    return false;
                }

            if let Some(ref priorities) = filter.priority
                && !priorities.contains(&fm.priority.to_string()) {
                    return false;
                }

            if let Some(ref epic) = filter.epic {
                match &fm.epic {
                    Some(e) if e == epic => {}
                    _ => return false,
                }
            }

            if let Some(ref tags) = filter.tags
                && !tags.iter().any(|t| fm.tags.contains(t)) {
                    return false;
                }

            if let Some(ref assignee) = filter.assignee {
                match &fm.assignee {
                    Some(a) if a == assignee => {}
                    _ => return false,
                }
            }

            if let Some(ref types) = filter.item_type
                && !types.contains(&fm.item_type.to_string()) {
                    return false;
                }

            true
        });

        // Sort by priority (critical first), then position, then updated (newest first), then ID
        items.sort_by(|a, b| {
            let pa = priority_rank(&a.frontmatter.priority);
            let pb = priority_rank(&b.frontmatter.priority);
            pa.cmp(&pb)
                .then_with(|| {
                    match (&a.frontmatter.position, &b.frontmatter.position) {
                        (Some(ap), Some(bp)) => ap.cmp(bp),
                        (Some(_), None) => std::cmp::Ordering::Less,
                        (None, Some(_)) => std::cmp::Ordering::Greater,
                        (None, None) => std::cmp::Ordering::Equal,
                    }
                })
                .then_with(|| b.frontmatter.updated.cmp(&a.frontmatter.updated))
                .then_with(|| a.frontmatter.id.cmp(&b.frontmatter.id))
        });

        Ok(items)
    }

    /// List all epics. Pass `archived: true` to list archived epics only.
    pub fn list_epics_filtered(&self, archived: bool) -> Result<Vec<MarkplaneDocument<Epic>>> {
        let dir = self.item_dir(&IdPrefix::Epic);
        let scope = if archived { ScanScope::Archived } else { ScanScope::Active };
        let mut items = scan_directory::<Epic>(&dir, scope)?;
        items.sort_by(|a, b| {
            let pa = priority_rank(&a.frontmatter.priority);
            let pb = priority_rank(&b.frontmatter.priority);
            pa.cmp(&pb).then_with(|| a.frontmatter.id.cmp(&b.frontmatter.id))
        });
        Ok(items)
    }

    /// List all active epics (convenience wrapper).
    pub fn list_epics(&self) -> Result<Vec<MarkplaneDocument<Epic>>> {
        self.list_epics_filtered(false)
    }

    /// List all plans. Pass `archived: true` to list archived plans only.
    pub fn list_plans_filtered(&self, archived: bool) -> Result<Vec<MarkplaneDocument<Plan>>> {
        let dir = self.item_dir(&IdPrefix::Plan);
        let scope = if archived { ScanScope::Archived } else { ScanScope::Active };
        let mut items = scan_directory::<Plan>(&dir, scope)?;
        items.sort_by(|a, b| a.frontmatter.id.cmp(&b.frontmatter.id));
        Ok(items)
    }

    /// List all active plans (convenience wrapper).
    pub fn list_plans(&self) -> Result<Vec<MarkplaneDocument<Plan>>> {
        self.list_plans_filtered(false)
    }

    /// List all notes. Pass `archived: true` to list archived notes only.
    pub fn list_notes_filtered(&self, archived: bool) -> Result<Vec<MarkplaneDocument<Note>>> {
        let dir = self.item_dir(&IdPrefix::Note);
        let scope = if archived { ScanScope::Archived } else { ScanScope::Active };
        let mut items = scan_directory::<Note>(&dir, scope)?;
        items.sort_by(|a, b| a.frontmatter.id.cmp(&b.frontmatter.id));
        Ok(items)
    }

    /// List all active notes (convenience wrapper).
    pub fn list_notes(&self) -> Result<Vec<MarkplaneDocument<Note>>> {
        self.list_notes_filtered(false)
    }
}

/// Scan a single directory for `.md` files, parse each one, and append to `results`.
/// Skips INDEX.md, ideas.md, decisions.md, and any files that fail to parse.
fn scan_dir_entries<T: serde::de::DeserializeOwned>(
    scan_dir: &std::path::Path,
    results: &mut Vec<MarkplaneDocument<T>>,
) {
    if !scan_dir.exists() {
        return;
    }

    let pattern = scan_dir.join("*.md").to_string_lossy().to_string();
    for path in glob::glob(&pattern).unwrap_or_else(|_| glob::glob("").unwrap()).flatten() {
        let filename = path.file_name().unwrap_or_default().to_string_lossy();
        if filename == "INDEX.md" || filename == "ideas.md" || filename == "decisions.md" {
            continue;
        }

        let content = match fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        match parse_frontmatter::<T>(&content) {
            Ok(doc) => results.push(doc),
            Err(_) => continue,
        }
    }
}

/// Scan a directory for `.md` files based on the given scope.
/// `Active` scans items/, `Archived` scans archive/, `All` scans both.
fn scan_directory<T: serde::de::DeserializeOwned>(
    dir: &std::path::Path,
    scope: ScanScope,
) -> Result<Vec<MarkplaneDocument<T>>> {
    let mut results = Vec::new();

    let items_dir = dir.join("items");
    let archive_dir = dir.join("archive");

    match scope {
        ScanScope::Active => {
            scan_dir_entries(&items_dir, &mut results);
        }
        ScanScope::Archived => {
            scan_dir_entries(&archive_dir, &mut results);
        }
        ScanScope::All => {
            scan_dir_entries(&items_dir, &mut results);
            scan_dir_entries(&archive_dir, &mut results);
        }
    }

    Ok(results)
}

/// Return a numeric rank for a priority (lower = higher priority).
fn priority_rank(priority: &Priority) -> u8 {
    match priority {
        Priority::Critical => 0,
        Priority::High => 1,
        Priority::Medium => 2,
        Priority::Low => 3,
        Priority::Someday => 4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_project() -> (TempDir, Project) {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path().join(".markplane");
        let project = Project::init(root, "Test", "Test project").unwrap();
        (tmp, project)
    }

    #[test]
    fn test_list_tasks_empty() {
        let (_tmp, project) = setup_project();
        let items = project.list_tasks(&QueryFilter::default()).unwrap();
        assert!(items.is_empty());
    }

    #[test]
    fn test_list_tasks_with_items() {
        let (_tmp, project) = setup_project();

        project
            .create_task(
                "Low item",
                "chore",
                Priority::Low,
                Effort::Xs,
                None,
                vec![],
                None,
            )
            .unwrap();
        project
            .create_task(
                "High item",
                "feature",
                Priority::High,
                Effort::Medium,
                None,
                vec![],
                None,
            )
            .unwrap();
        project
            .create_task(
                "Critical item",
                "bug",
                Priority::Critical,
                Effort::Small,
                None,
                vec![],
                None,
            )
            .unwrap();

        let items = project.list_tasks(&QueryFilter::default()).unwrap();
        assert_eq!(items.len(), 3);

        // Should be sorted: critical, high, low
        assert_eq!(items[0].frontmatter.priority, Priority::Critical);
        assert_eq!(items[1].frontmatter.priority, Priority::High);
        assert_eq!(items[2].frontmatter.priority, Priority::Low);
    }

    #[test]
    fn test_list_tasks_filter_status() {
        let (_tmp, project) = setup_project();

        let task1 = project
            .create_task(
                "Draft item",
                "feature",
                Priority::Medium,
                Effort::Small,
                None,
                vec![],
                None,
            )
            .unwrap();
        let task2 = project
            .create_task(
                "Another draft",
                "bug",
                Priority::High,
                Effort::Medium,
                None,
                vec![],
                None,
            )
            .unwrap();

        // Change one to in-progress
        project.update_status(&task1.id, "in-progress").unwrap();

        let filter = QueryFilter {
            status: Some(vec!["draft".to_string()]),
            ..Default::default()
        };
        let items = project.list_tasks(&filter).unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].frontmatter.id, task2.id);
    }

    #[test]
    fn test_list_tasks_filter_priority() {
        let (_tmp, project) = setup_project();

        project
            .create_task(
                "High",
                "feature",
                Priority::High,
                Effort::Small,
                None,
                vec![],
                None,
            )
            .unwrap();
        project
            .create_task(
                "Low",
                "feature",
                Priority::Low,
                Effort::Small,
                None,
                vec![],
                None,
            )
            .unwrap();

        let filter = QueryFilter {
            priority: Some(vec!["high".to_string()]),
            ..Default::default()
        };
        let items = project.list_tasks(&filter).unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].frontmatter.title, "High");
    }

    #[test]
    fn test_list_tasks_filter_tags() {
        let (_tmp, project) = setup_project();

        project
            .create_task(
                "UI item",
                "feature",
                Priority::Medium,
                Effort::Small,
                None,
                vec!["ui".to_string(), "frontend".to_string()],
                None,
            )
            .unwrap();
        project
            .create_task(
                "API item",
                "feature",
                Priority::Medium,
                Effort::Small,
                None,
                vec!["api".to_string()],
                None,
            )
            .unwrap();

        let filter = QueryFilter {
            tags: Some(vec!["ui".to_string()]),
            ..Default::default()
        };
        let items = project.list_tasks(&filter).unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].frontmatter.title, "UI item");
    }

    #[test]
    fn test_list_epics() {
        let (_tmp, project) = setup_project();

        project.create_epic("Phase 1", Priority::High, None).unwrap();
        project.create_epic("Phase 2", Priority::Medium, None).unwrap();

        let epics = project.list_epics().unwrap();
        assert_eq!(epics.len(), 2);
        // High priority first
        assert_eq!(epics[0].frontmatter.title, "Phase 1");
    }

    #[test]
    fn test_list_plans() {
        let (_tmp, project) = setup_project();

        project
            .create_plan("Plan A", vec![], None)
            .unwrap();
        project
            .create_plan("Plan B", vec![], None)
            .unwrap();

        let plans = project.list_plans().unwrap();
        assert_eq!(plans.len(), 2);
    }

    #[test]
    fn test_list_notes() {
        let (_tmp, project) = setup_project();

        project
            .create_note("Research A", "research", vec![], None)
            .unwrap();
        project
            .create_note("Analysis B", "analysis", vec![], None)
            .unwrap();

        let notes = project.list_notes().unwrap();
        assert_eq!(notes.len(), 2);
    }

    // ── Archive scan scope tests ─────────────────────────────────────────

    #[test]
    fn test_scan_directory_active_only() {
        let (_tmp, project) = setup_project();

        let task1 = project
            .create_task("Active task", "feature", Priority::Medium, Effort::Small, None, vec![], None)
            .unwrap();
        let task2 = project
            .create_task("To archive", "feature", Priority::Medium, Effort::Small, None, vec![], None)
            .unwrap();

        // Archive one task
        project.archive_item(&task2.id).unwrap();

        // Default (active) should return only active items
        let items = project.list_tasks(&QueryFilter::default()).unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].frontmatter.id, task1.id);
    }

    #[test]
    fn test_scan_directory_archived_only() {
        let (_tmp, project) = setup_project();

        project
            .create_task("Active task", "feature", Priority::Medium, Effort::Small, None, vec![], None)
            .unwrap();
        let task2 = project
            .create_task("To archive", "feature", Priority::Medium, Effort::Small, None, vec![], None)
            .unwrap();

        project.archive_item(&task2.id).unwrap();

        let filter = QueryFilter {
            scope: ScanScope::Archived,
            ..Default::default()
        };
        let items = project.list_tasks(&filter).unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].frontmatter.id, task2.id);
    }

    #[test]
    fn test_list_tasks_archived_filter() {
        let (_tmp, project) = setup_project();

        project
            .create_task("Active", "feature", Priority::High, Effort::Small, None, vec![], None)
            .unwrap();
        let task2 = project
            .create_task("Archived", "bug", Priority::Low, Effort::Medium, None, vec![], None)
            .unwrap();
        let task3 = project
            .create_task("Also archived", "chore", Priority::Medium, Effort::Xs, None, vec![], None)
            .unwrap();

        project.archive_item(&task2.id).unwrap();
        project.archive_item(&task3.id).unwrap();

        // Active: only the first task
        let active = project.list_tasks(&QueryFilter::default()).unwrap();
        assert_eq!(active.len(), 1);

        // Archived: task2 and task3
        let archived = project.list_tasks(&QueryFilter { scope: ScanScope::Archived, ..Default::default() }).unwrap();
        assert_eq!(archived.len(), 2);
    }

    #[test]
    fn test_list_epics_archived() {
        let (_tmp, project) = setup_project();

        let epic1 = project.create_epic("Active epic", Priority::High, None).unwrap();
        let epic2 = project.create_epic("Done epic", Priority::Medium, None).unwrap();

        project.archive_item(&epic2.id).unwrap();

        let active = project.list_epics().unwrap();
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].frontmatter.id, epic1.id);

        let archived = project.list_epics_filtered(true).unwrap();
        assert_eq!(archived.len(), 1);
        assert_eq!(archived[0].frontmatter.id, epic2.id);
    }

    #[test]
    fn test_list_plans_archived() {
        let (_tmp, project) = setup_project();

        project.create_plan("Active plan", vec![], None).unwrap();
        let plan2 = project.create_plan("Done plan", vec![], None).unwrap();

        project.archive_item(&plan2.id).unwrap();

        let active = project.list_plans().unwrap();
        assert_eq!(active.len(), 1);

        let archived = project.list_plans_filtered(true).unwrap();
        assert_eq!(archived.len(), 1);
        assert_eq!(archived[0].frontmatter.id, plan2.id);
    }

    #[test]
    fn test_list_notes_archived() {
        let (_tmp, project) = setup_project();

        project.create_note("Active note", "research", vec![], None).unwrap();
        let note2 = project.create_note("Done note", "idea", vec![], None).unwrap();

        project.archive_item(&note2.id).unwrap();

        let active = project.list_notes().unwrap();
        assert_eq!(active.len(), 1);

        let archived = project.list_notes_filtered(true).unwrap();
        assert_eq!(archived.len(), 1);
        assert_eq!(archived[0].frontmatter.id, note2.id);
    }
}
