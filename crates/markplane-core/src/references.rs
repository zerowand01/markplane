use std::collections::{HashMap, HashSet};
use std::path::Path;

use crate::error::Result;
use crate::models::{parse_id, IdPrefix};
use crate::project::Project;

/// Return a glob pattern for scanning .md files in a directory.
/// Prefers items/ subdirectory if it exists, falls back to flat layout.
fn scan_pattern(dir: &Path) -> String {
    let items_dir = dir.join("items");
    let scan_dir = if items_dir.is_dir() { items_dir } else { dir.to_path_buf() };
    scan_dir.join("*.md").to_string_lossy().to_string()
}

/// A broken reference found during validation.
#[derive(Debug)]
pub struct BrokenReference {
    /// The file containing the broken reference.
    pub source_file: String,
    /// The referenced ID that could not be resolved.
    pub target_id: String,
}

/// Extract all `[[ID]]` references from content using marker scanning (no regex).
pub fn extract_references(content: &str) -> Vec<String> {
    let mut refs = Vec::new();
    let bytes = content.as_bytes();
    let len = bytes.len();
    let mut i = 0;

    while i + 1 < len {
        if bytes[i] == b'[' && bytes[i + 1] == b'[' {
            // Found opening [[
            let start = i + 2;
            if let Some(end_offset) = find_closing_brackets(&bytes[start..]) {
                let ref_str = &content[start..start + end_offset];
                // Only include if it looks like a valid ID (PREFIX-NUMBER)
                if parse_id(ref_str).is_ok() {
                    refs.push(ref_str.to_string());
                }
                i = start + end_offset + 2; // skip past ]]
            } else {
                i += 2;
            }
        } else {
            i += 1;
        }
    }

    refs
}

/// Find the position of `]]` in a byte slice, returning the offset of the first `]`.
fn find_closing_brackets(bytes: &[u8]) -> Option<usize> {
    let len = bytes.len();
    for j in 0..len.saturating_sub(1) {
        if bytes[j] == b']' && bytes[j + 1] == b']' {
            return Some(j);
        }
        // Don't allow newlines inside references
        if bytes[j] == b'\n' {
            return None;
        }
    }
    None
}

/// Validate all cross-references in the project.
/// Returns a list of broken references (IDs that don't resolve to existing files).
pub fn validate_references(project: &Project) -> Result<Vec<BrokenReference>> {
    let mut broken = Vec::new();

    // Scan all .md files in all directories
    let dirs = [
        IdPrefix::Epic,
        IdPrefix::Task,
        IdPrefix::Plan,
        IdPrefix::Note,
    ];

    for prefix in &dirs {
        let dir = project.item_dir(prefix);
        if !dir.exists() {
            continue;
        }
        let pattern = scan_pattern(&dir);
        for path in glob::glob(&pattern).unwrap_or_else(|_| glob::glob("").unwrap()).flatten() {
            let filename = path.file_name().unwrap_or_default().to_string_lossy();
            if filename == "INDEX.md" {
                continue;
            }
            let content = std::fs::read_to_string(&path)?;
            let refs = extract_references(&content);
            for ref_id in refs {
                if project.item_path(&ref_id).is_err() {
                    broken.push(BrokenReference {
                        source_file: path.to_string_lossy().to_string(),
                        target_id: ref_id,
                    });
                }
            }
        }
    }

    Ok(broken)
}

/// Find orphan items — items with no incoming references from other items.
pub fn find_orphans(project: &Project) -> Result<Vec<String>> {
    // Build a set of all item IDs and a set of all referenced IDs
    let mut all_ids: HashSet<String> = HashSet::new();
    let mut referenced_ids: HashSet<String> = HashSet::new();

    let dirs = [
        IdPrefix::Epic,
        IdPrefix::Task,
        IdPrefix::Plan,
        IdPrefix::Note,
    ];

    for prefix in &dirs {
        let dir = project.item_dir(prefix);
        if !dir.exists() {
            continue;
        }
        let pattern = scan_pattern(&dir);
        for path in glob::glob(&pattern).unwrap_or_else(|_| glob::glob("").unwrap()).flatten() {
            let filename = path.file_name().unwrap_or_default().to_string_lossy();
            if filename == "INDEX.md" || filename == "ideas.md" || filename == "decisions.md" {
                continue;
            }
            let stem = path.file_stem().unwrap_or_default().to_string_lossy();
            if parse_id(&stem).is_ok() {
                all_ids.insert(stem.to_string());
            }

            let content = std::fs::read_to_string(&path)?;
            for ref_id in extract_references(&content) {
                referenced_ids.insert(ref_id);
            }

            let content_refs = extract_frontmatter_references(&content);
            for ref_id in content_refs {
                referenced_ids.insert(ref_id);
            }
        }
    }

    // Orphans are IDs that exist but are never referenced
    let mut orphans: Vec<String> = all_ids
        .difference(&referenced_ids)
        .cloned()
        .collect();
    orphans.sort();
    Ok(orphans)
}

/// Extract references from frontmatter fields like `epic`, `plan`, `depends_on`,
/// `blocks`, `implements`, `related`.
fn extract_frontmatter_references(content: &str) -> Vec<String> {
    let mut refs = Vec::new();

    // Parse frontmatter region only
    let trimmed = content.trim_start();
    if !trimmed.starts_with("---") {
        return refs;
    }
    let after = &trimmed[3..];
    let after = after.strip_prefix('\n').unwrap_or(after);
    let end = after.find("\n---");
    let yaml_section = match end {
        Some(pos) => &after[..pos],
        None => return refs,
    };

    // Look for ID-like values in the frontmatter
    for line in yaml_section.lines() {
        // Check for fields that hold references
        let trimmed_line = line.trim();
        // Skip the id field itself
        if trimmed_line.starts_with("id:") {
            continue;
        }
        // Extract anything that looks like PREFIX-NNN
        extract_ids_from_line(trimmed_line, &mut refs);
    }

    refs
}

/// Extract ID patterns (PREFIX-SUFFIX) from a line of text.
fn extract_ids_from_line(line: &str, refs: &mut Vec<String>) {
    let prefixes = ["EPIC-", "TASK-", "PLAN-", "NOTE-"];
    for prefix in &prefixes {
        let mut start = 0;
        while let Some(pos) = line[start..].find(prefix) {
            let abs_pos = start + pos;
            let id_start = abs_pos;
            let after_prefix = abs_pos + prefix.len();
            // Collect alphanumeric chars after the prefix (handles both random and legacy IDs)
            let suffix_end = line[after_prefix..]
                .find(|c: char| !c.is_ascii_alphanumeric())
                .map(|p| after_prefix + p)
                .unwrap_or(line.len());
            if suffix_end > after_prefix {
                let id = &line[id_start..suffix_end];
                if parse_id(id).is_ok() {
                    refs.push(id.to_string());
                }
            }
            start = suffix_end;
        }
    }
}

/// Build a reference graph: maps each ID to the set of IDs it references.
pub fn build_reference_graph(project: &Project) -> Result<HashMap<String, Vec<String>>> {
    let mut graph: HashMap<String, Vec<String>> = HashMap::new();

    let dirs = [
        IdPrefix::Epic,
        IdPrefix::Task,
        IdPrefix::Plan,
        IdPrefix::Note,
    ];

    for prefix in &dirs {
        let dir = project.item_dir(prefix);
        if !dir.exists() {
            continue;
        }
        let pattern = scan_pattern(&dir);
        for path in glob::glob(&pattern).unwrap_or_else(|_| glob::glob("").unwrap()).flatten() {
            let filename = path.file_name().unwrap_or_default().to_string_lossy();
            if filename == "INDEX.md" || filename == "ideas.md" || filename == "decisions.md" {
                continue;
            }
            let stem = path.file_stem().unwrap_or_default().to_string_lossy().to_string();
            if parse_id(&stem).is_err() {
                continue;
            }

            let content = std::fs::read_to_string(&path)?;
            let mut all_refs: Vec<String> = extract_references(&content);
            all_refs.extend(extract_frontmatter_references(&content));
            all_refs.sort();
            all_refs.dedup();
            graph.insert(stem, all_refs);
        }
    }

    Ok(graph)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_references_basic() {
        let content = "See [[TASK-042]] and [[EPIC-003]] for details.";
        let refs = extract_references(content);
        assert_eq!(refs, vec!["TASK-042", "EPIC-003"]);
    }

    #[test]
    fn test_extract_references_empty() {
        let content = "No references here.";
        let refs = extract_references(content);
        assert!(refs.is_empty());
    }

    #[test]
    fn test_extract_references_multiline() {
        let content = "- [[TASK-001]]\n- [[PLAN-012]]\n- [[NOTE-007]]";
        let refs = extract_references(content);
        assert_eq!(refs, vec!["TASK-001", "PLAN-012", "NOTE-007"]);
    }

    #[test]
    fn test_extract_references_invalid_id() {
        let content = "[[INVALID-001]] and [[TASK-042]]";
        let refs = extract_references(content);
        assert_eq!(refs, vec!["TASK-042"]);
    }

    #[test]
    fn test_extract_references_nested_brackets() {
        let content = "[[TASK-042]] then [not a ref] then [[PLAN-001]]";
        let refs = extract_references(content);
        assert_eq!(refs, vec!["TASK-042", "PLAN-001"]);
    }

    #[test]
    fn test_extract_references_no_newline_in_ref() {
        let content = "[[TASK\n-042]]";
        let refs = extract_references(content);
        assert!(refs.is_empty());
    }

    #[test]
    fn test_extract_ids_from_line() {
        let mut refs = Vec::new();
        extract_ids_from_line("epic: EPIC-003", &mut refs);
        assert_eq!(refs, vec!["EPIC-003"]);
    }

    #[test]
    fn test_extract_ids_from_line_list() {
        let mut refs = Vec::new();
        extract_ids_from_line("implements: [TASK-042, TASK-043]", &mut refs);
        assert_eq!(refs, vec!["TASK-042", "TASK-043"]);
    }

    // ── extract_references edge cases ────────────────────────────────────

    #[test]
    fn test_extract_references_empty_brackets() {
        let content = "Empty ref: [[]] should not match.";
        let refs = extract_references(content);
        assert!(refs.is_empty());
    }

    #[test]
    fn test_extract_references_unclosed() {
        let content = "Unclosed [[TASK-001 never closed.";
        let refs = extract_references(content);
        assert!(refs.is_empty());
    }

    #[test]
    fn test_extract_references_at_start() {
        let content = "[[TASK-001]] is at the start.";
        let refs = extract_references(content);
        assert_eq!(refs, vec!["TASK-001"]);
    }

    #[test]
    fn test_extract_references_at_end() {
        let content = "Ref at the end: [[TASK-001]]";
        let refs = extract_references(content);
        assert_eq!(refs, vec!["TASK-001"]);
    }

    #[test]
    fn test_extract_references_adjacent() {
        let content = "[[TASK-001]][[PLAN-002]]";
        let refs = extract_references(content);
        assert_eq!(refs, vec!["TASK-001", "PLAN-002"]);
    }

    #[test]
    fn test_extract_references_single_char_content() {
        let content = "[[x]]";
        let refs = extract_references(content);
        assert!(refs.is_empty()); // "x" is not a valid ID
    }

    // ── validate_references ──────────────────────────────────────────────

    #[test]
    fn test_validate_references_all_valid() {
        use tempfile::TempDir;
        use crate::project::Project;
        use crate::models::{Priority, Effort};

        let tmp = TempDir::new().unwrap();
        let root = tmp.path().join(".markplane");
        let project = Project::init(root, "Test", "Test").unwrap();

        let task_a = project
            .create_task("Item A", "feature", Priority::Medium, Effort::Small, None, vec![], None)
            .unwrap();
        let task_b = project
            .create_task("Item B", "feature", Priority::Medium, Effort::Small, None, vec![], None)
            .unwrap();

        // Add a valid reference from task_a to task_b in the body
        let mut doc: crate::models::MarkplaneDocument<crate::models::Task> =
            project.read_item(&task_a.id).unwrap();
        doc.body = format!("# Item A\n\nSee [[{}]] for details.\n", task_b.id);
        project.write_item(&task_a.id, &doc).unwrap();

        let broken = validate_references(&project).unwrap();
        assert!(broken.is_empty());
    }

    #[test]
    fn test_validate_references_broken_ref() {
        use tempfile::TempDir;
        use crate::project::Project;
        use crate::models::{Priority, Effort};

        let tmp = TempDir::new().unwrap();
        let root = tmp.path().join(".markplane");
        let project = Project::init(root, "Test", "Test").unwrap();

        let task_a = project
            .create_task("Item A", "feature", Priority::Medium, Effort::Small, None, vec![], None)
            .unwrap();

        // Add a broken reference to a non-existent item
        let mut doc: crate::models::MarkplaneDocument<crate::models::Task> =
            project.read_item(&task_a.id).unwrap();
        doc.body = "# Item A\n\nSee [[TASK-zzzzz]] for details.\n".to_string();
        project.write_item(&task_a.id, &doc).unwrap();

        let broken = validate_references(&project).unwrap();
        assert_eq!(broken.len(), 1);
        assert_eq!(broken[0].target_id, "TASK-zzzzz");
    }

    // ── find_orphans ─────────────────────────────────────────────────────

    #[test]
    fn test_find_orphans_all_referenced() {
        use tempfile::TempDir;
        use crate::project::Project;
        use crate::models::{Priority, Effort};

        let tmp = TempDir::new().unwrap();
        let root = tmp.path().join(".markplane");
        let project = Project::init(root, "Test", "Test").unwrap();

        let task_a = project
            .create_task("Item A", "feature", Priority::Medium, Effort::Small, None, vec![], None)
            .unwrap();
        let task_b = project
            .create_task("Item B", "feature", Priority::Medium, Effort::Small, None, vec![], None)
            .unwrap();

        // A references B, B references A
        let mut doc_a: crate::models::MarkplaneDocument<crate::models::Task> =
            project.read_item(&task_a.id).unwrap();
        doc_a.body = format!("# A\nSee [[{}]]\n", task_b.id);
        project.write_item(&task_a.id, &doc_a).unwrap();

        let mut doc_b: crate::models::MarkplaneDocument<crate::models::Task> =
            project.read_item(&task_b.id).unwrap();
        doc_b.body = format!("# B\nSee [[{}]]\n", task_a.id);
        project.write_item(&task_b.id, &doc_b).unwrap();

        let orphans = find_orphans(&project).unwrap();
        assert!(orphans.is_empty());
    }

    #[test]
    fn test_find_orphans_with_orphan() {
        use tempfile::TempDir;
        use crate::project::Project;
        use crate::models::{Priority, Effort};

        let tmp = TempDir::new().unwrap();
        let root = tmp.path().join(".markplane");
        let project = Project::init(root, "Test", "Test").unwrap();

        let task_a = project
            .create_task("Referenced", "feature", Priority::Medium, Effort::Small, None, vec![], None)
            .unwrap();
        let _task_b = project
            .create_task("Orphan", "feature", Priority::Medium, Effort::Small, None, vec![], None)
            .unwrap();

        // Neither references the other — both are orphans
        let mut doc: crate::models::MarkplaneDocument<crate::models::Task> =
            project.read_item(&task_a.id).unwrap();
        doc.body = "# Referenced\nStandalone.\n".to_string();
        project.write_item(&task_a.id, &doc).unwrap();

        let orphans = find_orphans(&project).unwrap();
        // Both items are orphans since neither references the other
        assert!(!orphans.is_empty());
    }

    // ── build_reference_graph ────────────────────────────────────────────

    #[test]
    fn test_build_reference_graph() {
        use tempfile::TempDir;
        use crate::project::Project;
        use crate::models::{Priority, Effort};

        let tmp = TempDir::new().unwrap();
        let root = tmp.path().join(".markplane");
        let project = Project::init(root, "Test", "Test").unwrap();

        let task_a = project
            .create_task("A", "feature", Priority::Medium, Effort::Small, None, vec![], None)
            .unwrap();
        let task_b = project
            .create_task("B", "feature", Priority::Medium, Effort::Small, None, vec![], None)
            .unwrap();

        // A's body references B
        let mut doc: crate::models::MarkplaneDocument<crate::models::Task> =
            project.read_item(&task_a.id).unwrap();
        doc.body = format!("# A\n\nSee [[{}]].\n", task_b.id);
        project.write_item(&task_a.id, &doc).unwrap();

        let graph = build_reference_graph(&project).unwrap();
        assert!(graph.contains_key(task_a.id.as_str()));
        assert!(graph[task_a.id.as_str()].contains(&task_b.id));

        // B doesn't reference A
        if let Some(b_refs) = graph.get(task_b.id.as_str()) {
            assert!(!b_refs.contains(&task_a.id));
        }
    }

    #[test]
    fn test_build_reference_graph_empty() {
        use tempfile::TempDir;
        use crate::project::Project;

        let tmp = TempDir::new().unwrap();
        let root = tmp.path().join(".markplane");
        let project = Project::init(root, "Test", "Test").unwrap();

        let graph = build_reference_graph(&project).unwrap();
        assert!(graph.is_empty());
    }
}
