use std::fs;
use std::path::{Path, PathBuf};

use chrono::Local;
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::error::{MarkplaneError, Result};
use crate::frontmatter::{parse_frontmatter, write_frontmatter};
use crate::models::*;
use crate::templates::{self, render_template};

/// Represents a `.markplane/` project directory.
pub struct Project {
    /// Path to the `.markplane/` directory.
    root: PathBuf,
}

impl Project {
    /// Create a Project from the path to a `.markplane/` directory.
    pub fn new(root: PathBuf) -> Self {
        Project { root }
    }

    /// Find `.markplane/` by walking up from the current working directory.
    pub fn from_current_dir() -> Result<Self> {
        let mut dir = std::env::current_dir().map_err(MarkplaneError::Io)?;
        loop {
            let candidate = dir.join(".markplane");
            if candidate.is_dir() && candidate.join("config.yaml").is_file() {
                return Ok(Project::new(candidate));
            }
            if !dir.pop() {
                return Err(MarkplaneError::NotInitialized(
                    "No .markplane/ directory found in current or parent directories".into(),
                ));
            }
        }
    }

    /// Get the root path of the `.markplane/` directory.
    pub fn root(&self) -> &Path {
        &self.root
    }

    // ── Config ────────────────────────────────────────────────────────────

    /// Read and parse `config.yaml`.
    pub fn load_config(&self) -> Result<Config> {
        let path = self.root.join("config.yaml");
        let content = fs::read_to_string(&path).map_err(|e| {
            MarkplaneError::NotInitialized(format!("Cannot read config.yaml: {}", e))
        })?;
        let config: Config = serde_yaml::from_str(&content)?;
        Ok(config)
    }

    /// Write `config.yaml`.
    pub fn save_config(&self, config: &Config) -> Result<()> {
        let path = self.root.join("config.yaml");
        let yaml = serde_yaml::to_string(config)?;
        fs::write(&path, yaml)?;
        Ok(())
    }

    // ── ID Management ─────────────────────────────────────────────────────

    /// Get the next ID for a given prefix and increment the counter in config.
    pub fn next_id(&self, prefix: &IdPrefix) -> Result<String> {
        let mut config = self.load_config()?;
        let key = prefix.as_str().to_string();
        let counter = config.counters.entry(key).or_insert(0);
        *counter += 1;
        let id = format_id(prefix, *counter);
        self.save_config(&config)?;
        Ok(id)
    }

    /// Resolve an item ID to its file path.
    /// Checks active directory first, then archive/.
    pub fn item_path(&self, id: &str) -> Result<PathBuf> {
        let (prefix, _) = parse_id(id)?;
        let dir = self.item_dir(&prefix);

        // Check active directory
        let active_path = dir.join(format!("{}.md", id));
        if active_path.is_file() {
            return Ok(active_path);
        }

        // Check archive directory
        let archive_path = dir.join("archive").join(format!("{}.md", id));
        if archive_path.is_file() {
            return Ok(archive_path);
        }

        Err(MarkplaneError::NotFound(format!(
            "Item {} not found in {} or its archive",
            id,
            dir.display()
        )))
    }

    /// Get the directory for a given prefix type.
    pub fn item_dir(&self, prefix: &IdPrefix) -> PathBuf {
        self.root.join(prefix.directory())
    }

    // ── CRUD Operations ───────────────────────────────────────────────────

    /// Create a new backlog item.
    pub fn create_backlog_item(
        &self,
        title: &str,
        item_type: ItemType,
        priority: Priority,
        effort: Effort,
        epic: Option<String>,
        tags: Vec<String>,
    ) -> Result<BacklogItem> {
        let id = self.next_id(&IdPrefix::Back)?;
        let today = Local::now().date_naive();
        let date_str = today.format("%Y-%m-%d").to_string();

        let tags_yaml = format_yaml_list(&tags);
        let epic_yaml = epic.as_deref().unwrap_or("null");

        let content = render_template(
            templates::BACKLOG_TEMPLATE,
            &[
                ("{ID}", &id),
                ("{TITLE}", title),
                ("{STATUS}", "draft"),
                ("{PRIORITY}", &priority.to_string()),
                ("{TYPE}", &item_type.to_string()),
                ("{EFFORT}", &effort.to_string()),
                ("{TAGS}", &tags_yaml),
                ("{EPIC}", epic_yaml),
                ("{DATE}", &date_str),
            ],
        );

        let dir = self.item_dir(&IdPrefix::Back);
        fs::create_dir_all(&dir)?;
        let path = dir.join(format!("{}.md", id));
        fs::write(&path, &content)?;

        let item = BacklogItem {
            id,
            title: title.to_string(),
            status: BacklogStatus::Draft,
            priority,
            item_type,
            effort,
            tags,
            epic,
            plan: None,
            depends_on: vec![],
            blocks: vec![],
            assignee: None,
            created: today,
            updated: today,
        };

        Ok(item)
    }

    /// Create a new epic.
    pub fn create_epic(&self, title: &str, priority: Priority) -> Result<Epic> {
        let id = self.next_id(&IdPrefix::Epic)?;

        let content = render_template(
            templates::EPIC_TEMPLATE,
            &[
                ("{ID}", &id),
                ("{TITLE}", title),
                ("{PRIORITY}", &priority.to_string()),
            ],
        );

        let dir = self.item_dir(&IdPrefix::Epic);
        fs::create_dir_all(&dir)?;
        let path = dir.join(format!("{}.md", id));
        fs::write(&path, &content)?;

        let epic = Epic {
            id,
            title: title.to_string(),
            status: EpicStatus::Planned,
            priority,
            started: None,
            target: None,
            tags: vec![],
            depends_on: vec![],
        };

        Ok(epic)
    }

    /// Create a new plan.
    pub fn create_plan(
        &self,
        title: &str,
        implements: Vec<String>,
        epic: Option<String>,
    ) -> Result<Plan> {
        let id = self.next_id(&IdPrefix::Plan)?;
        let today = Local::now().date_naive();
        let date_str = today.format("%Y-%m-%d").to_string();

        let implements_yaml = format_yaml_list(&implements);
        let epic_yaml = epic.as_deref().unwrap_or("null");

        let content = render_template(
            templates::PLAN_IMPLEMENTATION_TEMPLATE,
            &[
                ("{ID}", &id),
                ("{TITLE}", title),
                ("{IMPLEMENTS}", &implements_yaml),
                ("{EPIC}", epic_yaml),
                ("{DATE}", &date_str),
            ],
        );

        let dir = self.item_dir(&IdPrefix::Plan);
        fs::create_dir_all(&dir)?;
        let path = dir.join(format!("{}.md", id));
        fs::write(&path, &content)?;

        let plan = Plan {
            id,
            title: title.to_string(),
            status: PlanStatus::Draft,
            implements,
            epic,
            created: today,
            updated: today,
        };

        Ok(plan)
    }

    /// Create a new note.
    pub fn create_note(
        &self,
        title: &str,
        note_type: NoteType,
        tags: Vec<String>,
    ) -> Result<Note> {
        let id = self.next_id(&IdPrefix::Note)?;
        let today = Local::now().date_naive();
        let date_str = today.format("%Y-%m-%d").to_string();

        let tags_yaml = format_yaml_list(&tags);
        let type_str = note_type.to_string();

        let template = match note_type {
            NoteType::Research => templates::NOTE_RESEARCH_TEMPLATE,
            NoteType::Analysis => templates::NOTE_ANALYSIS_TEMPLATE,
            _ => templates::NOTE_GENERIC_TEMPLATE,
        };

        let content = render_template(
            template,
            &[
                ("{ID}", &id),
                ("{TITLE}", title),
                ("{TYPE}", &type_str),
                ("{TAGS}", &tags_yaml),
                ("{RELATED}", "[]"),
                ("{DATE}", &date_str),
            ],
        );

        let dir = self.item_dir(&IdPrefix::Note);
        fs::create_dir_all(&dir)?;
        let path = dir.join(format!("{}.md", id));
        fs::write(&path, &content)?;

        let note = Note {
            id,
            title: title.to_string(),
            note_type,
            status: NoteStatus::Draft,
            tags,
            related: vec![],
            created: today,
            updated: today,
        };

        Ok(note)
    }

    /// Read any item by ID, deserializing the frontmatter into type `T`.
    pub fn read_item<T: DeserializeOwned>(&self, id: &str) -> Result<MarkplaneDocument<T>> {
        let path = self.item_path(id)?;
        let content = fs::read_to_string(&path)?;
        parse_frontmatter(&content)
    }

    /// Write any item by ID, serializing the frontmatter from type `T`.
    pub fn write_item<T: Serialize>(&self, id: &str, doc: &MarkplaneDocument<T>) -> Result<()> {
        let path = self.item_path(id)?;
        let content = write_frontmatter(doc)?;
        fs::write(&path, content)?;
        Ok(())
    }

    /// Update the status field of any item (auto-detects type from ID prefix).
    pub fn update_status(&self, id: &str, new_status: &str) -> Result<()> {
        let (prefix, _) = parse_id(id)?;
        let today = Local::now().date_naive();

        match prefix {
            IdPrefix::Back => {
                let mut doc: MarkplaneDocument<BacklogItem> = self.read_item(id)?;
                doc.frontmatter.status = new_status.parse()?;
                doc.frontmatter.updated = today;
                self.write_item(id, &doc)?;
            }
            IdPrefix::Epic => {
                let mut doc: MarkplaneDocument<Epic> = self.read_item(id)?;
                doc.frontmatter.status = new_status.parse()?;
                self.write_item(id, &doc)?;
            }
            IdPrefix::Plan => {
                let mut doc: MarkplaneDocument<Plan> = self.read_item(id)?;
                doc.frontmatter.status = new_status.parse()?;
                doc.frontmatter.updated = today;
                self.write_item(id, &doc)?;
            }
            IdPrefix::Note => {
                let mut doc: MarkplaneDocument<Note> = self.read_item(id)?;
                doc.frontmatter.status = new_status.parse()?;
                doc.frontmatter.updated = today;
                self.write_item(id, &doc)?;
            }
        }

        Ok(())
    }

    /// Move an item to the archive/ subdirectory.
    pub fn archive_item(&self, id: &str) -> Result<()> {
        let (prefix, _) = parse_id(id)?;
        let dir = self.item_dir(&prefix);
        let active_path = dir.join(format!("{}.md", id));

        if !active_path.is_file() {
            return Err(MarkplaneError::NotFound(format!(
                "Item {} not found in active directory",
                id
            )));
        }

        let archive_dir = dir.join("archive");
        fs::create_dir_all(&archive_dir)?;
        let archive_path = archive_dir.join(format!("{}.md", id));
        fs::rename(&active_path, &archive_path)?;
        Ok(())
    }

    // ── Init ──────────────────────────────────────────────────────────────

    /// Initialize a new `.markplane/` directory structure.
    pub fn init(root: PathBuf, project_name: &str, description: &str) -> Result<Self> {
        if root.join("config.yaml").is_file() {
            return Err(MarkplaneError::Config(format!(
                "Markplane already initialized at {}",
                root.display()
            )));
        }

        let today = Local::now().date_naive().format("%Y-%m-%d").to_string();

        // Create directory structure
        let dirs = [
            "",
            "roadmap",
            "roadmap/archive",
            "backlog",
            "backlog/archive",
            "plans",
            "plans/archive",
            "plans/templates",
            "notes",
            "notes/archive",
            "kb",
            "templates",
            ".context",
        ];
        for dir in &dirs {
            fs::create_dir_all(root.join(dir))?;
        }

        // Write config.yaml
        let mut config = Config::default();
        config.project.name = project_name.to_string();
        config.project.description = description.to_string();
        let config_yaml = serde_yaml::to_string(&config)?;
        fs::write(root.join("config.yaml"), config_yaml)?;

        // Write INDEX.md files
        let root_index = render_template(
            templates::ROOT_INDEX_TEMPLATE,
            &[("{PROJECT_NAME}", project_name), ("{DATE}", &today)],
        );
        fs::write(root.join("INDEX.md"), root_index)?;
        fs::write(root.join("roadmap/INDEX.md"), templates::ROADMAP_INDEX_TEMPLATE)?;
        fs::write(root.join("backlog/INDEX.md"), templates::BACKLOG_INDEX_TEMPLATE)?;
        fs::write(root.join("plans/INDEX.md"), templates::PLANS_INDEX_TEMPLATE)?;
        fs::write(root.join("notes/INDEX.md"), templates::NOTES_INDEX_TEMPLATE)?;
        fs::write(root.join("kb/INDEX.md"), templates::KB_INDEX_TEMPLATE)?;

        // Write special note files
        fs::write(root.join("notes/ideas.md"), templates::IDEAS_TEMPLATE)?;
        fs::write(root.join("notes/decisions.md"), templates::DECISIONS_TEMPLATE)?;

        // Write template files
        fs::write(
            root.join("templates/backlog-item.md"),
            templates::BACKLOG_TEMPLATE,
        )?;
        fs::write(root.join("templates/epic.md"), templates::EPIC_TEMPLATE)?;
        fs::write(
            root.join("templates/plan-implementation.md"),
            templates::PLAN_IMPLEMENTATION_TEMPLATE,
        )?;
        fs::write(
            root.join("templates/plan-refactor.md"),
            templates::PLAN_REFACTOR_TEMPLATE,
        )?;
        fs::write(
            root.join("templates/note-research.md"),
            templates::NOTE_RESEARCH_TEMPLATE,
        )?;
        fs::write(
            root.join("templates/note-analysis.md"),
            templates::NOTE_ANALYSIS_TEMPLATE,
        )?;

        Ok(Project::new(root))
    }
}

/// Format a list of strings as a YAML inline list: `[a, b, c]` or `[]`.
fn format_yaml_list(items: &[String]) -> String {
    if items.is_empty() {
        "[]".to_string()
    } else {
        format!("[{}]", items.join(", "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_project() -> (TempDir, Project) {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path().join(".markplane");
        let project = Project::init(root, "Test Project", "A test project").unwrap();
        (tmp, project)
    }

    #[test]
    fn test_init_creates_structure() {
        let (tmp, _project) = setup_project();
        let root = tmp.path().join(".markplane");

        assert!(root.join("config.yaml").is_file());
        assert!(root.join("INDEX.md").is_file());
        assert!(root.join("roadmap/INDEX.md").is_file());
        assert!(root.join("roadmap/archive").is_dir());
        assert!(root.join("backlog/INDEX.md").is_file());
        assert!(root.join("backlog/archive").is_dir());
        assert!(root.join("plans/INDEX.md").is_file());
        assert!(root.join("plans/archive").is_dir());
        assert!(root.join("plans/templates").is_dir());
        assert!(root.join("notes/INDEX.md").is_file());
        assert!(root.join("notes/archive").is_dir());
        assert!(root.join("notes/ideas.md").is_file());
        assert!(root.join("notes/decisions.md").is_file());
        assert!(root.join("kb/INDEX.md").is_file());
        assert!(root.join("templates/backlog-item.md").is_file());
        assert!(root.join("templates/epic.md").is_file());
        assert!(root.join(".context").is_dir());
    }

    #[test]
    fn test_init_config() {
        let (_tmp, project) = setup_project();
        let config = project.load_config().unwrap();
        assert_eq!(config.project.name, "Test Project");
        assert_eq!(config.project.description, "A test project");
        assert_eq!(config.version, 1);
        assert_eq!(config.counters.get("BACK"), Some(&0));
    }

    #[test]
    fn test_init_already_exists() {
        let (tmp, _project) = setup_project();
        let root = tmp.path().join(".markplane");
        let result = Project::init(root, "Another", "desc");
        assert!(result.is_err());
    }

    #[test]
    fn test_next_id() {
        let (_tmp, project) = setup_project();
        let id1 = project.next_id(&IdPrefix::Back).unwrap();
        assert_eq!(id1, "BACK-001");
        let id2 = project.next_id(&IdPrefix::Back).unwrap();
        assert_eq!(id2, "BACK-002");
        let id3 = project.next_id(&IdPrefix::Epic).unwrap();
        assert_eq!(id3, "EPIC-001");
    }

    #[test]
    fn test_create_backlog_item() {
        let (_tmp, project) = setup_project();
        let item = project
            .create_backlog_item(
                "Fix login bug",
                ItemType::Bug,
                Priority::High,
                Effort::Small,
                None,
                vec!["auth".to_string()],
            )
            .unwrap();

        assert_eq!(item.id, "BACK-001");
        assert_eq!(item.title, "Fix login bug");
        assert_eq!(item.status, BacklogStatus::Draft);
        assert_eq!(item.priority, Priority::High);
        assert_eq!(item.item_type, ItemType::Bug);

        // Verify file exists and is parseable
        let doc: MarkplaneDocument<BacklogItem> = project.read_item("BACK-001").unwrap();
        assert_eq!(doc.frontmatter.id, "BACK-001");
        assert_eq!(doc.frontmatter.title, "Fix login bug");
        assert!(doc.body.contains("# Fix login bug"));
    }

    #[test]
    fn test_create_epic() {
        let (_tmp, project) = setup_project();
        let epic = project.create_epic("Phase 1: Foundation", Priority::High).unwrap();

        assert_eq!(epic.id, "EPIC-001");
        assert_eq!(epic.status, EpicStatus::Planned);

        let doc: MarkplaneDocument<Epic> = project.read_item("EPIC-001").unwrap();
        assert_eq!(doc.frontmatter.title, "Phase 1: Foundation");
    }

    #[test]
    fn test_create_plan() {
        let (_tmp, project) = setup_project();
        // Create a backlog item first
        project
            .create_backlog_item(
                "Dark mode",
                ItemType::Feature,
                Priority::High,
                Effort::Medium,
                None,
                vec![],
            )
            .unwrap();

        let plan = project
            .create_plan(
                "Dark mode implementation",
                vec!["BACK-001".to_string()],
                None,
            )
            .unwrap();

        assert_eq!(plan.id, "PLAN-001");
        assert_eq!(plan.status, PlanStatus::Draft);
        assert_eq!(plan.implements, vec!["BACK-001"]);
    }

    #[test]
    fn test_create_note() {
        let (_tmp, project) = setup_project();
        let note = project
            .create_note(
                "Caching research",
                NoteType::Research,
                vec!["cache".to_string(), "performance".to_string()],
            )
            .unwrap();

        assert_eq!(note.id, "NOTE-001");
        assert_eq!(note.note_type, NoteType::Research);
        assert_eq!(note.status, NoteStatus::Draft);
    }

    #[test]
    fn test_update_status() {
        let (_tmp, project) = setup_project();
        project
            .create_backlog_item(
                "Test item",
                ItemType::Feature,
                Priority::Medium,
                Effort::Small,
                None,
                vec![],
            )
            .unwrap();

        project.update_status("BACK-001", "in-progress").unwrap();

        let doc: MarkplaneDocument<BacklogItem> = project.read_item("BACK-001").unwrap();
        assert_eq!(doc.frontmatter.status, BacklogStatus::InProgress);
    }

    #[test]
    fn test_update_status_invalid() {
        let (_tmp, project) = setup_project();
        project
            .create_backlog_item(
                "Test item",
                ItemType::Feature,
                Priority::Medium,
                Effort::Small,
                None,
                vec![],
            )
            .unwrap();

        let result = project.update_status("BACK-001", "invalid-status");
        assert!(result.is_err());
    }

    #[test]
    fn test_archive_item() {
        let (_tmp, project) = setup_project();
        project
            .create_backlog_item(
                "To archive",
                ItemType::Chore,
                Priority::Low,
                Effort::Xs,
                None,
                vec![],
            )
            .unwrap();

        project.archive_item("BACK-001").unwrap();

        // Should now be found in archive
        let path = project.item_path("BACK-001").unwrap();
        assert!(path.to_string_lossy().contains("archive"));

        // Reading should still work
        let doc: MarkplaneDocument<BacklogItem> = project.read_item("BACK-001").unwrap();
        assert_eq!(doc.frontmatter.title, "To archive");
    }

    #[test]
    fn test_archive_nonexistent() {
        let (_tmp, project) = setup_project();
        let result = project.archive_item("BACK-999");
        assert!(result.is_err());
    }

    #[test]
    fn test_item_path_not_found() {
        let (_tmp, project) = setup_project();
        let result = project.item_path("BACK-999");
        assert!(result.is_err());
    }

    #[test]
    fn test_item_dir() {
        let (_tmp, project) = setup_project();
        let dir = project.item_dir(&IdPrefix::Back);
        assert!(dir.ends_with("backlog"));
        let dir = project.item_dir(&IdPrefix::Epic);
        assert!(dir.ends_with("roadmap"));
    }

    #[test]
    fn test_write_item() {
        let (_tmp, project) = setup_project();
        project
            .create_backlog_item(
                "Original title",
                ItemType::Feature,
                Priority::Medium,
                Effort::Small,
                None,
                vec![],
            )
            .unwrap();

        let mut doc: MarkplaneDocument<BacklogItem> = project.read_item("BACK-001").unwrap();
        doc.frontmatter.priority = Priority::High;
        doc.body = "# Updated body\n\nNew content.\n".to_string();
        project.write_item("BACK-001", &doc).unwrap();

        let updated: MarkplaneDocument<BacklogItem> = project.read_item("BACK-001").unwrap();
        assert_eq!(updated.frontmatter.priority, Priority::High);
        assert!(updated.body.contains("Updated body"));
    }

    #[test]
    fn test_format_yaml_list() {
        assert_eq!(format_yaml_list(&[]), "[]");
        assert_eq!(
            format_yaml_list(&["a".to_string(), "b".to_string()]),
            "[a, b]"
        );
    }

    #[test]
    fn test_save_and_load_config() {
        let (_tmp, project) = setup_project();
        let mut config = project.load_config().unwrap();
        config.project.name = "Updated Name".to_string();
        project.save_config(&config).unwrap();

        let reloaded = project.load_config().unwrap();
        assert_eq!(reloaded.project.name, "Updated Name");
    }
}
