use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::error::{MarkplaneError, Result};
use crate::models::MarkplaneDocument;

const FRONTMATTER_DELIMITER: &str = "---";

/// Split raw file content into the YAML frontmatter string and the markdown body.
///
/// Expects the format:
/// ```text
/// ---
/// yaml: here
/// ---
/// markdown body here
/// ```
///
/// Returns `(yaml_string, body_string)`.
pub fn parse_frontmatter_raw(content: &str) -> Result<(String, String)> {
    let trimmed = content.trim_start();

    if !trimmed.starts_with(FRONTMATTER_DELIMITER) {
        return Err(MarkplaneError::Frontmatter(
            "File does not start with frontmatter delimiter '---'".into(),
        ));
    }

    // Skip the opening "---" line
    let after_opening = &trimmed[FRONTMATTER_DELIMITER.len()..];
    let after_opening = after_opening.strip_prefix('\n').unwrap_or(after_opening);

    // Find the closing "---"
    let closing_pos = find_closing_delimiter(after_opening)?;

    let yaml = &after_opening[..closing_pos];
    let remainder = &after_opening[closing_pos + FRONTMATTER_DELIMITER.len()..];

    // The body is everything after the closing delimiter line.
    // Strip the line break after `---` and the conventional blank line separator.
    let body = remainder.strip_prefix('\n').unwrap_or(remainder);
    let body = body.strip_prefix('\n').unwrap_or(body).to_string();

    Ok((yaml.to_string(), body))
}

/// Parse a markdown file with YAML frontmatter into a typed `MarkplaneDocument<T>`.
pub fn parse_frontmatter<T: DeserializeOwned>(content: &str) -> Result<MarkplaneDocument<T>> {
    let (yaml, body) = parse_frontmatter_raw(content)?;
    let frontmatter: T = serde_yaml::from_str(&yaml)?;
    Ok(MarkplaneDocument { frontmatter, body })
}

/// Serialize a `MarkplaneDocument<T>` back into the `---\nyaml\n---\nbody` format.
pub fn write_frontmatter<T: Serialize>(doc: &MarkplaneDocument<T>) -> Result<String> {
    let yaml = serde_yaml::to_string(&doc.frontmatter)?;
    let mut output = String::new();
    output.push_str(FRONTMATTER_DELIMITER);
    output.push('\n');
    output.push_str(&yaml);
    // serde_yaml already adds a trailing newline
    if !yaml.ends_with('\n') {
        output.push('\n');
    }
    output.push_str(FRONTMATTER_DELIMITER);
    output.push('\n');
    output.push('\n'); // conventional blank line between frontmatter and body
    if !doc.body.is_empty() {
        output.push_str(&doc.body);
        if !doc.body.ends_with('\n') {
            output.push('\n');
        }
    }
    Ok(output)
}

/// Find the position of the closing `---` delimiter in the content after the opening one.
fn find_closing_delimiter(content: &str) -> Result<usize> {
    for (pos, line) in content.lines().scan(0usize, |offset, line| {
        let current = *offset;
        *offset += line.len() + 1; // +1 for the newline
        Some((current, line))
    }) {
        if line.trim() == FRONTMATTER_DELIMITER {
            return Ok(pos);
        }
    }
    Err(MarkplaneError::Frontmatter(
        "Missing closing frontmatter delimiter '---'".into(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Task;

    #[test]
    fn test_parse_frontmatter_raw_basic() {
        let content = "---\nid: TASK-001\ntitle: Test\n---\n# Body\n";
        let (yaml, body) = parse_frontmatter_raw(content).unwrap();
        assert_eq!(yaml, "id: TASK-001\ntitle: Test\n");
        assert_eq!(body, "# Body\n");
    }

    #[test]
    fn test_parse_frontmatter_raw_empty_body() {
        let content = "---\nid: TASK-001\n---\n";
        let (yaml, body) = parse_frontmatter_raw(content).unwrap();
        assert_eq!(yaml, "id: TASK-001\n");
        assert_eq!(body, "");
    }

    #[test]
    fn test_parse_frontmatter_raw_no_trailing_newline() {
        let content = "---\nid: TASK-001\n---";
        let (yaml, body) = parse_frontmatter_raw(content).unwrap();
        assert_eq!(yaml, "id: TASK-001\n");
        assert_eq!(body, "");
    }

    #[test]
    fn test_parse_frontmatter_raw_no_delimiter() {
        let content = "# Just markdown\nNo frontmatter here.";
        let result = parse_frontmatter_raw(content);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_frontmatter_raw_missing_closing() {
        let content = "---\nid: TASK-001\ntitle: Test\n";
        let result = parse_frontmatter_raw(content);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_frontmatter_raw_leading_whitespace() {
        let content = "\n---\nid: TASK-001\n---\nBody\n";
        let (yaml, body) = parse_frontmatter_raw(content).unwrap();
        assert_eq!(yaml, "id: TASK-001\n");
        assert_eq!(body, "Body\n");
    }

    #[test]
    fn test_parse_frontmatter_typed() {
        let content = r#"---
id: TASK-042
title: "Add dark mode"
status: in-progress
priority: high
type: feature
effort: medium
epic: null
plan: null
depends_on: []
blocks: []
related: []
assignee: null
tags: [ui]
created: 2026-01-15
updated: 2026-02-09
---
# Add dark mode

Some description here.
"#;
        let doc: MarkplaneDocument<Task> = parse_frontmatter(content).unwrap();
        assert_eq!(doc.frontmatter.id, "TASK-042");
        assert_eq!(doc.frontmatter.status, "in-progress");
        assert_eq!(doc.frontmatter.item_type, "feature");
        assert!(doc.body.starts_with("# Add dark mode"));
    }

    #[test]
    fn test_write_frontmatter_roundtrip() {
        let content = r#"---
id: TASK-042
title: "Add dark mode"
status: in-progress
priority: high
type: feature
effort: medium
epic: null
plan: null
depends_on: []
blocks: []
related: []
assignee: null
tags: [ui]
created: 2026-01-15
updated: 2026-02-09
---
# Add dark mode

Some description here.
"#;
        let doc: MarkplaneDocument<Task> = parse_frontmatter(content).unwrap();
        let written = write_frontmatter(&doc).unwrap();

        // Should be parseable again
        let reparsed: MarkplaneDocument<Task> = parse_frontmatter(&written).unwrap();
        assert_eq!(reparsed.frontmatter.id, doc.frontmatter.id);
        assert_eq!(reparsed.frontmatter.status, doc.frontmatter.status);
        assert!(reparsed.body.contains("# Add dark mode"));
        assert!(reparsed.body.contains("Some description here."));
    }

    #[test]
    fn test_write_frontmatter_empty_body() {
        let doc = MarkplaneDocument {
            frontmatter: crate::models::Config::default(),
            body: String::new(),
        };
        let output = write_frontmatter(&doc).unwrap();
        assert!(output.starts_with("---\n"));
        assert!(output.contains("---\n"));
        // Should be parseable
        let (_, body) = parse_frontmatter_raw(&output).unwrap();
        assert_eq!(body, "");
    }

    #[test]
    fn test_multiline_body_preserved() {
        let content = "---\nid: TASK-001\ntitle: Test\nstatus: draft\npriority: low\ntype: chore\neffort: xs\ncreated: 2026-01-01\nupdated: 2026-01-01\n---\n# Title\n\nParagraph one.\n\nParagraph two.\n\n- list item\n";
        let doc: MarkplaneDocument<Task> = parse_frontmatter(content).unwrap();
        assert!(doc.body.contains("Paragraph one."));
        assert!(doc.body.contains("Paragraph two."));
        assert!(doc.body.contains("- list item"));
    }
}
