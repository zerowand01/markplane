use std::fs;

use crate::error::Result;
use crate::frontmatter::parse_frontmatter;
use crate::models::*;
use crate::project::Project;

/// Filter criteria for querying items.
#[derive(Debug, Default)]
pub struct QueryFilter {
    pub status: Option<Vec<String>>,
    pub priority: Option<Vec<String>>,
    pub epic: Option<String>,
    pub tags: Option<Vec<String>>,
    pub assignee: Option<String>,
    pub item_type: Option<Vec<String>>,
}

impl Project {
    /// List backlog items, optionally filtered.
    /// Results are sorted by priority (critical first), then by ID.
    pub fn list_backlog_items(
        &self,
        filter: &QueryFilter,
    ) -> Result<Vec<MarkplaneDocument<BacklogItem>>> {
        let dir = self.item_dir(&IdPrefix::Back);
        let mut items = scan_directory::<BacklogItem>(&dir)?;

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

        // Sort by priority (critical first), then by ID
        items.sort_by(|a, b| {
            let pa = priority_rank(&a.frontmatter.priority);
            let pb = priority_rank(&b.frontmatter.priority);
            pa.cmp(&pb).then_with(|| a.frontmatter.id.cmp(&b.frontmatter.id))
        });

        Ok(items)
    }

    /// List all epics.
    pub fn list_epics(&self) -> Result<Vec<MarkplaneDocument<Epic>>> {
        let dir = self.item_dir(&IdPrefix::Epic);
        let mut items = scan_directory::<Epic>(&dir)?;
        items.sort_by(|a, b| {
            let pa = priority_rank(&a.frontmatter.priority);
            let pb = priority_rank(&b.frontmatter.priority);
            pa.cmp(&pb).then_with(|| a.frontmatter.id.cmp(&b.frontmatter.id))
        });
        Ok(items)
    }

    /// List all plans.
    pub fn list_plans(&self) -> Result<Vec<MarkplaneDocument<Plan>>> {
        let dir = self.item_dir(&IdPrefix::Plan);
        let mut items = scan_directory::<Plan>(&dir)?;
        items.sort_by(|a, b| a.frontmatter.id.cmp(&b.frontmatter.id));
        Ok(items)
    }

    /// List all notes.
    pub fn list_notes(&self) -> Result<Vec<MarkplaneDocument<Note>>> {
        let dir = self.item_dir(&IdPrefix::Note);
        let mut items = scan_directory::<Note>(&dir)?;
        items.sort_by(|a, b| a.frontmatter.id.cmp(&b.frontmatter.id));
        Ok(items)
    }
}

/// Scan a directory for `.md` files, parse each one, and return the parsed documents.
/// Prefers items/ subdirectory if it exists, falls back to flat layout.
/// Skips INDEX.md, ideas.md, decisions.md, and any files that fail to parse.
fn scan_directory<T: serde::de::DeserializeOwned>(
    dir: &std::path::Path,
) -> Result<Vec<MarkplaneDocument<T>>> {
    let mut results = Vec::new();

    let items_dir = dir.join("items");
    let scan_dir = if items_dir.is_dir() { &items_dir } else { dir };

    if !scan_dir.exists() {
        return Ok(results);
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
    fn test_list_backlog_items_empty() {
        let (_tmp, project) = setup_project();
        let items = project.list_backlog_items(&QueryFilter::default()).unwrap();
        assert!(items.is_empty());
    }

    #[test]
    fn test_list_backlog_items_with_items() {
        let (_tmp, project) = setup_project();

        project
            .create_backlog_item(
                "Low item",
                ItemType::Chore,
                Priority::Low,
                Effort::Xs,
                None,
                vec![],
            )
            .unwrap();
        project
            .create_backlog_item(
                "High item",
                ItemType::Feature,
                Priority::High,
                Effort::Medium,
                None,
                vec![],
            )
            .unwrap();
        project
            .create_backlog_item(
                "Critical item",
                ItemType::Bug,
                Priority::Critical,
                Effort::Small,
                None,
                vec![],
            )
            .unwrap();

        let items = project.list_backlog_items(&QueryFilter::default()).unwrap();
        assert_eq!(items.len(), 3);

        // Should be sorted: critical, high, low
        assert_eq!(items[0].frontmatter.priority, Priority::Critical);
        assert_eq!(items[1].frontmatter.priority, Priority::High);
        assert_eq!(items[2].frontmatter.priority, Priority::Low);
    }

    #[test]
    fn test_list_backlog_items_filter_status() {
        let (_tmp, project) = setup_project();

        project
            .create_backlog_item(
                "Draft item",
                ItemType::Feature,
                Priority::Medium,
                Effort::Small,
                None,
                vec![],
            )
            .unwrap();
        project
            .create_backlog_item(
                "Another draft",
                ItemType::Bug,
                Priority::High,
                Effort::Medium,
                None,
                vec![],
            )
            .unwrap();

        // Change one to in-progress
        project.update_status("BACK-001", "in-progress").unwrap();

        let filter = QueryFilter {
            status: Some(vec!["draft".to_string()]),
            ..Default::default()
        };
        let items = project.list_backlog_items(&filter).unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].frontmatter.id, "BACK-002");
    }

    #[test]
    fn test_list_backlog_items_filter_priority() {
        let (_tmp, project) = setup_project();

        project
            .create_backlog_item(
                "High",
                ItemType::Feature,
                Priority::High,
                Effort::Small,
                None,
                vec![],
            )
            .unwrap();
        project
            .create_backlog_item(
                "Low",
                ItemType::Feature,
                Priority::Low,
                Effort::Small,
                None,
                vec![],
            )
            .unwrap();

        let filter = QueryFilter {
            priority: Some(vec!["high".to_string()]),
            ..Default::default()
        };
        let items = project.list_backlog_items(&filter).unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].frontmatter.title, "High");
    }

    #[test]
    fn test_list_backlog_items_filter_tags() {
        let (_tmp, project) = setup_project();

        project
            .create_backlog_item(
                "UI item",
                ItemType::Feature,
                Priority::Medium,
                Effort::Small,
                None,
                vec!["ui".to_string(), "frontend".to_string()],
            )
            .unwrap();
        project
            .create_backlog_item(
                "API item",
                ItemType::Feature,
                Priority::Medium,
                Effort::Small,
                None,
                vec!["api".to_string()],
            )
            .unwrap();

        let filter = QueryFilter {
            tags: Some(vec!["ui".to_string()]),
            ..Default::default()
        };
        let items = project.list_backlog_items(&filter).unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].frontmatter.title, "UI item");
    }

    #[test]
    fn test_list_epics() {
        let (_tmp, project) = setup_project();

        project.create_epic("Phase 1", Priority::High).unwrap();
        project.create_epic("Phase 2", Priority::Medium).unwrap();

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
            .create_note("Research A", NoteType::Research, vec![])
            .unwrap();
        project
            .create_note("Analysis B", NoteType::Analysis, vec![])
            .unwrap();

        let notes = project.list_notes().unwrap();
        assert_eq!(notes.len(), 2);
    }
}
