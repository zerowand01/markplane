//! Template manifest: customizable template resolution for Markplane.
//!
//! Templates are markdown body scaffolds stored in `.markplane/templates/`.
//! A `manifest.yaml` file in the templates directory maps item kinds and types
//! to template files. Built-in constants serve as fallback when no custom
//! templates exist.

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use serde::Deserialize;

use crate::templates;

/// Metadata about a template variant.
#[derive(Clone, Debug, Deserialize)]
pub struct TemplateInfo {
    pub description: String,
}

/// Configuration for templates of a given kind (task, epic, plan, note).
#[derive(Clone, Debug, Default, Deserialize)]
pub struct KindConfig {
    /// Default template name for this kind (used when no explicit or type-based match).
    pub default: Option<String>,
    /// Maps item sub-types (e.g. "bug", "research") to template names.
    #[serde(default)]
    pub type_defaults: HashMap<String, String>,
    /// Available templates for this kind.
    #[serde(default)]
    pub templates: HashMap<String, TemplateInfo>,
}

/// Full template manifest: maps kind names to their configuration.
pub type Manifest = HashMap<String, KindConfig>;

/// Load and parse `manifest.yaml` from the templates directory.
/// Returns `None` if the file doesn't exist, `Err` on parse failure.
pub fn load_manifest(root: &Path) -> Result<Option<Manifest>, String> {
    let path = root.join("templates/manifest.yaml");
    if !path.is_file() {
        return Ok(None);
    }
    let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let manifest: Manifest = serde_yaml::from_str(&content).map_err(|e| e.to_string())?;
    Ok(Some(manifest))
}

/// Validate that a template name contains only safe characters: `[a-zA-Z0-9_-]`.
/// Rejects names containing `/`, `\`, `..`, or any other characters that could
/// enable path traversal.
pub fn validate_template_name(name: &str) -> Result<(), String> {
    if name.is_empty() {
        return Err("Template name must not be empty".to_string());
    }
    if !name.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-') {
        return Err(format!(
            "Invalid template name '{name}': only [a-zA-Z0-9_-] allowed"
        ));
    }
    Ok(())
}

/// Map a (kind, template name) pair to a filename.
/// "default" → `"{kind}.md"`, anything else → `"{kind}-{name}.md"`.
///
/// The caller must validate `name` with [`validate_template_name`] first.
pub fn template_filename(kind: &str, name: &str) -> String {
    if name == "default" {
        format!("{kind}.md")
    } else {
        format!("{kind}-{name}.md")
    }
}

/// Look up a built-in template constant by (kind, name).
/// Falls back to the kind's default template for unknown names.
pub fn builtin_template(kind: &str, name: &str) -> &'static str {
    match (kind, name) {
        ("task", "bug") => templates::TASK_BUG_TEMPLATE,
        ("task", _) => templates::TASK_TEMPLATE,
        ("epic", _) => templates::EPIC_TEMPLATE,
        ("plan", "refactor") => templates::PLAN_REFACTOR_TEMPLATE,
        ("plan", _) => templates::PLAN_IMPLEMENTATION_TEMPLATE,
        ("note", "research") => templates::NOTE_RESEARCH_TEMPLATE,
        ("note", "analysis") => templates::NOTE_ANALYSIS_TEMPLATE,
        ("note", _) => templates::NOTE_GENERIC_TEMPLATE,
        _ => templates::TASK_TEMPLATE, // ultimate fallback
    }
}

/// Default manifest content written during `markplane init`.
pub const DEFAULT_MANIFEST: &str = r#"# Template Manifest
# Maps item kinds and types to template files in this directory.
# Built-in templates are used as fallback when files are missing.

task:
  default: default
  type_defaults:
    bug: bug
  templates:
    default:
      description: Standard task template
    bug:
      description: Bug report with reproduction steps

epic:
  default: default
  templates:
    default:
      description: Standard epic template

plan:
  default: implementation
  type_defaults:
    refactor: refactor
  templates:
    implementation:
      description: Implementation plan with phases
    refactor:
      description: Refactor plan with migration steps

note:
  default: default
  type_defaults:
    research: research
    analysis: analysis
  templates:
    default:
      description: Generic note
    research:
      description: Research note with findings
    analysis:
      description: Analysis note with conclusions
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_manifest() {
        let manifest: Manifest = serde_yaml::from_str(DEFAULT_MANIFEST).unwrap();

        assert!(manifest.contains_key("task"));
        assert!(manifest.contains_key("epic"));
        assert!(manifest.contains_key("plan"));
        assert!(manifest.contains_key("note"));

        let task = &manifest["task"];
        assert_eq!(task.default.as_deref(), Some("default"));
        assert_eq!(task.type_defaults.get("bug").map(|s| s.as_str()), Some("bug"));
        assert!(task.templates.contains_key("default"));
        assert!(task.templates.contains_key("bug"));
    }

    #[test]
    fn test_template_filename_default() {
        assert_eq!(template_filename("task", "default"), "task.md");
        assert_eq!(template_filename("epic", "default"), "epic.md");
    }

    #[test]
    fn test_template_filename_named() {
        assert_eq!(template_filename("task", "bug"), "task-bug.md");
        assert_eq!(template_filename("plan", "refactor"), "plan-refactor.md");
        assert_eq!(template_filename("note", "research"), "note-research.md");
    }

    #[test]
    fn test_builtin_template_task() {
        assert_eq!(builtin_template("task", "default"), templates::TASK_TEMPLATE);
        assert_eq!(builtin_template("task", "bug"), templates::TASK_BUG_TEMPLATE);
        assert_eq!(builtin_template("task", "anything"), templates::TASK_TEMPLATE);
    }

    #[test]
    fn test_builtin_template_epic() {
        assert_eq!(builtin_template("epic", "default"), templates::EPIC_TEMPLATE);
        assert_eq!(builtin_template("epic", "anything"), templates::EPIC_TEMPLATE);
    }

    #[test]
    fn test_builtin_template_plan() {
        assert_eq!(builtin_template("plan", "default"), templates::PLAN_IMPLEMENTATION_TEMPLATE);
        assert_eq!(builtin_template("plan", "implementation"), templates::PLAN_IMPLEMENTATION_TEMPLATE);
        assert_eq!(builtin_template("plan", "refactor"), templates::PLAN_REFACTOR_TEMPLATE);
    }

    #[test]
    fn test_builtin_template_note() {
        assert_eq!(builtin_template("note", "default"), templates::NOTE_GENERIC_TEMPLATE);
        assert_eq!(builtin_template("note", "research"), templates::NOTE_RESEARCH_TEMPLATE);
        assert_eq!(builtin_template("note", "analysis"), templates::NOTE_ANALYSIS_TEMPLATE);
        assert_eq!(builtin_template("note", "anything"), templates::NOTE_GENERIC_TEMPLATE);
    }

    #[test]
    fn test_validate_template_name_valid() {
        assert!(validate_template_name("default").is_ok());
        assert!(validate_template_name("bug").is_ok());
        assert!(validate_template_name("my-template").is_ok());
        assert!(validate_template_name("my_template").is_ok());
        assert!(validate_template_name("Template123").is_ok());
    }

    #[test]
    fn test_validate_template_name_rejects_traversal() {
        assert!(validate_template_name("x/../../README").is_err());
        assert!(validate_template_name("../etc/passwd").is_err());
        assert!(validate_template_name("..").is_err());
        assert!(validate_template_name("foo/bar").is_err());
        assert!(validate_template_name("foo\\bar").is_err());
        assert!(validate_template_name("").is_err());
        assert!(validate_template_name("foo.bar").is_err());
    }

    #[test]
    fn test_load_manifest_missing_file() {
        let tmp = tempfile::TempDir::new().unwrap();
        let result = load_manifest(tmp.path());
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_load_manifest_valid() {
        let tmp = tempfile::TempDir::new().unwrap();
        let templates_dir = tmp.path().join("templates");
        std::fs::create_dir_all(&templates_dir).unwrap();
        std::fs::write(templates_dir.join("manifest.yaml"), DEFAULT_MANIFEST).unwrap();

        let result = load_manifest(tmp.path());
        assert!(result.is_ok());
        let manifest = result.unwrap().unwrap();
        assert!(manifest.contains_key("task"));
    }
}
