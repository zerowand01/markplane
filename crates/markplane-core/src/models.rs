use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::error::{MarkplaneError, Result};

// ── ID System ──────────────────────────────────────────────────────────────

/// Prefix for Markplane item IDs: EPIC, BACK, PLAN, NOTE.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum IdPrefix {
    Epic,
    Back,
    Plan,
    Note,
}

impl IdPrefix {
    /// The uppercase prefix string used in IDs (e.g. "EPIC").
    pub fn as_str(&self) -> &'static str {
        match self {
            IdPrefix::Epic => "EPIC",
            IdPrefix::Back => "BACK",
            IdPrefix::Plan => "PLAN",
            IdPrefix::Note => "NOTE",
        }
    }

    /// The directory name where items of this type live.
    pub fn directory(&self) -> &'static str {
        match self {
            IdPrefix::Epic => "roadmap",
            IdPrefix::Back => "backlog",
            IdPrefix::Plan => "plans",
            IdPrefix::Note => "notes",
        }
    }

    /// Parse a prefix from a string like "EPIC", "BACK", etc.
    pub fn parse(s: &str) -> Result<Self> {
        match s.to_uppercase().as_str() {
            "EPIC" => Ok(IdPrefix::Epic),
            "BACK" => Ok(IdPrefix::Back),
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

/// Parse an item ID string like "BACK-042" into its prefix and number.
pub fn parse_id(id: &str) -> Result<(IdPrefix, u32)> {
    let parts: Vec<&str> = id.splitn(2, '-').collect();
    if parts.len() != 2 {
        return Err(MarkplaneError::InvalidId(format!(
            "Expected format PREFIX-NUMBER, got: {}",
            id
        )));
    }
    let prefix = IdPrefix::parse(parts[0])?;
    let number = parts[1]
        .parse::<u32>()
        .map_err(|_| MarkplaneError::InvalidId(format!("Invalid number in ID: {}", id)))?;
    Ok((prefix, number))
}

/// Format a prefix and number into an ID string like "BACK-042".
pub fn format_id(prefix: &IdPrefix, number: u32) -> String {
    format!("{}-{:03}", prefix.as_str(), number)
}

// ── Status Enums ───────────────────────────────────────────────────────────

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum BacklogStatus {
    Draft,
    Backlog,
    Planned,
    InProgress,
    Done,
    Cancelled,
}

impl fmt::Display for BacklogStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BacklogStatus::Draft => write!(f, "draft"),
            BacklogStatus::Backlog => write!(f, "backlog"),
            BacklogStatus::Planned => write!(f, "planned"),
            BacklogStatus::InProgress => write!(f, "in-progress"),
            BacklogStatus::Done => write!(f, "done"),
            BacklogStatus::Cancelled => write!(f, "cancelled"),
        }
    }
}

impl FromStr for BacklogStatus {
    type Err = MarkplaneError;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "draft" => Ok(BacklogStatus::Draft),
            "backlog" => Ok(BacklogStatus::Backlog),
            "planned" => Ok(BacklogStatus::Planned),
            "in-progress" => Ok(BacklogStatus::InProgress),
            "done" => Ok(BacklogStatus::Done),
            "cancelled" => Ok(BacklogStatus::Cancelled),
            _ => Err(MarkplaneError::InvalidStatus(s.into())),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum EpicStatus {
    Planned,
    Active,
    Done,
    Paused,
}

impl fmt::Display for EpicStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EpicStatus::Planned => write!(f, "planned"),
            EpicStatus::Active => write!(f, "active"),
            EpicStatus::Done => write!(f, "done"),
            EpicStatus::Paused => write!(f, "paused"),
        }
    }
}

impl FromStr for EpicStatus {
    type Err = MarkplaneError;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "planned" => Ok(EpicStatus::Planned),
            "active" => Ok(EpicStatus::Active),
            "done" => Ok(EpicStatus::Done),
            "paused" => Ok(EpicStatus::Paused),
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
pub struct BacklogItem {
    pub id: String,
    pub title: String,
    pub status: BacklogStatus,
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
    pub counters: HashMap<String, u32>,
    pub context: ContextConfig,
    pub archive: ArchiveConfig,
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
        let mut counters = HashMap::new();
        counters.insert("EPIC".to_string(), 0);
        counters.insert("BACK".to_string(), 0);
        counters.insert("PLAN".to_string(), 0);
        counters.insert("NOTE".to_string(), 0);

        Config {
            version: 1,
            project: ProjectInfo {
                name: "My Project".to_string(),
                description: "Project description".to_string(),
            },
            counters,
            context: ContextConfig {
                token_budget: 1000,
                recent_days: 7,
                auto_generate: true,
            },
            archive: ArchiveConfig {
                auto_archive_after_days: 30,
                keep_cancelled: true,
            },
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
        assert_eq!(IdPrefix::parse("back").unwrap(), IdPrefix::Back);
        assert_eq!(IdPrefix::Epic.as_str(), "EPIC");
        assert_eq!(IdPrefix::Back.directory(), "backlog");
    }

    #[test]
    fn test_parse_id() {
        let (prefix, num) = parse_id("BACK-042").unwrap();
        assert_eq!(prefix, IdPrefix::Back);
        assert_eq!(num, 42);
    }

    #[test]
    fn test_format_id() {
        assert_eq!(format_id(&IdPrefix::Back, 42), "BACK-042");
        assert_eq!(format_id(&IdPrefix::Epic, 1), "EPIC-001");
    }

    #[test]
    fn test_backlog_status_display() {
        assert_eq!(BacklogStatus::InProgress.to_string(), "in-progress");
        assert_eq!(BacklogStatus::Draft.to_string(), "draft");
    }

    #[test]
    fn test_backlog_status_serde_roundtrip() {
        let yaml = serde_yaml::to_string(&BacklogStatus::InProgress).unwrap();
        assert!(yaml.contains("in-progress"));
        let parsed: BacklogStatus = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(parsed, BacklogStatus::InProgress);
    }

    #[test]
    fn test_backlog_item_serde() {
        let yaml = r#"
id: BACK-042
title: "Add dark mode"
status: in-progress
priority: high
type: feature
effort: medium
tags: [ui, theming]
epic: EPIC-003
plan: null
depends_on: [BACK-038]
blocks: [BACK-045]
assignee: daniel
created: 2026-01-15
updated: 2026-02-09
"#;
        let item: BacklogItem = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(item.id, "BACK-042");
        assert_eq!(item.status, BacklogStatus::InProgress);
        assert_eq!(item.item_type, ItemType::Feature);
        assert_eq!(item.tags, vec!["ui", "theming"]);
        assert_eq!(item.epic, Some("EPIC-003".to_string()));
        assert!(item.plan.is_none());
        assert_eq!(item.depends_on, vec!["BACK-038"]);

        // Round-trip
        let serialized = serde_yaml::to_string(&item).unwrap();
        let reparsed: BacklogItem = serde_yaml::from_str(&serialized).unwrap();
        assert_eq!(reparsed.id, item.id);
        assert_eq!(reparsed.status, item.status);
    }

    #[test]
    fn test_epic_serde() {
        let yaml = r#"
id: EPIC-003
title: "User Dashboard & Theming"
status: active
priority: high
started: 2026-01-01
target: null
tags: [frontend]
depends_on: [EPIC-001]
"#;
        let epic: Epic = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(epic.id, "EPIC-003");
        assert_eq!(epic.status, EpicStatus::Active);
        assert!(epic.started.is_some());
        assert!(epic.target.is_none());
    }

    #[test]
    fn test_plan_serde() {
        let yaml = r#"
id: PLAN-012
title: "Dark mode implementation"
status: in-progress
implements: [BACK-042, BACK-043]
epic: EPIC-003
created: 2026-02-01
updated: 2026-02-09
"#;
        let plan: Plan = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(plan.id, "PLAN-012");
        assert_eq!(plan.status, PlanStatus::InProgress);
        assert_eq!(plan.implements, vec!["BACK-042", "BACK-043"]);
    }

    #[test]
    fn test_note_serde() {
        let yaml = r#"
id: NOTE-007
title: "Caching strategies research"
type: research
status: active
tags: [cache, performance]
related: [BACK-042, PLAN-012]
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
        assert_eq!(config.counters.get("BACK"), Some(&0));
        assert_eq!(config.context.token_budget, 1000);
        assert!(config.archive.keep_cancelled);
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
        assert!(parse_id("BACK-").is_err());
    }

    #[test]
    fn test_parse_id_no_prefix() {
        assert!(parse_id("-042").is_err());
    }

    #[test]
    fn test_parse_id_lowercase() {
        // IdPrefix::parse does to_uppercase, so lowercase should work
        let (prefix, num) = parse_id("back-042").unwrap();
        assert_eq!(prefix, IdPrefix::Back);
        assert_eq!(num, 42);
    }

    #[test]
    fn test_parse_id_no_separator() {
        assert!(parse_id("BACK042").is_err());
    }

    #[test]
    fn test_parse_id_invalid_prefix() {
        assert!(parse_id("INVALID-042").is_err());
    }

    #[test]
    fn test_parse_id_all_prefixes() {
        let cases = [
            ("EPIC-001", IdPrefix::Epic, 1),
            ("BACK-042", IdPrefix::Back, 42),
            ("PLAN-003", IdPrefix::Plan, 3),
            ("NOTE-007", IdPrefix::Note, 7),
        ];
        for (input, expected_prefix, expected_num) in cases {
            let (prefix, num) = parse_id(input).unwrap();
            assert_eq!(prefix, expected_prefix);
            assert_eq!(num, expected_num);
        }
    }

    // ── Status updates for all types ─────────────────────────────────────

    #[test]
    fn test_epic_status_from_str() {
        assert_eq!("planned".parse::<EpicStatus>().unwrap(), EpicStatus::Planned);
        assert_eq!("active".parse::<EpicStatus>().unwrap(), EpicStatus::Active);
        assert_eq!("done".parse::<EpicStatus>().unwrap(), EpicStatus::Done);
        assert_eq!("paused".parse::<EpicStatus>().unwrap(), EpicStatus::Paused);
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
        for status in [EpicStatus::Planned, EpicStatus::Active, EpicStatus::Done, EpicStatus::Paused] {
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
