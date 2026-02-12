use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

fn cmd() -> Command {
    assert_cmd::cargo::cargo_bin_cmd!("markplane")
}

/// Run markplane inside a temp dir, initializing it first.
fn setup_project() -> TempDir {
    let tmp = TempDir::new().unwrap();
    cmd()
        .current_dir(tmp.path())
        .args(["init", "--name", "Test Project"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Initialized Markplane project"));
    tmp
}

// ── Init ─────────────────────────────────────────────────────────────────

#[test]
fn test_init_creates_structure() {
    let tmp = TempDir::new().unwrap();
    cmd()
        .current_dir(tmp.path())
        .args(["init", "--name", "My Project", "--description", "A test"])
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
        .arg("init")
        .assert()
        .success()
        .stdout(predicate::str::contains("Initialized Markplane project"));
}

#[test]
fn test_init_already_initialized() {
    let tmp = setup_project();
    cmd()
        .current_dir(tmp.path())
        .arg("init")
        .assert()
        .failure();
}

// ── Add ──────────────────────────────────────────────────────────────────

#[test]
fn test_add_basic() {
    let tmp = setup_project();
    cmd()
        .current_dir(tmp.path())
        .args(["add", "Fix login bug"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Created TASK-001"))
        .stdout(predicate::str::contains("Fix login bug"));

    assert!(tmp.path().join(".markplane/backlog/items/TASK-001.md").is_file());
}

#[test]
fn test_add_with_flags() {
    let tmp = setup_project();
    cmd()
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
        .assert()
        .success()
        .stdout(predicate::str::contains("Created TASK-001"));

    // Verify the file contains the right metadata
    let content =
        std::fs::read_to_string(tmp.path().join(".markplane/backlog/items/TASK-001.md")).unwrap();
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
    cmd()
        .current_dir(tmp.path())
        .args(["epic", "Phase 1"])
        .assert()
        .success();

    cmd()
        .current_dir(tmp.path())
        .args(["add", "Task in epic", "--epic", "EPIC-001"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Created TASK-001"));

    let content =
        std::fs::read_to_string(tmp.path().join(".markplane/backlog/items/TASK-001.md")).unwrap();
    assert!(content.contains("epic: EPIC-001"));
}

#[test]
fn test_add_sequential_ids() {
    let tmp = setup_project();
    cmd()
        .current_dir(tmp.path())
        .args(["add", "First"])
        .assert()
        .success()
        .stdout(predicate::str::contains("TASK-001"));
    cmd()
        .current_dir(tmp.path())
        .args(["add", "Second"])
        .assert()
        .success()
        .stdout(predicate::str::contains("TASK-002"));
    cmd()
        .current_dir(tmp.path())
        .args(["add", "Third"])
        .assert()
        .success()
        .stdout(predicate::str::contains("TASK-003"));
}

// ── Show ─────────────────────────────────────────────────────────────────

#[test]
fn test_show_task() {
    let tmp = setup_project();
    cmd()
        .current_dir(tmp.path())
        .args(["add", "Show me", "--priority", "high"])
        .assert()
        .success();

    cmd()
        .current_dir(tmp.path())
        .args(["show", "TASK-001"])
        .assert()
        .success()
        .stdout(predicate::str::contains("TASK-001"))
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
    cmd()
        .current_dir(tmp.path())
        .args(["epic", "Phase 1", "--priority", "high"])
        .assert()
        .success();

    cmd()
        .current_dir(tmp.path())
        .args(["show", "EPIC-001"])
        .assert()
        .success()
        .stdout(predicate::str::contains("EPIC-001"))
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
    cmd()
        .current_dir(tmp.path())
        .args(["add", "Task A"])
        .assert()
        .success();
    cmd()
        .current_dir(tmp.path())
        .args(["add", "Task B"])
        .assert()
        .success();

    cmd()
        .current_dir(tmp.path())
        .arg("ls")
        .assert()
        .success()
        .stdout(predicate::str::contains("TASK-001"))
        .stdout(predicate::str::contains("TASK-002"))
        .stdout(predicate::str::contains("Task A"))
        .stdout(predicate::str::contains("Task B"));
}

#[test]
fn test_ls_filter_status() {
    let tmp = setup_project();
    cmd()
        .current_dir(tmp.path())
        .args(["add", "Draft item"])
        .assert()
        .success();
    cmd()
        .current_dir(tmp.path())
        .args(["add", "Progress item"])
        .assert()
        .success();
    cmd()
        .current_dir(tmp.path())
        .args(["status", "TASK-002", "in-progress"])
        .assert()
        .success();

    cmd()
        .current_dir(tmp.path())
        .args(["ls", "--status", "in-progress"])
        .assert()
        .success()
        .stdout(predicate::str::contains("TASK-002"))
        .stdout(predicate::str::contains("Draft item").not());
}

#[test]
fn test_ls_epics() {
    let tmp = setup_project();
    cmd()
        .current_dir(tmp.path())
        .args(["epic", "Phase 1"])
        .assert()
        .success();

    cmd()
        .current_dir(tmp.path())
        .args(["ls", "epics"])
        .assert()
        .success()
        .stdout(predicate::str::contains("EPIC-001"))
        .stdout(predicate::str::contains("Phase 1"));
}

#[test]
fn test_ls_plans() {
    let tmp = setup_project();
    cmd()
        .current_dir(tmp.path())
        .args(["add", "Some task"])
        .assert()
        .success();
    cmd()
        .current_dir(tmp.path())
        .args(["plan", "TASK-001"])
        .assert()
        .success();

    cmd()
        .current_dir(tmp.path())
        .args(["ls", "plans"])
        .assert()
        .success()
        .stdout(predicate::str::contains("PLAN-001"));
}

#[test]
fn test_ls_notes() {
    let tmp = setup_project();
    cmd()
        .current_dir(tmp.path())
        .args(["note", "Research topic", "--type", "research"])
        .assert()
        .success();

    cmd()
        .current_dir(tmp.path())
        .args(["ls", "notes"])
        .assert()
        .success()
        .stdout(predicate::str::contains("NOTE-001"))
        .stdout(predicate::str::contains("Research topic"));
}

// ── Status ───────────────────────────────────────────────────────────────

#[test]
fn test_status_update() {
    let tmp = setup_project();
    cmd()
        .current_dir(tmp.path())
        .args(["add", "Status test"])
        .assert()
        .success();

    cmd()
        .current_dir(tmp.path())
        .args(["status", "TASK-001", "in-progress"])
        .assert()
        .success()
        .stdout(predicate::str::contains("TASK-001"))
        .stdout(predicate::str::contains("in-progress"));

    // Verify the change
    cmd()
        .current_dir(tmp.path())
        .args(["show", "TASK-001"])
        .assert()
        .success()
        .stdout(predicate::str::contains("in-progress"));
}

#[test]
fn test_status_invalid() {
    let tmp = setup_project();
    cmd()
        .current_dir(tmp.path())
        .args(["add", "Status test"])
        .assert()
        .success();

    cmd()
        .current_dir(tmp.path())
        .args(["status", "TASK-001", "invalid-status"])
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
    cmd()
        .current_dir(tmp.path())
        .args(["add", "Start/done test"])
        .assert()
        .success();

    cmd()
        .current_dir(tmp.path())
        .args(["start", "TASK-001", "--user", "alice"])
        .assert()
        .success()
        .stdout(predicate::str::contains("in-progress"))
        .stdout(predicate::str::contains("alice"));

    // Verify status and assignee
    let content =
        std::fs::read_to_string(tmp.path().join(".markplane/backlog/items/TASK-001.md")).unwrap();
    assert!(content.contains("status: in-progress"));
    assert!(content.contains("assignee: alice"));

    cmd()
        .current_dir(tmp.path())
        .args(["done", "TASK-001"])
        .assert()
        .success()
        .stdout(predicate::str::contains("done"));
}

// ── Epic / Note / Plan ───────────────────────────────────────────────────

#[test]
fn test_epic_creation() {
    let tmp = setup_project();
    cmd()
        .current_dir(tmp.path())
        .args(["epic", "Phase 1", "--priority", "high"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Created EPIC-001"))
        .stdout(predicate::str::contains("Phase 1"));

    assert!(tmp
        .path()
        .join(".markplane/roadmap/items/EPIC-001.md")
        .is_file());
}

#[test]
fn test_note_creation() {
    let tmp = setup_project();
    cmd()
        .current_dir(tmp.path())
        .args(["note", "Research caching", "--type", "research", "--tags", "cache,perf"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Created NOTE-001"))
        .stdout(predicate::str::contains("Research caching"));

    assert!(tmp
        .path()
        .join(".markplane/notes/items/NOTE-001.md")
        .is_file());
}

#[test]
fn test_plan_creation() {
    let tmp = setup_project();
    cmd()
        .current_dir(tmp.path())
        .args(["add", "Dark mode"])
        .assert()
        .success();

    cmd()
        .current_dir(tmp.path())
        .args(["plan", "TASK-001", "--title", "Dark mode plan"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Created PLAN-001"))
        .stdout(predicate::str::contains("Dark mode plan"))
        .stdout(predicate::str::contains("Linked to TASK-001"));

    // Verify task has plan linked
    let content =
        std::fs::read_to_string(tmp.path().join(".markplane/backlog/items/TASK-001.md")).unwrap();
    assert!(content.contains("PLAN-001"));
}

// ── Promote ──────────────────────────────────────────────────────────────

#[test]
fn test_promote_note_to_task() {
    let tmp = setup_project();
    cmd()
        .current_dir(tmp.path())
        .args(["note", "Good idea", "--type", "idea", "--tags", "cool"])
        .assert()
        .success();

    cmd()
        .current_dir(tmp.path())
        .args(["promote", "NOTE-001", "--priority", "high"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Promoted NOTE-001"))
        .stdout(predicate::str::contains("TASK-001"));

    assert!(tmp
        .path()
        .join(".markplane/backlog/items/TASK-001.md")
        .is_file());
}

#[test]
fn test_promote_non_note_fails() {
    let tmp = setup_project();
    cmd()
        .current_dir(tmp.path())
        .args(["add", "Not a note"])
        .assert()
        .success();

    cmd()
        .current_dir(tmp.path())
        .args(["promote", "TASK-001"])
        .assert()
        .failure();
}

// ── Assign ───────────────────────────────────────────────────────────────

#[test]
fn test_assign() {
    let tmp = setup_project();
    cmd()
        .current_dir(tmp.path())
        .args(["add", "Assign test"])
        .assert()
        .success();

    cmd()
        .current_dir(tmp.path())
        .args(["assign", "TASK-001", "@daniel"])
        .assert()
        .success()
        .stdout(predicate::str::contains("assigned to daniel"));

    let content =
        std::fs::read_to_string(tmp.path().join(".markplane/backlog/items/TASK-001.md")).unwrap();
    assert!(content.contains("assignee: daniel"));
}

// ── Link ─────────────────────────────────────────────────────────────────

#[test]
fn test_link_blocks() {
    let tmp = setup_project();
    cmd()
        .current_dir(tmp.path())
        .args(["add", "Blocker"])
        .assert()
        .success();
    cmd()
        .current_dir(tmp.path())
        .args(["add", "Blocked"])
        .assert()
        .success();

    cmd()
        .current_dir(tmp.path())
        .args(["link", "TASK-001", "--blocks", "TASK-002"])
        .assert()
        .success()
        .stdout(predicate::str::contains("TASK-001 blocks TASK-002"));

    // Verify bidirectional
    let blocker =
        std::fs::read_to_string(tmp.path().join(".markplane/backlog/items/TASK-001.md")).unwrap();
    assert!(blocker.contains("TASK-002"));

    let blocked =
        std::fs::read_to_string(tmp.path().join(".markplane/backlog/items/TASK-002.md")).unwrap();
    assert!(blocked.contains("TASK-001"));
}

#[test]
fn test_link_no_flags_fails() {
    let tmp = setup_project();
    cmd()
        .current_dir(tmp.path())
        .args(["add", "Lonely"])
        .assert()
        .success();

    cmd()
        .current_dir(tmp.path())
        .args(["link", "TASK-001"])
        .assert()
        .failure();
}

// ── Tag ──────────────────────────────────────────────────────────────────

#[test]
fn test_tag() {
    let tmp = setup_project();
    cmd()
        .current_dir(tmp.path())
        .args(["add", "Tag test"])
        .assert()
        .success();

    cmd()
        .current_dir(tmp.path())
        .args(["tag", "TASK-001", "ui,frontend"])
        .assert()
        .success()
        .stdout(predicate::str::contains("tagged with: ui, frontend"));

    let content =
        std::fs::read_to_string(tmp.path().join(".markplane/backlog/items/TASK-001.md")).unwrap();
    assert!(content.contains("ui"));
    assert!(content.contains("frontend"));
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
        .stdout(predicate::str::contains("assign"))
        .stdout(predicate::str::contains("link"))
        .stdout(predicate::str::contains("tag"))
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
    cmd()
        .current_dir(tmp.path())
        .args(["add", "Old item"])
        .assert()
        .success();

    // Manually backdate the item's updated field to make it stale
    let item_path = tmp.path().join(".markplane/backlog/items/TASK-001.md");
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
        .stdout(predicate::str::contains("TASK-001"))
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

    cmd()
        .current_dir(tmp.path())
        .arg("archive")
        .assert()
        .success()
        .stdout(predicate::str::contains("No items eligible"));
}

#[test]
fn test_archive_dry_run() {
    let tmp = setup_project();
    cmd()
        .current_dir(tmp.path())
        .args(["add", "To archive"])
        .assert()
        .success();
    cmd()
        .current_dir(tmp.path())
        .args(["status", "TASK-001", "done"])
        .assert()
        .success();

    // Backdate to make archivable
    let item_path = tmp.path().join(".markplane/backlog/items/TASK-001.md");
    let content = std::fs::read_to_string(&item_path).unwrap();
    let today = chrono::Local::now()
        .date_naive()
        .format("%Y-%m-%d")
        .to_string();
    let content = content.replace(
        &format!("updated: {}", today),
        "updated: 2020-01-01",
    );
    std::fs::write(&item_path, content).unwrap();

    cmd()
        .current_dir(tmp.path())
        .args(["archive", "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Would archive"))
        .stdout(predicate::str::contains("TASK-001"));

    // File should still be in active dir (not moved)
    assert!(tmp.path().join(".markplane/backlog/items/TASK-001.md").is_file());
}

#[test]
fn test_archive_actual() {
    let tmp = setup_project();
    cmd()
        .current_dir(tmp.path())
        .args(["add", "To archive"])
        .assert()
        .success();
    cmd()
        .current_dir(tmp.path())
        .args(["status", "TASK-001", "done"])
        .assert()
        .success();

    // Backdate
    let item_path = tmp.path().join(".markplane/backlog/items/TASK-001.md");
    let content = std::fs::read_to_string(&item_path).unwrap();
    let today = chrono::Local::now()
        .date_naive()
        .format("%Y-%m-%d")
        .to_string();
    let content = content.replace(
        &format!("updated: {}", today),
        "updated: 2020-01-01",
    );
    std::fs::write(&item_path, content).unwrap();

    cmd()
        .current_dir(tmp.path())
        .arg("archive")
        .assert()
        .success()
        .stdout(predicate::str::contains("Archived TASK-001"));

    // File should be in archive dir
    assert!(tmp
        .path()
        .join(".markplane/backlog/archive/TASK-001.md")
        .is_file());
    assert!(!tmp.path().join(".markplane/backlog/items/TASK-001.md").is_file());
}

#[test]
fn test_archive_keep_cancelled() {
    let tmp = setup_project();
    cmd()
        .current_dir(tmp.path())
        .args(["add", "To cancel"])
        .assert()
        .success();
    cmd()
        .current_dir(tmp.path())
        .args(["status", "TASK-001", "cancelled"])
        .assert()
        .success();

    // Backdate
    let item_path = tmp.path().join(".markplane/backlog/items/TASK-001.md");
    let content = std::fs::read_to_string(&item_path).unwrap();
    let today = chrono::Local::now()
        .date_naive()
        .format("%Y-%m-%d")
        .to_string();
    let content = content.replace(
        &format!("updated: {}", today),
        "updated: 2020-01-01",
    );
    std::fs::write(&item_path, content).unwrap();

    // Default config has keep_cancelled: true, so cancelled items should NOT be archived
    cmd()
        .current_dir(tmp.path())
        .arg("archive")
        .assert()
        .success()
        .stdout(predicate::str::contains("No items eligible"));

    // File should still be in active dir
    assert!(tmp.path().join(".markplane/backlog/items/TASK-001.md").is_file());
}

// ── Graph ─────────────────────────────────────────────────────────────────

#[test]
fn test_graph() {
    let tmp = setup_project();
    cmd()
        .current_dir(tmp.path())
        .args(["add", "Blocker"])
        .assert()
        .success();
    cmd()
        .current_dir(tmp.path())
        .args(["add", "Blocked"])
        .assert()
        .success();
    cmd()
        .current_dir(tmp.path())
        .args(["link", "TASK-001", "--blocks", "TASK-002"])
        .assert()
        .success();

    cmd()
        .current_dir(tmp.path())
        .args(["graph", "TASK-001"])
        .assert()
        .success()
        .stdout(predicate::str::contains("TASK-001"))
        .stdout(predicate::str::contains("TASK-002"));
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
fn test_context_for_item() {
    let tmp = setup_project();
    cmd()
        .current_dir(tmp.path())
        .args(["add", "Context item", "--priority", "high"])
        .assert()
        .success();

    cmd()
        .current_dir(tmp.path())
        .args(["context", "--item", "TASK-001"])
        .assert()
        .success()
        .stdout(predicate::str::contains("TASK-001"))
        .stdout(predicate::str::contains("Context item"));
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
    cmd()
        .current_dir(tmp.path())
        .args(["epic", "Phase 1"])
        .assert()
        .success();

    cmd()
        .current_dir(tmp.path())
        .args(["plan", "EPIC-001"])
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
    cmd()
        .current_dir(tmp.path())
        .args(["epic", "Phase 1", "--priority", "high"])
        .assert()
        .success();

    // Create tasks
    cmd()
        .current_dir(tmp.path())
        .args([
            "add",
            "Build API",
            "--priority",
            "high",
            "--epic",
            "EPIC-001",
            "--tags",
            "api",
        ])
        .assert()
        .success();
    cmd()
        .current_dir(tmp.path())
        .args([
            "add",
            "Build UI",
            "--priority",
            "medium",
            "--epic",
            "EPIC-001",
            "--tags",
            "ui",
        ])
        .assert()
        .success();

    // Link
    cmd()
        .current_dir(tmp.path())
        .args(["link", "TASK-002", "--depends-on", "TASK-001"])
        .assert()
        .success();

    // Start work
    cmd()
        .current_dir(tmp.path())
        .args(["start", "TASK-001", "--user", "alice"])
        .assert()
        .success();

    // Create plan
    cmd()
        .current_dir(tmp.path())
        .args(["plan", "TASK-001"])
        .assert()
        .success();

    // Finish
    cmd()
        .current_dir(tmp.path())
        .args(["done", "TASK-001"])
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
        .stdout(predicate::str::contains("TASK-001"))
        .stdout(predicate::str::contains("TASK-002"));

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
