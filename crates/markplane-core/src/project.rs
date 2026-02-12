use std::collections::HashSet;
use std::fs::{self, File};
use std::path::{Path, PathBuf};

use chrono::Local;
use fs2::FileExt;
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::error::{MarkplaneError, Result};
use crate::frontmatter::{parse_frontmatter, write_frontmatter};
use crate::models::*;
use crate::templates::{self, render_template};

/// Maximum allowed title length in characters.
const MAX_TITLE_LENGTH: usize = 500;

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
    /// Uses advisory file locking to prevent concurrent ID conflicts.
    pub fn next_id(&self, prefix: &IdPrefix) -> Result<String> {
        let config_path = self.root.join("config.yaml");
        let lock_file = File::open(&config_path).map_err(|e| {
            MarkplaneError::NotInitialized(format!("Cannot open config.yaml for locking: {}", e))
        })?;
        lock_file.lock_exclusive().map_err(MarkplaneError::Io)?;

        let result = (|| {
            let mut config = self.load_config()?;
            let key = prefix.as_str().to_string();
            let counter = config.counters.entry(key).or_insert(0);
            *counter += 1;
            let id = format_id(prefix, *counter);
            self.save_config(&config)?;
            Ok(id)
        })();

        lock_file.unlock().map_err(MarkplaneError::Io)?;
        result
    }

    /// Resolve an item ID to its file path.
    /// Checks items/ subdirectory first, then archive/, then legacy flat layout.
    pub fn item_path(&self, id: &str) -> Result<PathBuf> {
        let (prefix, _) = parse_id(id)?;
        let dir = self.item_dir(&prefix);

        // New layout: items/ subdirectory
        let items_path = dir.join("items").join(format!("{}.md", id));
        if items_path.is_file() {
            return Ok(items_path);
        }

        // Archive directory
        let archive_path = dir.join("archive").join(format!("{}.md", id));
        if archive_path.is_file() {
            return Ok(archive_path);
        }

        // Legacy fallback: flat directory
        let legacy_path = dir.join(format!("{}.md", id));
        if legacy_path.is_file() {
            return Ok(legacy_path);
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

    /// Create a new task.
    pub fn create_task(
        &self,
        title: &str,
        item_type: ItemType,
        priority: Priority,
        effort: Effort,
        epic: Option<String>,
        tags: Vec<String>,
    ) -> Result<Task> {
        validate_title_length(title)?;
        let id = self.next_id(&IdPrefix::Task)?;
        let today = Local::now().date_naive();
        let date_str = today.format("%Y-%m-%d").to_string();

        let safe_title = sanitize_yaml_string(title);
        let tags_yaml = format_yaml_list(&tags);
        let epic_yaml = epic.as_deref().unwrap_or("null");

        let content = render_template(
            templates::TASK_TEMPLATE,
            &[
                ("{ID}", &id),
                ("{TITLE}", &safe_title),
                ("{STATUS}", "draft"),
                ("{PRIORITY}", &priority.to_string()),
                ("{TYPE}", &item_type.to_string()),
                ("{EFFORT}", &effort.to_string()),
                ("{TAGS}", &tags_yaml),
                ("{EPIC}", epic_yaml),
                ("{DATE}", &date_str),
            ],
        );

        let items_dir = self.item_dir(&IdPrefix::Task).join("items");
        fs::create_dir_all(&items_dir)?;
        let path = items_dir.join(format!("{}.md", id));
        fs::write(&path, &content)?;

        let item = Task {
            id,
            title: title.to_string(),
            status: TaskStatus::Draft,
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
        validate_title_length(title)?;
        let id = self.next_id(&IdPrefix::Epic)?;

        let safe_title = sanitize_yaml_string(title);
        let content = render_template(
            templates::EPIC_TEMPLATE,
            &[
                ("{ID}", &id),
                ("{TITLE}", &safe_title),
                ("{PRIORITY}", &priority.to_string()),
            ],
        );

        let items_dir = self.item_dir(&IdPrefix::Epic).join("items");
        fs::create_dir_all(&items_dir)?;
        let path = items_dir.join(format!("{}.md", id));
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
        validate_title_length(title)?;
        let id = self.next_id(&IdPrefix::Plan)?;
        let today = Local::now().date_naive();
        let date_str = today.format("%Y-%m-%d").to_string();

        let safe_title = sanitize_yaml_string(title);
        let implements_yaml = format_yaml_list(&implements);
        let epic_yaml = epic.as_deref().unwrap_or("null");

        let content = render_template(
            templates::PLAN_IMPLEMENTATION_TEMPLATE,
            &[
                ("{ID}", &id),
                ("{TITLE}", &safe_title),
                ("{IMPLEMENTS}", &implements_yaml),
                ("{EPIC}", epic_yaml),
                ("{DATE}", &date_str),
            ],
        );

        let items_dir = self.item_dir(&IdPrefix::Plan).join("items");
        fs::create_dir_all(&items_dir)?;
        let path = items_dir.join(format!("{}.md", id));
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
        validate_title_length(title)?;
        let id = self.next_id(&IdPrefix::Note)?;
        let today = Local::now().date_naive();
        let date_str = today.format("%Y-%m-%d").to_string();

        let safe_title = sanitize_yaml_string(title);
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
                ("{TITLE}", &safe_title),
                ("{TYPE}", &type_str),
                ("{TAGS}", &tags_yaml),
                ("{RELATED}", "[]"),
                ("{DATE}", &date_str),
            ],
        );

        let items_dir = self.item_dir(&IdPrefix::Note).join("items");
        fs::create_dir_all(&items_dir)?;
        let path = items_dir.join(format!("{}.md", id));
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
            IdPrefix::Task => {
                let mut doc: MarkplaneDocument<Task> = self.read_item(id)?;
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

    /// Update the markdown body of any item, preserving frontmatter.
    /// Also updates the `updated` date in frontmatter where applicable.
    pub fn update_body(&self, id: &str, new_body: &str) -> Result<()> {
        let (prefix, _) = parse_id(id)?;
        let today = Local::now().date_naive();

        match prefix {
            IdPrefix::Task => {
                let mut doc: MarkplaneDocument<Task> = self.read_item(id)?;
                doc.frontmatter.updated = today;
                doc.body = new_body.to_string();
                self.write_item(id, &doc)?;
            }
            IdPrefix::Epic => {
                let mut doc: MarkplaneDocument<Epic> = self.read_item(id)?;
                doc.body = new_body.to_string();
                self.write_item(id, &doc)?;
            }
            IdPrefix::Plan => {
                let mut doc: MarkplaneDocument<Plan> = self.read_item(id)?;
                doc.frontmatter.updated = today;
                doc.body = new_body.to_string();
                self.write_item(id, &doc)?;
            }
            IdPrefix::Note => {
                let mut doc: MarkplaneDocument<Note> = self.read_item(id)?;
                doc.frontmatter.updated = today;
                doc.body = new_body.to_string();
                self.write_item(id, &doc)?;
            }
        }

        Ok(())
    }

    /// Move an item to the archive/ subdirectory.
    pub fn archive_item(&self, id: &str) -> Result<()> {
        let (prefix, _) = parse_id(id)?;
        let source = self.item_path(id)?;

        // Don't archive if already in archive
        if source.to_string_lossy().contains("/archive/") {
            return Err(MarkplaneError::NotFound(format!(
                "Item {} is already archived",
                id
            )));
        }

        let archive_dir = self.item_dir(&prefix).join("archive");
        fs::create_dir_all(&archive_dir)?;
        let archive_path = archive_dir.join(format!("{}.md", id));
        fs::rename(&source, &archive_path)?;
        Ok(())
    }

    // ── Documentation ────────────────────────────────────────────────────

    /// List documentation files from configured `documentation_paths`.
    /// Returns `(display_name, relative_path_from_markplane_root)` pairs.
    pub fn list_documentation_files(&self) -> Result<Vec<(String, String)>> {
        let config = self.load_config()?;
        let repo_root = self.root().parent().ok_or_else(|| {
            MarkplaneError::Config("Cannot determine repo root".into())
        })?;
        let mut docs = Vec::new();
        for doc_path in &config.documentation_paths {
            let abs_dir = repo_root.join(doc_path);
            if !abs_dir.is_dir() {
                continue;
            }
            let pattern = abs_dir.join("*.md").to_string_lossy().to_string();
            let mut entries: Vec<_> = glob::glob(&pattern)
                .map_err(|e| MarkplaneError::Config(e.to_string()))?
                .filter_map(|e| e.ok())
                .collect();
            entries.sort();
            for entry in entries {
                let file_name = entry.file_name().unwrap().to_string_lossy().to_string();
                let rel_path = format!("../{}/{}", doc_path, file_name);
                let display = file_name.trim_end_matches(".md").to_string();
                docs.push((display, rel_path));
            }
        }
        Ok(docs)
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
            "roadmap/items",
            "roadmap/archive",
            "backlog",
            "backlog/items",
            "backlog/archive",
            "plans",
            "plans/items",
            "plans/archive",
            "notes",
            "notes/items",
            "notes/archive",
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
        fs::write(root.join("backlog/INDEX.md"), templates::TASK_INDEX_TEMPLATE)?;
        fs::write(root.join("plans/INDEX.md"), templates::PLANS_INDEX_TEMPLATE)?;
        fs::write(root.join("notes/INDEX.md"), templates::NOTES_INDEX_TEMPLATE)?;
        // Write special note files
        fs::write(root.join("notes/ideas.md"), templates::IDEAS_TEMPLATE)?;
        fs::write(root.join("notes/decisions.md"), templates::DECISIONS_TEMPLATE)?;

        // Write template files
        fs::write(
            root.join("templates/task.md"),
            templates::TASK_TEMPLATE,
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

/// Escape a string for safe inclusion in YAML double-quoted values.
/// Escapes `\`, `"`, and newlines.
fn sanitize_yaml_string(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
}

/// Validate that a title does not exceed the maximum length.
fn validate_title_length(title: &str) -> Result<()> {
    if title.len() > MAX_TITLE_LENGTH {
        return Err(MarkplaneError::Config(format!(
            "Title exceeds maximum length of {} characters (got {})",
            MAX_TITLE_LENGTH,
            title.len()
        )));
    }
    Ok(())
}

/// Format a list of strings as a YAML inline list: `["a", "b", "c"]` or `[]`.
/// Each value is quoted to prevent YAML injection.
fn format_yaml_list(items: &[String]) -> String {
    if items.is_empty() {
        "[]".to_string()
    } else {
        format!(
            "[{}]",
            items
                .iter()
                .map(|s| format!("\"{}\"", s.replace('"', "\\\"")))
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

/// Find tasks that are blocked (have unresolved dependencies).
/// An item is blocked if it's not done/cancelled and has at least one
/// dependency that isn't done.
pub fn find_blocked_items(
    items: &[MarkplaneDocument<Task>],
) -> Vec<&MarkplaneDocument<Task>> {
    let done_ids: HashSet<&str> = items
        .iter()
        .filter(|i| i.frontmatter.status == TaskStatus::Done)
        .map(|i| i.frontmatter.id.as_str())
        .collect();

    items
        .iter()
        .filter(|i| {
            i.frontmatter.status != TaskStatus::Done
                && i.frontmatter.status != TaskStatus::Cancelled
                && !i.frontmatter.depends_on.is_empty()
                && i.frontmatter
                    .depends_on
                    .iter()
                    .any(|dep| !done_ids.contains(dep.as_str()))
        })
        .collect()
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
        assert!(root.join("roadmap/items").is_dir());
        assert!(root.join("roadmap/archive").is_dir());
        assert!(root.join("backlog/INDEX.md").is_file());
        assert!(root.join("backlog/items").is_dir());
        assert!(root.join("backlog/archive").is_dir());
        assert!(root.join("plans/INDEX.md").is_file());
        assert!(root.join("plans/items").is_dir());
        assert!(root.join("plans/archive").is_dir());
        assert!(root.join("notes/INDEX.md").is_file());
        assert!(root.join("notes/items").is_dir());
        assert!(root.join("notes/archive").is_dir());
        assert!(root.join("notes/ideas.md").is_file());
        assert!(root.join("notes/decisions.md").is_file());
        assert!(root.join("templates/task.md").is_file());
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
        assert_eq!(config.counters.get("TASK"), Some(&0));
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
        let id1 = project.next_id(&IdPrefix::Task).unwrap();
        assert_eq!(id1, "TASK-001");
        let id2 = project.next_id(&IdPrefix::Task).unwrap();
        assert_eq!(id2, "TASK-002");
        let id3 = project.next_id(&IdPrefix::Epic).unwrap();
        assert_eq!(id3, "EPIC-001");
    }

    #[test]
    fn test_create_task() {
        let (_tmp, project) = setup_project();
        let item = project
            .create_task(
                "Fix login bug",
                ItemType::Bug,
                Priority::High,
                Effort::Small,
                None,
                vec!["auth".to_string()],
            )
            .unwrap();

        assert_eq!(item.id, "TASK-001");
        assert_eq!(item.title, "Fix login bug");
        assert_eq!(item.status, TaskStatus::Draft);
        assert_eq!(item.priority, Priority::High);
        assert_eq!(item.item_type, ItemType::Bug);

        // Verify file exists and is parseable
        let doc: MarkplaneDocument<Task> = project.read_item("TASK-001").unwrap();
        assert_eq!(doc.frontmatter.id, "TASK-001");
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
        // Create a task first
        project
            .create_task(
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
                vec!["TASK-001".to_string()],
                None,
            )
            .unwrap();

        assert_eq!(plan.id, "PLAN-001");
        assert_eq!(plan.status, PlanStatus::Draft);
        assert_eq!(plan.implements, vec!["TASK-001"]);
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
            .create_task(
                "Test item",
                ItemType::Feature,
                Priority::Medium,
                Effort::Small,
                None,
                vec![],
            )
            .unwrap();

        project.update_status("TASK-001", "in-progress").unwrap();

        let doc: MarkplaneDocument<Task> = project.read_item("TASK-001").unwrap();
        assert_eq!(doc.frontmatter.status, TaskStatus::InProgress);
    }

    #[test]
    fn test_update_status_invalid() {
        let (_tmp, project) = setup_project();
        project
            .create_task(
                "Test item",
                ItemType::Feature,
                Priority::Medium,
                Effort::Small,
                None,
                vec![],
            )
            .unwrap();

        let result = project.update_status("TASK-001", "invalid-status");
        assert!(result.is_err());
    }

    #[test]
    fn test_archive_item() {
        let (_tmp, project) = setup_project();
        project
            .create_task(
                "To archive",
                ItemType::Chore,
                Priority::Low,
                Effort::Xs,
                None,
                vec![],
            )
            .unwrap();

        project.archive_item("TASK-001").unwrap();

        // Should now be found in archive
        let path = project.item_path("TASK-001").unwrap();
        assert!(path.to_string_lossy().contains("archive"));

        // Reading should still work
        let doc: MarkplaneDocument<Task> = project.read_item("TASK-001").unwrap();
        assert_eq!(doc.frontmatter.title, "To archive");
    }

    #[test]
    fn test_archive_nonexistent() {
        let (_tmp, project) = setup_project();
        let result = project.archive_item("TASK-999");
        assert!(result.is_err());
    }

    #[test]
    fn test_item_path_not_found() {
        let (_tmp, project) = setup_project();
        let result = project.item_path("TASK-999");
        assert!(result.is_err());
    }

    #[test]
    fn test_item_dir() {
        let (_tmp, project) = setup_project();
        let dir = project.item_dir(&IdPrefix::Task);
        assert!(dir.ends_with("backlog"));
        let dir = project.item_dir(&IdPrefix::Epic);
        assert!(dir.ends_with("roadmap"));
    }

    #[test]
    fn test_write_item() {
        let (_tmp, project) = setup_project();
        project
            .create_task(
                "Original title",
                ItemType::Feature,
                Priority::Medium,
                Effort::Small,
                None,
                vec![],
            )
            .unwrap();

        let mut doc: MarkplaneDocument<Task> = project.read_item("TASK-001").unwrap();
        doc.frontmatter.priority = Priority::High;
        doc.body = "# Updated body\n\nNew content.\n".to_string();
        project.write_item("TASK-001", &doc).unwrap();

        let updated: MarkplaneDocument<Task> = project.read_item("TASK-001").unwrap();
        assert_eq!(updated.frontmatter.priority, Priority::High);
        assert!(updated.body.contains("Updated body"));
    }

    #[test]
    fn test_format_yaml_list() {
        assert_eq!(format_yaml_list(&[]), "[]");
        assert_eq!(
            format_yaml_list(&["a".to_string(), "b".to_string()]),
            "[\"a\", \"b\"]"
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

    // ── Status updates for Epic, Plan, Note types ────────────────────────

    #[test]
    fn test_update_status_epic() {
        let (_tmp, project) = setup_project();
        project.create_epic("Phase 1", Priority::High).unwrap();

        project.update_status("EPIC-001", "active").unwrap();
        let doc: MarkplaneDocument<Epic> = project.read_item("EPIC-001").unwrap();
        assert_eq!(doc.frontmatter.status, EpicStatus::Active);

        project.update_status("EPIC-001", "done").unwrap();
        let doc: MarkplaneDocument<Epic> = project.read_item("EPIC-001").unwrap();
        assert_eq!(doc.frontmatter.status, EpicStatus::Done);

    }

    #[test]
    fn test_update_status_plan() {
        let (_tmp, project) = setup_project();
        project.create_plan("Plan A", vec![], None).unwrap();

        project.update_status("PLAN-001", "approved").unwrap();
        let doc: MarkplaneDocument<Plan> = project.read_item("PLAN-001").unwrap();
        assert_eq!(doc.frontmatter.status, PlanStatus::Approved);

        project.update_status("PLAN-001", "in-progress").unwrap();
        let doc: MarkplaneDocument<Plan> = project.read_item("PLAN-001").unwrap();
        assert_eq!(doc.frontmatter.status, PlanStatus::InProgress);

        project.update_status("PLAN-001", "done").unwrap();
        let doc: MarkplaneDocument<Plan> = project.read_item("PLAN-001").unwrap();
        assert_eq!(doc.frontmatter.status, PlanStatus::Done);
    }

    #[test]
    fn test_update_status_note() {
        let (_tmp, project) = setup_project();
        project
            .create_note("Research A", NoteType::Research, vec![])
            .unwrap();

        project.update_status("NOTE-001", "active").unwrap();
        let doc: MarkplaneDocument<Note> = project.read_item("NOTE-001").unwrap();
        assert_eq!(doc.frontmatter.status, NoteStatus::Active);

        project.update_status("NOTE-001", "archived").unwrap();
        let doc: MarkplaneDocument<Note> = project.read_item("NOTE-001").unwrap();
        assert_eq!(doc.frontmatter.status, NoteStatus::Archived);
    }

    #[test]
    fn test_update_status_epic_invalid() {
        let (_tmp, project) = setup_project();
        project.create_epic("Phase 1", Priority::High).unwrap();
        assert!(project.update_status("EPIC-001", "in-progress").is_err());
    }

    #[test]
    fn test_update_status_plan_invalid() {
        let (_tmp, project) = setup_project();
        project.create_plan("Plan A", vec![], None).unwrap();
        assert!(project.update_status("PLAN-001", "cancelled").is_err());
    }

    // ── find_blocked_items ───────────────────────────────────────────────

    #[test]
    fn test_find_blocked_items_none_blocked() {
        let (_tmp, project) = setup_project();
        project
            .create_task("A", ItemType::Feature, Priority::Medium, Effort::Small, None, vec![])
            .unwrap();
        project
            .create_task("B", ItemType::Feature, Priority::Medium, Effort::Small, None, vec![])
            .unwrap();

        let items = project.list_tasks(&crate::query::QueryFilter::default()).unwrap();
        let blocked = find_blocked_items(&items);
        assert!(blocked.is_empty());
    }

    #[test]
    fn test_find_blocked_items_with_blocked() {
        let (_tmp, project) = setup_project();
        project
            .create_task("Blocker", ItemType::Feature, Priority::Medium, Effort::Small, None, vec![])
            .unwrap();
        project
            .create_task("Blocked", ItemType::Feature, Priority::Medium, Effort::Small, None, vec![])
            .unwrap();

        // Set TASK-002 to depend on TASK-001
        let mut doc: MarkplaneDocument<Task> = project.read_item("TASK-002").unwrap();
        doc.frontmatter.depends_on = vec!["TASK-001".to_string()];
        project.write_item("TASK-002", &doc).unwrap();

        let items = project.list_tasks(&crate::query::QueryFilter::default()).unwrap();
        let blocked = find_blocked_items(&items);
        assert_eq!(blocked.len(), 1);
        assert_eq!(blocked[0].frontmatter.id, "TASK-002");
    }

    #[test]
    fn test_find_blocked_items_resolved_dependency() {
        let (_tmp, project) = setup_project();
        project
            .create_task("Blocker", ItemType::Feature, Priority::Medium, Effort::Small, None, vec![])
            .unwrap();
        project
            .create_task("Blocked", ItemType::Feature, Priority::Medium, Effort::Small, None, vec![])
            .unwrap();

        // Set dependency
        let mut doc: MarkplaneDocument<Task> = project.read_item("TASK-002").unwrap();
        doc.frontmatter.depends_on = vec!["TASK-001".to_string()];
        project.write_item("TASK-002", &doc).unwrap();

        // Mark blocker as done
        project.update_status("TASK-001", "done").unwrap();

        let items = project.list_tasks(&crate::query::QueryFilter::default()).unwrap();
        let blocked = find_blocked_items(&items);
        assert!(blocked.is_empty()); // No longer blocked
    }

    // ── UTF-8 truncate safety (bug fix verification) ─────────────────────

    #[test]
    fn test_format_yaml_list_with_special_chars() {
        // Tags with quotes should be escaped properly
        let tags = vec!["c++".to_string(), "it's".to_string(), "key\"value".to_string()];
        let result = format_yaml_list(&tags);
        assert!(result.contains("c++"));
        assert!(result.contains("it's"));
        // Double-quote inside should be escaped
        assert!(result.contains("key\\\"value"));
    }

    #[test]
    fn test_create_task_with_emoji_title() {
        let (_tmp, project) = setup_project();
        let item = project
            .create_task(
                "Fix login bug 🔥🚀",
                ItemType::Bug,
                Priority::High,
                Effort::Small,
                None,
                vec![],
            )
            .unwrap();

        assert_eq!(item.title, "Fix login bug 🔥🚀");
        // Read it back and verify
        let doc: MarkplaneDocument<Task> = project.read_item("TASK-001").unwrap();
        assert_eq!(doc.frontmatter.title, "Fix login bug 🔥🚀");
    }

    // ── Title length validation ──────────────────────────────────────────

    #[test]
    fn test_validate_title_length_ok() {
        assert!(validate_title_length("Normal title").is_ok());
    }

    #[test]
    fn test_validate_title_length_too_long() {
        let long_title = "x".repeat(501);
        assert!(validate_title_length(&long_title).is_err());
    }

    #[test]
    fn test_validate_title_length_at_limit() {
        let title = "x".repeat(500);
        assert!(validate_title_length(&title).is_ok());
    }

    // ── sanitize_yaml_string ─────────────────────────────────────────────

    #[test]
    fn test_sanitize_yaml_string() {
        assert_eq!(sanitize_yaml_string("hello"), "hello");
        assert_eq!(sanitize_yaml_string("it's \"fine\""), "it's \\\"fine\\\"");
        assert_eq!(sanitize_yaml_string("line\nbreak"), "line\\nbreak");
        assert_eq!(sanitize_yaml_string("back\\slash"), "back\\\\slash");
    }

    // ── update_body ────────────────────────────────────────────────────

    #[test]
    fn test_update_body_task() {
        let (_tmp, project) = setup_project();
        project
            .create_task("Test item", ItemType::Feature, Priority::Medium, Effort::Small, None, vec![])
            .unwrap();

        let original: MarkplaneDocument<Task> = project.read_item("TASK-001").unwrap();
        assert!(original.body.contains("[What needs to be done"));

        project
            .update_body("TASK-001", "# Test item\n\nActual description here.\n")
            .unwrap();

        let updated: MarkplaneDocument<Task> = project.read_item("TASK-001").unwrap();
        assert!(updated.body.contains("Actual description here."));
        assert_eq!(updated.frontmatter.id, "TASK-001");
        assert_eq!(updated.frontmatter.title, "Test item");
    }

    #[test]
    fn test_update_body_epic() {
        let (_tmp, project) = setup_project();
        project.create_epic("Phase 1", Priority::High).unwrap();

        project
            .update_body("EPIC-001", "# Phase 1\n\n## Objective\n\nBuild the foundation.\n")
            .unwrap();

        let updated: MarkplaneDocument<Epic> = project.read_item("EPIC-001").unwrap();
        assert!(updated.body.contains("Build the foundation."));
        assert_eq!(updated.frontmatter.id, "EPIC-001");
    }

    #[test]
    fn test_update_body_plan() {
        let (_tmp, project) = setup_project();
        project.create_plan("Plan A", vec![], None).unwrap();

        project
            .update_body("PLAN-001", "# Plan A\n\nDetailed steps.\n")
            .unwrap();

        let updated: MarkplaneDocument<Plan> = project.read_item("PLAN-001").unwrap();
        assert!(updated.body.contains("Detailed steps."));
    }

    #[test]
    fn test_update_body_note() {
        let (_tmp, project) = setup_project();
        project
            .create_note("Research A", NoteType::Research, vec![])
            .unwrap();

        project
            .update_body("NOTE-001", "# Research A\n\nFindings here.\n")
            .unwrap();

        let updated: MarkplaneDocument<Note> = project.read_item("NOTE-001").unwrap();
        assert!(updated.body.contains("Findings here."));
    }

    #[test]
    fn test_update_body_nonexistent() {
        let (_tmp, project) = setup_project();
        let result = project.update_body("TASK-999", "new body");
        assert!(result.is_err());
    }

    #[test]
    fn test_list_documentation_files_empty() {
        let (_tmp, project) = setup_project();
        let docs = project.list_documentation_files().unwrap();
        assert!(docs.is_empty());
    }

    #[test]
    fn test_list_documentation_files_with_docs() {
        let (tmp, project) = setup_project();
        // Create a docs directory at the repo root (parent of .markplane/)
        let docs_dir = tmp.path().join("docs");
        fs::create_dir_all(&docs_dir).unwrap();
        fs::write(docs_dir.join("architecture.md"), "# Architecture").unwrap();
        fs::write(docs_dir.join("getting-started.md"), "# Getting Started").unwrap();
        fs::write(docs_dir.join("not-markdown.txt"), "ignored").unwrap();

        // Update config to include documentation_paths
        let mut config = project.load_config().unwrap();
        config.documentation_paths = vec!["docs".to_string()];
        project.save_config(&config).unwrap();

        let docs = project.list_documentation_files().unwrap();
        assert_eq!(docs.len(), 2);
        assert_eq!(docs[0].0, "architecture");
        assert_eq!(docs[0].1, "../docs/architecture.md");
        assert_eq!(docs[1].0, "getting-started");
        assert_eq!(docs[1].1, "../docs/getting-started.md");
    }

    #[test]
    fn test_list_documentation_files_missing_dir() {
        let (_tmp, project) = setup_project();
        let mut config = project.load_config().unwrap();
        config.documentation_paths = vec!["nonexistent".to_string()];
        project.save_config(&config).unwrap();

        let docs = project.list_documentation_files().unwrap();
        assert!(docs.is_empty());
    }
}
