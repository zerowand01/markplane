use std::collections::HashSet;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

use chrono::{Local, NaiveDate};
use fs2::FileExt;
use serde::Serialize;
use serde::de::DeserializeOwned;
use tempfile::NamedTempFile;

use crate::error::{MarkplaneError, Result};
use crate::frontmatter::{parse_frontmatter, write_frontmatter};
use crate::links::{LinkAction, LinkRelation};
use crate::manifest;
use crate::models::*;
use crate::query::QueryFilter;
use crate::templates::{self, render_template};

// ── Patch<T> ────────────────────────────────────────────────────────────────

/// Three-state type for clearable optional fields in updates.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum Patch<T> {
    /// Don't touch this field.
    #[default]
    Unchanged,
    /// Set the field to None.
    Clear,
    /// Set the field to a value.
    Set(T),
}

// ── Per-type update structs ─────────────────────────────────────────────────

/// Fields that can be updated on a Task.
#[derive(Clone, Debug, Default)]
pub struct TaskUpdate {
    pub title: Option<String>,
    pub status: Option<String>,
    pub priority: Option<String>,
    pub effort: Option<String>,
    pub item_type: Option<String>,
    pub assignee: Patch<String>,
    pub position: Patch<String>,
    pub add_tags: Vec<String>,
    pub remove_tags: Vec<String>,
    pub body: Option<String>,
}

/// Fields that can be updated on an Epic.
#[derive(Clone, Debug, Default)]
pub struct EpicUpdate {
    pub title: Option<String>,
    pub status: Option<String>,
    pub priority: Option<String>,
    pub add_tags: Vec<String>,
    pub remove_tags: Vec<String>,
    pub started: Patch<NaiveDate>,
    pub target: Patch<NaiveDate>,
    pub body: Option<String>,
}

/// Fields that can be updated on a Plan.
#[derive(Clone, Debug, Default)]
pub struct PlanUpdate {
    pub title: Option<String>,
    pub status: Option<String>,
    pub body: Option<String>,
}

/// Fields that can be updated on a Note.
#[derive(Clone, Debug, Default)]
pub struct NoteUpdate {
    pub title: Option<String>,
    pub status: Option<String>,
    pub note_type: Option<String>,
    pub add_tags: Vec<String>,
    pub remove_tags: Vec<String>,
    pub body: Option<String>,
}

/// Generic union of all per-type update fields, for MCP/CLI dispatch.
/// Positioning directive for `move_item()`.
#[derive(Clone, Debug)]
pub enum MoveDirective {
    /// Move to the top of the item's priority group.
    Top,
    /// Move to the bottom of the item's priority group.
    Bottom,
    /// Position immediately before a specific item.
    Before(String),
    /// Position immediately after a specific item.
    After(String),
}

/// `update_item()` parses the prefix, validates inapplicable fields, and delegates.
#[derive(Clone, Debug, Default)]
pub struct UpdateFields {
    pub title: Option<String>,
    pub status: Option<String>,
    pub priority: Option<String>,
    pub effort: Option<String>,
    pub item_type: Option<String>,
    pub assignee: Patch<String>,
    pub position: Patch<String>,
    pub add_tags: Vec<String>,
    pub remove_tags: Vec<String>,
    pub started: Patch<NaiveDate>,
    pub target: Patch<NaiveDate>,
    pub note_type: Option<String>,
}

/// Apply tag changes: retain non-removed tags, then push non-duplicate adds.
pub fn apply_tag_changes(current: &mut Vec<String>, add: &[String], remove: &[String]) {
    current.retain(|t| !remove.contains(t));
    for tag in add {
        if !current.contains(tag) {
            current.push(tag.clone());
        }
    }
}

/// Maximum allowed title length in characters.
const MAX_TITLE_LENGTH: usize = 500;

/// Atomically create a new file, failing if it already exists.
/// Uses `File::create_new()` (O_CREAT | O_EXCL) to prevent TOCTOU races.
fn write_new_file(path: &Path, content: &str) -> Result<()> {
    let mut file = File::create_new(path).map_err(|e| {
        if e.kind() == std::io::ErrorKind::AlreadyExists {
            MarkplaneError::DuplicateId(
                path.file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown")
                    .to_string(),
            )
        } else {
            e.into()
        }
    })?;
    file.write_all(content.as_bytes())?;
    Ok(())
}

/// Atomically write `content` to `path` by writing to a tempfile in the same
/// directory, then renaming. The rename is atomic on the same filesystem,
/// so a crash mid-write can never leave a truncated/corrupted target file.
fn atomic_write(path: &Path, content: &[u8]) -> Result<()> {
    let parent = path.parent().ok_or_else(|| {
        MarkplaneError::Config(format!(
            "Cannot determine parent directory of {}",
            path.display()
        ))
    })?;
    let mut tmp = NamedTempFile::new_in(parent)?;
    tmp.write_all(content)?;
    tmp.persist(path)?;
    Ok(())
}

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

    /// Generate a unique random ID for a given prefix.
    /// Retries up to 100 times to avoid collisions with existing files.
    pub fn next_id(&self, prefix: &IdPrefix) -> Result<String> {
        let dir = self.item_dir(prefix);
        for _ in 0..100 {
            let id = generate_random_id(prefix);
            let items_path = dir.join("items").join(format!("{}.md", id));
            let archive_path = dir.join("archive").join(format!("{}.md", id));
            if !items_path.exists() && !archive_path.exists() {
                return Ok(id);
            }
        }
        Err(MarkplaneError::Config(
            "Failed to generate unique ID after 100 attempts".into(),
        ))
    }

    /// Resolve an item ID to its file path.
    /// Checks items/ subdirectory first, then archive/.
    pub fn item_path(&self, id: &str) -> Result<PathBuf> {
        let (prefix, _) = parse_id(id)?;
        let dir = self.item_dir(&prefix);

        let items_path = dir.join("items").join(format!("{}.md", id));
        if items_path.is_file() {
            return Ok(items_path);
        }

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

    /// Compute a position key that appends to the end of a priority group.
    fn append_position(&self, priority: &Priority) -> Result<String> {
        let tasks = self.list_tasks(&crate::query::QueryFilter::default())?;
        let max_pos = tasks
            .iter()
            .filter(|t| &t.frontmatter.priority == priority)
            .filter_map(|t| t.frontmatter.position.as_deref())
            .max();
        Ok(crate::position::generate_key_between(max_pos, None)?
            .expect("generate_key_between(_, None) always returns Some"))
    }

    // ── Template Resolution ──────────────────────────────────────────────

    /// Resolve a template body for the given kind.
    ///
    /// Resolution chain:
    /// 1. `explicit` name if provided
    /// 2. `type_defaults[item_type]` from manifest
    /// 3. `default` for the kind from manifest
    /// 4. Fall through to "default"
    ///
    /// Then: try reading `templates/{filename}` from disk, fall back to built-in.
    pub fn resolve_template_body(
        &self,
        kind: &str,
        explicit: Option<&str>,
        item_type: Option<&str>,
    ) -> Result<String> {
        // Determine the template name via the resolution chain
        let name = if let Some(name) = explicit {
            name.to_string()
        } else {
            let manifest = match manifest::load_manifest(&self.root) {
                Ok(m) => m,
                Err(e) => {
                    eprintln!(
                        "warning: failed to parse manifest.yaml, using default templates: {e}"
                    );
                    None
                }
            };
            if let Some(m) = manifest {
                if let Some(kind_config) = m.get(kind) {
                    if let Some(it) = item_type {
                        kind_config
                            .type_defaults
                            .get(it)
                            .cloned()
                            .or_else(|| kind_config.default.clone())
                            .unwrap_or_else(|| "default".to_string())
                    } else {
                        kind_config
                            .default
                            .clone()
                            .unwrap_or_else(|| "default".to_string())
                    }
                } else {
                    "default".to_string()
                }
            } else {
                "default".to_string()
            }
        };

        // Validate template name to prevent path traversal
        manifest::validate_template_name(&name).map_err(MarkplaneError::Config)?;

        // Try reading from disk first
        let filename = manifest::template_filename(kind, &name);
        let path = self.root.join("templates").join(&filename);
        if let Ok(content) = fs::read_to_string(&path) {
            return Ok(content);
        }

        // Fall back to built-in
        Ok(manifest::builtin_template(kind, &name).to_string())
    }

    // ── Validation ─────────────────────────────────────────────────────────

    /// Validate that a task type is in the configured list.
    pub fn validate_task_type(&self, item_type: &str) -> Result<()> {
        let config = self.load_config()?;
        if config.task_types.iter().any(|t| t == item_type) {
            Ok(())
        } else {
            Err(MarkplaneError::Config(format!(
                "Unknown task type: '{}'. Valid types: {}",
                item_type,
                config.task_types.join(", ")
            )))
        }
    }

    /// Validate that a note type is in the configured list.
    pub fn validate_note_type(&self, note_type: &str) -> Result<()> {
        let config = self.load_config()?;
        if config.note_types.iter().any(|t| t == note_type) {
            Ok(())
        } else {
            Err(MarkplaneError::Config(format!(
                "Unknown note type: '{}'. Valid types: {}",
                note_type,
                config.note_types.join(", ")
            )))
        }
    }

    // ── CRUD Operations ───────────────────────────────────────────────────

    /// Validate that a task status is in the configured workflow.
    pub fn validate_task_status(&self, status: &str) -> Result<()> {
        let config = self.load_config()?;
        if config.workflows.task.contains(status) {
            Ok(())
        } else {
            Err(MarkplaneError::Config(format!(
                "Unknown task status: '{}'. Valid statuses: {}",
                status,
                config.workflows.task.all_statuses().join(", ")
            )))
        }
    }

    /// Create a new task.
    #[allow(clippy::too_many_arguments)]
    pub fn create_task(
        &self,
        title: &str,
        item_type: &str,
        priority: Priority,
        effort: Effort,
        epic: Option<String>,
        tags: Vec<String>,
        template: Option<&str>,
    ) -> Result<Task> {
        validate_title_length(title)?;
        self.validate_task_type(item_type)?;
        let config = self.load_config()?;
        let id = self.next_id(&IdPrefix::Task)?;
        let today = Local::now().date_naive();
        let position = self.append_position(&priority)?;
        let tmpl = self.resolve_template_body("task", template, Some(item_type))?;

        let task = Task {
            id,
            title: title.to_string(),
            status: config.default_task_status().to_string(),
            priority,
            item_type: item_type.to_string(),
            effort,
            tags,
            epic,
            plan: None,
            depends_on: vec![],
            blocks: vec![],
            related: vec![],
            assignee: None,
            position: Some(position),
            created: today,
            updated: today,
        };

        let body = render_template(&tmpl, &[("{TITLE}", title)]);
        let doc = MarkplaneDocument {
            frontmatter: &task,
            body,
        };
        let content = write_frontmatter(&doc)?;

        let items_dir = self.item_dir(&IdPrefix::Task).join("items");
        fs::create_dir_all(&items_dir)?;
        let path = items_dir.join(format!("{}.md", task.id));
        write_new_file(&path, &content)?;

        Ok(task)
    }

    /// Create a new epic.
    pub fn create_epic(
        &self,
        title: &str,
        priority: Priority,
        template: Option<&str>,
    ) -> Result<Epic> {
        validate_title_length(title)?;
        let id = self.next_id(&IdPrefix::Epic)?;

        let today = Local::now().date_naive();
        let epic = Epic {
            id,
            title: title.to_string(),
            status: EpicStatus::Later,
            priority,
            started: None,
            target: None,
            tags: vec![],
            related: vec![],
            created: today,
            updated: today,
        };

        let tmpl = self.resolve_template_body("epic", template, None)?;
        let body = render_template(&tmpl, &[("{TITLE}", title)]);
        let doc = MarkplaneDocument {
            frontmatter: &epic,
            body,
        };
        let content = write_frontmatter(&doc)?;

        let items_dir = self.item_dir(&IdPrefix::Epic).join("items");
        fs::create_dir_all(&items_dir)?;
        let path = items_dir.join(format!("{}.md", epic.id));
        write_new_file(&path, &content)?;

        Ok(epic)
    }

    /// Create a new plan.
    pub fn create_plan(
        &self,
        title: &str,
        implements: Vec<String>,
        template: Option<&str>,
    ) -> Result<Plan> {
        validate_title_length(title)?;
        let id = self.next_id(&IdPrefix::Plan)?;
        let today = Local::now().date_naive();

        let plan = Plan {
            id,
            title: title.to_string(),
            status: PlanStatus::Draft,
            implements,
            related: vec![],
            created: today,
            updated: today,
        };

        let tmpl = self.resolve_template_body("plan", template, None)?;
        let body = render_template(&tmpl, &[("{TITLE}", title)]);
        let doc = MarkplaneDocument {
            frontmatter: &plan,
            body,
        };
        let content = write_frontmatter(&doc)?;

        let items_dir = self.item_dir(&IdPrefix::Plan).join("items");
        fs::create_dir_all(&items_dir)?;
        let path = items_dir.join(format!("{}.md", plan.id));
        write_new_file(&path, &content)?;

        Ok(plan)
    }

    /// Create a new note.
    pub fn create_note(
        &self,
        title: &str,
        note_type: &str,
        tags: Vec<String>,
        template: Option<&str>,
    ) -> Result<Note> {
        validate_title_length(title)?;
        self.validate_note_type(note_type)?;
        let id = self.next_id(&IdPrefix::Note)?;
        let today = Local::now().date_naive();
        let tmpl = self.resolve_template_body("note", template, Some(note_type))?;

        let note = Note {
            id,
            title: title.to_string(),
            note_type: note_type.to_string(),
            status: NoteStatus::Draft,
            tags,
            related: vec![],
            created: today,
            updated: today,
        };

        let body = render_template(&tmpl, &[("{TITLE}", title)]);
        let doc = MarkplaneDocument {
            frontmatter: &note,
            body,
        };
        let content = write_frontmatter(&doc)?;

        let items_dir = self.item_dir(&IdPrefix::Note).join("items");
        fs::create_dir_all(&items_dir)?;
        let path = items_dir.join(format!("{}.md", note.id));
        write_new_file(&path, &content)?;

        Ok(note)
    }

    /// Read any item by ID, deserializing the frontmatter into type `T`.
    pub fn read_item<T: DeserializeOwned>(&self, id: &str) -> Result<MarkplaneDocument<T>> {
        let path = self.item_path(id)?;
        let content = fs::read_to_string(&path)?;
        parse_frontmatter(&content)
    }

    /// Write any item by ID, serializing the frontmatter from type `T`.
    ///
    /// Uses atomic write (tempfile + rename) for crash safety. The caller is
    /// responsible for holding an advisory lock if this is part of a
    /// read-modify-write cycle — see [`lock_item`] and [`lock_items`].
    pub fn write_item<T: Serialize>(&self, id: &str, doc: &MarkplaneDocument<T>) -> Result<()> {
        let path = self.item_path(id)?;
        let content = write_frontmatter(doc)?;
        atomic_write(&path, content.as_bytes())?;
        Ok(())
    }

    /// Acquire an advisory exclusive lock on an item file.
    ///
    /// Returns the locked `File` handle. The lock is held until the handle is
    /// dropped. The caller must hold the returned handle for the duration of
    /// the read-modify-write cycle.
    pub fn lock_item(&self, id: &str) -> Result<File> {
        let path = self.item_path(id)?;
        let file = File::open(&path)?;
        file.lock_exclusive()?;
        Ok(file)
    }

    /// Acquire advisory exclusive locks on multiple item files in deterministic
    /// (sorted) order to prevent deadlocks.
    ///
    /// Returns locked `File` handles. All locks are held until the handles are
    /// dropped.
    pub fn lock_items(&self, ids: &mut [&str]) -> Result<Vec<File>> {
        ids.sort();
        let mut locks = Vec::with_capacity(ids.len());
        for id in ids {
            locks.push(self.lock_item(id)?);
        }
        Ok(locks)
    }

    /// Update the status field of any item (auto-detects type from ID prefix).
    ///
    /// Holds an advisory file lock for the full read-modify-write cycle.
    pub fn update_status(&self, id: &str, new_status: &str) -> Result<()> {
        let (prefix, _) = parse_id(id)?;
        let today = Local::now().date_naive();
        let _lock = self.lock_item(id)?;

        match prefix {
            IdPrefix::Task => {
                self.validate_task_status(new_status)?;
                let mut doc: MarkplaneDocument<Task> = self.read_item(id)?;
                doc.frontmatter.status = new_status.to_string();
                doc.frontmatter.updated = today;
                self.write_item(id, &doc)?;
            }
            IdPrefix::Epic => {
                let mut doc: MarkplaneDocument<Epic> = self.read_item(id)?;
                doc.frontmatter.status = new_status.parse()?;
                doc.frontmatter.updated = today;
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

    // ── Typed Update Methods ──────────────────────────────────────────────

    /// Update properties on a Task.
    ///
    /// Holds an advisory file lock for the full read-modify-write cycle.
    pub fn update_task(&self, id: &str, u: &TaskUpdate) -> Result<()> {
        let _lock = self.lock_item(id)?;
        let mut doc: MarkplaneDocument<Task> = self.read_item(id)?;
        let fm = &mut doc.frontmatter;

        if let Some(ref title) = u.title {
            validate_title_length(title)?;
            fm.title = title.clone();
        }
        if let Some(ref status) = u.status {
            self.validate_task_status(status)?;
            fm.status = status.clone();
        }
        if let Some(ref priority) = u.priority {
            fm.priority = priority.parse()?;
        }
        if let Some(ref effort) = u.effort {
            fm.effort = effort.parse()?;
        }
        if let Some(ref item_type) = u.item_type {
            self.validate_task_type(item_type)?;
            fm.item_type = item_type.clone();
        }
        match &u.assignee {
            Patch::Set(v) => fm.assignee = Some(v.clone()),
            Patch::Clear => fm.assignee = None,
            Patch::Unchanged => {}
        }
        match &u.position {
            Patch::Set(v) => fm.position = Some(v.clone()),
            Patch::Clear => fm.position = None,
            Patch::Unchanged => {}
        }
        apply_tag_changes(&mut fm.tags, &u.add_tags, &u.remove_tags);
        if let Some(ref new_body) = u.body {
            doc.body = new_body.clone();
        }
        fm.updated = Local::now().date_naive();
        self.write_item(id, &doc)
    }

    /// Update properties on an Epic.
    ///
    /// Holds an advisory file lock for the full read-modify-write cycle.
    pub fn update_epic(&self, id: &str, u: &EpicUpdate) -> Result<()> {
        let _lock = self.lock_item(id)?;
        let mut doc: MarkplaneDocument<Epic> = self.read_item(id)?;
        let fm = &mut doc.frontmatter;

        if let Some(ref title) = u.title {
            validate_title_length(title)?;
            fm.title = title.clone();
        }
        if let Some(ref status) = u.status {
            fm.status = status.parse()?;
        }
        if let Some(ref priority) = u.priority {
            fm.priority = priority.parse()?;
        }
        apply_tag_changes(&mut fm.tags, &u.add_tags, &u.remove_tags);
        match &u.started {
            Patch::Set(v) => fm.started = Some(*v),
            Patch::Clear => fm.started = None,
            Patch::Unchanged => {}
        }
        match &u.target {
            Patch::Set(v) => fm.target = Some(*v),
            Patch::Clear => fm.target = None,
            Patch::Unchanged => {}
        }
        if let Some(ref new_body) = u.body {
            doc.body = new_body.clone();
        }
        fm.updated = Local::now().date_naive();
        self.write_item(id, &doc)
    }

    /// Update properties on a Plan.
    ///
    /// Holds an advisory file lock for the full read-modify-write cycle.
    pub fn update_plan(&self, id: &str, u: &PlanUpdate) -> Result<()> {
        let _lock = self.lock_item(id)?;
        let mut doc: MarkplaneDocument<Plan> = self.read_item(id)?;
        let fm = &mut doc.frontmatter;

        if let Some(ref title) = u.title {
            validate_title_length(title)?;
            fm.title = title.clone();
        }
        if let Some(ref status) = u.status {
            fm.status = status.parse()?;
        }
        if let Some(ref new_body) = u.body {
            doc.body = new_body.clone();
        }
        fm.updated = Local::now().date_naive();
        self.write_item(id, &doc)
    }

    /// Update properties on a Note.
    ///
    /// Holds an advisory file lock for the full read-modify-write cycle.
    pub fn update_note(&self, id: &str, u: &NoteUpdate) -> Result<()> {
        let _lock = self.lock_item(id)?;
        let mut doc: MarkplaneDocument<Note> = self.read_item(id)?;
        let fm = &mut doc.frontmatter;

        if let Some(ref title) = u.title {
            validate_title_length(title)?;
            fm.title = title.clone();
        }
        if let Some(ref status) = u.status {
            fm.status = status.parse()?;
        }
        if let Some(ref note_type) = u.note_type {
            self.validate_note_type(note_type)?;
            fm.note_type = note_type.clone();
        }
        apply_tag_changes(&mut fm.tags, &u.add_tags, &u.remove_tags);
        if let Some(ref new_body) = u.body {
            doc.body = new_body.clone();
        }
        fm.updated = Local::now().date_naive();
        self.write_item(id, &doc)
    }

    /// Generic dispatch: parse prefix from ID, validate inapplicable fields, delegate.
    pub fn update_item(&self, id: &str, fields: UpdateFields) -> Result<()> {
        let (prefix, _) = parse_id(id)?;

        match prefix {
            IdPrefix::Task => {
                // Reject fields not applicable to tasks
                if !matches!(fields.started, Patch::Unchanged)
                    || !matches!(fields.target, Patch::Unchanged)
                {
                    return Err(MarkplaneError::Config(
                        "Tasks do not support started/target fields".into(),
                    ));
                }
                if fields.note_type.is_some() {
                    return Err(MarkplaneError::Config(
                        "Tasks do not support the note_type field".into(),
                    ));
                }
                self.update_task(
                    id,
                    &TaskUpdate {
                        title: fields.title,
                        status: fields.status,
                        priority: fields.priority,
                        effort: fields.effort,
                        item_type: fields.item_type,
                        assignee: fields.assignee,
                        position: fields.position,
                        add_tags: fields.add_tags,
                        remove_tags: fields.remove_tags,
                        body: None,
                    },
                )
            }
            IdPrefix::Epic => {
                // Reject fields not applicable to epics
                if fields.effort.is_some() {
                    return Err(MarkplaneError::Config(
                        "Epics do not support the effort field".into(),
                    ));
                }
                if fields.item_type.is_some() {
                    return Err(MarkplaneError::Config(
                        "Epics do not support the type field".into(),
                    ));
                }
                if !matches!(fields.assignee, Patch::Unchanged) {
                    return Err(MarkplaneError::Config(
                        "Epics do not support the assignee field".into(),
                    ));
                }
                if !matches!(fields.position, Patch::Unchanged) {
                    return Err(MarkplaneError::Config(
                        "Epics do not support the position field".into(),
                    ));
                }
                if fields.note_type.is_some() {
                    return Err(MarkplaneError::Config(
                        "Epics do not support the note_type field".into(),
                    ));
                }
                self.update_epic(
                    id,
                    &EpicUpdate {
                        title: fields.title,
                        status: fields.status,
                        priority: fields.priority,
                        add_tags: fields.add_tags,
                        remove_tags: fields.remove_tags,
                        started: fields.started,
                        target: fields.target,
                        body: None,
                    },
                )
            }
            IdPrefix::Plan => {
                // Reject fields not applicable to plans
                for (name, present) in [
                    ("priority", fields.priority.is_some()),
                    ("effort", fields.effort.is_some()),
                    ("type", fields.item_type.is_some()),
                    ("note_type", fields.note_type.is_some()),
                ] {
                    if present {
                        return Err(MarkplaneError::Config(format!(
                            "Plans do not support the {} field",
                            name
                        )));
                    }
                }
                if !matches!(fields.assignee, Patch::Unchanged) {
                    return Err(MarkplaneError::Config(
                        "Plans do not support the assignee field".into(),
                    ));
                }
                if !matches!(fields.position, Patch::Unchanged) {
                    return Err(MarkplaneError::Config(
                        "Plans do not support the position field".into(),
                    ));
                }
                if !fields.add_tags.is_empty() || !fields.remove_tags.is_empty() {
                    return Err(MarkplaneError::Config("Plans do not support tags".into()));
                }
                if !matches!(fields.started, Patch::Unchanged)
                    || !matches!(fields.target, Patch::Unchanged)
                {
                    return Err(MarkplaneError::Config(
                        "Plans do not support started/target fields".into(),
                    ));
                }
                self.update_plan(
                    id,
                    &PlanUpdate {
                        title: fields.title,
                        status: fields.status,
                        body: None,
                    },
                )
            }
            IdPrefix::Note => {
                // Reject fields not applicable to notes
                if fields.priority.is_some() {
                    return Err(MarkplaneError::Config(
                        "Notes do not support the priority field".into(),
                    ));
                }
                if fields.effort.is_some() {
                    return Err(MarkplaneError::Config(
                        "Notes do not support the effort field".into(),
                    ));
                }
                if fields.item_type.is_some() {
                    return Err(MarkplaneError::Config(
                        "Notes do not support the type field".into(),
                    ));
                }
                if !matches!(fields.assignee, Patch::Unchanged) {
                    return Err(MarkplaneError::Config(
                        "Notes do not support the assignee field".into(),
                    ));
                }
                if !matches!(fields.position, Patch::Unchanged) {
                    return Err(MarkplaneError::Config(
                        "Notes do not support the position field".into(),
                    ));
                }
                if !matches!(fields.started, Patch::Unchanged)
                    || !matches!(fields.target, Patch::Unchanged)
                {
                    return Err(MarkplaneError::Config(
                        "Notes do not support started/target fields".into(),
                    ));
                }
                self.update_note(
                    id,
                    &NoteUpdate {
                        title: fields.title,
                        status: fields.status,
                        note_type: fields.note_type,
                        add_tags: fields.add_tags,
                        remove_tags: fields.remove_tags,
                        body: None,
                    },
                )
            }
        }
    }

    // ── Move / Reorder ─────────────────────────────────────────────────

    /// Move a task to a new position within its priority group.
    ///
    /// Computes the correct fractional-indexing position key and updates the
    /// task's frontmatter. If positions are missing or there is no room for a
    /// new key, the priority group is normalized automatically.
    pub fn move_item(&self, id: &str, directive: MoveDirective) -> Result<()> {
        use crate::position::{generate_key_between, sequential_keys};

        // Validate the ID is a task
        let (prefix, _) = parse_id(id)?;
        if prefix != IdPrefix::Task {
            return Err(MarkplaneError::Config(format!(
                "{} is not a task — only tasks support positioning",
                id
            )));
        }

        let doc: MarkplaneDocument<Task> = self.read_item(id)?;
        let priority = doc.frontmatter.priority.clone();

        // Get all tasks in the same priority group, sorted by position
        let filter = crate::query::QueryFilter {
            priority: Some(vec![priority.to_string()]),
            ..Default::default()
        };
        let tasks = self.list_tasks(&filter)?;

        // If any task lacks a position, normalize the group first and re-read
        let tasks = if tasks.iter().any(|t| t.frontmatter.position.is_none()) {
            self.normalize_priority_group(&tasks)?;
            self.list_tasks(&filter)?
        } else {
            tasks
        };

        // Build sorted list excluding the item being moved
        let others: Vec<_> = tasks.iter().filter(|t| t.frontmatter.id != id).collect();

        let (insert_index, new_pos) = match &directive {
            MoveDirective::Top => {
                let first_pos = others
                    .first()
                    .and_then(|t| t.frontmatter.position.as_deref());
                (0, generate_key_between(None, first_pos)?)
            }
            MoveDirective::Bottom => {
                let last_pos = others
                    .last()
                    .and_then(|t| t.frontmatter.position.as_deref());
                (others.len(), generate_key_between(last_pos, None)?)
            }
            MoveDirective::Before(target_id) => {
                if target_id == id {
                    return Err(MarkplaneError::InvalidLink(
                        "Cannot position an item relative to itself".into(),
                    ));
                }
                let idx = self.find_move_target(&others, target_id, id, &priority)?;
                let before = if idx > 0 {
                    others[idx - 1].frontmatter.position.as_deref()
                } else {
                    None
                };
                let after = others[idx].frontmatter.position.as_deref();
                (idx, generate_key_between(before, after)?)
            }
            MoveDirective::After(target_id) => {
                if target_id == id {
                    return Err(MarkplaneError::InvalidLink(
                        "Cannot position an item relative to itself".into(),
                    ));
                }
                let idx = self.find_move_target(&others, target_id, id, &priority)?;
                let before = others[idx].frontmatter.position.as_deref();
                let after = others
                    .get(idx + 1)
                    .and_then(|t| t.frontmatter.position.as_deref());
                (idx + 1, generate_key_between(before, after)?)
            }
        };

        match new_pos {
            Some(pos) => self.update_task(
                id,
                &TaskUpdate {
                    position: Patch::Set(pos),
                    ..Default::default()
                },
            ),
            None => {
                // No room for a fractional key — normalize the group with the
                // moved item at the desired index.
                let mut ordered: Vec<_> = tasks.iter().filter(|t| t.frontmatter.id != id).collect();
                let moved = tasks
                    .iter()
                    .find(|t| t.frontmatter.id == id)
                    .ok_or_else(|| {
                        MarkplaneError::NotFound(format!(
                            "task {} not found in priority group during move",
                            id
                        ))
                    })?;
                let at = insert_index.min(ordered.len());
                ordered.insert(at, moved);

                let keys = sequential_keys(ordered.len());
                for (doc, new_key) in ordered.iter().zip(keys.iter()) {
                    if doc.frontmatter.position.as_deref() != Some(new_key.as_str()) {
                        self.update_task(
                            &doc.frontmatter.id,
                            &TaskUpdate {
                                position: Patch::Set(new_key.clone()),
                                ..Default::default()
                            },
                        )?;
                    }
                }
                Ok(())
            }
        }
    }

    /// Find the index of `target_id` in the `others` list, or return an
    /// appropriate error (not found / different priority group).
    fn find_move_target(
        &self,
        others: &[&MarkplaneDocument<Task>],
        target_id: &str,
        moved_id: &str,
        priority: &Priority,
    ) -> Result<usize> {
        others
            .iter()
            .position(|t| t.frontmatter.id == target_id)
            .ok_or_else(|| match self.read_item::<Task>(target_id) {
                Ok(target_doc) => MarkplaneError::InvalidLink(format!(
                    "{} is in priority '{}' but {} is in '{}'",
                    target_id, target_doc.frontmatter.priority, moved_id, priority,
                )),
                Err(e) => e,
            })
    }

    /// Assign clean sequential position keys to every task in a list.
    fn normalize_priority_group(&self, tasks: &[MarkplaneDocument<Task>]) -> Result<()> {
        use crate::position::sequential_keys;

        let keys = sequential_keys(tasks.len());
        for (doc, new_pos) in tasks.iter().zip(keys.iter()) {
            if doc.frontmatter.position.as_deref() != Some(new_pos.as_str()) {
                self.update_task(
                    &doc.frontmatter.id,
                    &TaskUpdate {
                        position: Patch::Set(new_pos.clone()),
                        ..Default::default()
                    },
                )?;
            }
        }
        Ok(())
    }

    /// Find all active items that reference the given ID in their frontmatter fields.
    ///
    /// Returns a list of `(referencing_id, relation, action)` tuples that can be
    /// used to remove the inbound references via `link_items()`.
    pub fn find_inbound_references(
        &self,
        target_id: &str,
    ) -> Result<Vec<(String, LinkRelation, LinkAction)>> {
        let mut refs = Vec::new();

        // Scan tasks for references to target_id
        let tasks = self.list_tasks(&QueryFilter::default())?;
        for doc in &tasks {
            let fm = &doc.frontmatter;
            if fm.id == target_id {
                continue;
            }
            if fm.blocks.iter().any(|b| b == target_id) {
                refs.push((fm.id.clone(), LinkRelation::Blocks, LinkAction::Remove));
            }
            if fm.depends_on.iter().any(|d| d == target_id) {
                refs.push((fm.id.clone(), LinkRelation::DependsOn, LinkAction::Remove));
            }
            if fm.epic.as_deref() == Some(target_id) {
                refs.push((fm.id.clone(), LinkRelation::Epic, LinkAction::Remove));
            }
            if fm.plan.as_deref() == Some(target_id) {
                refs.push((fm.id.clone(), LinkRelation::Plan, LinkAction::Remove));
            }
            if fm.related.iter().any(|r| r == target_id) {
                refs.push((fm.id.clone(), LinkRelation::Related, LinkAction::Remove));
            }
        }

        // Scan plans for references to target_id
        let plans = self.list_plans()?;
        for doc in &plans {
            let fm = &doc.frontmatter;
            if fm.id == target_id {
                continue;
            }
            if fm.implements.iter().any(|i| i == target_id) {
                refs.push((fm.id.clone(), LinkRelation::Implements, LinkAction::Remove));
            }
            if fm.related.iter().any(|r| r == target_id) {
                refs.push((fm.id.clone(), LinkRelation::Related, LinkAction::Remove));
            }
        }

        // Scan epics for references to target_id
        let epics = self.list_epics()?;
        for doc in &epics {
            let fm = &doc.frontmatter;
            if fm.id == target_id {
                continue;
            }
            if fm.related.iter().any(|r| r == target_id) {
                refs.push((fm.id.clone(), LinkRelation::Related, LinkAction::Remove));
            }
        }

        // Scan notes for references to target_id
        let notes = self.list_notes()?;
        for doc in &notes {
            let fm = &doc.frontmatter;
            if fm.id == target_id {
                continue;
            }
            if fm.related.iter().any(|r| r == target_id) {
                refs.push((fm.id.clone(), LinkRelation::Related, LinkAction::Remove));
            }
        }

        Ok(refs)
    }

    /// Move an item to the archive/ subdirectory.
    ///
    /// Also cleans up inbound references from active items so they don't
    /// point to an archived item.
    pub fn archive_item(&self, id: &str) -> Result<()> {
        let (prefix, _) = parse_id(id)?;
        let source = self.item_path(id)?;

        // Don't archive if already in archive
        if path_is_archived(&source) {
            return Err(MarkplaneError::NotFound(format!(
                "Item {} is already archived",
                id
            )));
        }

        // Find inbound references before archiving
        let inbound_refs = self.find_inbound_references(id)?;

        // Move the file to archive
        let archive_dir = self.item_dir(&prefix).join("archive");
        fs::create_dir_all(&archive_dir)?;
        let archive_path = archive_dir.join(format!("{}.md", id));
        fs::rename(&source, &archive_path)?;

        // Clean up inbound references from active items
        for (source_id, relation, action) in inbound_refs {
            // Best-effort: if cleanup fails, the item is still archived
            let _ = self.link_items(&source_id, id, relation, action);
        }

        Ok(())
    }

    /// Move an item from the archive/ subdirectory back to items/.
    pub fn unarchive_item(&self, id: &str) -> Result<()> {
        let (prefix, _) = parse_id(id)?;
        let source = self.item_path(id)?;

        // Only unarchive if currently in archive
        if !path_is_archived(&source) {
            return Err(MarkplaneError::Config(format!(
                "Item {} is not archived",
                id
            )));
        }

        let items_dir = self.item_dir(&prefix).join("items");
        fs::create_dir_all(&items_dir)?;
        let items_path = items_dir.join(format!("{}.md", id));
        fs::rename(&source, &items_path)?;
        Ok(())
    }

    /// Check whether an item is currently archived.
    pub fn is_archived(&self, id: &str) -> Result<bool> {
        let path = self.item_path(id)?;
        Ok(path_is_archived(&path))
    }

    // ── Documentation ────────────────────────────────────────────────────

    /// List documentation files from configured `documentation_paths`.
    /// Returns `(display_name, relative_path_from_markplane_root)` pairs.
    pub fn list_documentation_files(&self) -> Result<Vec<(String, String)>> {
        let config = self.load_config()?;
        let repo_root = self
            .root()
            .parent()
            .ok_or_else(|| MarkplaneError::Config("Cannot determine repo root".into()))?;
        let canonical_root = repo_root
            .canonicalize()
            .map_err(|e| MarkplaneError::Config(format!("Cannot canonicalize repo root: {e}")))?;
        let mut docs = Vec::new();
        for doc_path in &config.documentation_paths {
            let abs_dir = repo_root.join(doc_path);
            if !abs_dir.is_dir() {
                continue;
            }
            // Verify the resolved path is within the repo root
            let canonical_dir = abs_dir.canonicalize().map_err(|e| {
                MarkplaneError::Config(format!("Cannot canonicalize path '{}': {e}", doc_path))
            })?;
            if !canonical_dir.starts_with(&canonical_root) {
                continue;
            }
            let pattern = abs_dir.join("*.md").to_string_lossy().to_string();
            let mut entries: Vec<_> = glob::glob(&pattern)
                .map_err(|e| MarkplaneError::Config(e.to_string()))?
                .filter_map(|e| e.ok())
                .collect();
            entries.sort();
            for entry in entries {
                let Some(file_name_os) = entry.file_name() else {
                    continue;
                };
                let file_name = file_name_os.to_string_lossy().to_string();
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

        // Write .gitignore for derived files
        fs::write(root.join(".gitignore"), templates::GITIGNORE_TEMPLATE)?;

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
        fs::write(
            root.join("roadmap/INDEX.md"),
            templates::ROADMAP_INDEX_TEMPLATE,
        )?;
        fs::write(
            root.join("backlog/INDEX.md"),
            templates::TASK_INDEX_TEMPLATE,
        )?;
        fs::write(root.join("plans/INDEX.md"), templates::PLANS_INDEX_TEMPLATE)?;
        fs::write(root.join("notes/INDEX.md"), templates::NOTES_INDEX_TEMPLATE)?;
        // Write special note files
        fs::write(root.join("notes/ideas.md"), templates::IDEAS_TEMPLATE)?;
        fs::write(
            root.join("notes/decisions.md"),
            templates::DECISIONS_TEMPLATE,
        )?;

        // Write template manifest and files
        fs::write(
            root.join("templates/manifest.yaml"),
            manifest::DEFAULT_MANIFEST,
        )?;
        fs::write(root.join("templates/task.md"), templates::TASK_TEMPLATE)?;
        fs::write(
            root.join("templates/task-bug.md"),
            templates::TASK_BUG_TEMPLATE,
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
            root.join("templates/note.md"),
            templates::NOTE_GENERIC_TEMPLATE,
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

    /// Seed the project with starter content that demonstrates the system.
    ///
    /// Creates an epic, two tasks, a plan, and a note with realistic body
    /// content and cross-references. Called by `markplane init` by default
    /// (skipped with `--empty`).
    pub fn seed_starter_content(&self) -> Result<Vec<String>> {
        use crate::links::{LinkAction, LinkRelation};

        // 1. Create epic
        let epic = self.create_epic("Project Setup", Priority::Medium, None)?;
        let epic_id = epic.id.clone();

        // 2. Create task 1: review setup
        let setup_task = self.create_task(
            "Review and customize your markplane setup",
            "chore",
            Priority::Medium,
            Effort::Small,
            Some(epic_id.clone()),
            vec!["onboarding".to_string()],
            None,
        )?;
        let setup_task_id = setup_task.id.clone();

        // 3. Create task 2: import existing work
        let import_task = self.create_task(
            "Import existing work into markplane",
            "chore",
            Priority::Medium,
            Effort::Medium,
            Some(epic_id.clone()),
            vec!["onboarding".to_string()],
            None,
        )?;
        let import_task_id = import_task.id.clone();

        // 4. Create plan for the import task
        let plan = self.create_plan("Import existing work into markplane", vec![], None)?;
        let plan_id = plan.id.clone();

        // 5. Create note
        let note = self.create_note(
            "Project decisions",
            "decision",
            vec!["onboarding".to_string()],
            None,
        )?;
        let note_id = note.id.clone();

        // 6. Link plan ↔ import task
        self.link_items(
            &import_task_id,
            &plan_id,
            LinkRelation::Plan,
            LinkAction::Add,
        )?;

        // 7. Set statuses and body content in one pass per item
        let today = Local::now().format("%Y-%m-%d").to_string();
        let vars = &[
            ("{EPIC_ID}", epic_id.as_str()),
            ("{SETUP_TASK_ID}", setup_task_id.as_str()),
            ("{IMPORT_TASK_ID}", import_task_id.as_str()),
            ("{PLAN_ID}", plan_id.as_str()),
            ("{TODAY}", today.as_str()),
        ];

        self.update_epic(
            &epic_id,
            &EpicUpdate {
                status: Some("now".to_string()),
                body: Some(render_template(templates::STARTER_EPIC_BODY, vars)),
                ..Default::default()
            },
        )?;
        self.update_task(
            &setup_task_id,
            &TaskUpdate {
                status: Some("backlog".to_string()),
                body: Some(render_template(templates::STARTER_SETUP_TASK_BODY, vars)),
                ..Default::default()
            },
        )?;
        self.update_task(
            &import_task_id,
            &TaskUpdate {
                status: Some("backlog".to_string()),
                body: Some(render_template(templates::STARTER_IMPORT_TASK_BODY, vars)),
                ..Default::default()
            },
        )?;
        self.update_plan(
            &plan_id,
            &PlanUpdate {
                body: Some(render_template(templates::STARTER_PLAN_BODY, vars)),
                ..Default::default()
            },
        )?;
        self.update_note(
            &note_id,
            &NoteUpdate {
                status: Some("active".to_string()),
                body: Some(render_template(templates::STARTER_NOTE_BODY, vars)),
                ..Default::default()
            },
        )?;

        Ok(vec![
            epic_id,
            setup_task_id,
            import_task_id,
            plan_id,
            note_id,
        ])
    }
}

/// Check whether an item file lives inside an `archive/` parent directory.
fn path_is_archived(path: &Path) -> bool {
    path.parent()
        .and_then(|p| p.file_name())
        .is_some_and(|name| name == "archive")
}

/// Validate that a title does not exceed the maximum length.
fn validate_title_length(title: &str) -> Result<()> {
    let char_count = title.chars().count();
    if char_count > MAX_TITLE_LENGTH {
        return Err(MarkplaneError::Config(format!(
            "Title exceeds maximum length of {} characters (got {})",
            MAX_TITLE_LENGTH, char_count
        )));
    }
    Ok(())
}

/// Find tasks that are blocked (have unresolved dependencies).
/// An item is blocked if it's not done/cancelled and has at least one
/// dependency that isn't done.
pub fn find_blocked_items<'a>(
    items: &'a [MarkplaneDocument<Task>],
    workflow: &TaskWorkflow,
) -> Vec<&'a MarkplaneDocument<Task>> {
    let completed_statuses: HashSet<&str> = workflow
        .statuses_in(StatusCategory::Completed)
        .iter()
        .map(|s| s.as_str())
        .collect();
    let closed_statuses: HashSet<&str> = workflow
        .statuses_in(StatusCategory::Completed)
        .iter()
        .chain(workflow.statuses_in(StatusCategory::Cancelled).iter())
        .map(|s| s.as_str())
        .collect();

    let done_ids: HashSet<&str> = items
        .iter()
        .filter(|i| completed_statuses.contains(i.frontmatter.status.as_str()))
        .map(|i| i.frontmatter.id.as_str())
        .collect();

    items
        .iter()
        .filter(|i| {
            !closed_statuses.contains(i.frontmatter.status.as_str())
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
        assert!(root.join("templates/manifest.yaml").is_file());
        assert!(root.join("templates/task.md").is_file());
        assert!(root.join("templates/task-bug.md").is_file());
        assert!(root.join("templates/epic.md").is_file());
        assert!(root.join("templates/plan-implementation.md").is_file());
        assert!(root.join("templates/plan-refactor.md").is_file());
        assert!(root.join("templates/note.md").is_file());
        assert!(root.join("templates/note-research.md").is_file());
        assert!(root.join("templates/note-analysis.md").is_file());
        assert!(root.join(".context").is_dir());
        assert!(root.join(".gitignore").is_file());
    }

    #[test]
    fn test_init_config() {
        let (_tmp, project) = setup_project();
        let config = project.load_config().unwrap();
        assert_eq!(config.project.name, "Test Project");
        assert_eq!(config.project.description, "A test project");
        assert_eq!(config.version, 1);
        assert_eq!(config.task_types, default_task_types());
        assert_eq!(config.note_types, default_note_types());
        assert_eq!(config.default_task_type(), "feature");
        assert_eq!(config.default_note_type(), "research");
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
        assert!(
            id1.starts_with("TASK-"),
            "Expected TASK- prefix, got: {}",
            id1
        );
        assert!(parse_id(&id1).is_ok());

        let id2 = project.next_id(&IdPrefix::Task).unwrap();
        assert!(id2.starts_with("TASK-"));
        assert_ne!(id1, id2, "IDs should be unique");

        let id3 = project.next_id(&IdPrefix::Epic).unwrap();
        assert!(id3.starts_with("EPIC-"));
    }

    #[test]
    fn test_create_task() {
        let (_tmp, project) = setup_project();
        let item = project
            .create_task(
                "Fix login bug",
                "bug",
                Priority::High,
                Effort::Small,
                None,
                vec!["auth".to_string()],
                None,
            )
            .unwrap();

        assert!(item.id.starts_with("TASK-"));
        assert_eq!(item.title, "Fix login bug");
        assert_eq!(item.status, "draft");
        assert_eq!(item.priority, Priority::High);
        assert_eq!(item.item_type, "bug");

        // Verify file exists and is parseable
        let doc: MarkplaneDocument<Task> = project.read_item(&item.id).unwrap();
        assert_eq!(doc.frontmatter.id, item.id);
        assert_eq!(doc.frontmatter.title, "Fix login bug");
        assert!(doc.body.contains("# Fix login bug"));
    }

    #[test]
    fn test_create_epic() {
        let (_tmp, project) = setup_project();
        let epic = project
            .create_epic("Phase 1: Foundation", Priority::High, None)
            .unwrap();

        assert!(epic.id.starts_with("EPIC-"));
        assert_eq!(epic.status, EpicStatus::Later);

        let doc: MarkplaneDocument<Epic> = project.read_item(&epic.id).unwrap();
        assert_eq!(doc.frontmatter.title, "Phase 1: Foundation");
    }

    #[test]
    fn test_create_plan() {
        let (_tmp, project) = setup_project();
        // Create a task first
        let task = project
            .create_task(
                "Dark mode",
                "feature",
                Priority::High,
                Effort::Medium,
                None,
                vec![],
                None,
            )
            .unwrap();

        let plan = project
            .create_plan("Dark mode implementation", vec![task.id.clone()], None)
            .unwrap();

        assert!(plan.id.starts_with("PLAN-"));
        assert_eq!(plan.status, PlanStatus::Draft);
        assert_eq!(plan.implements, vec![task.id]);
    }

    #[test]
    fn test_create_note() {
        let (_tmp, project) = setup_project();
        let note = project
            .create_note(
                "Caching research",
                "research",
                vec!["cache".to_string(), "performance".to_string()],
                None,
            )
            .unwrap();

        assert!(note.id.starts_with("NOTE-"));
        assert_eq!(note.note_type, "research");
        assert_eq!(note.status, NoteStatus::Draft);
    }

    #[test]
    fn test_update_status() {
        let (_tmp, project) = setup_project();
        let task = project
            .create_task(
                "Test item",
                "feature",
                Priority::Medium,
                Effort::Small,
                None,
                vec![],
                None,
            )
            .unwrap();

        project.update_status(&task.id, "in-progress").unwrap();

        let doc: MarkplaneDocument<Task> = project.read_item(&task.id).unwrap();
        assert_eq!(doc.frontmatter.status, "in-progress");
    }

    #[test]
    fn test_update_status_invalid() {
        let (_tmp, project) = setup_project();
        let task = project
            .create_task(
                "Test item",
                "feature",
                Priority::Medium,
                Effort::Small,
                None,
                vec![],
                None,
            )
            .unwrap();

        let result = project.update_status(&task.id, "invalid-status");
        assert!(result.is_err());
    }

    #[test]
    fn test_archive_item() {
        let (_tmp, project) = setup_project();
        let task = project
            .create_task(
                "To archive",
                "chore",
                Priority::Low,
                Effort::Xs,
                None,
                vec![],
                None,
            )
            .unwrap();

        project.archive_item(&task.id).unwrap();

        // Should now be found in archive
        let path = project.item_path(&task.id).unwrap();
        assert!(path.to_string_lossy().contains("archive"));

        // Reading should still work
        let doc: MarkplaneDocument<Task> = project.read_item(&task.id).unwrap();
        assert_eq!(doc.frontmatter.title, "To archive");
    }

    #[test]
    fn test_archive_nonexistent() {
        let (_tmp, project) = setup_project();
        let result = project.archive_item("TASK-zzzzz");
        assert!(result.is_err());
    }

    #[test]
    fn test_item_path_not_found() {
        let (_tmp, project) = setup_project();
        let result = project.item_path("TASK-zzzzz");
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
        let task = project
            .create_task(
                "Original title",
                "feature",
                Priority::Medium,
                Effort::Small,
                None,
                vec![],
                None,
            )
            .unwrap();

        let mut doc: MarkplaneDocument<Task> = project.read_item(&task.id).unwrap();
        doc.frontmatter.priority = Priority::High;
        doc.body = "# Updated body\n\nNew content.\n".to_string();
        project.write_item(&task.id, &doc).unwrap();

        let updated: MarkplaneDocument<Task> = project.read_item(&task.id).unwrap();
        assert_eq!(updated.frontmatter.priority, Priority::High);
        assert!(updated.body.contains("Updated body"));
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

    // ── Create→roundtrip byte-identity ────────────────────────────────

    #[test]
    fn test_create_roundtrip_byte_identical() {
        let (_tmp, project) = setup_project();

        // Task with tags, epic, special chars
        let epic = project
            .create_epic("Phase 1", Priority::High, None)
            .unwrap();
        let task = project
            .create_task(
                "Fix \"login\" bug's edge-case",
                "bug",
                Priority::High,
                Effort::Small,
                Some(epic.id.clone()),
                vec!["auth".to_string(), "urgent".to_string()],
                None,
            )
            .unwrap();

        let path = project.item_path(&task.id).unwrap();
        let original = fs::read_to_string(&path).unwrap();

        // Read and immediately write back — should produce identical bytes
        let doc: MarkplaneDocument<Task> = project.read_item(&task.id).unwrap();
        project.write_item(&task.id, &doc).unwrap();
        let after_roundtrip = fs::read_to_string(&path).unwrap();

        assert_eq!(
            original, after_roundtrip,
            "create output must be byte-identical to read→write roundtrip"
        );

        // Same for epic
        let epic_path = project.item_path(&epic.id).unwrap();
        let epic_original = fs::read_to_string(&epic_path).unwrap();
        let epic_doc: MarkplaneDocument<Epic> = project.read_item(&epic.id).unwrap();
        project.write_item(&epic.id, &epic_doc).unwrap();
        let epic_after = fs::read_to_string(&epic_path).unwrap();
        assert_eq!(epic_original, epic_after);

        // Plan with implements list
        let plan = project
            .create_plan("Plan A", vec![task.id.clone()], None)
            .unwrap();
        let plan_path = project.item_path(&plan.id).unwrap();
        let plan_original = fs::read_to_string(&plan_path).unwrap();
        let plan_doc: MarkplaneDocument<Plan> = project.read_item(&plan.id).unwrap();
        project.write_item(&plan.id, &plan_doc).unwrap();
        let plan_after = fs::read_to_string(&plan_path).unwrap();
        assert_eq!(plan_original, plan_after);

        // Note with tags
        let note = project
            .create_note("Research", "research", vec!["perf".to_string()], None)
            .unwrap();
        let note_path = project.item_path(&note.id).unwrap();
        let note_original = fs::read_to_string(&note_path).unwrap();
        let note_doc: MarkplaneDocument<Note> = project.read_item(&note.id).unwrap();
        project.write_item(&note.id, &note_doc).unwrap();
        let note_after = fs::read_to_string(&note_path).unwrap();
        assert_eq!(note_original, note_after);
    }

    // ── Status updates for Epic, Plan, Note types ────────────────────────

    #[test]
    fn test_update_status_epic() {
        let (_tmp, project) = setup_project();
        let epic = project
            .create_epic("Phase 1", Priority::High, None)
            .unwrap();

        project.update_status(&epic.id, "next").unwrap();
        let doc: MarkplaneDocument<Epic> = project.read_item(&epic.id).unwrap();
        assert_eq!(doc.frontmatter.status, EpicStatus::Next);

        project.update_status(&epic.id, "now").unwrap();
        let doc: MarkplaneDocument<Epic> = project.read_item(&epic.id).unwrap();
        assert_eq!(doc.frontmatter.status, EpicStatus::Now);

        project.update_status(&epic.id, "done").unwrap();
        let doc: MarkplaneDocument<Epic> = project.read_item(&epic.id).unwrap();
        assert_eq!(doc.frontmatter.status, EpicStatus::Done);
    }

    #[test]
    fn test_update_status_plan() {
        let (_tmp, project) = setup_project();
        let plan = project.create_plan("Plan A", vec![], None).unwrap();

        project.update_status(&plan.id, "approved").unwrap();
        let doc: MarkplaneDocument<Plan> = project.read_item(&plan.id).unwrap();
        assert_eq!(doc.frontmatter.status, PlanStatus::Approved);

        project.update_status(&plan.id, "in-progress").unwrap();
        let doc: MarkplaneDocument<Plan> = project.read_item(&plan.id).unwrap();
        assert_eq!(doc.frontmatter.status, PlanStatus::InProgress);

        project.update_status(&plan.id, "done").unwrap();
        let doc: MarkplaneDocument<Plan> = project.read_item(&plan.id).unwrap();
        assert_eq!(doc.frontmatter.status, PlanStatus::Done);
    }

    #[test]
    fn test_update_status_note() {
        let (_tmp, project) = setup_project();
        let note = project
            .create_note("Research A", "research", vec![], None)
            .unwrap();

        project.update_status(&note.id, "active").unwrap();
        let doc: MarkplaneDocument<Note> = project.read_item(&note.id).unwrap();
        assert_eq!(doc.frontmatter.status, NoteStatus::Active);

        project.update_status(&note.id, "archived").unwrap();
        let doc: MarkplaneDocument<Note> = project.read_item(&note.id).unwrap();
        assert_eq!(doc.frontmatter.status, NoteStatus::Archived);
    }

    #[test]
    fn test_update_status_epic_invalid() {
        let (_tmp, project) = setup_project();
        let epic = project
            .create_epic("Phase 1", Priority::High, None)
            .unwrap();
        assert!(project.update_status(&epic.id, "in-progress").is_err());
        assert!(project.update_status(&epic.id, "planned").is_err());
        assert!(project.update_status(&epic.id, "active").is_err());
    }

    #[test]
    fn test_update_status_plan_invalid() {
        let (_tmp, project) = setup_project();
        let plan = project.create_plan("Plan A", vec![], None).unwrap();
        assert!(project.update_status(&plan.id, "cancelled").is_err());
    }

    // ── find_blocked_items ───────────────────────────────────────────────

    #[test]
    fn test_find_blocked_items_none_blocked() {
        let (_tmp, project) = setup_project();
        project
            .create_task(
                "A",
                "feature",
                Priority::Medium,
                Effort::Small,
                None,
                vec![],
                None,
            )
            .unwrap();
        project
            .create_task(
                "B",
                "feature",
                Priority::Medium,
                Effort::Small,
                None,
                vec![],
                None,
            )
            .unwrap();

        let config = project.load_config().unwrap();
        let items = project
            .list_tasks(&crate::query::QueryFilter::default())
            .unwrap();
        let blocked = find_blocked_items(&items, &config.workflows.task);
        assert!(blocked.is_empty());
    }

    #[test]
    fn test_find_blocked_items_with_blocked() {
        let (_tmp, project) = setup_project();
        let blocker = project
            .create_task(
                "Blocker",
                "feature",
                Priority::Medium,
                Effort::Small,
                None,
                vec![],
                None,
            )
            .unwrap();
        let blocked_task = project
            .create_task(
                "Blocked",
                "feature",
                Priority::Medium,
                Effort::Small,
                None,
                vec![],
                None,
            )
            .unwrap();

        // Set blocked_task to depend on blocker
        let mut doc: MarkplaneDocument<Task> = project.read_item(&blocked_task.id).unwrap();
        doc.frontmatter.depends_on = vec![blocker.id.clone()];
        project.write_item(&blocked_task.id, &doc).unwrap();

        let config = project.load_config().unwrap();
        let items = project
            .list_tasks(&crate::query::QueryFilter::default())
            .unwrap();
        let blocked = find_blocked_items(&items, &config.workflows.task);
        assert_eq!(blocked.len(), 1);
        assert_eq!(blocked[0].frontmatter.id, blocked_task.id);
    }

    #[test]
    fn test_find_blocked_items_resolved_dependency() {
        let (_tmp, project) = setup_project();
        let blocker = project
            .create_task(
                "Blocker",
                "feature",
                Priority::Medium,
                Effort::Small,
                None,
                vec![],
                None,
            )
            .unwrap();
        let blocked_task = project
            .create_task(
                "Blocked",
                "feature",
                Priority::Medium,
                Effort::Small,
                None,
                vec![],
                None,
            )
            .unwrap();

        // Set dependency
        let mut doc: MarkplaneDocument<Task> = project.read_item(&blocked_task.id).unwrap();
        doc.frontmatter.depends_on = vec![blocker.id.clone()];
        project.write_item(&blocked_task.id, &doc).unwrap();

        // Mark blocker as done
        project.update_status(&blocker.id, "done").unwrap();

        let config = project.load_config().unwrap();
        let items = project
            .list_tasks(&crate::query::QueryFilter::default())
            .unwrap();
        let blocked = find_blocked_items(&items, &config.workflows.task);
        assert!(blocked.is_empty()); // No longer blocked
    }

    #[test]
    fn test_create_task_with_emoji_title() {
        let (_tmp, project) = setup_project();
        let item = project
            .create_task(
                "Fix login bug 🔥🚀",
                "bug",
                Priority::High,
                Effort::Small,
                None,
                vec![],
                None,
            )
            .unwrap();

        assert_eq!(item.title, "Fix login bug 🔥🚀");
        // Read it back and verify
        let doc: MarkplaneDocument<Task> = project.read_item(&item.id).unwrap();
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

    // ── body via typed update methods ──────────────────────────────────

    #[test]
    fn test_update_task_body() {
        let (_tmp, project) = setup_project();
        let task = project
            .create_task(
                "Test item",
                "feature",
                Priority::Medium,
                Effort::Small,
                None,
                vec![],
                None,
            )
            .unwrap();

        let original: MarkplaneDocument<Task> = project.read_item(&task.id).unwrap();
        assert!(original.body.contains("[What needs to be done"));

        project
            .update_task(
                &task.id,
                &TaskUpdate {
                    body: Some("# Test item\n\nActual description here.\n".into()),
                    ..Default::default()
                },
            )
            .unwrap();

        let updated: MarkplaneDocument<Task> = project.read_item(&task.id).unwrap();
        assert!(updated.body.contains("Actual description here."));
        assert_eq!(updated.frontmatter.id, task.id);
        assert_eq!(updated.frontmatter.title, "Test item");
    }

    #[test]
    fn test_update_epic_body() {
        let (_tmp, project) = setup_project();
        let epic = project
            .create_epic("Phase 1", Priority::High, None)
            .unwrap();

        project
            .update_epic(
                &epic.id,
                &EpicUpdate {
                    body: Some("# Phase 1\n\n## Objective\n\nBuild the foundation.\n".into()),
                    ..Default::default()
                },
            )
            .unwrap();

        let updated: MarkplaneDocument<Epic> = project.read_item(&epic.id).unwrap();
        assert!(updated.body.contains("Build the foundation."));
        assert_eq!(updated.frontmatter.id, epic.id);
    }

    #[test]
    fn test_update_plan_body() {
        let (_tmp, project) = setup_project();
        let plan = project.create_plan("Plan A", vec![], None).unwrap();

        project
            .update_plan(
                &plan.id,
                &PlanUpdate {
                    body: Some("# Plan A\n\nDetailed steps.\n".into()),
                    ..Default::default()
                },
            )
            .unwrap();

        let updated: MarkplaneDocument<Plan> = project.read_item(&plan.id).unwrap();
        assert!(updated.body.contains("Detailed steps."));
    }

    #[test]
    fn test_update_note_body() {
        let (_tmp, project) = setup_project();
        let note = project
            .create_note("Research A", "research", vec![], None)
            .unwrap();

        project
            .update_note(
                &note.id,
                &NoteUpdate {
                    body: Some("# Research A\n\nFindings here.\n".into()),
                    ..Default::default()
                },
            )
            .unwrap();

        let updated: MarkplaneDocument<Note> = project.read_item(&note.id).unwrap();
        assert!(updated.body.contains("Findings here."));
    }

    #[test]
    fn test_update_task_body_nonexistent() {
        let (_tmp, project) = setup_project();
        let result = project.update_task(
            "TASK-zzzzz",
            &TaskUpdate {
                body: Some("new body".into()),
                ..Default::default()
            },
        );
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

    #[test]
    fn test_list_documentation_files_rejects_traversal() {
        let (tmp, project) = setup_project();
        // repo_root = tmp.path() (parent of .markplane/)
        // Create a docs dir that exists but is reached via traversal outside repo root.
        // Place it inside the repo root but reference it via a path that escapes first.
        let docs_dir = tmp.path().join("docs");
        fs::create_dir_all(&docs_dir).unwrap();
        fs::write(docs_dir.join("secret.md"), "# Secret").unwrap();

        // Get the repo root dir name so we can construct a traversal that
        // escapes and re-enters: "../{dirname}/docs" canonicalizes to the
        // same place as "docs", but the non-canonical path leaves repo_root.
        // Actually, canonicalize() resolves this back inside, so it would pass.
        //
        // Instead, test with a truly outside directory:
        // Create a sibling directory next to the tmp dir
        let parent = tmp.path().parent().unwrap();
        let outside_name = format!("markplane_test_outside_{}", std::process::id());
        let outside_dir = parent.join(&outside_name);
        fs::create_dir_all(&outside_dir).unwrap();
        fs::write(outside_dir.join("secret.md"), "# Secret").unwrap();

        let mut config = project.load_config().unwrap();
        config.documentation_paths = vec![format!("../{outside_name}")];
        project.save_config(&config).unwrap();

        let docs = project.list_documentation_files().unwrap();
        // Clean up the outside dir before asserting
        let _ = fs::remove_dir_all(&outside_dir);
        assert!(docs.is_empty(), "Path traversal should be blocked");
    }

    // ── unarchive_item ──────────────────────────────────────────────────

    #[test]
    fn test_unarchive_item() {
        let (_tmp, project) = setup_project();
        let task = project
            .create_task(
                "To archive",
                "chore",
                Priority::Low,
                Effort::Xs,
                None,
                vec![],
                None,
            )
            .unwrap();

        project.archive_item(&task.id).unwrap();
        assert!(project.is_archived(&task.id).unwrap());

        project.unarchive_item(&task.id).unwrap();
        assert!(!project.is_archived(&task.id).unwrap());

        // Should still be readable
        let doc: MarkplaneDocument<Task> = project.read_item(&task.id).unwrap();
        assert_eq!(doc.frontmatter.title, "To archive");
    }

    #[test]
    fn test_unarchive_not_archived_errors() {
        let (_tmp, project) = setup_project();
        let task = project
            .create_task(
                "Active item",
                "feature",
                Priority::Medium,
                Effort::Small,
                None,
                vec![],
                None,
            )
            .unwrap();

        let result = project.unarchive_item(&task.id);
        assert!(result.is_err());
    }

    // ── is_archived ─────────────────────────────────────────────────────

    #[test]
    fn test_is_archived() {
        let (_tmp, project) = setup_project();
        let task = project
            .create_task(
                "Test",
                "feature",
                Priority::Medium,
                Effort::Small,
                None,
                vec![],
                None,
            )
            .unwrap();

        assert!(!project.is_archived(&task.id).unwrap());
        project.archive_item(&task.id).unwrap();
        assert!(project.is_archived(&task.id).unwrap());
    }

    // ── apply_tag_changes ──────────────────────────────────────────────

    #[test]
    fn test_apply_tag_changes_add() {
        let mut tags = vec!["a".to_string()];
        apply_tag_changes(&mut tags, &["b".to_string(), "c".to_string()], &[]);
        assert_eq!(tags, vec!["a", "b", "c"]);
    }

    #[test]
    fn test_apply_tag_changes_remove() {
        let mut tags = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        apply_tag_changes(&mut tags, &[], &["b".to_string()]);
        assert_eq!(tags, vec!["a", "c"]);
    }

    #[test]
    fn test_apply_tag_changes_add_and_remove() {
        let mut tags = vec!["a".to_string(), "b".to_string()];
        apply_tag_changes(&mut tags, &["c".to_string()], &["a".to_string()]);
        assert_eq!(tags, vec!["b", "c"]);
    }

    #[test]
    fn test_apply_tag_changes_no_duplicates() {
        let mut tags = vec!["a".to_string(), "b".to_string()];
        apply_tag_changes(&mut tags, &["a".to_string(), "b".to_string()], &[]);
        assert_eq!(tags, vec!["a", "b"]);
    }

    #[test]
    fn test_apply_tag_changes_remove_then_add_same() {
        let mut tags = vec!["a".to_string(), "b".to_string()];
        // Remove "a" then re-add "a" — should still have "a" (remove first, then add)
        apply_tag_changes(&mut tags, &["a".to_string()], &["a".to_string()]);
        assert_eq!(tags, vec!["b", "a"]);
    }

    // ── update_task ────────────────────────────────────────────────────

    #[test]
    fn test_update_task_title() {
        let (_tmp, project) = setup_project();
        let task = project
            .create_task(
                "Original",
                "feature",
                Priority::Medium,
                Effort::Small,
                None,
                vec![],
                None,
            )
            .unwrap();

        project
            .update_task(
                &task.id,
                &TaskUpdate {
                    title: Some("Updated title".to_string()),
                    ..Default::default()
                },
            )
            .unwrap();

        let doc: MarkplaneDocument<Task> = project.read_item(&task.id).unwrap();
        assert_eq!(doc.frontmatter.title, "Updated title");
    }

    #[test]
    fn test_update_task_multiple_fields() {
        let (_tmp, project) = setup_project();
        let task = project
            .create_task(
                "Multi",
                "feature",
                Priority::Low,
                Effort::Small,
                None,
                vec!["old".to_string()],
                None,
            )
            .unwrap();

        project
            .update_task(
                &task.id,
                &TaskUpdate {
                    priority: Some("high".to_string()),
                    effort: Some("large".to_string()),
                    item_type: Some("bug".to_string()),
                    assignee: Patch::Set("daniel".to_string()),
                    add_tags: vec!["new".to_string()],
                    remove_tags: vec!["old".to_string()],
                    ..Default::default()
                },
            )
            .unwrap();

        let doc: MarkplaneDocument<Task> = project.read_item(&task.id).unwrap();
        assert_eq!(doc.frontmatter.priority, Priority::High);
        assert_eq!(doc.frontmatter.effort, Effort::Large);
        assert_eq!(doc.frontmatter.item_type, "bug");
        assert_eq!(doc.frontmatter.assignee, Some("daniel".to_string()));
        assert_eq!(doc.frontmatter.tags, vec!["new"]);
    }

    #[test]
    fn test_update_task_clear_assignee() {
        let (_tmp, project) = setup_project();
        let task = project
            .create_task(
                "Clear test",
                "feature",
                Priority::Medium,
                Effort::Small,
                None,
                vec![],
                None,
            )
            .unwrap();

        // Set assignee first
        project
            .update_task(
                &task.id,
                &TaskUpdate {
                    assignee: Patch::Set("daniel".to_string()),
                    ..Default::default()
                },
            )
            .unwrap();
        let doc: MarkplaneDocument<Task> = project.read_item(&task.id).unwrap();
        assert_eq!(doc.frontmatter.assignee, Some("daniel".to_string()));

        // Clear it
        project
            .update_task(
                &task.id,
                &TaskUpdate {
                    assignee: Patch::Clear,
                    ..Default::default()
                },
            )
            .unwrap();
        let doc: MarkplaneDocument<Task> = project.read_item(&task.id).unwrap();
        assert_eq!(doc.frontmatter.assignee, None);
    }

    #[test]
    fn test_update_task_clear_position() {
        let (_tmp, project) = setup_project();
        let task = project
            .create_task(
                "Pos test",
                "feature",
                Priority::Medium,
                Effort::Small,
                None,
                vec![],
                None,
            )
            .unwrap();

        // Set position first
        project
            .update_task(
                &task.id,
                &TaskUpdate {
                    position: Patch::Set("aaa".to_string()),
                    ..Default::default()
                },
            )
            .unwrap();
        let doc: MarkplaneDocument<Task> = project.read_item(&task.id).unwrap();
        assert_eq!(doc.frontmatter.position, Some("aaa".to_string()));

        // Clear it
        project
            .update_task(
                &task.id,
                &TaskUpdate {
                    position: Patch::Clear,
                    ..Default::default()
                },
            )
            .unwrap();
        let doc: MarkplaneDocument<Task> = project.read_item(&task.id).unwrap();
        assert_eq!(doc.frontmatter.position, None);
    }

    #[test]
    fn test_update_epic_clear_started_and_target() {
        let (_tmp, project) = setup_project();
        let epic = project
            .create_epic("Date test", Priority::Medium, None)
            .unwrap();

        let start = chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
        let end = chrono::NaiveDate::from_ymd_opt(2026, 6, 1).unwrap();

        // Set dates
        project
            .update_epic(
                &epic.id,
                &EpicUpdate {
                    started: Patch::Set(start),
                    target: Patch::Set(end),
                    ..Default::default()
                },
            )
            .unwrap();
        let doc: MarkplaneDocument<Epic> = project.read_item(&epic.id).unwrap();
        assert_eq!(doc.frontmatter.started, Some(start));
        assert_eq!(doc.frontmatter.target, Some(end));

        // Clear started
        project
            .update_epic(
                &epic.id,
                &EpicUpdate {
                    started: Patch::Clear,
                    ..Default::default()
                },
            )
            .unwrap();
        let doc: MarkplaneDocument<Epic> = project.read_item(&epic.id).unwrap();
        assert_eq!(doc.frontmatter.started, None);
        assert_eq!(doc.frontmatter.target, Some(end));

        // Clear target
        project
            .update_epic(
                &epic.id,
                &EpicUpdate {
                    target: Patch::Clear,
                    ..Default::default()
                },
            )
            .unwrap();
        let doc: MarkplaneDocument<Epic> = project.read_item(&epic.id).unwrap();
        assert_eq!(doc.frontmatter.target, None);
    }

    #[test]
    fn test_update_task_invalid_status() {
        let (_tmp, project) = setup_project();
        let task = project
            .create_task(
                "Bad status",
                "feature",
                Priority::Medium,
                Effort::Small,
                None,
                vec![],
                None,
            )
            .unwrap();

        let result = project.update_task(
            &task.id,
            &TaskUpdate {
                status: Some("bogus".to_string()),
                ..Default::default()
            },
        );
        assert!(result.is_err());
    }

    // ── update_epic ────────────────────────────────────────────────────

    #[test]
    fn test_update_epic_fields() {
        let (_tmp, project) = setup_project();
        let epic = project
            .create_epic("Phase 1", Priority::Medium, None)
            .unwrap();

        let date = chrono::NaiveDate::from_ymd_opt(2026, 6, 1).unwrap();
        project
            .update_epic(
                &epic.id,
                &EpicUpdate {
                    title: Some("Phase 1 Updated".to_string()),
                    priority: Some("high".to_string()),
                    started: Patch::Set(chrono::NaiveDate::from_ymd_opt(2026, 2, 20).unwrap()),
                    target: Patch::Set(date),
                    add_tags: vec!["core".to_string()],
                    ..Default::default()
                },
            )
            .unwrap();

        let doc: MarkplaneDocument<Epic> = project.read_item(&epic.id).unwrap();
        assert_eq!(doc.frontmatter.title, "Phase 1 Updated");
        assert_eq!(doc.frontmatter.priority, Priority::High);
        assert!(doc.frontmatter.started.is_some());
        assert_eq!(doc.frontmatter.target, Some(date));
        assert_eq!(doc.frontmatter.tags, vec!["core"]);
    }

    // ── update_plan ────────────────────────────────────────────────────

    #[test]
    fn test_update_plan_fields() {
        let (_tmp, project) = setup_project();
        let plan = project.create_plan("Plan A", vec![], None).unwrap();

        project
            .update_plan(
                &plan.id,
                &PlanUpdate {
                    title: Some("Plan A v2".to_string()),
                    status: Some("approved".to_string()),
                    ..Default::default()
                },
            )
            .unwrap();

        let doc: MarkplaneDocument<Plan> = project.read_item(&plan.id).unwrap();
        assert_eq!(doc.frontmatter.title, "Plan A v2");
        assert_eq!(doc.frontmatter.status, PlanStatus::Approved);
    }

    // ── update_note ────────────────────────────────────────────────────

    #[test]
    fn test_update_note_fields() {
        let (_tmp, project) = setup_project();
        let note = project
            .create_note("Research", "idea", vec!["wip".to_string()], None)
            .unwrap();

        project
            .update_note(
                &note.id,
                &NoteUpdate {
                    title: Some("Decision: Use Redis".to_string()),
                    note_type: Some("decision".to_string()),
                    add_tags: vec!["arch".to_string()],
                    remove_tags: vec!["wip".to_string()],
                    ..Default::default()
                },
            )
            .unwrap();

        let doc: MarkplaneDocument<Note> = project.read_item(&note.id).unwrap();
        assert_eq!(doc.frontmatter.title, "Decision: Use Redis");
        assert_eq!(doc.frontmatter.note_type, "decision");
        assert_eq!(doc.frontmatter.tags, vec!["arch"]);
    }

    // ── update_item (generic dispatch) ─────────────────────────────────

    #[test]
    fn test_update_item_task() {
        let (_tmp, project) = setup_project();
        let task = project
            .create_task(
                "Dispatch test",
                "feature",
                Priority::Medium,
                Effort::Small,
                None,
                vec![],
                None,
            )
            .unwrap();

        project
            .update_item(
                &task.id,
                UpdateFields {
                    effort: Some("large".to_string()),
                    priority: Some("high".to_string()),
                    ..Default::default()
                },
            )
            .unwrap();

        let doc: MarkplaneDocument<Task> = project.read_item(&task.id).unwrap();
        assert_eq!(doc.frontmatter.effort, Effort::Large);
        assert_eq!(doc.frontmatter.priority, Priority::High);
    }

    #[test]
    fn test_update_item_rejects_invalid_field_for_type() {
        let (_tmp, project) = setup_project();
        let epic = project.create_epic("Epic", Priority::Medium, None).unwrap();

        // effort is not valid for epics
        let result = project.update_item(
            &epic.id,
            UpdateFields {
                effort: Some("large".to_string()),
                ..Default::default()
            },
        );
        assert!(result.is_err());

        let plan = project.create_plan("Plan", vec![], None).unwrap();

        // priority is not valid for plans
        let result = project.update_item(
            &plan.id,
            UpdateFields {
                priority: Some("high".to_string()),
                ..Default::default()
            },
        );
        assert!(result.is_err());

        let note = project.create_note("Note", "idea", vec![], None).unwrap();

        // assignee is not valid for notes
        let result = project.update_item(
            &note.id,
            UpdateFields {
                assignee: Patch::Set("someone".to_string()),
                ..Default::default()
            },
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_update_item_title_too_long() {
        let (_tmp, project) = setup_project();
        let task = project
            .create_task(
                "Title test",
                "feature",
                Priority::Medium,
                Effort::Small,
                None,
                vec![],
                None,
            )
            .unwrap();

        let long_title = "x".repeat(501);
        let result = project.update_item(
            &task.id,
            UpdateFields {
                title: Some(long_title),
                ..Default::default()
            },
        );
        assert!(result.is_err());
    }

    // ── Template resolution ────────────────────────────────────────────

    #[test]
    fn test_resolve_template_builtin_fallback() {
        let (_tmp, project) = setup_project();
        // Delete template files so we fall back to builtins
        let _ = fs::remove_file(project.root().join("templates/task.md"));
        let _ = fs::remove_file(project.root().join("templates/manifest.yaml"));

        let body = project.resolve_template_body("task", None, None).unwrap();
        assert!(body.contains("## Description"));
    }

    #[test]
    fn test_resolve_template_explicit_override() {
        let (_tmp, project) = setup_project();
        let body = project
            .resolve_template_body("task", Some("bug"), None)
            .unwrap();
        assert!(body.contains("## Steps to Reproduce"));
    }

    #[test]
    fn test_resolve_template_type_defaults() {
        let (_tmp, project) = setup_project();
        // With manifest present, bug type should resolve to bug template
        let body = project
            .resolve_template_body("task", None, Some("bug"))
            .unwrap();
        assert!(body.contains("## Steps to Reproduce"));
    }

    #[test]
    fn test_resolve_template_kind_default() {
        let (_tmp, project) = setup_project();
        // Plan kind default is "implementation"
        let body = project.resolve_template_body("plan", None, None).unwrap();
        assert!(body.contains("## Phases"));
    }

    #[test]
    fn test_resolve_template_reads_custom_file() {
        let (_tmp, project) = setup_project();
        // Write a custom template file
        fs::write(
            project.root().join("templates/task-custom.md"),
            "# {TITLE}\n\nCustom template body.\n",
        )
        .unwrap();

        let body = project
            .resolve_template_body("task", Some("custom"), None)
            .unwrap();
        assert!(body.contains("Custom template body."));
    }

    #[test]
    fn test_resolve_template_rejects_path_traversal() {
        let (_tmp, project) = setup_project();
        assert!(
            project
                .resolve_template_body("task", Some("x/../../README"), None)
                .is_err()
        );
        assert!(
            project
                .resolve_template_body("task", Some("../etc/passwd"), None)
                .is_err()
        );
        assert!(
            project
                .resolve_template_body("task", Some("foo\\bar"), None)
                .is_err()
        );
    }

    #[test]
    fn test_create_task_with_explicit_template() {
        let (_tmp, project) = setup_project();
        let item = project
            .create_task(
                "Bug report",
                "bug",
                Priority::High,
                Effort::Small,
                None,
                vec![],
                Some("bug"),
            )
            .unwrap();

        let doc: MarkplaneDocument<Task> = project.read_item(&item.id).unwrap();
        assert!(doc.body.contains("## Steps to Reproduce"));
    }

    #[test]
    fn test_create_plan_with_refactor_template() {
        let (_tmp, project) = setup_project();
        let plan = project
            .create_plan("Refactor auth", vec![], Some("refactor"))
            .unwrap();

        let doc: MarkplaneDocument<Plan> = project.read_item(&plan.id).unwrap();
        assert!(doc.body.contains("## Motivation"));
        assert!(doc.body.contains("## Current State"));
    }

    #[test]
    fn test_create_note_with_explicit_template() {
        let (_tmp, project) = setup_project();
        let note = project
            .create_note("Research notes", "idea", vec![], Some("research"))
            .unwrap();

        let doc: MarkplaneDocument<Note> = project.read_item(&note.id).unwrap();
        assert!(doc.body.contains("## Findings"));
    }

    #[test]
    fn test_init_generates_manifest_and_templates() {
        let (tmp, _project) = setup_project();
        let root = tmp.path().join(".markplane");

        // Verify manifest exists and is valid YAML
        let manifest_content = fs::read_to_string(root.join("templates/manifest.yaml")).unwrap();
        let manifest: crate::manifest::Manifest = serde_yaml::from_str(&manifest_content).unwrap();
        assert!(manifest.contains_key("task"));
        assert!(manifest.contains_key("epic"));
        assert!(manifest.contains_key("plan"));
        assert!(manifest.contains_key("note"));

        // Verify all 8 template files exist
        assert!(root.join("templates/task.md").is_file());
        assert!(root.join("templates/task-bug.md").is_file());
        assert!(root.join("templates/epic.md").is_file());
        assert!(root.join("templates/plan-implementation.md").is_file());
        assert!(root.join("templates/plan-refactor.md").is_file());
        assert!(root.join("templates/note.md").is_file());
        assert!(root.join("templates/note-research.md").is_file());
        assert!(root.join("templates/note-analysis.md").is_file());
    }

    // ── move_item tests ─────────────────────────────────────────────────

    /// Helper: create a task with a given priority and position.
    fn create_task_with_position(
        project: &Project,
        title: &str,
        priority: Priority,
        position: &str,
    ) -> String {
        let task = project
            .create_task(
                title,
                "feature",
                priority,
                Effort::Medium,
                None,
                vec![],
                None,
            )
            .unwrap();
        project
            .update_task(
                &task.id,
                &TaskUpdate {
                    position: Patch::Set(position.to_string()),
                    ..Default::default()
                },
            )
            .unwrap();
        task.id
    }

    /// Helper: read position from a task.
    fn get_position(project: &Project, id: &str) -> Option<String> {
        let doc: MarkplaneDocument<Task> = project.read_item(id).unwrap();
        doc.frontmatter.position.clone()
    }

    #[test]
    fn test_move_item_top() {
        let (_tmp, project) = setup_project();
        let t1 = create_task_with_position(&project, "First", Priority::High, "a2");
        let t2 = create_task_with_position(&project, "Second", Priority::High, "a5");
        let t3 = create_task_with_position(&project, "Third", Priority::High, "a8");

        project.move_item(&t3, MoveDirective::Top).unwrap();

        let p1 = get_position(&project, &t1).unwrap();
        let p2 = get_position(&project, &t2).unwrap();
        let p3 = get_position(&project, &t3).unwrap();
        assert!(p3 < p1, "t3 ({}) should be before t1 ({})", p3, p1);
        assert!(p1 < p2, "t1 ({}) should be before t2 ({})", p1, p2);
    }

    #[test]
    fn test_move_item_bottom() {
        let (_tmp, project) = setup_project();
        let t1 = create_task_with_position(&project, "First", Priority::High, "a2");
        let t2 = create_task_with_position(&project, "Second", Priority::High, "a5");
        let t3 = create_task_with_position(&project, "Third", Priority::High, "a8");

        project.move_item(&t1, MoveDirective::Bottom).unwrap();

        let p1 = get_position(&project, &t1).unwrap();
        let p2 = get_position(&project, &t2).unwrap();
        let p3 = get_position(&project, &t3).unwrap();
        assert!(p2 < p3, "t2 ({}) should be before t3 ({})", p2, p3);
        assert!(p3 < p1, "t3 ({}) should be before t1 ({})", p3, p1);
    }

    #[test]
    fn test_move_item_before() {
        let (_tmp, project) = setup_project();
        let t1 = create_task_with_position(&project, "First", Priority::High, "a2");
        let t2 = create_task_with_position(&project, "Second", Priority::High, "a5");
        let t3 = create_task_with_position(&project, "Third", Priority::High, "a8");

        project
            .move_item(&t3, MoveDirective::Before(t2.clone()))
            .unwrap();

        let p1 = get_position(&project, &t1).unwrap();
        let p2 = get_position(&project, &t2).unwrap();
        let p3 = get_position(&project, &t3).unwrap();
        assert!(p1 < p3, "t1 ({}) < t3 ({})", p1, p3);
        assert!(p3 < p2, "t3 ({}) < t2 ({})", p3, p2);
    }

    #[test]
    fn test_move_item_after() {
        let (_tmp, project) = setup_project();
        let t1 = create_task_with_position(&project, "First", Priority::High, "a2");
        let t2 = create_task_with_position(&project, "Second", Priority::High, "a5");
        let t3 = create_task_with_position(&project, "Third", Priority::High, "a8");

        project
            .move_item(&t1, MoveDirective::After(t2.clone()))
            .unwrap();

        let p1 = get_position(&project, &t1).unwrap();
        let p2 = get_position(&project, &t2).unwrap();
        let p3 = get_position(&project, &t3).unwrap();
        assert!(p2 < p1, "t2 ({}) < t1 ({})", p2, p1);
        assert!(p1 < p3, "t1 ({}) < t3 ({})", p1, p3);
    }

    #[test]
    fn test_move_item_single_task() {
        let (_tmp, project) = setup_project();
        let t1 = create_task_with_position(&project, "Only", Priority::High, "a5");

        project.move_item(&t1, MoveDirective::Top).unwrap();
        let pos = get_position(&project, &t1).unwrap();
        assert!(!pos.is_empty());

        project.move_item(&t1, MoveDirective::Bottom).unwrap();
        let pos = get_position(&project, &t1).unwrap();
        assert!(!pos.is_empty());
    }

    #[test]
    fn test_move_item_no_position_normalizes() {
        let (_tmp, project) = setup_project();
        // Create tasks without positions
        let t1 = project
            .create_task(
                "First",
                "feature",
                Priority::High,
                Effort::Small,
                None,
                vec![],
                None,
            )
            .unwrap()
            .id;
        let t2 = project
            .create_task(
                "Second",
                "feature",
                Priority::High,
                Effort::Small,
                None,
                vec![],
                None,
            )
            .unwrap()
            .id;

        // Move should normalize first, then position correctly
        project.move_item(&t2, MoveDirective::Top).unwrap();
        let p1 = get_position(&project, &t1).unwrap();
        let p2 = get_position(&project, &t2).unwrap();
        assert!(p2 < p1, "t2 ({}) should be before t1 ({})", p2, p1);
    }

    #[test]
    fn test_move_item_different_priority_error() {
        let (_tmp, project) = setup_project();
        let t_high = create_task_with_position(&project, "High", Priority::High, "a0");
        let t_low = create_task_with_position(&project, "Low", Priority::Low, "a0");

        let result = project.move_item(&t_high, MoveDirective::Before(t_low));
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("priority"),
            "error should mention priority: {}",
            err
        );
    }

    #[test]
    fn test_move_item_self_reference_error() {
        let (_tmp, project) = setup_project();
        let t1 = create_task_with_position(&project, "Task", Priority::High, "a0");

        let result = project.move_item(&t1, MoveDirective::Before(t1.clone()));
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("itself"), "error should mention self: {}", err);
    }

    #[test]
    fn test_move_item_nonexistent_target_error() {
        let (_tmp, project) = setup_project();
        let t1 = create_task_with_position(&project, "Task", Priority::High, "a0");

        let result = project.move_item(&t1, MoveDirective::After("TASK-zzzzz".to_string()));
        assert!(result.is_err());
    }

    #[test]
    fn test_move_item_non_task_error() {
        let (_tmp, project) = setup_project();
        let epic = project.create_epic("Epic", Priority::High, None).unwrap();

        let result = project.move_item(&epic.id, MoveDirective::Top);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("not a task"),
            "error should say not a task: {}",
            err
        );
    }

    #[test]
    fn test_seed_starter_content() {
        let (_tmp, project) = setup_project();
        let ids = project.seed_starter_content().unwrap();
        assert_eq!(ids.len(), 5, "should return 5 IDs");

        let epic_id = &ids[0];
        let setup_task_id = &ids[1];
        let import_task_id = &ids[2];
        let plan_id = &ids[3];
        let note_id = &ids[4];

        // Verify all items are readable
        let epic_doc: MarkplaneDocument<Epic> = project.read_item(epic_id).unwrap();
        assert_eq!(epic_doc.frontmatter.title, "Project Setup");
        assert_eq!(epic_doc.frontmatter.status, EpicStatus::Now);

        let setup_doc: MarkplaneDocument<Task> = project.read_item(setup_task_id).unwrap();
        assert_eq!(
            setup_doc.frontmatter.title,
            "Review and customize your markplane setup"
        );
        assert_eq!(setup_doc.frontmatter.status, "backlog");
        assert_eq!(
            setup_doc.frontmatter.epic.as_deref(),
            Some(epic_id.as_str())
        );
        assert!(
            setup_doc
                .frontmatter
                .tags
                .contains(&"onboarding".to_string())
        );

        let import_doc: MarkplaneDocument<Task> = project.read_item(import_task_id).unwrap();
        assert_eq!(
            import_doc.frontmatter.title,
            "Import existing work into markplane"
        );
        assert_eq!(import_doc.frontmatter.status, "backlog");
        assert_eq!(
            import_doc.frontmatter.epic.as_deref(),
            Some(epic_id.as_str())
        );
        assert_eq!(
            import_doc.frontmatter.plan.as_deref(),
            Some(plan_id.as_str())
        );

        let plan_doc: MarkplaneDocument<Plan> = project.read_item(plan_id).unwrap();
        assert_eq!(
            plan_doc.frontmatter.title,
            "Import existing work into markplane"
        );
        assert!(plan_doc.frontmatter.implements.contains(import_task_id));

        let note_doc: MarkplaneDocument<Note> = project.read_item(note_id).unwrap();
        assert_eq!(note_doc.frontmatter.title, "Project decisions");
        assert_eq!(note_doc.frontmatter.status, NoteStatus::Active);

        // Verify cross-references in body content
        assert!(epic_doc.body.contains(&format!("[[{}]]", setup_task_id)));
        assert!(epic_doc.body.contains(&format!("[[{}]]", import_task_id)));
        assert!(import_doc.body.contains(&format!("[[{}]]", plan_id)));
        assert!(plan_doc.body.contains(&format!("[[{}]]", import_task_id)));
        assert!(note_doc.body.contains(&format!("[[{}]]", epic_id)));

        // Verify items exist after sync
        project.sync_all().unwrap();
        let tasks = project.list_tasks(&Default::default()).unwrap();
        let epics = project.list_epics().unwrap();
        let plans = project.list_plans().unwrap();
        let notes = project.list_notes().unwrap();
        assert_eq!(epics.len(), 1);
        assert_eq!(tasks.len(), 2);
        assert_eq!(plans.len(), 1);
        assert_eq!(notes.len(), 1);
    }
}
