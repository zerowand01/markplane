use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use rand::Rng;

use crate::error::{MarkplaneError, Result};

// ── ID System ──────────────────────────────────────────────────────────────

/// Alphabet for random ID suffixes: a-z minus o,l + 2-9 (32 chars, no ambiguous chars).
pub const RANDOM_ID_ALPHABET: &[u8] = b"abcdefghijkmnpqrstuvwxyz23456789";

/// Length of the random suffix in generated IDs.
pub const RANDOM_ID_LENGTH: usize = 5;

/// Prefix for Markplane item IDs: EPIC, TASK, PLAN, NOTE.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum IdPrefix {
    Epic,
    Task,
    Plan,
    Note,
}

impl IdPrefix {
    /// The uppercase prefix string used in IDs (e.g. "EPIC").
    pub fn as_str(&self) -> &'static str {
        match self {
            IdPrefix::Epic => "EPIC",
            IdPrefix::Task => "TASK",
            IdPrefix::Plan => "PLAN",
            IdPrefix::Note => "NOTE",
        }
    }

    /// The directory name where items of this type live.
    pub fn directory(&self) -> &'static str {
        match self {
            IdPrefix::Epic => "roadmap",
            IdPrefix::Task => "backlog",
            IdPrefix::Plan => "plans",
            IdPrefix::Note => "notes",
        }
    }

    /// Parse a prefix from a string like "EPIC", "TASK", etc.
    pub fn parse(s: &str) -> Result<Self> {
        match s.to_uppercase().as_str() {
            "EPIC" => Ok(IdPrefix::Epic),
            "TASK" => Ok(IdPrefix::Task),
            "PLAN" => Ok(IdPrefix::Plan),
            "NOTE" => Ok(IdPrefix::Note),
            _ => Err(MarkplaneError::InvalidId(format!(
                "Unknown prefix: {}",
                s
            ))),
        }
    }
}

impl fmt::Display for IdPrefix {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Parse an item ID string like "TASK-k7x9m" into its prefix and suffix.
/// Accepts both random IDs (5 alphanumeric chars) and legacy sequential IDs (digits).
pub fn parse_id(id: &str) -> Result<(IdPrefix, &str)> {
    let parts: Vec<&str> = id.splitn(2, '-').collect();
    if parts.len() != 2 {
        return Err(MarkplaneError::InvalidId(format!(
            "Expected format PREFIX-SUFFIX, got: {}",
            id
        )));
    }
    let prefix = IdPrefix::parse(parts[0])?;
    let suffix = parts[1];
    if suffix.is_empty() {
        return Err(MarkplaneError::InvalidId(format!(
            "Empty suffix in ID: {}",
            id
        )));
    }
    // Validate suffix: all chars must be alphanumeric (covers both random and legacy formats)
    if !suffix.bytes().all(|b| b.is_ascii_alphanumeric()) {
        return Err(MarkplaneError::InvalidId(format!(
            "Invalid suffix in ID: {}",
            id
        )));
    }
    Ok((prefix, suffix))
}

/// Generate a random ID like "TASK-k7x9m" using the random alphabet.
pub fn generate_random_id(prefix: &IdPrefix) -> String {
    let mut rng = rand::rng();
    let suffix: String = (0..RANDOM_ID_LENGTH)
        .map(|_| {
            let idx = rng.random_range(0..RANDOM_ID_ALPHABET.len());
            RANDOM_ID_ALPHABET[idx] as char
        })
        .collect();
    format!("{}-{}", prefix.as_str(), suffix)
}

// ── Status Enums ───────────────────────────────────────────────────────────

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TaskStatus {
    Draft,
    Backlog,
    Planned,
    InProgress,
    Done,
    Cancelled,
}

impl fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TaskStatus::Draft => write!(f, "draft"),
            TaskStatus::Backlog => write!(f, "backlog"),
            TaskStatus::Planned => write!(f, "planned"),
            TaskStatus::InProgress => write!(f, "in-progress"),
            TaskStatus::Done => write!(f, "done"),
            TaskStatus::Cancelled => write!(f, "cancelled"),
        }
    }
}

impl FromStr for TaskStatus {
    type Err = MarkplaneError;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "draft" => Ok(TaskStatus::Draft),
            "backlog" => Ok(TaskStatus::Backlog),
            "planned" => Ok(TaskStatus::Planned),
            "in-progress" => Ok(TaskStatus::InProgress),
            "done" => Ok(TaskStatus::Done),
            "cancelled" => Ok(TaskStatus::Cancelled),
            _ => Err(MarkplaneError::InvalidStatus(s.into())),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum EpicStatus {
    Now,
    Next,
    Later,
    Done,
}

impl fmt::Display for EpicStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EpicStatus::Now => write!(f, "now"),
            EpicStatus::Next => write!(f, "next"),
            EpicStatus::Later => write!(f, "later"),
            EpicStatus::Done => write!(f, "done"),
        }
    }
}

impl FromStr for EpicStatus {
    type Err = MarkplaneError;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "now" => Ok(EpicStatus::Now),
            "next" => Ok(EpicStatus::Next),
            "later" => Ok(EpicStatus::Later),
            "done" => Ok(EpicStatus::Done),
            _ => Err(MarkplaneError::InvalidStatus(s.into())),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PlanStatus {
    Draft,
    Approved,
    InProgress,
    Done,
}

impl fmt::Display for PlanStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PlanStatus::Draft => write!(f, "draft"),
            PlanStatus::Approved => write!(f, "approved"),
            PlanStatus::InProgress => write!(f, "in-progress"),
            PlanStatus::Done => write!(f, "done"),
        }
    }
}

impl FromStr for PlanStatus {
    type Err = MarkplaneError;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "draft" => Ok(PlanStatus::Draft),
            "approved" => Ok(PlanStatus::Approved),
            "in-progress" => Ok(PlanStatus::InProgress),
            "done" => Ok(PlanStatus::Done),
            _ => Err(MarkplaneError::InvalidStatus(s.into())),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum NoteStatus {
    Draft,
    Active,
    Archived,
}

impl fmt::Display for NoteStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NoteStatus::Draft => write!(f, "draft"),
            NoteStatus::Active => write!(f, "active"),
            NoteStatus::Archived => write!(f, "archived"),
        }
    }
}

impl FromStr for NoteStatus {
    type Err = MarkplaneError;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "draft" => Ok(NoteStatus::Draft),
            "active" => Ok(NoteStatus::Active),
            "archived" => Ok(NoteStatus::Archived),
            _ => Err(MarkplaneError::InvalidStatus(s.into())),
        }
    }
}

// ── Classification Enums ───────────────────────────────────────────────────

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Priority {
    Critical,
    High,
    Medium,
    Low,
    Someday,
}

impl fmt::Display for Priority {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Priority::Critical => write!(f, "critical"),
            Priority::High => write!(f, "high"),
            Priority::Medium => write!(f, "medium"),
            Priority::Low => write!(f, "low"),
            Priority::Someday => write!(f, "someday"),
        }
    }
}

impl FromStr for Priority {
    type Err = MarkplaneError;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "critical" => Ok(Priority::Critical),
            "high" => Ok(Priority::High),
            "medium" => Ok(Priority::Medium),
            "low" => Ok(Priority::Low),
            "someday" => Ok(Priority::Someday),
            _ => Err(MarkplaneError::Config(format!(
                "Unknown priority: {}",
                s
            ))),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ItemType {
    Feature,
    Bug,
    Enhancement,
    Chore,
    Research,
    Spike,
}

impl fmt::Display for ItemType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ItemType::Feature => write!(f, "feature"),
            ItemType::Bug => write!(f, "bug"),
            ItemType::Enhancement => write!(f, "enhancement"),
            ItemType::Chore => write!(f, "chore"),
            ItemType::Research => write!(f, "research"),
            ItemType::Spike => write!(f, "spike"),
        }
    }
}

impl FromStr for ItemType {
    type Err = MarkplaneError;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "feature" => Ok(ItemType::Feature),
            "bug" => Ok(ItemType::Bug),
            "enhancement" => Ok(ItemType::Enhancement),
            "chore" => Ok(ItemType::Chore),
            "research" => Ok(ItemType::Research),
            "spike" => Ok(ItemType::Spike),
            _ => Err(MarkplaneError::Config(format!(
                "Unknown item type: {}",
                s
            ))),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Effort {
    Xs,
    Small,
    Medium,
    Large,
    Xl,
}

impl fmt::Display for Effort {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Effort::Xs => write!(f, "xs"),
            Effort::Small => write!(f, "small"),
            Effort::Medium => write!(f, "medium"),
            Effort::Large => write!(f, "large"),
            Effort::Xl => write!(f, "xl"),
        }
    }
}

impl FromStr for Effort {
    type Err = MarkplaneError;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "xs" => Ok(Effort::Xs),
            "small" => Ok(Effort::Small),
            "medium" => Ok(Effort::Medium),
            "large" => Ok(Effort::Large),
            "xl" => Ok(Effort::Xl),
            _ => Err(MarkplaneError::Config(format!(
                "Unknown effort size: {}",
                s
            ))),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum NoteType {
    Research,
    Analysis,
    Idea,
    Decision,
    Meeting,
}

impl fmt::Display for NoteType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NoteType::Research => write!(f, "research"),
            NoteType::Analysis => write!(f, "analysis"),
            NoteType::Idea => write!(f, "idea"),
            NoteType::Decision => write!(f, "decision"),
            NoteType::Meeting => write!(f, "meeting"),
        }
    }
}

impl FromStr for NoteType {
    type Err = MarkplaneError;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "research" => Ok(NoteType::Research),
            "analysis" => Ok(NoteType::Analysis),
            "idea" => Ok(NoteType::Idea),
            "decision" => Ok(NoteType::Decision),
            "meeting" => Ok(NoteType::Meeting),
            _ => Err(MarkplaneError::Config(format!(
                "Unknown note type: {}",
                s
            ))),
        }
    }
}

// ── Entity Structs ─────────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub title: String,
    pub status: TaskStatus,
    pub priority: Priority,
    #[serde(rename = "type")]
    pub item_type: ItemType,
    pub effort: Effort,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub epic: Option<String>,
    #[serde(default)]
    pub plan: Option<String>,
    #[serde(default)]
    pub depends_on: Vec<String>,
    #[serde(default)]
    pub blocks: Vec<String>,
    #[serde(default)]
    pub assignee: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub position: Option<String>,
    pub created: NaiveDate,
    pub updated: NaiveDate,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Epic {
    pub id: String,
    pub title: String,
    pub status: EpicStatus,
    pub priority: Priority,
    #[serde(default)]
    pub started: Option<NaiveDate>,
    #[serde(default)]
    pub target: Option<NaiveDate>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub depends_on: Vec<String>,
    pub created: NaiveDate,
    pub updated: NaiveDate,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Plan {
    pub id: String,
    pub title: String,
    pub status: PlanStatus,
    #[serde(default)]
    pub implements: Vec<String>,
    #[serde(default)]
    pub epic: Option<String>,
    pub created: NaiveDate,
    pub updated: NaiveDate,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Note {
    pub id: String,
    pub title: String,
    #[serde(rename = "type")]
    pub note_type: NoteType,
    pub status: NoteStatus,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub related: Vec<String>,
    pub created: NaiveDate,
    pub updated: NaiveDate,
}

// ── Config ─────────────────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    pub version: u32,
    pub project: ProjectInfo,
    /// Legacy counter field — silently ignored on read, never written.
    #[serde(default, skip_serializing)]
    pub counters: Option<HashMap<String, u32>>,
    pub context: ContextConfig,
    /// Deprecated: archive is now an explicit operation, not time-based.
    /// This field is kept for backward compatibility with existing config files
    /// but silently dropped on next write.
    #[serde(default, skip_serializing)]
    pub archive: Option<ArchiveConfig>,
    #[serde(default)]
    pub documentation_paths: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProjectInfo {
    pub name: String,
    pub description: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ContextConfig {
    pub token_budget: u32,
    pub recent_days: u32,
    pub auto_generate: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ArchiveConfig {
    pub auto_archive_after_days: u32,
    pub keep_cancelled: bool,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            version: 1,
            project: ProjectInfo {
                name: "My Project".to_string(),
                description: "Project description".to_string(),
            },
            counters: None,
            context: ContextConfig {
                token_budget: 1000,
                recent_days: 7,
                auto_generate: true,
            },
            archive: None,
            documentation_paths: Vec::new(),
        }
    }
}

// ── Document Wrapper ───────────────────────────────────────────────────────

/// Generic wrapper for any Markplane entity with its markdown body.
#[derive(Clone, Debug)]
pub struct MarkplaneDocument<T> {
    pub frontmatter: T,
    pub body: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_id_prefix_roundtrip() {
        assert_eq!(IdPrefix::parse("EPIC").unwrap(), IdPrefix::Epic);
        assert_eq!(IdPrefix::parse("task").unwrap(), IdPrefix::Task);
        assert_eq!(IdPrefix::Epic.as_str(), "EPIC");
        assert_eq!(IdPrefix::Task.as_str(), "TASK");
        assert_eq!(IdPrefix::Task.directory(), "backlog");
    }

    #[test]
    fn test_parse_id_legacy() {
        let (prefix, suffix) = parse_id("TASK-042").unwrap();
        assert_eq!(prefix, IdPrefix::Task);
        assert_eq!(suffix, "042");
    }

    #[test]
    fn test_parse_id_random() {
        let (prefix, suffix) = parse_id("TASK-k7x9m").unwrap();
        assert_eq!(prefix, IdPrefix::Task);
        assert_eq!(suffix, "k7x9m");
    }

    #[test]
    fn test_generate_random_id() {
        let id = generate_random_id(&IdPrefix::Task);
        assert!(id.starts_with("TASK-"));
        let (prefix, suffix) = parse_id(&id).unwrap();
        assert_eq!(prefix, IdPrefix::Task);
        assert_eq!(suffix.len(), RANDOM_ID_LENGTH);
        // All chars should be from the alphabet
        for c in suffix.bytes() {
            assert!(RANDOM_ID_ALPHABET.contains(&c), "Invalid char: {}", c as char);
        }
    }

    #[test]
    fn test_generate_random_id_uniqueness() {
        use std::collections::HashSet;
        let mut ids = HashSet::new();
        for _ in 0..1000 {
            let id = generate_random_id(&IdPrefix::Task);
            assert!(ids.insert(id), "Duplicate ID generated");
        }
    }

    #[test]
    fn test_task_status_display() {
        assert_eq!(TaskStatus::InProgress.to_string(), "in-progress");
        assert_eq!(TaskStatus::Draft.to_string(), "draft");
    }

    #[test]
    fn test_task_status_serde_roundtrip() {
        let yaml = serde_yaml::to_string(&TaskStatus::InProgress).unwrap();
        assert!(yaml.contains("in-progress"));
        let parsed: TaskStatus = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(parsed, TaskStatus::InProgress);
    }

    #[test]
    fn test_task_serde() {
        let yaml = r#"
id: TASK-042
title: "Add dark mode"
status: in-progress
priority: high
type: feature
effort: medium
tags: [ui, theming]
epic: EPIC-003
plan: null
depends_on: [TASK-038]
blocks: [TASK-045]
assignee: daniel
created: 2026-01-15
updated: 2026-02-09
"#;
        let item: Task = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(item.id, "TASK-042");
        assert_eq!(item.status, TaskStatus::InProgress);
        assert_eq!(item.item_type, ItemType::Feature);
        assert_eq!(item.tags, vec!["ui", "theming"]);
        assert_eq!(item.epic, Some("EPIC-003".to_string()));
        assert!(item.plan.is_none());
        assert_eq!(item.depends_on, vec!["TASK-038"]);

        // Round-trip
        let serialized = serde_yaml::to_string(&item).unwrap();
        let reparsed: Task = serde_yaml::from_str(&serialized).unwrap();
        assert_eq!(reparsed.id, item.id);
        assert_eq!(reparsed.status, item.status);
    }

    #[test]
    fn test_epic_serde() {
        let yaml = r#"
id: EPIC-003
title: "User Dashboard & Theming"
status: now
priority: high
started: 2026-01-01
target: null
tags: [frontend]
depends_on: [EPIC-001]
created: 2026-01-01
updated: 2026-02-09
"#;
        let epic: Epic = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(epic.id, "EPIC-003");
        assert_eq!(epic.status, EpicStatus::Now);
        assert!(epic.started.is_some());
        assert!(epic.target.is_none());
        assert_eq!(epic.created, NaiveDate::from_ymd_opt(2026, 1, 1).unwrap());
        assert_eq!(epic.updated, NaiveDate::from_ymd_opt(2026, 2, 9).unwrap());
    }

    #[test]
    fn test_plan_serde() {
        let yaml = r#"
id: PLAN-012
title: "Dark mode implementation"
status: in-progress
implements: [TASK-042, TASK-043]
epic: EPIC-003
created: 2026-02-01
updated: 2026-02-09
"#;
        let plan: Plan = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(plan.id, "PLAN-012");
        assert_eq!(plan.status, PlanStatus::InProgress);
        assert_eq!(plan.implements, vec!["TASK-042", "TASK-043"]);
    }

    #[test]
    fn test_note_serde() {
        let yaml = r#"
id: NOTE-007
title: "Caching strategies research"
type: research
status: active
tags: [cache, performance]
related: [TASK-042, PLAN-012]
created: 2026-02-05
updated: 2026-02-09
"#;
        let note: Note = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(note.id, "NOTE-007");
        assert_eq!(note.note_type, NoteType::Research);
        assert_eq!(note.status, NoteStatus::Active);
    }

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert_eq!(config.version, 1);
        assert!(config.counters.is_none());
        assert_eq!(config.context.token_budget, 1000);
        assert!(config.archive.is_none());
    }

    #[test]
    fn test_config_serde() {
        let config = Config::default();
        let yaml = serde_yaml::to_string(&config).unwrap();
        let reparsed: Config = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(reparsed.version, config.version);
        assert_eq!(reparsed.project.name, config.project.name);
    }

    // ── parse_id edge cases ──────────────────────────────────────────────

    #[test]
    fn test_parse_id_empty_string() {
        assert!(parse_id("").is_err());
    }

    #[test]
    fn test_parse_id_no_number() {
        assert!(parse_id("TASK-").is_err());
    }

    #[test]
    fn test_parse_id_no_prefix() {
        assert!(parse_id("-042").is_err());
    }

    #[test]
    fn test_parse_id_lowercase() {
        // IdPrefix::parse does to_uppercase, so lowercase should work
        let (prefix, suffix) = parse_id("task-042").unwrap();
        assert_eq!(prefix, IdPrefix::Task);
        assert_eq!(suffix, "042");
    }

    #[test]
    fn test_parse_id_no_separator() {
        assert!(parse_id("TASK042").is_err());
    }

    #[test]
    fn test_parse_id_invalid_prefix() {
        assert!(parse_id("INVALID-042").is_err());
    }

    #[test]
    fn test_parse_id_all_prefixes() {
        let cases = [
            ("EPIC-001", IdPrefix::Epic, "001"),
            ("TASK-042", IdPrefix::Task, "042"),
            ("PLAN-003", IdPrefix::Plan, "003"),
            ("NOTE-007", IdPrefix::Note, "007"),
            ("TASK-k7x9m", IdPrefix::Task, "k7x9m"),
        ];
        for (input, expected_prefix, expected_suffix) in cases {
            let (prefix, suffix) = parse_id(input).unwrap();
            assert_eq!(prefix, expected_prefix);
            assert_eq!(suffix, expected_suffix);
        }
    }

    // ── Status updates for all types ─────────────────────────────────────

    #[test]
    fn test_epic_status_from_str() {
        assert_eq!("now".parse::<EpicStatus>().unwrap(), EpicStatus::Now);
        assert_eq!("next".parse::<EpicStatus>().unwrap(), EpicStatus::Next);
        assert_eq!("later".parse::<EpicStatus>().unwrap(), EpicStatus::Later);
        assert_eq!("done".parse::<EpicStatus>().unwrap(), EpicStatus::Done);
        assert!("planned".parse::<EpicStatus>().is_err());
        assert!("active".parse::<EpicStatus>().is_err());
        assert!("invalid".parse::<EpicStatus>().is_err());
    }

    #[test]
    fn test_plan_status_from_str() {
        assert_eq!("draft".parse::<PlanStatus>().unwrap(), PlanStatus::Draft);
        assert_eq!("approved".parse::<PlanStatus>().unwrap(), PlanStatus::Approved);
        assert_eq!("in-progress".parse::<PlanStatus>().unwrap(), PlanStatus::InProgress);
        assert_eq!("done".parse::<PlanStatus>().unwrap(), PlanStatus::Done);
        assert!("invalid".parse::<PlanStatus>().is_err());
    }

    #[test]
    fn test_note_status_from_str() {
        assert_eq!("draft".parse::<NoteStatus>().unwrap(), NoteStatus::Draft);
        assert_eq!("active".parse::<NoteStatus>().unwrap(), NoteStatus::Active);
        assert_eq!("archived".parse::<NoteStatus>().unwrap(), NoteStatus::Archived);
        assert!("invalid".parse::<NoteStatus>().is_err());
    }

    #[test]
    fn test_epic_status_display_roundtrip() {
        for status in [EpicStatus::Now, EpicStatus::Next, EpicStatus::Later, EpicStatus::Done] {
            let s = status.to_string();
            let parsed: EpicStatus = s.parse().unwrap();
            assert_eq!(parsed, status);
        }
    }

    #[test]
    fn test_plan_status_display_roundtrip() {
        for status in [PlanStatus::Draft, PlanStatus::Approved, PlanStatus::InProgress, PlanStatus::Done] {
            let s = status.to_string();
            let parsed: PlanStatus = s.parse().unwrap();
            assert_eq!(parsed, status);
        }
    }

    #[test]
    fn test_note_status_display_roundtrip() {
        for status in [NoteStatus::Draft, NoteStatus::Active, NoteStatus::Archived] {
            let s = status.to_string();
            let parsed: NoteStatus = s.parse().unwrap();
            assert_eq!(parsed, status);
        }
    }
}
