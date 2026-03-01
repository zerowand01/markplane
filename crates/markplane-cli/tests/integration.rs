use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

fn cmd() -> Command {
    assert_cmd::cargo::cargo_bin_cmd!("markplane")
}

/// Run markplane inside a temp dir, initializing it first (empty, no starter content).
fn setup_project() -> TempDir {
    let tmp = TempDir::new().unwrap();
    cmd()
        .current_dir(tmp.path())
        .args(["init", "--name", "Test Project", "--empty"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Initialized Markplane project"));
    tmp
}

/// Extract the generated ID from CLI output like "Created TASK-k7x9m — title"
fn extract_id(output: &[u8]) -> String {
    let stdout = String::from_utf8_lossy(output);
    stdout
        .split_whitespace()
        .find(|w| {
            w.starts_with("TASK-")
                || w.starts_with("EPIC-")
                || w.starts_with("PLAN-")
                || w.starts_with("NOTE-")
        })
        .unwrap_or_else(|| panic!("No ID found in output: {}", stdout))
        .to_string()
}

// ── Init ─────────────────────────────────────────────────────────────────

#[test]
fn test_init_creates_structure() {
    let tmp = TempDir::new().unwrap();
    cmd()
        .current_dir(tmp.path())
        .args(["init", "--name", "My Project", "--description", "A test", "--empty"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Initialized Markplane project: My Project"))
        .stdout(predicate::str::contains(".markplane/"));

    assert!(tmp.path().join(".markplane/config.yaml").is_file());
    assert!(tmp.path().join(".markplane/INDEX.md").is_file());
    assert!(tmp.path().join(".markplane/backlog").is_dir());
    assert!(tmp.path().join(".markplane/roadmap").is_dir());
    assert!(tmp.path().join(".markplane/plans").is_dir());
    assert!(tmp.path().join(".markplane/notes").is_dir());
    assert!(tmp.path().join(".markplane/.context").is_dir());
}

#[test]
fn test_init_defaults_to_dir_name() {
    let tmp = TempDir::new().unwrap();
    cmd()
        .current_dir(tmp.path())
        .args(["init", "--empty"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Initialized Markplane project"));
}

#[test]
fn test_init_already_initialized() {
    let tmp = setup_project();
    cmd()
        .current_dir(tmp.path())
        .args(["init", "--empty"])
        .assert()
        .failure();
}

#[test]
fn test_init_with_starter_content() {
    let tmp = TempDir::new().unwrap();
    cmd()
        .current_dir(tmp.path())
        .args(["init", "--name", "Seeded Project"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Seeded with starter content"))
        .stdout(predicate::str::contains("Next steps:"));

    // Tasks should exist
    cmd()
        .current_dir(tmp.path())
        .arg("ls")
        .assert()
        .success()
        .stdout(predicate::str::contains("Review and customize"))
        .stdout(predicate::str::contains("Import existing work"));

    // Epic should exist
    cmd()
        .current_dir(tmp.path())
        .args(["ls", "epics"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Project Setup"));

    // Plan should exist
    cmd()
        .current_dir(tmp.path())
        .args(["ls", "plans"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Import existing work"));

    // Note should exist
    cmd()
        .current_dir(tmp.path())
        .args(["ls", "notes"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Project decisions"));

    // No broken references
    cmd()
        .current_dir(tmp.path())
        .arg("check")
        .assert()
        .success()
        .stdout(predicate::str::contains("No broken references"));
}

#[test]
fn test_init_empty() {
    let tmp = TempDir::new().unwrap();
    cmd()
        .current_dir(tmp.path())
        .args(["init", "--name", "Empty Project", "--empty"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Initialized Markplane project"))
        .stdout(predicate::str::contains("Get started:"));

    cmd()
        .current_dir(tmp.path())
        .arg("ls")
        .assert()
        .success()
        .stdout(predicate::str::contains("No tasks found"));
}

// ── Add ──────────────────────────────────────────────────────────────────

#[test]
fn test_add_basic() {
    let tmp = setup_project();
    let output = cmd()
        .current_dir(tmp.path())
        .args(["add", "Fix login bug"])
        .output()
        .unwrap();
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
    let task_id = extract_id(&output.stdout);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Created"));
    assert!(stdout.contains(&task_id));
    assert!(stdout.contains("Fix login bug"));

    assert!(tmp.path().join(format!(".markplane/backlog/items/{}.md", task_id)).is_file());
}

#[test]
fn test_add_with_flags() {
    let tmp = setup_project();
    let output = cmd()
        .current_dir(tmp.path())
        .args([
            "add",
            "Add dark mode",
            "--type",
            "feature",
            "--priority",
            "high",
            "--effort",
            "large",
            "--tags",
            "ui,frontend",
        ])
        .output()
        .unwrap();
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
    let task_id = extract_id(&output.stdout);

    // Verify the file contains the right metadata
    let content =
        std::fs::read_to_string(tmp.path().join(format!(".markplane/backlog/items/{}.md", task_id))).unwrap();
    assert!(content.contains("priority: high"));
    assert!(content.contains("type: feature"));
    assert!(content.contains("effort: large"));
    assert!(content.contains("ui"));
    assert!(content.contains("frontend"));
}

#[test]
fn test_add_with_epic() {
    let tmp = setup_project();
    // Create an epic first
    let epic_output = cmd()
        .current_dir(tmp.path())
        .args(["epic", "Phase 1"])
        .output()
        .unwrap();
    assert!(epic_output.status.success(), "stderr: {}", String::from_utf8_lossy(&epic_output.stderr));
    let epic_id = extract_id(&epic_output.stdout);

    let output = cmd()
        .current_dir(tmp.path())
        .args(["add", "Task in epic", "--epic", &epic_id])
        .output()
        .unwrap();
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
    let task_id = extract_id(&output.stdout);

    let content =
        std::fs::read_to_string(tmp.path().join(format!(".markplane/backlog/items/{}.md", task_id))).unwrap();
    assert!(content.contains(&format!("epic: {}", epic_id)));
}

#[test]
fn test_add_random_ids() {
    let tmp = setup_project();
    let out1 = cmd()
        .current_dir(tmp.path())
        .args(["add", "First"])
        .output()
        .unwrap();
    assert!(out1.status.success(), "stderr: {}", String::from_utf8_lossy(&out1.stderr));
    let id1 = extract_id(&out1.stdout);
    assert!(id1.starts_with("TASK-"));

    let out2 = cmd()
        .current_dir(tmp.path())
        .args(["add", "Second"])
        .output()
        .unwrap();
    assert!(out2.status.success(), "stderr: {}", String::from_utf8_lossy(&out2.stderr));
    let id2 = extract_id(&out2.stdout);
    assert!(id2.starts_with("TASK-"));

    let out3 = cmd()
        .current_dir(tmp.path())
        .args(["add", "Third"])
        .output()
        .unwrap();
    assert!(out3.status.success(), "stderr: {}", String::from_utf8_lossy(&out3.stderr));
    let id3 = extract_id(&out3.stdout);
    assert!(id3.starts_with("TASK-"));

    // All IDs should be unique
    assert_ne!(id1, id2);
    assert_ne!(id2, id3);
    assert_ne!(id1, id3);
}

// ── Show ─────────────────────────────────────────────────────────────────

#[test]
fn test_show_task() {
    let tmp = setup_project();
    let output = cmd()
        .current_dir(tmp.path())
        .args(["add", "Show me", "--priority", "high"])
        .output()
        .unwrap();
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
    let task_id = extract_id(&output.stdout);

    cmd()
        .current_dir(tmp.path())
        .args(["show", &task_id])
        .assert()
        .success()
        .stdout(predicate::str::contains(&task_id))
        .stdout(predicate::str::contains("Show me"))
        .stdout(predicate::str::contains("high"));
}

#[test]
fn test_show_not_found() {
    let tmp = setup_project();
    cmd()
        .current_dir(tmp.path())
        .args(["show", "TASK-999"])
        .assert()
        .failure();
}

#[test]
fn test_show_epic() {
    let tmp = setup_project();
    let output = cmd()
        .current_dir(tmp.path())
        .args(["epic", "Phase 1", "--priority", "high"])
        .output()
        .unwrap();
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
    let epic_id = extract_id(&output.stdout);

    cmd()
        .current_dir(tmp.path())
        .args(["show", &epic_id])
        .assert()
        .success()
        .stdout(predicate::str::contains(&epic_id))
        .stdout(predicate::str::contains("Phase 1"));
}

// ── Ls ───────────────────────────────────────────────────────────────────

#[test]
fn test_ls_empty() {
    let tmp = setup_project();
    cmd()
        .current_dir(tmp.path())
        .arg("ls")
        .assert()
        .success()
        .stdout(predicate::str::contains("No tasks found"));
}

#[test]
fn test_ls_with_items() {
    let tmp = setup_project();
    let out1 = cmd()
        .current_dir(tmp.path())
        .args(["add", "Task A"])
        .output()
        .unwrap();
    assert!(out1.status.success(), "stderr: {}", String::from_utf8_lossy(&out1.stderr));
    let id1 = extract_id(&out1.stdout);

    let out2 = cmd()
        .current_dir(tmp.path())
        .args(["add", "Task B"])
        .output()
        .unwrap();
    assert!(out2.status.success(), "stderr: {}", String::from_utf8_lossy(&out2.stderr));
    let id2 = extract_id(&out2.stdout);

    cmd()
        .current_dir(tmp.path())
        .arg("ls")
        .assert()
        .success()
        .stdout(predicate::str::contains(&id1))
        .stdout(predicate::str::contains(&id2))
        .stdout(predicate::str::contains("Task A"))
        .stdout(predicate::str::contains("Task B"));
}

#[test]
fn test_ls_filter_status() {
    let tmp = setup_project();
    let out1 = cmd()
        .current_dir(tmp.path())
        .args(["add", "Draft item"])
        .output()
        .unwrap();
    assert!(out1.status.success(), "stderr: {}", String::from_utf8_lossy(&out1.stderr));

    let out2 = cmd()
        .current_dir(tmp.path())
        .args(["add", "Progress item"])
        .output()
        .unwrap();
    assert!(out2.status.success(), "stderr: {}", String::from_utf8_lossy(&out2.stderr));
    let id2 = extract_id(&out2.stdout);

    cmd()
        .current_dir(tmp.path())
        .args(["status", &id2, "in-progress"])
        .assert()
        .success();

    cmd()
        .current_dir(tmp.path())
        .args(["ls", "--status", "in-progress"])
        .assert()
        .success()
        .stdout(predicate::str::contains(&id2))
        .stdout(predicate::str::contains("Draft item").not());
}

#[test]
fn test_ls_epics() {
    let tmp = setup_project();
    let output = cmd()
        .current_dir(tmp.path())
        .args(["epic", "Phase 1"])
        .output()
        .unwrap();
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
    let epic_id = extract_id(&output.stdout);

    cmd()
        .current_dir(tmp.path())
        .args(["ls", "epics"])
        .assert()
        .success()
        .stdout(predicate::str::contains(&epic_id))
        .stdout(predicate::str::contains("Phase 1"));
}

#[test]
fn test_ls_plans() {
    let tmp = setup_project();
    let task_out = cmd()
        .current_dir(tmp.path())
        .args(["add", "Some task"])
        .output()
        .unwrap();
    assert!(task_out.status.success(), "stderr: {}", String::from_utf8_lossy(&task_out.stderr));
    let task_id = extract_id(&task_out.stdout);

    let plan_out = cmd()
        .current_dir(tmp.path())
        .args(["plan", &task_id])
        .output()
        .unwrap();
    assert!(plan_out.status.success(), "stderr: {}", String::from_utf8_lossy(&plan_out.stderr));
    let plan_id = extract_id(&plan_out.stdout);

    cmd()
        .current_dir(tmp.path())
        .args(["ls", "plans"])
        .assert()
        .success()
        .stdout(predicate::str::contains(&plan_id));
}

#[test]
fn test_ls_notes() {
    let tmp = setup_project();
    let output = cmd()
        .current_dir(tmp.path())
        .args(["note", "Research topic", "--type", "research"])
        .output()
        .unwrap();
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
    let note_id = extract_id(&output.stdout);

    cmd()
        .current_dir(tmp.path())
        .args(["ls", "notes"])
        .assert()
        .success()
        .stdout(predicate::str::contains(&note_id))
        .stdout(predicate::str::contains("Research topic"));
}

// ── Status ───────────────────────────────────────────────────────────────

#[test]
fn test_status_update() {
    let tmp = setup_project();
    let output = cmd()
        .current_dir(tmp.path())
        .args(["add", "Status test"])
        .output()
        .unwrap();
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
    let task_id = extract_id(&output.stdout);

    cmd()
        .current_dir(tmp.path())
        .args(["status", &task_id, "in-progress"])
        .assert()
        .success()
        .stdout(predicate::str::contains(&task_id))
        .stdout(predicate::str::contains("in-progress"));

    // Verify the change
    cmd()
        .current_dir(tmp.path())
        .args(["show", &task_id])
        .assert()
        .success()
        .stdout(predicate::str::contains("in-progress"));
}

#[test]
fn test_status_invalid() {
    let tmp = setup_project();
    let output = cmd()
        .current_dir(tmp.path())
        .args(["add", "Status test"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let task_id = extract_id(&output.stdout);

    cmd()
        .current_dir(tmp.path())
        .args(["status", &task_id, "invalid-status"])
        .assert()
        .failure();
}

// ── Sync ─────────────────────────────────────────────────────────────────

#[test]
fn test_sync() {
    let tmp = setup_project();
    cmd()
        .current_dir(tmp.path())
        .args(["add", "Sync test"])
        .assert()
        .success();

    cmd()
        .current_dir(tmp.path())
        .arg("sync")
        .assert()
        .success();

    // INDEX.md files should be regenerated
    let root_index =
        std::fs::read_to_string(tmp.path().join(".markplane/INDEX.md")).unwrap();
    assert!(root_index.contains("Generated by markplane sync"));

    // Context files should exist
    assert!(tmp
        .path()
        .join(".markplane/.context/summary.md")
        .is_file());
}

// ── Start / Done ─────────────────────────────────────────────────────────

#[test]
fn test_start_and_done() {
    let tmp = setup_project();
    let output = cmd()
        .current_dir(tmp.path())
        .args(["add", "Start/done test"])
        .output()
        .unwrap();
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
    let task_id = extract_id(&output.stdout);

    cmd()
        .current_dir(tmp.path())
        .args(["start", &task_id, "--user", "alice"])
        .assert()
        .success()
        .stdout(predicate::str::contains("in-progress"))
        .stdout(predicate::str::contains("alice"));

    // Verify status and assignee
    let content =
        std::fs::read_to_string(tmp.path().join(format!(".markplane/backlog/items/{}.md", task_id))).unwrap();
    assert!(content.contains("status: in-progress"));
    assert!(content.contains("assignee: alice"));

    cmd()
        .current_dir(tmp.path())
        .args(["done", &task_id])
        .assert()
        .success()
        .stdout(predicate::str::contains("done"));
}

// ── Epic / Note / Plan ───────────────────────────────────────────────────

#[test]
fn test_epic_creation() {
    let tmp = setup_project();
    let output = cmd()
        .current_dir(tmp.path())
        .args(["epic", "Phase 1", "--priority", "high"])
        .output()
        .unwrap();
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
    let epic_id = extract_id(&output.stdout);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Created"));
    assert!(stdout.contains(&epic_id));
    assert!(stdout.contains("Phase 1"));

    assert!(tmp
        .path()
        .join(format!(".markplane/roadmap/items/{}.md", epic_id))
        .is_file());
}

#[test]
fn test_note_creation() {
    let tmp = setup_project();
    let output = cmd()
        .current_dir(tmp.path())
        .args(["note", "Research caching", "--type", "research", "--tags", "cache,perf"])
        .output()
        .unwrap();
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
    let note_id = extract_id(&output.stdout);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Created"));
    assert!(stdout.contains(&note_id));
    assert!(stdout.contains("Research caching"));

    assert!(tmp
        .path()
        .join(format!(".markplane/notes/items/{}.md", note_id))
        .is_file());
}

#[test]
fn test_plan_creation() {
    let tmp = setup_project();
    let task_out = cmd()
        .current_dir(tmp.path())
        .args(["add", "Dark mode"])
        .output()
        .unwrap();
    assert!(task_out.status.success(), "stderr: {}", String::from_utf8_lossy(&task_out.stderr));
    let task_id = extract_id(&task_out.stdout);

    let plan_out = cmd()
        .current_dir(tmp.path())
        .args(["plan", &task_id, "--title", "Dark mode plan"])
        .output()
        .unwrap();
    assert!(plan_out.status.success(), "stderr: {}", String::from_utf8_lossy(&plan_out.stderr));
    let plan_id = extract_id(&plan_out.stdout);
    let stdout = String::from_utf8_lossy(&plan_out.stdout);
    assert!(stdout.contains("Created"));
    assert!(stdout.contains(&plan_id));
    assert!(stdout.contains("Dark mode plan"));
    assert!(stdout.contains(&format!("Linked to {}", task_id)));

    // Verify task has plan linked
    let content =
        std::fs::read_to_string(tmp.path().join(format!(".markplane/backlog/items/{}.md", task_id))).unwrap();
    assert!(content.contains(&plan_id));
}

// ── Promote ──────────────────────────────────────────────────────────────

#[test]
fn test_promote_note_to_task() {
    let tmp = setup_project();
    let note_out = cmd()
        .current_dir(tmp.path())
        .args(["note", "Good idea", "--type", "idea", "--tags", "cool"])
        .output()
        .unwrap();
    assert!(note_out.status.success(), "stderr: {}", String::from_utf8_lossy(&note_out.stderr));
    let note_id = extract_id(&note_out.stdout);

    let promote_out = cmd()
        .current_dir(tmp.path())
        .args(["promote", &note_id, "--priority", "high"])
        .output()
        .unwrap();
    assert!(promote_out.status.success(), "stderr: {}", String::from_utf8_lossy(&promote_out.stderr));
    let stdout = String::from_utf8_lossy(&promote_out.stdout);
    assert!(stdout.contains(&format!("Promoted {}", note_id)));
    // Extract the TASK ID from "Promoted NOTE-xxx → TASK-yyy — title"
    let task_id = stdout
        .split_whitespace()
        .find(|w| w.starts_with("TASK-"))
        .expect("No TASK ID found in promote output")
        .to_string();

    assert!(tmp
        .path()
        .join(format!(".markplane/backlog/items/{}.md", task_id))
        .is_file());
}

#[test]
fn test_promote_non_note_fails() {
    let tmp = setup_project();
    let output = cmd()
        .current_dir(tmp.path())
        .args(["add", "Not a note"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let task_id = extract_id(&output.stdout);

    cmd()
        .current_dir(tmp.path())
        .args(["promote", &task_id])
        .assert()
        .failure();
}

// ── Update ───────────────────────────────────────────────────────────────

#[test]
fn test_update_assignee() {
    let tmp = setup_project();
    let output = cmd()
        .current_dir(tmp.path())
        .args(["add", "Update assignee test"])
        .output()
        .unwrap();
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
    let task_id = extract_id(&output.stdout);

    cmd()
        .current_dir(tmp.path())
        .args(["update", &task_id, "--assignee", "@daniel"])
        .assert()
        .success()
        .stdout(predicate::str::contains(format!("Updated {}", task_id)));

    let content =
        std::fs::read_to_string(tmp.path().join(format!(".markplane/backlog/items/{}.md", task_id))).unwrap();
    assert!(content.contains("assignee: daniel"));
}

#[test]
fn test_update_clear_assignee() {
    let tmp = setup_project();
    let output = cmd()
        .current_dir(tmp.path())
        .args(["add", "Clear assignee test"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let task_id = extract_id(&output.stdout);

    // Set assignee first
    cmd()
        .current_dir(tmp.path())
        .args(["update", &task_id, "--assignee", "daniel"])
        .assert()
        .success();

    // Clear it
    cmd()
        .current_dir(tmp.path())
        .args(["update", &task_id, "--clear-assignee"])
        .assert()
        .success();

    let content =
        std::fs::read_to_string(tmp.path().join(format!(".markplane/backlog/items/{}.md", task_id))).unwrap();
    assert!(!content.contains("assignee: daniel"));
}

#[test]
fn test_update_tags() {
    let tmp = setup_project();
    let output = cmd()
        .current_dir(tmp.path())
        .args(["add", "Tag test"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let task_id = extract_id(&output.stdout);

    cmd()
        .current_dir(tmp.path())
        .args(["update", &task_id, "--add-tag", "ui,frontend"])
        .assert()
        .success();

    let content =
        std::fs::read_to_string(tmp.path().join(format!(".markplane/backlog/items/{}.md", task_id))).unwrap();
    assert!(content.contains("- ui\n"));
    assert!(content.contains("- frontend\n"));

    // Remove one tag
    cmd()
        .current_dir(tmp.path())
        .args(["update", &task_id, "--remove-tag", "ui"])
        .assert()
        .success();

    let content =
        std::fs::read_to_string(tmp.path().join(format!(".markplane/backlog/items/{}.md", task_id))).unwrap();
    assert!(!content.contains("- ui\n"));
    assert!(content.contains("- frontend\n"));
}

#[test]
fn test_update_effort_priority() {
    let tmp = setup_project();
    let output = cmd()
        .current_dir(tmp.path())
        .args(["add", "Effort test"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let task_id = extract_id(&output.stdout);

    cmd()
        .current_dir(tmp.path())
        .args(["update", &task_id, "--effort", "large", "--priority", "high"])
        .assert()
        .success();

    let content =
        std::fs::read_to_string(tmp.path().join(format!(".markplane/backlog/items/{}.md", task_id))).unwrap();
    assert!(content.contains("effort: large"));
    assert!(content.contains("priority: high"));
}

#[test]
fn test_update_title_and_type() {
    let tmp = setup_project();
    let output = cmd()
        .current_dir(tmp.path())
        .args(["add", "Title test"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let task_id = extract_id(&output.stdout);

    cmd()
        .current_dir(tmp.path())
        .args(["update", &task_id, "--title", "Renamed task", "--type", "bug"])
        .assert()
        .success();

    let content =
        std::fs::read_to_string(tmp.path().join(format!(".markplane/backlog/items/{}.md", task_id))).unwrap();
    assert!(content.contains("title: Renamed task"));
    assert!(content.contains("type: bug"));
}

#[test]
fn test_update_position() {
    let tmp = setup_project();
    let output = cmd()
        .current_dir(tmp.path())
        .args(["add", "Position test"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let task_id = extract_id(&output.stdout);

    // Set position
    cmd()
        .current_dir(tmp.path())
        .args(["update", &task_id, "--position", "aaa"])
        .assert()
        .success();

    let content =
        std::fs::read_to_string(tmp.path().join(format!(".markplane/backlog/items/{}.md", task_id))).unwrap();
    assert!(content.contains("position: aaa"));

    // Clear position
    cmd()
        .current_dir(tmp.path())
        .args(["update", &task_id, "--clear-position"])
        .assert()
        .success();

    let content =
        std::fs::read_to_string(tmp.path().join(format!(".markplane/backlog/items/{}.md", task_id))).unwrap();
    assert!(!content.contains("position: aaa"));
}

#[test]
fn test_update_epic_dates() {
    let tmp = setup_project();
    let output = cmd()
        .current_dir(tmp.path())
        .args(["epic", "Dates test"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let epic_id = extract_id(&output.stdout);

    // Set started and target dates
    cmd()
        .current_dir(tmp.path())
        .args(["update", &epic_id, "--started", "2026-02-20", "--target", "2026-06-01"])
        .assert()
        .success();

    let content =
        std::fs::read_to_string(tmp.path().join(format!(".markplane/roadmap/items/{}.md", epic_id))).unwrap();
    assert!(content.contains("started: 2026-02-20"));
    assert!(content.contains("target: 2026-06-01"));

    // Clear started
    cmd()
        .current_dir(tmp.path())
        .args(["update", &epic_id, "--clear-started"])
        .assert()
        .success();

    let content =
        std::fs::read_to_string(tmp.path().join(format!(".markplane/roadmap/items/{}.md", epic_id))).unwrap();
    assert!(!content.contains("started: 2026-02-20"));
    assert!(content.contains("target: 2026-06-01"));
}

#[test]
fn test_update_note_type() {
    let tmp = setup_project();
    let output = cmd()
        .current_dir(tmp.path())
        .args(["note", "Note type test", "--type", "idea"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let note_id = extract_id(&output.stdout);

    // Change note type
    cmd()
        .current_dir(tmp.path())
        .args(["update", &note_id, "--note-type", "decision"])
        .assert()
        .success();

    let content =
        std::fs::read_to_string(tmp.path().join(format!(".markplane/notes/items/{}.md", note_id))).unwrap();
    assert!(content.contains("type: decision"));
}

#[test]
fn test_update_rejects_invalid_field() {
    let tmp = setup_project();
    // Create an epic
    let output = cmd()
        .current_dir(tmp.path())
        .args(["epic", "Test Epic"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let epic_id = extract_id(&output.stdout);

    // effort is not valid for epics
    cmd()
        .current_dir(tmp.path())
        .args(["update", &epic_id, "--effort", "large"])
        .assert()
        .failure();
}

// ── Link ─────────────────────────────────────────────────────────────────

#[test]
fn test_link_blocks() {
    let tmp = setup_project();
    let out1 = cmd()
        .current_dir(tmp.path())
        .args(["add", "Blocker"])
        .output()
        .unwrap();
    assert!(out1.status.success(), "stderr: {}", String::from_utf8_lossy(&out1.stderr));
    let id1 = extract_id(&out1.stdout);

    let out2 = cmd()
        .current_dir(tmp.path())
        .args(["add", "Blocked"])
        .output()
        .unwrap();
    assert!(out2.status.success(), "stderr: {}", String::from_utf8_lossy(&out2.stderr));
    let id2 = extract_id(&out2.stdout);

    cmd()
        .current_dir(tmp.path())
        .args(["link", &id1, &id2, "--relation", "blocks"])
        .assert()
        .success()
        .stdout(predicate::str::contains(format!("{} blocks {}", id1, id2)));

    // Verify bidirectional
    let blocker =
        std::fs::read_to_string(tmp.path().join(format!(".markplane/backlog/items/{}.md", id1))).unwrap();
    assert!(blocker.contains(&id2));

    let blocked =
        std::fs::read_to_string(tmp.path().join(format!(".markplane/backlog/items/{}.md", id2))).unwrap();
    assert!(blocked.contains(&id1));
}

#[test]
fn test_link_missing_args_fails() {
    let tmp = setup_project();
    let output = cmd()
        .current_dir(tmp.path())
        .args(["add", "Lonely"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let task_id = extract_id(&output.stdout);

    // Missing TO and --relation
    cmd()
        .current_dir(tmp.path())
        .args(["link", &task_id])
        .assert()
        .failure();
}

#[test]
fn test_link_epic() {
    let tmp = setup_project();
    let task_out = cmd()
        .current_dir(tmp.path())
        .args(["add", "A task"])
        .output()
        .unwrap();
    assert!(task_out.status.success());
    let task_id = extract_id(&task_out.stdout);

    let epic_out = cmd()
        .current_dir(tmp.path())
        .args(["epic", "An epic"])
        .output()
        .unwrap();
    assert!(epic_out.status.success());
    let epic_id = extract_id(&epic_out.stdout);

    cmd()
        .current_dir(tmp.path())
        .args(["link", &task_id, &epic_id, "--relation", "epic"])
        .assert()
        .success();

    let task_content =
        std::fs::read_to_string(tmp.path().join(format!(".markplane/backlog/items/{}.md", task_id))).unwrap();
    assert!(task_content.contains(&epic_id));
}

#[test]
fn test_link_plan() {
    let tmp = setup_project();
    let task_out = cmd()
        .current_dir(tmp.path())
        .args(["add", "A task"])
        .output()
        .unwrap();
    assert!(task_out.status.success());
    let task_id = extract_id(&task_out.stdout);

    // Create plan via the plan command (which now uses link_items internally)
    let plan_out = cmd()
        .current_dir(tmp.path())
        .args(["plan", &task_id])
        .output()
        .unwrap();
    assert!(plan_out.status.success(), "stderr: {}", String::from_utf8_lossy(&plan_out.stderr));

    // Verify task.plan is set
    let task_content =
        std::fs::read_to_string(tmp.path().join(format!(".markplane/backlog/items/{}.md", task_id))).unwrap();
    assert!(task_content.contains("plan: PLAN-"));
}

#[test]
fn test_link_remove() {
    let tmp = setup_project();
    let out1 = cmd()
        .current_dir(tmp.path())
        .args(["add", "A"])
        .output()
        .unwrap();
    assert!(out1.status.success());
    let id1 = extract_id(&out1.stdout);

    let out2 = cmd()
        .current_dir(tmp.path())
        .args(["add", "B"])
        .output()
        .unwrap();
    assert!(out2.status.success());
    let id2 = extract_id(&out2.stdout);

    // Add link
    cmd()
        .current_dir(tmp.path())
        .args(["link", &id1, &id2, "--relation", "blocks"])
        .assert()
        .success();

    // Remove link
    cmd()
        .current_dir(tmp.path())
        .args(["link", &id1, &id2, "--relation", "blocks", "--remove"])
        .assert()
        .success();

    let content =
        std::fs::read_to_string(tmp.path().join(format!(".markplane/backlog/items/{}.md", id1))).unwrap();
    assert!(!content.contains(&id2));
}

#[test]
fn test_link_related_bidirectional() {
    let tmp = setup_project();
    let out1 = cmd()
        .current_dir(tmp.path())
        .args(["add", "Task A"])
        .output()
        .unwrap();
    assert!(out1.status.success());
    let id1 = extract_id(&out1.stdout);

    let out2 = cmd()
        .current_dir(tmp.path())
        .args(["add", "Task B"])
        .output()
        .unwrap();
    assert!(out2.status.success());
    let id2 = extract_id(&out2.stdout);

    // Link with related
    cmd()
        .current_dir(tmp.path())
        .args(["link", &id1, &id2, "--relation", "related"])
        .assert()
        .success();

    // Verify both files contain the reciprocal link
    let content1 =
        std::fs::read_to_string(tmp.path().join(format!(".markplane/backlog/items/{}.md", id1))).unwrap();
    let content2 =
        std::fs::read_to_string(tmp.path().join(format!(".markplane/backlog/items/{}.md", id2))).unwrap();
    assert!(content1.contains(&id2), "Task A should have related link to Task B");
    assert!(content2.contains(&id1), "Task B should have related link to Task A");
}

// ── Check ────────────────────────────────────────────────────────────────

#[test]
fn test_check_clean() {
    let tmp = setup_project();
    cmd()
        .current_dir(tmp.path())
        .args(["add", "Clean item"])
        .assert()
        .success();

    cmd()
        .current_dir(tmp.path())
        .arg("check")
        .assert()
        .success()
        .stdout(predicate::str::contains("No broken references"));
}

// ── Metrics ──────────────────────────────────────────────────────────────

#[test]
fn test_metrics() {
    let tmp = setup_project();
    cmd()
        .current_dir(tmp.path())
        .args(["add", "Metrics item"])
        .assert()
        .success();

    cmd()
        .current_dir(tmp.path())
        .arg("metrics")
        .assert()
        .success()
        .stdout(predicate::str::contains("Task Status"))
        .stdout(predicate::str::contains("Total:"));
}

// ── Dashboard ────────────────────────────────────────────────────────────

#[test]
fn test_dashboard() {
    let tmp = setup_project();
    cmd()
        .current_dir(tmp.path())
        .args(["add", "Dashboard test"])
        .assert()
        .success();

    cmd()
        .current_dir(tmp.path())
        .arg("dashboard")
        .assert()
        .success()
        .stdout(predicate::str::contains("Project Dashboard"));
}

// ── No project ───────────────────────────────────────────────────────────

#[test]
fn test_commands_fail_without_init() {
    let tmp = TempDir::new().unwrap();

    cmd()
        .current_dir(tmp.path())
        .args(["add", "Should fail"])
        .assert()
        .failure();

    cmd()
        .current_dir(tmp.path())
        .arg("ls")
        .assert()
        .failure();

    cmd()
        .current_dir(tmp.path())
        .args(["show", "TASK-001"])
        .assert()
        .failure();
}

// ── Help ─────────────────────────────────────────────────────────────────

#[test]
fn test_help() {
    cmd()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("AI-native project management"))
        .stdout(predicate::str::contains("init"))
        .stdout(predicate::str::contains("add"))
        .stdout(predicate::str::contains("show"))
        .stdout(predicate::str::contains("ls"))
        .stdout(predicate::str::contains("status"))
        .stdout(predicate::str::contains("sync"))
        .stdout(predicate::str::contains("start"))
        .stdout(predicate::str::contains("done"))
        .stdout(predicate::str::contains("promote"))
        .stdout(predicate::str::contains("plan"))
        .stdout(predicate::str::contains("epic"))
        .stdout(predicate::str::contains("note"))
        .stdout(predicate::str::contains("update"))
        .stdout(predicate::str::contains("link"))
        .stdout(predicate::str::contains("check"))
        .stdout(predicate::str::contains("stale"))
        .stdout(predicate::str::contains("archive"))
        .stdout(predicate::str::contains("context"))
        .stdout(predicate::str::contains("metrics"))
        .stdout(predicate::str::contains("graph"))
        .stdout(predicate::str::contains("dashboard"));
}

// ── Stale ─────────────────────────────────────────────────────────────────

#[test]
fn test_stale_no_stale_items() {
    let tmp = setup_project();
    cmd()
        .current_dir(tmp.path())
        .args(["add", "Fresh item"])
        .assert()
        .success();

    cmd()
        .current_dir(tmp.path())
        .args(["stale", "--days", "1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("No items stale"));
}

#[test]
fn test_stale_with_old_items() {
    let tmp = setup_project();
    let output = cmd()
        .current_dir(tmp.path())
        .args(["add", "Old item"])
        .output()
        .unwrap();
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
    let task_id = extract_id(&output.stdout);

    // Manually backdate the item's updated field to make it stale
    let item_path = tmp.path().join(format!(".markplane/backlog/items/{}.md", task_id));
    let content = std::fs::read_to_string(&item_path).unwrap();
    let old_date = "2020-01-01";
    let today = chrono::Local::now()
        .date_naive()
        .format("%Y-%m-%d")
        .to_string();
    let content = content.replace(
        &format!("updated: {}", today),
        &format!("updated: {}", old_date),
    );
    std::fs::write(&item_path, content).unwrap();

    cmd()
        .current_dir(tmp.path())
        .args(["stale", "--days", "30"])
        .assert()
        .success()
        .stdout(predicate::str::contains(&task_id))
        .stdout(predicate::str::contains("Old item"));
}

// ── Archive ───────────────────────────────────────────────────────────────

#[test]
fn test_archive_nothing_to_archive() {
    let tmp = setup_project();
    cmd()
        .current_dir(tmp.path())
        .args(["add", "Draft item"])
        .assert()
        .success();

    // --all-done with only draft items should report nothing
    cmd()
        .current_dir(tmp.path())
        .args(["archive", "--all-done"])
        .assert()
        .success()
        .stdout(predicate::str::contains("No completed items"));
}

#[test]
fn test_archive_dry_run() {
    let tmp = setup_project();
    let output = cmd()
        .current_dir(tmp.path())
        .args(["add", "To archive"])
        .output()
        .unwrap();
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
    let task_id = extract_id(&output.stdout);

    cmd()
        .current_dir(tmp.path())
        .args(["status", &task_id, "done"])
        .assert()
        .success();

    cmd()
        .current_dir(tmp.path())
        .args(["archive", "--all-done", "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Would archive"))
        .stdout(predicate::str::contains(&task_id));

    // File should still be in active dir (not moved)
    assert!(tmp.path().join(format!(".markplane/backlog/items/{}.md", task_id)).is_file());
}

#[test]
fn test_archive_actual() {
    let tmp = setup_project();
    let output = cmd()
        .current_dir(tmp.path())
        .args(["add", "To archive"])
        .output()
        .unwrap();
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
    let task_id = extract_id(&output.stdout);

    cmd()
        .current_dir(tmp.path())
        .args(["status", &task_id, "done"])
        .assert()
        .success();

    // Single-item archive by ID
    cmd()
        .current_dir(tmp.path())
        .args(["archive", &task_id])
        .assert()
        .success()
        .stdout(predicate::str::contains(format!("Archived {}", task_id)));

    // File should be in archive dir
    assert!(tmp
        .path()
        .join(format!(".markplane/backlog/archive/{}.md", task_id))
        .is_file());
    assert!(!tmp.path().join(format!(".markplane/backlog/items/{}.md", task_id)).is_file());
}

#[test]
fn test_archive_keep_cancelled() {
    let tmp = setup_project();
    let output = cmd()
        .current_dir(tmp.path())
        .args(["add", "To cancel"])
        .output()
        .unwrap();
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
    let task_id = extract_id(&output.stdout);

    cmd()
        .current_dir(tmp.path())
        .args(["status", &task_id, "cancelled"])
        .assert()
        .success();

    // --all-done now archives cancelled tasks too
    cmd()
        .current_dir(tmp.path())
        .args(["archive", "--all-done"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Archived"));

    // File should be in archive dir
    assert!(tmp
        .path()
        .join(format!(".markplane/backlog/archive/{}.md", task_id))
        .is_file());
}

#[test]
fn test_unarchive() {
    let tmp = setup_project();
    let output = cmd()
        .current_dir(tmp.path())
        .args(["add", "To archive and restore"])
        .output()
        .unwrap();
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
    let task_id = extract_id(&output.stdout);

    // Archive it
    cmd()
        .current_dir(tmp.path())
        .args(["archive", &task_id])
        .assert()
        .success();

    assert!(tmp.path().join(format!(".markplane/backlog/archive/{}.md", task_id)).is_file());

    // Unarchive it
    cmd()
        .current_dir(tmp.path())
        .args(["unarchive", &task_id])
        .assert()
        .success()
        .stdout(predicate::str::contains(format!("Restored {}", task_id)));

    assert!(tmp.path().join(format!(".markplane/backlog/items/{}.md", task_id)).is_file());
    assert!(!tmp.path().join(format!(".markplane/backlog/archive/{}.md", task_id)).is_file());
}

#[test]
fn test_ls_archived() {
    let tmp = setup_project();
    let out1 = cmd()
        .current_dir(tmp.path())
        .args(["add", "Active item"])
        .output()
        .unwrap();
    assert!(out1.status.success(), "stderr: {}", String::from_utf8_lossy(&out1.stderr));
    let id1 = extract_id(&out1.stdout);

    let out2 = cmd()
        .current_dir(tmp.path())
        .args(["add", "Archived item"])
        .output()
        .unwrap();
    assert!(out2.status.success(), "stderr: {}", String::from_utf8_lossy(&out2.stderr));
    let id2 = extract_id(&out2.stdout);

    // Archive second item
    cmd()
        .current_dir(tmp.path())
        .args(["archive", &id2])
        .assert()
        .success();

    // Normal ls should only show active
    cmd()
        .current_dir(tmp.path())
        .arg("ls")
        .assert()
        .success()
        .stdout(predicate::str::contains(id1.as_str()))
        .stdout(predicate::str::contains(id2.as_str()).not());

    // ls --archived should only show archived
    cmd()
        .current_dir(tmp.path())
        .args(["ls", "--archived"])
        .assert()
        .success()
        .stdout(predicate::str::contains(id2.as_str()))
        .stdout(predicate::str::contains(id1.as_str()).not());
}

// ── Graph ─────────────────────────────────────────────────────────────────

#[test]
fn test_graph() {
    let tmp = setup_project();
    let out1 = cmd()
        .current_dir(tmp.path())
        .args(["add", "Blocker"])
        .output()
        .unwrap();
    assert!(out1.status.success(), "stderr: {}", String::from_utf8_lossy(&out1.stderr));
    let id1 = extract_id(&out1.stdout);

    let out2 = cmd()
        .current_dir(tmp.path())
        .args(["add", "Blocked"])
        .output()
        .unwrap();
    assert!(out2.status.success(), "stderr: {}", String::from_utf8_lossy(&out2.stderr));
    let id2 = extract_id(&out2.stdout);

    cmd()
        .current_dir(tmp.path())
        .args(["link", &id1, &id2, "--relation", "blocks"])
        .assert()
        .success();

    cmd()
        .current_dir(tmp.path())
        .args(["graph", &id1])
        .assert()
        .success()
        .stdout(predicate::str::contains(&id1))
        .stdout(predicate::str::contains(&id2));
}

// ── Context ───────────────────────────────────────────────────────────────

#[test]
fn test_context_regenerate() {
    let tmp = setup_project();
    cmd()
        .current_dir(tmp.path())
        .arg("context")
        .assert()
        .success()
        .stdout(predicate::str::contains("Context files regenerated"));

    assert!(tmp
        .path()
        .join(".markplane/.context/summary.md")
        .is_file());
}

#[test]
fn test_context_focus_active_work() {
    let tmp = setup_project();
    cmd()
        .current_dir(tmp.path())
        .args(["context", "--focus", "active-work"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Active Work"));
}

#[test]
fn test_context_focus_blocked() {
    let tmp = setup_project();
    cmd()
        .current_dir(tmp.path())
        .args(["context", "--focus", "blocked"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Blocked"));
}

#[test]
fn test_context_focus_metrics() {
    let tmp = setup_project();
    cmd()
        .current_dir(tmp.path())
        .args(["context", "--focus", "metrics"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Metrics"));
}

#[test]
fn test_context_focus_summary() {
    let tmp = setup_project();
    cmd()
        .current_dir(tmp.path())
        .args(["context", "--focus", "summary"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Test Project"));
}

#[test]
fn test_context_focus_invalid() {
    let tmp = setup_project();
    cmd()
        .current_dir(tmp.path())
        .args(["context", "--focus", "bogus"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Unknown focus area"));
}

// ── Error Cases ──────────────────────────────────────────────────────────

#[test]
fn test_add_invalid_type() {
    let tmp = setup_project();
    cmd()
        .current_dir(tmp.path())
        .args(["add", "Bad type", "--type", "invalid-type"])
        .assert()
        .failure();
}

#[test]
fn test_status_update_nonexistent_item() {
    let tmp = setup_project();
    cmd()
        .current_dir(tmp.path())
        .args(["status", "TASK-999", "in-progress"])
        .assert()
        .failure();
}

#[test]
fn test_plan_for_nonexistent_task() {
    let tmp = setup_project();
    cmd()
        .current_dir(tmp.path())
        .args(["plan", "TASK-999"])
        .assert()
        .failure();
}

#[test]
fn test_plan_for_non_task_item() {
    let tmp = setup_project();
    let output = cmd()
        .current_dir(tmp.path())
        .args(["epic", "Phase 1"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let epic_id = extract_id(&output.stdout);

    cmd()
        .current_dir(tmp.path())
        .args(["plan", &epic_id])
        .assert()
        .failure();
}

#[test]
fn test_show_invalid_id_format() {
    let tmp = setup_project();
    cmd()
        .current_dir(tmp.path())
        .args(["show", "not-a-valid-id"])
        .assert()
        .failure();
}

#[test]
fn test_add_invalid_priority() {
    let tmp = setup_project();
    cmd()
        .current_dir(tmp.path())
        .args(["add", "Bad priority", "--priority", "ultra-high"])
        .assert()
        .failure();
}

#[test]
fn test_add_invalid_effort() {
    let tmp = setup_project();
    cmd()
        .current_dir(tmp.path())
        .args(["add", "Bad effort", "--effort", "enormous"])
        .assert()
        .failure();
}

// ── Full workflow ────────────────────────────────────────────────────────

#[test]
fn test_full_workflow() {
    let tmp = setup_project();

    // Create epic
    let epic_out = cmd()
        .current_dir(tmp.path())
        .args(["epic", "Phase 1", "--priority", "high"])
        .output()
        .unwrap();
    assert!(epic_out.status.success(), "stderr: {}", String::from_utf8_lossy(&epic_out.stderr));
    let epic_id = extract_id(&epic_out.stdout);

    // Create tasks
    let task1_out = cmd()
        .current_dir(tmp.path())
        .args([
            "add",
            "Build API",
            "--priority",
            "high",
            "--epic",
            &epic_id,
            "--tags",
            "api",
        ])
        .output()
        .unwrap();
    assert!(task1_out.status.success(), "stderr: {}", String::from_utf8_lossy(&task1_out.stderr));
    let task1_id = extract_id(&task1_out.stdout);

    let task2_out = cmd()
        .current_dir(tmp.path())
        .args([
            "add",
            "Build UI",
            "--priority",
            "medium",
            "--epic",
            &epic_id,
            "--tags",
            "ui",
        ])
        .output()
        .unwrap();
    assert!(task2_out.status.success(), "stderr: {}", String::from_utf8_lossy(&task2_out.stderr));
    let task2_id = extract_id(&task2_out.stdout);

    // Link
    cmd()
        .current_dir(tmp.path())
        .args(["link", &task2_id, &task1_id, "--relation", "depends-on"])
        .assert()
        .success();

    // Start work
    cmd()
        .current_dir(tmp.path())
        .args(["start", &task1_id, "--user", "alice"])
        .assert()
        .success();

    // Create plan
    cmd()
        .current_dir(tmp.path())
        .args(["plan", &task1_id])
        .assert()
        .success();

    // Finish
    cmd()
        .current_dir(tmp.path())
        .args(["done", &task1_id])
        .assert()
        .success();

    // Sync everything
    cmd()
        .current_dir(tmp.path())
        .arg("sync")
        .assert()
        .success();

    // Check references
    cmd()
        .current_dir(tmp.path())
        .arg("check")
        .assert()
        .success();

    // List should show items
    cmd()
        .current_dir(tmp.path())
        .arg("ls")
        .assert()
        .success()
        .stdout(predicate::str::contains(&task1_id))
        .stdout(predicate::str::contains(&task2_id));

    // Dashboard should work
    cmd()
        .current_dir(tmp.path())
        .arg("dashboard")
        .assert()
        .success();

    // Metrics should work
    cmd()
        .current_dir(tmp.path())
        .arg("metrics")
        .assert()
        .success();
}

// ── Template flag ────────────────────────────────────────────────────

#[test]
fn test_add_with_template_bug() {
    let tmp = setup_project();
    let output = cmd()
        .current_dir(tmp.path())
        .args(["add", "Login crash", "--type", "bug", "--template", "bug"])
        .output()
        .unwrap();
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
    let task_id = extract_id(&output.stdout);

    let content =
        std::fs::read_to_string(tmp.path().join(format!(".markplane/backlog/items/{}.md", task_id))).unwrap();
    assert!(content.contains("## Steps to Reproduce"));
}

#[test]
fn test_plan_with_template_refactor() {
    let tmp = setup_project();
    let task_out = cmd()
        .current_dir(tmp.path())
        .args(["add", "Refactor auth"])
        .output()
        .unwrap();
    assert!(task_out.status.success(), "stderr: {}", String::from_utf8_lossy(&task_out.stderr));
    let task_id = extract_id(&task_out.stdout);

    let plan_out = cmd()
        .current_dir(tmp.path())
        .args(["plan", &task_id, "--template", "refactor"])
        .output()
        .unwrap();
    assert!(plan_out.status.success(), "stderr: {}", String::from_utf8_lossy(&plan_out.stderr));
    let plan_id = extract_id(&plan_out.stdout);

    let content =
        std::fs::read_to_string(tmp.path().join(format!(".markplane/plans/items/{}.md", plan_id))).unwrap();
    assert!(content.contains("## Motivation"));
    assert!(content.contains("## Current State"));
}

#[test]
fn test_note_with_template_research() {
    let tmp = setup_project();
    let output = cmd()
        .current_dir(tmp.path())
        .args(["note", "Caching study", "--template", "research"])
        .output()
        .unwrap();
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
    let note_id = extract_id(&output.stdout);

    let content =
        std::fs::read_to_string(tmp.path().join(format!(".markplane/notes/items/{}.md", note_id))).unwrap();
    assert!(content.contains("## Findings"));
    assert!(content.contains("## Recommendations"));
}
