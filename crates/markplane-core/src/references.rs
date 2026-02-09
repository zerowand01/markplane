use std::collections::{HashMap, HashSet};

use crate::error::Result;
use crate::models::{parse_id, IdPrefix};
use crate::project::Project;

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
        IdPrefix::Back,
        IdPrefix::Plan,
        IdPrefix::Note,
    ];

    for prefix in &dirs {
        let dir = project.item_dir(prefix);
        if !dir.exists() {
            continue;
        }
        let pattern = dir.join("*.md").to_string_lossy().to_string();
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
        IdPrefix::Back,
        IdPrefix::Plan,
        IdPrefix::Note,
    ];

    for prefix in &dirs {
        let dir = project.item_dir(prefix);
        if !dir.exists() {
            continue;
        }
        let pattern = dir.join("*.md").to_string_lossy().to_string();
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

/// Extract ID patterns (PREFIX-NNN) from a line of text.
fn extract_ids_from_line(line: &str, refs: &mut Vec<String>) {
    let prefixes = ["EPIC-", "BACK-", "PLAN-", "NOTE-"];
    for prefix in &prefixes {
        let mut start = 0;
        while let Some(pos) = line[start..].find(prefix) {
            let abs_pos = start + pos;
            let id_start = abs_pos;
            let after_prefix = abs_pos + prefix.len();
            // Collect digits after the prefix
            let num_end = line[after_prefix..]
                .find(|c: char| !c.is_ascii_digit())
                .map(|p| after_prefix + p)
                .unwrap_or(line.len());
            if num_end > after_prefix {
                let id = &line[id_start..num_end];
                if parse_id(id).is_ok() {
                    refs.push(id.to_string());
                }
            }
            start = num_end;
        }
    }
}

/// Build a reference graph: maps each ID to the set of IDs it references.
pub fn build_reference_graph(project: &Project) -> Result<HashMap<String, Vec<String>>> {
    let mut graph: HashMap<String, Vec<String>> = HashMap::new();

    let dirs = [
        IdPrefix::Epic,
        IdPrefix::Back,
        IdPrefix::Plan,
        IdPrefix::Note,
    ];

    for prefix in &dirs {
        let dir = project.item_dir(prefix);
        if !dir.exists() {
            continue;
        }
        let pattern = dir.join("*.md").to_string_lossy().to_string();
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
        let content = "See [[BACK-042]] and [[EPIC-003]] for details.";
        let refs = extract_references(content);
        assert_eq!(refs, vec!["BACK-042", "EPIC-003"]);
    }

    #[test]
    fn test_extract_references_empty() {
        let content = "No references here.";
        let refs = extract_references(content);
        assert!(refs.is_empty());
    }

    #[test]
    fn test_extract_references_multiline() {
        let content = "- [[BACK-001]]\n- [[PLAN-012]]\n- [[NOTE-007]]";
        let refs = extract_references(content);
        assert_eq!(refs, vec!["BACK-001", "PLAN-012", "NOTE-007"]);
    }

    #[test]
    fn test_extract_references_invalid_id() {
        let content = "[[INVALID-001]] and [[BACK-042]]";
        let refs = extract_references(content);
        assert_eq!(refs, vec!["BACK-042"]);
    }

    #[test]
    fn test_extract_references_nested_brackets() {
        let content = "[[BACK-042]] then [not a ref] then [[PLAN-001]]";
        let refs = extract_references(content);
        assert_eq!(refs, vec!["BACK-042", "PLAN-001"]);
    }

    #[test]
    fn test_extract_references_no_newline_in_ref() {
        let content = "[[BACK\n-042]]";
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
        extract_ids_from_line("implements: [BACK-042, BACK-043]", &mut refs);
        assert_eq!(refs, vec!["BACK-042", "BACK-043"]);
    }
}
