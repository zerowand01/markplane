use assert_cmd::Command;
use serde_json::{json, Value};
use tempfile::TempDir;

fn mcp_cmd() -> Command {
    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("markplane");
    cmd.arg("mcp");
    cmd
}

/// Initialize a markplane project in a tempdir and return the tempdir.
fn setup_project() -> TempDir {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().join(".markplane");
    markplane_core::Project::init(root, "Test Project", "A test project").unwrap();
    tmp
}

/// Helper: send a JSON-RPC request via stdin and return the parsed response.
fn send_request(tmp: &TempDir, request: &Value) -> Value {
    let input = format!("{}\n", serde_json::to_string(request).unwrap());
    let output = mcp_cmd()
        .arg("--project")
        .arg(tmp.path().to_str().unwrap())
        .write_stdin(input)
        .output()
        .expect("failed to run markplane mcp");

    let stdout = String::from_utf8_lossy(&output.stdout);
    // The response is on the first non-empty line
    for line in stdout.lines() {
        let trimmed = line.trim();
        if !trimmed.is_empty() {
            return serde_json::from_str(trimmed).expect("failed to parse JSON-RPC response");
        }
    }
    panic!("No response received from MCP server. stdout: {}", stdout);
}

/// Extract the ID from an MCP markplane_add tool response.
/// The response text is JSON like: {"id":"TASK-k7x9m","title":"..."}
fn extract_id_from_response(response: &Value) -> String {
    let text = response["result"]["content"][0]["text"].as_str().unwrap();
    let result: Value = serde_json::from_str(text).unwrap();
    result["id"].as_str().unwrap().to_string()
}

// ── Initialize ──────────────────────────────────────────────────────────

#[test]
fn test_initialize() {
    let tmp = setup_project();
    let response = send_request(
        &tmp,
        &json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {}
        }),
    );

    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 1);
    assert!(response["result"]["serverInfo"]["name"]
        .as_str()
        .unwrap()
        .contains("markplane"));
    assert!(response["result"]["capabilities"]["tools"].is_object());
    assert!(response["result"]["capabilities"]["resources"].is_object());
    // Protocol version should be 2025-11-25
    assert_eq!(response["result"]["protocolVersion"], "2025-11-25");
    // serverInfo should include a description
    let description = response["result"]["serverInfo"]["description"]
        .as_str()
        .unwrap();
    assert!(description.contains("markdown-first"));
    // instructions field should be present and include project name
    let instructions = response["result"]["instructions"].as_str().unwrap();
    assert!(!instructions.is_empty());
    assert!(instructions.contains("Test Project"));
    assert!(instructions.contains("TASK-NNN"));
    assert!(instructions.contains("EPIC-NNN"));
    assert!(instructions.contains("PLAN-NNN"));
    assert!(instructions.contains("NOTE-NNN"));
    // instructions should describe file editing workflow
    assert!(instructions.contains("File Editing"));
    assert!(instructions.contains("free-form markdown"));
}

// ── Ping ─────────────────────────────────────────────────────────────────

#[test]
fn test_ping() {
    let tmp = setup_project();
    let response = send_request(
        &tmp,
        &json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "ping",
            "params": {}
        }),
    );

    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 2);
    assert!(response["result"].is_object());
    assert!(response["error"].is_null());
}

// ── Method Not Found ─────────────────────────────────────────────────────

#[test]
fn test_unknown_method_returns_error() {
    let tmp = setup_project();
    let response = send_request(
        &tmp,
        &json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "nonexistent/method",
            "params": {}
        }),
    );

    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 3);
    assert!(response["error"].is_object());
    assert_eq!(response["error"]["code"], -32601); // METHOD_NOT_FOUND
    assert!(response["error"]["message"]
        .as_str()
        .unwrap()
        .contains("Method not found"));
}

// ── Malformed JSON ───────────────────────────────────────────────────────

#[test]
fn test_malformed_json_returns_parse_error() {
    let tmp = setup_project();
    let output = mcp_cmd()
        .arg("--project")
        .arg(tmp.path().to_str().unwrap())
        .write_stdin("this is not valid json\n")
        .output()
        .expect("failed to run");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let response: Value = stdout
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| serde_json::from_str(l.trim()).unwrap())
        .next()
        .expect("no response");

    assert_eq!(response["error"]["code"], -32700); // PARSE_ERROR
}

// ── Tools List ───────────────────────────────────────────────────────────

#[test]
fn test_tools_list() {
    let tmp = setup_project();
    let response = send_request(
        &tmp,
        &json!({
            "jsonrpc": "2.0",
            "id": 10,
            "method": "tools/list",
            "params": {}
        }),
    );

    assert!(response["error"].is_null());
    let tools = &response["result"]["tools"];
    assert!(tools.is_array());
    let tool_names: Vec<&str> = tools
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["name"].as_str().unwrap())
        .collect();
    assert!(tool_names.contains(&"markplane_summary"));
    assert!(tool_names.contains(&"markplane_query"));
    assert!(tool_names.contains(&"markplane_show"));
    assert!(tool_names.contains(&"markplane_add"));
    assert!(tool_names.contains(&"markplane_update"));
    assert!(tool_names.contains(&"markplane_start"));
    assert!(tool_names.contains(&"markplane_done"));
    assert!(tool_names.contains(&"markplane_sync"));
    assert!(tool_names.contains(&"markplane_context"));
    assert!(tool_names.contains(&"markplane_graph"));
    assert!(tool_names.contains(&"markplane_promote"));
    assert!(tool_names.contains(&"markplane_plan"));
    assert!(tool_names.contains(&"markplane_link"));
    assert!(tool_names.contains(&"markplane_check"));
    assert!(tool_names.contains(&"markplane_archive"));
    assert!(tool_names.contains(&"markplane_unarchive"));
    assert!(tool_names.contains(&"markplane_move"));
    assert!(!tool_names.contains(&"markplane_stale"), "markplane_stale should be removed");
    assert_eq!(tool_names.len(), 17, "Expected 17 tools, got: {:?}", tool_names);
}

// ── Resources List ───────────────────────────────────────────────────────

#[test]
fn test_resources_list() {
    let tmp = setup_project();
    let response = send_request(
        &tmp,
        &json!({
            "jsonrpc": "2.0",
            "id": 11,
            "method": "resources/list",
            "params": {}
        }),
    );

    assert!(response["error"].is_null());
    let resources = &response["result"]["resources"];
    assert!(resources.is_array());
    let uris: Vec<&str> = resources
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["uri"].as_str().unwrap())
        .collect();
    assert!(uris.contains(&"markplane://summary"));
    assert!(uris.contains(&"markplane://active-work"));
    assert!(uris.contains(&"markplane://blocked"));

    // PLAN and NOTE resource templates should be present
    let templates = &response["result"]["resourceTemplates"];
    assert!(templates.is_array());
    let template_uris: Vec<&str> = templates
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["uriTemplate"].as_str().unwrap())
        .collect();
    assert!(template_uris.contains(&"markplane://task/{id}"));
    assert!(template_uris.contains(&"markplane://epic/{id}"));
    assert!(template_uris.contains(&"markplane://plan/{id}"));
    assert!(template_uris.contains(&"markplane://note/{id}"));
}

// ── Tool: markplane_summary ──────────────────────────────────────────────

#[test]
fn test_tool_summary() {
    let tmp = setup_project();
    let response = send_request(
        &tmp,
        &json!({
            "jsonrpc": "2.0",
            "id": 20,
            "method": "tools/call",
            "params": {
                "name": "markplane_summary",
                "arguments": {}
            }
        }),
    );

    assert!(response["error"].is_null());
    let text = response["result"]["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("Test Project"));
    assert!(text.contains("Total items: 0") || text.contains("Backlog: 0 items"));
}

// ── Tool: markplane_add ──────────────────────────────────────────────────

#[test]
fn test_tool_add() {
    let tmp = setup_project();
    let response = send_request(
        &tmp,
        &json!({
            "jsonrpc": "2.0",
            "id": 21,
            "method": "tools/call",
            "params": {
                "name": "markplane_add",
                "arguments": {
                    "title": "Build the API",
                    "type": "feature",
                    "priority": "high",
                    "effort": "large",
                    "tags": ["api", "backend"]
                }
            }
        }),
    );

    assert!(response["error"].is_null());
    let text = response["result"]["content"][0]["text"].as_str().unwrap();
    let result: Value = serde_json::from_str(text).unwrap();
    let id = result["id"].as_str().unwrap();
    assert!(id.starts_with("TASK-"), "ID should start with TASK-, got: {}", id);
    assert_eq!(id.len(), 10, "ID should be 10 chars (TASK-xxxxx), got: {}", id);
    assert_eq!(result["title"], "Build the API");
}

#[test]
fn test_tool_add_missing_title() {
    let tmp = setup_project();
    let response = send_request(
        &tmp,
        &json!({
            "jsonrpc": "2.0",
            "id": 22,
            "method": "tools/call",
            "params": {
                "name": "markplane_add",
                "arguments": {}
            }
        }),
    );

    // Should be an error because title is required
    assert!(response["error"].is_object());
}

#[test]
fn test_tool_add_epic() {
    let tmp = setup_project();
    let response = send_request(
        &tmp,
        &json!({
            "jsonrpc": "2.0",
            "id": 23,
            "method": "tools/call",
            "params": {
                "name": "markplane_add",
                "arguments": {
                    "title": "Phase 1 rollout",
                    "kind": "epic",
                    "priority": "high"
                }
            }
        }),
    );

    assert!(response["error"].is_null());
    let text = response["result"]["content"][0]["text"].as_str().unwrap();
    let result: Value = serde_json::from_str(text).unwrap();
    let id = result["id"].as_str().unwrap();
    assert!(id.starts_with("EPIC-"), "ID should start with EPIC-, got: {}", id);
    assert_eq!(result["title"], "Phase 1 rollout");
}

#[test]
fn test_tool_add_note() {
    let tmp = setup_project();
    let response = send_request(
        &tmp,
        &json!({
            "jsonrpc": "2.0",
            "id": 24,
            "method": "tools/call",
            "params": {
                "name": "markplane_add",
                "arguments": {
                    "title": "Research caching strategies",
                    "kind": "note",
                    "note_type": "research",
                    "tags": ["perf"]
                }
            }
        }),
    );

    assert!(response["error"].is_null());
    let text = response["result"]["content"][0]["text"].as_str().unwrap();
    let result: Value = serde_json::from_str(text).unwrap();
    let id = result["id"].as_str().unwrap();
    assert!(id.starts_with("NOTE-"), "ID should start with NOTE-, got: {}", id);
    assert_eq!(result["title"], "Research caching strategies");
}

#[test]
fn test_tool_add_invalid_kind() {
    let tmp = setup_project();
    let response = send_request(
        &tmp,
        &json!({
            "jsonrpc": "2.0",
            "id": 25,
            "method": "tools/call",
            "params": {
                "name": "markplane_add",
                "arguments": {
                    "title": "Bad kind",
                    "kind": "widget"
                }
            }
        }),
    );

    assert!(response["error"].is_object());
}

// ── Tool: markplane_show ─────────────────────────────────────────────────

#[test]
fn test_tool_show() {
    let tmp = setup_project();
    // Add an item first
    let add_response = send_request(
        &tmp,
        &json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": {
                "name": "markplane_add",
                "arguments": { "title": "Show me" }
            }
        }),
    );
    let task_id = extract_id_from_response(&add_response);

    let response = send_request(
        &tmp,
        &json!({
            "jsonrpc": "2.0",
            "id": 30,
            "method": "tools/call",
            "params": {
                "name": "markplane_show",
                "arguments": { "id": task_id }
            }
        }),
    );

    assert!(response["error"].is_null());
    let text = response["result"]["content"][0]["text"].as_str().unwrap();
    assert!(text.contains(&task_id));
    assert!(text.contains("Show me"));
}

#[test]
fn test_tool_show_invalid_id() {
    let tmp = setup_project();
    let response = send_request(
        &tmp,
        &json!({
            "jsonrpc": "2.0",
            "id": 31,
            "method": "tools/call",
            "params": {
                "name": "markplane_show",
                "arguments": { "id": "TASK-999" }
            }
        }),
    );

    assert!(response["error"].is_object());
}

// ── Tool: markplane_query ────────────────────────────────────────────────

#[test]
fn test_tool_query() {
    let tmp = setup_project();
    // Add items
    send_request(
        &tmp,
        &json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": {
                "name": "markplane_add",
                "arguments": { "title": "Task A", "priority": "high" }
            }
        }),
    );
    send_request(
        &tmp,
        &json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/call",
            "params": {
                "name": "markplane_add",
                "arguments": { "title": "Task B", "priority": "low" }
            }
        }),
    );

    // Query all
    let response = send_request(
        &tmp,
        &json!({
            "jsonrpc": "2.0",
            "id": 40,
            "method": "tools/call",
            "params": {
                "name": "markplane_query",
                "arguments": {}
            }
        }),
    );

    assert!(response["error"].is_null());
    let text = response["result"]["content"][0]["text"].as_str().unwrap();
    let items: Vec<Value> = serde_json::from_str(text).unwrap();
    assert_eq!(items.len(), 2);

    // Query with priority filter
    let response = send_request(
        &tmp,
        &json!({
            "jsonrpc": "2.0",
            "id": 41,
            "method": "tools/call",
            "params": {
                "name": "markplane_query",
                "arguments": { "priority": ["high"] }
            }
        }),
    );

    assert!(response["error"].is_null());
    let text = response["result"]["content"][0]["text"].as_str().unwrap();
    let items: Vec<Value> = serde_json::from_str(text).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["title"], "Task A");
}

#[test]
fn test_tool_query_tasks_include_updated() {
    let tmp = setup_project();
    send_request(
        &tmp,
        &json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": {
                "name": "markplane_add",
                "arguments": { "title": "Check updated" }
            }
        }),
    );

    let response = send_request(
        &tmp,
        &json!({
            "jsonrpc": "2.0",
            "id": 42,
            "method": "tools/call",
            "params": {
                "name": "markplane_query",
                "arguments": {}
            }
        }),
    );

    assert!(response["error"].is_null());
    let text = response["result"]["content"][0]["text"].as_str().unwrap();
    let items: Vec<Value> = serde_json::from_str(text).unwrap();
    assert_eq!(items.len(), 1);
    assert!(items[0]["updated"].is_string(), "Task query should include updated date");
}

#[test]
fn test_tool_query_epics() {
    let tmp = setup_project();
    send_request(
        &tmp,
        &json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": {
                "name": "markplane_add",
                "arguments": { "title": "Epic A", "kind": "epic", "priority": "high" }
            }
        }),
    );

    let response = send_request(
        &tmp,
        &json!({
            "jsonrpc": "2.0",
            "id": 43,
            "method": "tools/call",
            "params": {
                "name": "markplane_query",
                "arguments": { "kind": "epics" }
            }
        }),
    );

    assert!(response["error"].is_null());
    let text = response["result"]["content"][0]["text"].as_str().unwrap();
    let items: Vec<Value> = serde_json::from_str(text).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["title"], "Epic A");
    assert_eq!(items[0]["status"], "later");
    assert_eq!(items[0]["priority"], "high");
    assert!(items[0]["created"].is_string(), "epic query should include created date");
    assert!(items[0]["updated"].is_string(), "epic query should include updated date");
}

#[test]
fn test_tool_query_plans() {
    let tmp = setup_project();
    // Create a task first, then a plan linked to it
    let add_response = send_request(
        &tmp,
        &json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": {
                "name": "markplane_add",
                "arguments": { "title": "Plan target task" }
            }
        }),
    );
    let task_id = extract_id_from_response(&add_response);

    send_request(
        &tmp,
        &json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/call",
            "params": {
                "name": "markplane_plan",
                "arguments": { "task_id": task_id }
            }
        }),
    );

    let response = send_request(
        &tmp,
        &json!({
            "jsonrpc": "2.0",
            "id": 44,
            "method": "tools/call",
            "params": {
                "name": "markplane_query",
                "arguments": { "kind": "plans" }
            }
        }),
    );

    assert!(response["error"].is_null());
    let text = response["result"]["content"][0]["text"].as_str().unwrap();
    let items: Vec<Value> = serde_json::from_str(text).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["status"], "draft");
    assert!(items[0]["updated"].is_string(), "Plan query should include updated date");
}

#[test]
fn test_tool_query_notes() {
    let tmp = setup_project();
    send_request(
        &tmp,
        &json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": {
                "name": "markplane_add",
                "arguments": { "title": "Note A", "kind": "note", "note_type": "idea" }
            }
        }),
    );

    let response = send_request(
        &tmp,
        &json!({
            "jsonrpc": "2.0",
            "id": 45,
            "method": "tools/call",
            "params": {
                "name": "markplane_query",
                "arguments": { "kind": "notes" }
            }
        }),
    );

    assert!(response["error"].is_null());
    let text = response["result"]["content"][0]["text"].as_str().unwrap();
    let items: Vec<Value> = serde_json::from_str(text).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["title"], "Note A");
    assert_eq!(items[0]["type"], "idea");
    assert!(items[0]["updated"].is_string(), "Note query should include updated date");
}

#[test]
fn test_tool_query_invalid_kind() {
    let tmp = setup_project();
    let response = send_request(
        &tmp,
        &json!({
            "jsonrpc": "2.0",
            "id": 46,
            "method": "tools/call",
            "params": {
                "name": "markplane_query",
                "arguments": { "kind": "widgets" }
            }
        }),
    );

    assert!(response["error"].is_object());
}

// ── Tool: markplane_update ───────────────────────────────────────────────

#[test]
fn test_tool_update() {
    let tmp = setup_project();
    let add_response = send_request(
        &tmp,
        &json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": {
                "name": "markplane_add",
                "arguments": { "title": "Update me" }
            }
        }),
    );
    let task_id = extract_id_from_response(&add_response);

    // Update status
    let response = send_request(
        &tmp,
        &json!({
            "jsonrpc": "2.0",
            "id": 50,
            "method": "tools/call",
            "params": {
                "name": "markplane_update",
                "arguments": { "id": task_id, "status": "in-progress" }
            }
        }),
    );

    assert!(response["error"].is_null());
    let text = response["result"]["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("success"));

    // Verify via show
    let show = send_request(
        &tmp,
        &json!({
            "jsonrpc": "2.0",
            "id": 51,
            "method": "tools/call",
            "params": {
                "name": "markplane_show",
                "arguments": { "id": task_id }
            }
        }),
    );
    let text = show["result"]["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("in-progress"));
}

// ── Tool: markplane_start ────────────────────────────────────────────────

#[test]
fn test_tool_start() {
    let tmp = setup_project();
    let add_response = send_request(
        &tmp,
        &json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": {
                "name": "markplane_add",
                "arguments": { "title": "Start me" }
            }
        }),
    );
    let task_id = extract_id_from_response(&add_response);

    let response = send_request(
        &tmp,
        &json!({
            "jsonrpc": "2.0",
            "id": 60,
            "method": "tools/call",
            "params": {
                "name": "markplane_start",
                "arguments": { "id": task_id }
            }
        }),
    );

    assert!(response["error"].is_null());
    let text = response["result"]["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("success"));
}

// ── Tool: markplane_done ─────────────────────────────────────────────────

#[test]
fn test_tool_done() {
    let tmp = setup_project();
    let add_response = send_request(
        &tmp,
        &json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": {
                "name": "markplane_add",
                "arguments": { "title": "Finish me" }
            }
        }),
    );
    let task_id = extract_id_from_response(&add_response);

    let response = send_request(
        &tmp,
        &json!({
            "jsonrpc": "2.0",
            "id": 70,
            "method": "tools/call",
            "params": {
                "name": "markplane_done",
                "arguments": { "id": task_id }
            }
        }),
    );

    assert!(response["error"].is_null());
}

// ── Tool: markplane_sync ─────────────────────────────────────────────────

#[test]
fn test_tool_sync() {
    let tmp = setup_project();
    let response = send_request(
        &tmp,
        &json!({
            "jsonrpc": "2.0",
            "id": 80,
            "method": "tools/call",
            "params": {
                "name": "markplane_sync",
                "arguments": {}
            }
        }),
    );

    assert!(response["error"].is_null());
    // Verify context files were created
    assert!(tmp.path().join(".markplane/.context/summary.md").is_file());
}

// ── Tool: markplane_context ──────────────────────────────────────────────

#[test]
fn test_tool_context_default() {
    let tmp = setup_project();
    let response = send_request(
        &tmp,
        &json!({
            "jsonrpc": "2.0",
            "id": 81,
            "method": "tools/call",
            "params": {
                "name": "markplane_context",
                "arguments": {}
            }
        }),
    );

    assert!(response["error"].is_null());
    let text = response["result"]["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("Test Project"));
}

#[test]
fn test_tool_context_focus_active_work() {
    let tmp = setup_project();
    let response = send_request(
        &tmp,
        &json!({
            "jsonrpc": "2.0",
            "id": 82,
            "method": "tools/call",
            "params": {
                "name": "markplane_context",
                "arguments": { "focus": "active-work" }
            }
        }),
    );

    assert!(response["error"].is_null());
    let text = response["result"]["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("Active Work"));
}

// ── Tool: markplane_check ────────────────────────────────────────────────

#[test]
fn test_tool_check_clean() {
    let tmp = setup_project();
    send_request(
        &tmp,
        &json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": {
                "name": "markplane_add",
                "arguments": { "title": "Clean item" }
            }
        }),
    );

    let response = send_request(
        &tmp,
        &json!({
            "jsonrpc": "2.0",
            "id": 90,
            "method": "tools/call",
            "params": {
                "name": "markplane_check",
                "arguments": {}
            }
        }),
    );

    assert!(response["error"].is_null());
    let text = response["result"]["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("All cross-references and task statuses are valid"));
}

// ── Tool: markplane_graph ────────────────────────────────────────────────

#[test]
fn test_tool_graph() {
    let tmp = setup_project();
    // Create linked items
    let add1 = send_request(
        &tmp,
        &json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": {
                "name": "markplane_add",
                "arguments": { "title": "Blocker" }
            }
        }),
    );
    let task1_id = extract_id_from_response(&add1);

    let add2 = send_request(
        &tmp,
        &json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/call",
            "params": {
                "name": "markplane_add",
                "arguments": { "title": "Blocked" }
            }
        }),
    );
    let task2_id = extract_id_from_response(&add2);

    send_request(
        &tmp,
        &json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "tools/call",
            "params": {
                "name": "markplane_link",
                "arguments": { "from": task1_id, "to": task2_id, "relation": "blocks" }
            }
        }),
    );

    let response = send_request(
        &tmp,
        &json!({
            "jsonrpc": "2.0",
            "id": 110,
            "method": "tools/call",
            "params": {
                "name": "markplane_graph",
                "arguments": { "id": task1_id }
            }
        }),
    );

    assert!(response["error"].is_null());
    let text = response["result"]["content"][0]["text"].as_str().unwrap();
    assert!(text.contains(&format!("Reference Graph for {}", task1_id)));
    assert!(text.contains(&task2_id));
}

// ── Tool: markplane_promote ──────────────────────────────────────────────

#[test]
fn test_tool_promote() {
    let tmp = setup_project();
    // Create a note first (directly using core)
    let root = tmp.path().join(".markplane");
    let project = markplane_core::Project::new(root);
    let note = project
        .create_note("Good idea", "idea", vec!["cool".to_string()], None)
        .unwrap();

    let response = send_request(
        &tmp,
        &json!({
            "jsonrpc": "2.0",
            "id": 120,
            "method": "tools/call",
            "params": {
                "name": "markplane_promote",
                "arguments": { "note_id": note.id, "priority": "high" }
            }
        }),
    );

    assert!(response["error"].is_null());
    let text = response["result"]["content"][0]["text"].as_str().unwrap();
    let result: Value = serde_json::from_str(text).unwrap();
    assert!(result["id"].as_str().unwrap().starts_with("TASK-"));
    assert_eq!(result["promoted_from"], note.id);
}

// ── Tool: markplane_plan ─────────────────────────────────────────────────

#[test]
fn test_tool_plan() {
    let tmp = setup_project();
    let add_response = send_request(
        &tmp,
        &json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": {
                "name": "markplane_add",
                "arguments": { "title": "Need a plan" }
            }
        }),
    );
    let task_id = extract_id_from_response(&add_response);

    let response = send_request(
        &tmp,
        &json!({
            "jsonrpc": "2.0",
            "id": 130,
            "method": "tools/call",
            "params": {
                "name": "markplane_plan",
                "arguments": { "task_id": task_id, "title": "My plan" }
            }
        }),
    );

    assert!(response["error"].is_null());
    let text = response["result"]["content"][0]["text"].as_str().unwrap();
    let result: Value = serde_json::from_str(text).unwrap();
    assert!(result["id"].as_str().unwrap().starts_with("PLAN-"));
    assert_eq!(result["implements"], task_id);
}

// ── Tool: markplane_link ─────────────────────────────────────────────────

#[test]
fn test_tool_link_blocks() {
    let tmp = setup_project();
    let add1 = send_request(
        &tmp,
        &json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": {
                "name": "markplane_add",
                "arguments": { "title": "A" }
            }
        }),
    );
    let task1_id = extract_id_from_response(&add1);

    let add2 = send_request(
        &tmp,
        &json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/call",
            "params": {
                "name": "markplane_add",
                "arguments": { "title": "B" }
            }
        }),
    );
    let task2_id = extract_id_from_response(&add2);

    let link_resp = send_request(
        &tmp,
        &json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "tools/call",
            "params": {
                "name": "markplane_link",
                "arguments": {
                    "from": task1_id,
                    "to": task2_id,
                    "relation": "blocks"
                }
            }
        }),
    );

    assert!(link_resp["error"].is_null());
    let text = link_resp["result"]["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("success"));
}

#[test]
fn test_tool_link_missing_params() {
    let tmp = setup_project();
    let response = send_request(
        &tmp,
        &json!({
            "jsonrpc": "2.0",
            "id": 140,
            "method": "tools/call",
            "params": {
                "name": "markplane_link",
                "arguments": { "from": "TASK-001" }
            }
        }),
    );

    assert!(response["error"].is_object());
}

#[test]
fn test_tool_link_epic() {
    let tmp = setup_project();
    let add_task = send_request(
        &tmp,
        &json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": {
                "name": "markplane_add",
                "arguments": { "title": "A task" }
            }
        }),
    );
    let task_id = extract_id_from_response(&add_task);

    // Create an epic by writing the file directly (no add_epic MCP tool)
    let epic_id = {
        use std::fs;
        let epic_dir = tmp.path().join(".markplane/roadmap/items");
        fs::create_dir_all(&epic_dir).unwrap();
        let id = "EPIC-test1";
        let content = format!(
            "---\nid: {}\ntitle: Test Epic\nstatus: planned\npriority: medium\ntags: []\ndepends_on: []\ncreated: 2026-01-01\nupdated: 2026-01-01\n---\n",
            id
        );
        fs::write(epic_dir.join(format!("{}.md", id)), content).unwrap();
        id.to_string()
    };

    let link_resp = send_request(
        &tmp,
        &json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/call",
            "params": {
                "name": "markplane_link",
                "arguments": {
                    "from": task_id,
                    "to": epic_id,
                    "relation": "epic"
                }
            }
        }),
    );

    assert!(link_resp["error"].is_null());
    let text = link_resp["result"]["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("success"));

    // Verify task.epic is set
    let task_content = std::fs::read_to_string(
        tmp.path().join(format!(".markplane/backlog/items/{}.md", task_id))
    ).unwrap();
    assert!(task_content.contains(&epic_id));
}

#[test]
fn test_tool_link_plan_reciprocal() {
    let tmp = setup_project();
    let add_task = send_request(
        &tmp,
        &json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": {
                "name": "markplane_add",
                "arguments": { "title": "A task" }
            }
        }),
    );
    let task_id = extract_id_from_response(&add_task);

    let plan_resp = send_request(
        &tmp,
        &json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/call",
            "params": {
                "name": "markplane_plan",
                "arguments": { "task_id": task_id }
            }
        }),
    );
    assert!(plan_resp["error"].is_null());
    let plan_text = plan_resp["result"]["content"][0]["text"].as_str().unwrap();
    let plan_data: serde_json::Value = serde_json::from_str(plan_text).unwrap();
    let plan_id = plan_data["id"].as_str().unwrap();

    // Verify task has plan set
    let task_content = std::fs::read_to_string(
        tmp.path().join(format!(".markplane/backlog/items/{}.md", task_id))
    ).unwrap();
    assert!(task_content.contains(plan_id));

    // Verify plan has implements set
    let plan_content = std::fs::read_to_string(
        tmp.path().join(format!(".markplane/plans/items/{}.md", plan_id))
    ).unwrap();
    assert!(plan_content.contains(&task_id));
}

#[test]
fn test_tool_link_remove() {
    let tmp = setup_project();
    let add1 = send_request(
        &tmp,
        &json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": {
                "name": "markplane_add",
                "arguments": { "title": "A" }
            }
        }),
    );
    let task1_id = extract_id_from_response(&add1);

    let add2 = send_request(
        &tmp,
        &json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/call",
            "params": {
                "name": "markplane_add",
                "arguments": { "title": "B" }
            }
        }),
    );
    let task2_id = extract_id_from_response(&add2);

    // Add link
    send_request(
        &tmp,
        &json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "tools/call",
            "params": {
                "name": "markplane_link",
                "arguments": {
                    "from": task1_id,
                    "to": task2_id,
                    "relation": "blocks"
                }
            }
        }),
    );

    // Remove link
    let remove_resp = send_request(
        &tmp,
        &json!({
            "jsonrpc": "2.0",
            "id": 4,
            "method": "tools/call",
            "params": {
                "name": "markplane_link",
                "arguments": {
                    "from": task1_id,
                    "to": task2_id,
                    "relation": "blocks",
                    "remove": true
                }
            }
        }),
    );

    assert!(remove_resp["error"].is_null());

    // Verify link is removed
    let task_content = std::fs::read_to_string(
        tmp.path().join(format!(".markplane/backlog/items/{}.md", task1_id))
    ).unwrap();
    assert!(!task_content.contains(&task2_id));
}

#[test]
fn test_tool_link_related_bidirectional() {
    let tmp = setup_project();

    // Create a task
    let add_task = send_request(
        &tmp,
        &json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": {
                "name": "markplane_add",
                "arguments": { "title": "A task" }
            }
        }),
    );
    let task_id = extract_id_from_response(&add_task);

    // Create an epic
    let add_epic = send_request(
        &tmp,
        &json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/call",
            "params": {
                "name": "markplane_add",
                "arguments": { "title": "An epic", "kind": "epic" }
            }
        }),
    );
    let epic_id = extract_id_from_response(&add_epic);

    // Link with related
    let link_resp = send_request(
        &tmp,
        &json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "tools/call",
            "params": {
                "name": "markplane_link",
                "arguments": {
                    "from": task_id,
                    "to": epic_id,
                    "relation": "related"
                }
            }
        }),
    );
    assert!(link_resp["error"].is_null());

    // Verify both files contain the reciprocal link
    let task_content = std::fs::read_to_string(
        tmp.path().join(format!(".markplane/backlog/items/{}.md", task_id))
    ).unwrap();
    let epic_content = std::fs::read_to_string(
        tmp.path().join(format!(".markplane/roadmap/items/{}.md", epic_id))
    ).unwrap();
    assert!(task_content.contains(&epic_id), "Task should have related link to Epic");
    assert!(epic_content.contains(&task_id), "Epic should have related link to Task");

    // Verify show output includes the related field
    let show_resp = send_request(
        &tmp,
        &json!({
            "jsonrpc": "2.0",
            "id": 4,
            "method": "tools/call",
            "params": {
                "name": "markplane_show",
                "arguments": { "id": task_id }
            }
        }),
    );
    let show_text = show_resp["result"]["content"][0]["text"].as_str().unwrap();
    assert!(show_text.contains(&epic_id), "Show output should contain related epic ID");
}

// ── Unknown tool ─────────────────────────────────────────────────────────

#[test]
fn test_unknown_tool_returns_error() {
    let tmp = setup_project();
    let response = send_request(
        &tmp,
        &json!({
            "jsonrpc": "2.0",
            "id": 150,
            "method": "tools/call",
            "params": {
                "name": "nonexistent_tool",
                "arguments": {}
            }
        }),
    );

    assert!(response["error"].is_object());
    assert!(response["error"]["message"]
        .as_str()
        .unwrap()
        .contains("Unknown tool"));
}

// ── Missing params ───────────────────────────────────────────────────────

#[test]
fn test_tools_call_missing_params() {
    let tmp = setup_project();
    let response = send_request(
        &tmp,
        &json!({
            "jsonrpc": "2.0",
            "id": 160,
            "method": "tools/call"
        }),
    );

    assert!(response["error"].is_object());
}

#[test]
fn test_tools_call_missing_name() {
    let tmp = setup_project();
    let response = send_request(
        &tmp,
        &json!({
            "jsonrpc": "2.0",
            "id": 161,
            "method": "tools/call",
            "params": { "arguments": {} }
        }),
    );

    assert!(response["error"].is_object());
}

// ── Resources Read ───────────────────────────────────────────────────────

#[test]
fn test_resource_summary() {
    let tmp = setup_project();
    let response = send_request(
        &tmp,
        &json!({
            "jsonrpc": "2.0",
            "id": 200,
            "method": "resources/read",
            "params": { "uri": "markplane://summary" }
        }),
    );

    assert!(response["error"].is_null());
    let contents = &response["result"]["contents"];
    assert!(contents.is_array());
    assert_eq!(contents[0]["uri"], "markplane://summary");
    assert_eq!(contents[0]["mimeType"], "text/markdown");
    let text = contents[0]["text"].as_str().unwrap();
    assert!(text.contains("Test Project"));
}

#[test]
fn test_resource_active_work() {
    let tmp = setup_project();
    let response = send_request(
        &tmp,
        &json!({
            "jsonrpc": "2.0",
            "id": 201,
            "method": "resources/read",
            "params": { "uri": "markplane://active-work" }
        }),
    );

    assert!(response["error"].is_null());
    let text = response["result"]["contents"][0]["text"]
        .as_str()
        .unwrap();
    assert!(text.contains("Active Work"));
    assert!(text.contains("No items currently in progress"));
}

#[test]
fn test_resource_blocked() {
    let tmp = setup_project();
    let response = send_request(
        &tmp,
        &json!({
            "jsonrpc": "2.0",
            "id": 202,
            "method": "resources/read",
            "params": { "uri": "markplane://blocked" }
        }),
    );

    assert!(response["error"].is_null());
    let text = response["result"]["contents"][0]["text"]
        .as_str()
        .unwrap();
    assert!(text.contains("Blocked Items"));
    assert!(text.contains("No items with unresolved dependencies"));
}

#[test]
fn test_resource_task_item() {
    let tmp = setup_project();
    // Create an item first
    let root = tmp.path().join(".markplane");
    let project = markplane_core::Project::new(root);
    let task = project
        .create_task(
            "Resource test",
            "feature",
            markplane_core::Priority::Medium,
            markplane_core::Effort::Small,
            None,
            vec![],
            None,
        )
        .unwrap();

    let response = send_request(
        &tmp,
        &json!({
            "jsonrpc": "2.0",
            "id": 203,
            "method": "resources/read",
            "params": { "uri": format!("markplane://task/{}", task.id) }
        }),
    );

    assert!(response["error"].is_null());
    let text = response["result"]["contents"][0]["text"]
        .as_str()
        .unwrap();
    assert!(text.contains(&task.id));
    assert!(text.contains("Resource test"));
}

#[test]
fn test_resource_epic_item() {
    let tmp = setup_project();
    let root = tmp.path().join(".markplane");
    let project = markplane_core::Project::new(root);
    let epic = project
        .create_epic("Epic resource test", markplane_core::Priority::High, None)
        .unwrap();

    let response = send_request(
        &tmp,
        &json!({
            "jsonrpc": "2.0",
            "id": 204,
            "method": "resources/read",
            "params": { "uri": format!("markplane://epic/{}", epic.id) }
        }),
    );

    assert!(response["error"].is_null());
    let text = response["result"]["contents"][0]["text"]
        .as_str()
        .unwrap();
    assert!(text.contains(&epic.id));
}

#[test]
fn test_resource_unknown_uri() {
    let tmp = setup_project();
    let response = send_request(
        &tmp,
        &json!({
            "jsonrpc": "2.0",
            "id": 205,
            "method": "resources/read",
            "params": { "uri": "markplane://nonexistent" }
        }),
    );

    assert!(response["error"].is_object());
    assert!(response["error"]["message"]
        .as_str()
        .unwrap()
        .contains("Unknown resource URI"));
}

#[test]
fn test_resource_missing_uri() {
    let tmp = setup_project();
    let response = send_request(
        &tmp,
        &json!({
            "jsonrpc": "2.0",
            "id": 206,
            "method": "resources/read",
            "params": {}
        }),
    );

    assert!(response["error"].is_object());
}

// ── Plan and Note resource reads ─────────────────────────────────────────

#[test]
fn test_resource_plan_item() {
    let tmp = setup_project();
    let root = tmp.path().join(".markplane");
    let project = markplane_core::Project::new(root);
    let plan = project
        .create_plan(
            "Plan resource test",
            vec![],
            None,
        )
        .unwrap();

    let response = send_request(
        &tmp,
        &json!({
            "jsonrpc": "2.0",
            "id": 210,
            "method": "resources/read",
            "params": { "uri": format!("markplane://plan/{}", plan.id) }
        }),
    );

    assert!(response["error"].is_null());
    let text = response["result"]["contents"][0]["text"]
        .as_str()
        .unwrap();
    assert!(text.contains(&plan.id));
    assert!(text.contains("Plan resource test"));
}

#[test]
fn test_resource_plan_wrong_prefix() {
    let tmp = setup_project();
    let root = tmp.path().join(".markplane");
    let project = markplane_core::Project::new(root);
    let task = project
        .create_task(
            "Not a plan",
            "feature",
            markplane_core::Priority::Medium,
            markplane_core::Effort::Small,
            None,
            vec![],
            None,
        )
        .unwrap();

    let response = send_request(
        &tmp,
        &json!({
            "jsonrpc": "2.0",
            "id": 211,
            "method": "resources/read",
            "params": { "uri": format!("markplane://plan/{}", task.id) }
        }),
    );

    assert!(response["error"].is_object());
    assert!(response["error"]["message"]
        .as_str()
        .unwrap()
        .contains("Expected PLAN-"));
}

#[test]
fn test_resource_note_item() {
    let tmp = setup_project();
    let root = tmp.path().join(".markplane");
    let project = markplane_core::Project::new(root);
    let note = project
        .create_note(
            "Note resource test",
            "research",
            vec![],
            None,
        )
        .unwrap();

    let response = send_request(
        &tmp,
        &json!({
            "jsonrpc": "2.0",
            "id": 212,
            "method": "resources/read",
            "params": { "uri": format!("markplane://note/{}", note.id) }
        }),
    );

    assert!(response["error"].is_null());
    let text = response["result"]["contents"][0]["text"]
        .as_str()
        .unwrap();
    assert!(text.contains(&note.id));
    assert!(text.contains("Note resource test"));
}

#[test]
fn test_resource_note_wrong_prefix() {
    let tmp = setup_project();
    let root = tmp.path().join(".markplane");
    let project = markplane_core::Project::new(root);
    let task = project
        .create_task(
            "Not a note",
            "feature",
            markplane_core::Priority::Medium,
            markplane_core::Effort::Small,
            None,
            vec![],
            None,
        )
        .unwrap();

    let response = send_request(
        &tmp,
        &json!({
            "jsonrpc": "2.0",
            "id": 213,
            "method": "resources/read",
            "params": { "uri": format!("markplane://note/{}", task.id) }
        }),
    );

    assert!(response["error"].is_object());
    assert!(response["error"]["message"]
        .as_str()
        .unwrap()
        .contains("Expected NOTE-"));
}

// ── Tool: markplane_update (expanded) ────────────────────────────────────

#[test]
fn test_tool_update_effort_and_type() {
    let tmp = setup_project();
    let add_response = send_request(
        &tmp,
        &json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": {
                "name": "markplane_add",
                "arguments": { "title": "Update effort test" }
            }
        }),
    );
    let task_id = extract_id_from_response(&add_response);

    let response = send_request(
        &tmp,
        &json!({
            "jsonrpc": "2.0",
            "id": 300,
            "method": "tools/call",
            "params": {
                "name": "markplane_update",
                "arguments": { "id": task_id, "effort": "large", "type": "bug" }
            }
        }),
    );

    assert!(response["error"].is_null());

    // Verify via show
    let show = send_request(
        &tmp,
        &json!({
            "jsonrpc": "2.0",
            "id": 301,
            "method": "tools/call",
            "params": {
                "name": "markplane_show",
                "arguments": { "id": task_id }
            }
        }),
    );
    let text = show["result"]["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("effort: large"));
    assert!(text.contains("type: bug"));
}

#[test]
fn test_tool_update_title() {
    let tmp = setup_project();
    let add_response = send_request(
        &tmp,
        &json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": {
                "name": "markplane_add",
                "arguments": { "title": "Old title" }
            }
        }),
    );
    let task_id = extract_id_from_response(&add_response);

    let response = send_request(
        &tmp,
        &json!({
            "jsonrpc": "2.0",
            "id": 310,
            "method": "tools/call",
            "params": {
                "name": "markplane_update",
                "arguments": { "id": task_id, "title": "New title" }
            }
        }),
    );
    assert!(response["error"].is_null());

    let show = send_request(
        &tmp,
        &json!({
            "jsonrpc": "2.0",
            "id": 311,
            "method": "tools/call",
            "params": {
                "name": "markplane_show",
                "arguments": { "id": task_id }
            }
        }),
    );
    let text = show["result"]["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("title: New title"));
}

#[test]
fn test_tool_update_tags() {
    let tmp = setup_project();
    let add_response = send_request(
        &tmp,
        &json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": {
                "name": "markplane_add",
                "arguments": { "title": "Tag test", "tags": ["old"] }
            }
        }),
    );
    let task_id = extract_id_from_response(&add_response);

    let response = send_request(
        &tmp,
        &json!({
            "jsonrpc": "2.0",
            "id": 320,
            "method": "tools/call",
            "params": {
                "name": "markplane_update",
                "arguments": { "id": task_id, "add_tags": ["new", "ui"], "remove_tags": ["old"] }
            }
        }),
    );
    assert!(response["error"].is_null());

    let show = send_request(
        &tmp,
        &json!({
            "jsonrpc": "2.0",
            "id": 321,
            "method": "tools/call",
            "params": {
                "name": "markplane_show",
                "arguments": { "id": task_id }
            }
        }),
    );
    let text = show["result"]["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("new"));
    assert!(text.contains("ui"));
    assert!(!text.contains("- old"));
}

#[test]
fn test_tool_update_clear_assignee() {
    let tmp = setup_project();
    let add_response = send_request(
        &tmp,
        &json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": {
                "name": "markplane_add",
                "arguments": { "title": "Clear assignee test" }
            }
        }),
    );
    let task_id = extract_id_from_response(&add_response);

    // Set assignee
    send_request(
        &tmp,
        &json!({
            "jsonrpc": "2.0",
            "id": 330,
            "method": "tools/call",
            "params": {
                "name": "markplane_update",
                "arguments": { "id": task_id, "assignee": "daniel" }
            }
        }),
    );

    // Clear assignee (null)
    let response = send_request(
        &tmp,
        &json!({
            "jsonrpc": "2.0",
            "id": 331,
            "method": "tools/call",
            "params": {
                "name": "markplane_update",
                "arguments": { "id": task_id, "assignee": null }
            }
        }),
    );
    assert!(response["error"].is_null());

    let show = send_request(
        &tmp,
        &json!({
            "jsonrpc": "2.0",
            "id": 332,
            "method": "tools/call",
            "params": {
                "name": "markplane_show",
                "arguments": { "id": task_id }
            }
        }),
    );
    let text = show["result"]["content"][0]["text"].as_str().unwrap();
    assert!(!text.contains("assignee: daniel"));
}

#[test]
fn test_tool_update_epic_priority_and_tags() {
    let tmp = setup_project();
    let root = tmp.path().join(".markplane");
    let project = markplane_core::Project::new(root);
    let epic = project
        .create_epic("Epic update test", markplane_core::Priority::Medium, None)
        .unwrap();

    let response = send_request(
        &tmp,
        &json!({
            "jsonrpc": "2.0",
            "id": 350,
            "method": "tools/call",
            "params": {
                "name": "markplane_update",
                "arguments": {
                    "id": epic.id,
                    "priority": "high",
                    "add_tags": ["core", "v2"],
                    "started": "2026-02-20"
                }
            }
        }),
    );
    assert!(response["error"].is_null());

    let show = send_request(
        &tmp,
        &json!({
            "jsonrpc": "2.0",
            "id": 351,
            "method": "tools/call",
            "params": {
                "name": "markplane_show",
                "arguments": { "id": epic.id }
            }
        }),
    );
    let text = show["result"]["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("priority: high"));
    assert!(text.contains("core"));
    assert!(text.contains("v2"));
    assert!(text.contains("started: 2026-02-20"));
}

#[test]
fn test_tool_update_note_type_and_tags() {
    let tmp = setup_project();
    let root = tmp.path().join(".markplane");
    let project = markplane_core::Project::new(root);
    let note = project
        .create_note("Note update test", "idea", vec!["wip".to_string()], None)
        .unwrap();

    let response = send_request(
        &tmp,
        &json!({
            "jsonrpc": "2.0",
            "id": 360,
            "method": "tools/call",
            "params": {
                "name": "markplane_update",
                "arguments": {
                    "id": note.id,
                    "note_type": "decision",
                    "add_tags": ["arch"],
                    "remove_tags": ["wip"]
                }
            }
        }),
    );
    assert!(response["error"].is_null());

    let show = send_request(
        &tmp,
        &json!({
            "jsonrpc": "2.0",
            "id": 361,
            "method": "tools/call",
            "params": {
                "name": "markplane_show",
                "arguments": { "id": note.id }
            }
        }),
    );
    let text = show["result"]["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("type: decision"));
    assert!(text.contains("arch"));
    assert!(!text.contains("- wip"));
}

#[test]
fn test_tool_update_clear_position_via_null() {
    let tmp = setup_project();
    let add_response = send_request(
        &tmp,
        &json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": {
                "name": "markplane_add",
                "arguments": { "title": "Position clear test" }
            }
        }),
    );
    let task_id = extract_id_from_response(&add_response);

    // Set position
    send_request(
        &tmp,
        &json!({
            "jsonrpc": "2.0",
            "id": 370,
            "method": "tools/call",
            "params": {
                "name": "markplane_update",
                "arguments": { "id": task_id, "position": "aaa" }
            }
        }),
    );

    // Clear via null
    let response = send_request(
        &tmp,
        &json!({
            "jsonrpc": "2.0",
            "id": 371,
            "method": "tools/call",
            "params": {
                "name": "markplane_update",
                "arguments": { "id": task_id, "position": null }
            }
        }),
    );
    assert!(response["error"].is_null());

    let show = send_request(
        &tmp,
        &json!({
            "jsonrpc": "2.0",
            "id": 372,
            "method": "tools/call",
            "params": {
                "name": "markplane_show",
                "arguments": { "id": task_id }
            }
        }),
    );
    let text = show["result"]["content"][0]["text"].as_str().unwrap();
    assert!(!text.contains("position: aaa"));
}

// ── Move tool ─────────────────────────────────────────────────────────

/// Helper: create a task with a given priority and position via core API.
fn create_positioned_task(
    tmp: &TempDir,
    title: &str,
    priority: markplane_core::Priority,
    position: &str,
) -> String {
    let root = tmp.path().join(".markplane");
    let project = markplane_core::Project::new(root);
    let task = project
        .create_task(
            title,
            "feature",
            priority,
            markplane_core::Effort::Medium,
            None,
            vec![],
            None,
        )
        .unwrap();
    project
        .update_task(
            &task.id,
            &markplane_core::TaskUpdate {
                position: markplane_core::Patch::Set(position.to_string()),
                ..Default::default()
            },
        )
        .unwrap();
    task.id
}

/// Helper: read a task's position from its frontmatter via MCP show.
fn read_position(tmp: &TempDir, task_id: &str) -> Option<String> {
    let show = send_request(
        tmp,
        &json!({
            "jsonrpc": "2.0",
            "id": 9000,
            "method": "tools/call",
            "params": { "name": "markplane_show", "arguments": { "id": task_id } }
        }),
    );
    let text = show["result"]["content"][0]["text"].as_str().unwrap();
    // Parse the YAML frontmatter for position
    for line in text.lines() {
        let trimmed = line.trim();
        if let Some(val) = trimmed.strip_prefix("position: ") {
            if val == "null" {
                return None;
            }
            return Some(val.to_string());
        }
    }
    None
}

#[test]
fn test_tool_move_to_top() {
    let tmp = setup_project();
    let t1 = create_positioned_task(&tmp, "First", markplane_core::Priority::High, "a2");
    let _t2 = create_positioned_task(&tmp, "Second", markplane_core::Priority::High, "a5");
    let t3 = create_positioned_task(&tmp, "Third", markplane_core::Priority::High, "a8");

    // Move t3 to top
    let response = send_request(
        &tmp,
        &json!({
            "jsonrpc": "2.0",
            "id": 400,
            "method": "tools/call",
            "params": {
                "name": "markplane_move",
                "arguments": { "id": t3, "to": "top" }
            }
        }),
    );
    assert!(response["error"].is_null(), "move failed: {:?}", response["error"]);

    let pos3 = read_position(&tmp, &t3).expect("t3 should have position");
    let pos1 = read_position(&tmp, &t1).expect("t1 should have position");
    assert!(pos3 < pos1, "t3 ({}) should sort before t1 ({})", pos3, pos1);
}

#[test]
fn test_tool_move_to_bottom() {
    let tmp = setup_project();
    let t1 = create_positioned_task(&tmp, "First", markplane_core::Priority::High, "a2");
    let _t2 = create_positioned_task(&tmp, "Second", markplane_core::Priority::High, "a5");
    let t3 = create_positioned_task(&tmp, "Third", markplane_core::Priority::High, "a8");

    // Move t1 to bottom
    let response = send_request(
        &tmp,
        &json!({
            "jsonrpc": "2.0",
            "id": 401,
            "method": "tools/call",
            "params": {
                "name": "markplane_move",
                "arguments": { "id": t1, "to": "bottom" }
            }
        }),
    );
    assert!(response["error"].is_null(), "move failed: {:?}", response["error"]);

    let pos1 = read_position(&tmp, &t1).expect("t1 should have position");
    let pos3 = read_position(&tmp, &t3).expect("t3 should have position");
    assert!(pos1 > pos3, "t1 ({}) should sort after t3 ({})", pos1, pos3);
}

#[test]
fn test_tool_move_before() {
    let tmp = setup_project();
    let t1 = create_positioned_task(&tmp, "First", markplane_core::Priority::High, "a2");
    let t2 = create_positioned_task(&tmp, "Second", markplane_core::Priority::High, "a5");
    let t3 = create_positioned_task(&tmp, "Third", markplane_core::Priority::High, "a8");

    // Move t3 before t2
    let response = send_request(
        &tmp,
        &json!({
            "jsonrpc": "2.0",
            "id": 402,
            "method": "tools/call",
            "params": {
                "name": "markplane_move",
                "arguments": { "id": t3, "before": t2 }
            }
        }),
    );
    assert!(response["error"].is_null(), "move failed: {:?}", response["error"]);

    let pos1 = read_position(&tmp, &t1).expect("t1 should have position");
    let pos3 = read_position(&tmp, &t3).expect("t3 should have position");
    let pos2 = read_position(&tmp, &t2).expect("t2 should have position");
    assert!(pos1 < pos3, "t1 ({}) < t3 ({})", pos1, pos3);
    assert!(pos3 < pos2, "t3 ({}) < t2 ({})", pos3, pos2);
}

#[test]
fn test_tool_move_after() {
    let tmp = setup_project();
    let t1 = create_positioned_task(&tmp, "First", markplane_core::Priority::High, "a2");
    let t2 = create_positioned_task(&tmp, "Second", markplane_core::Priority::High, "a5");
    let t3 = create_positioned_task(&tmp, "Third", markplane_core::Priority::High, "a8");

    // Move t1 after t2
    let response = send_request(
        &tmp,
        &json!({
            "jsonrpc": "2.0",
            "id": 403,
            "method": "tools/call",
            "params": {
                "name": "markplane_move",
                "arguments": { "id": t1, "after": t2 }
            }
        }),
    );
    assert!(response["error"].is_null(), "move failed: {:?}", response["error"]);

    let pos2 = read_position(&tmp, &t2).expect("t2 should have position");
    let pos1 = read_position(&tmp, &t1).expect("t1 should have position");
    let pos3 = read_position(&tmp, &t3).expect("t3 should have position");
    assert!(pos2 < pos1, "t2 ({}) < t1 ({})", pos2, pos1);
    assert!(pos1 < pos3, "t1 ({}) < t3 ({})", pos1, pos3);
}

#[test]
fn test_tool_move_different_priority_error() {
    let tmp = setup_project();
    let t1 = create_positioned_task(&tmp, "High task", markplane_core::Priority::High, "a0");
    let t2 = create_positioned_task(&tmp, "Low task", markplane_core::Priority::Low, "a0");

    let response = send_request(
        &tmp,
        &json!({
            "jsonrpc": "2.0",
            "id": 404,
            "method": "tools/call",
            "params": {
                "name": "markplane_move",
                "arguments": { "id": t1, "before": t2 }
            }
        }),
    );
    assert!(response["error"].is_object(), "should error for different priorities");
}

#[test]
fn test_tool_move_self_reference_error() {
    let tmp = setup_project();
    let t1 = create_positioned_task(&tmp, "Task", markplane_core::Priority::High, "a0");

    let response = send_request(
        &tmp,
        &json!({
            "jsonrpc": "2.0",
            "id": 405,
            "method": "tools/call",
            "params": {
                "name": "markplane_move",
                "arguments": { "id": t1, "before": t1 }
            }
        }),
    );
    assert!(response["error"].is_object(), "should error for self-reference");
}

#[test]
fn test_tool_move_invalid_directive() {
    let tmp = setup_project();
    let add_response = send_request(
        &tmp,
        &json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": { "name": "markplane_add", "arguments": { "title": "Test" } }
        }),
    );
    let task_id = extract_id_from_response(&add_response);

    // No positioning directive
    let response = send_request(
        &tmp,
        &json!({
            "jsonrpc": "2.0",
            "id": 406,
            "method": "tools/call",
            "params": {
                "name": "markplane_move",
                "arguments": { "id": task_id }
            }
        }),
    );
    assert!(response["error"].is_object(), "should error with no directive");

    // Invalid 'to' value
    let response = send_request(
        &tmp,
        &json!({
            "jsonrpc": "2.0",
            "id": 407,
            "method": "tools/call",
            "params": {
                "name": "markplane_move",
                "arguments": { "id": task_id, "to": "middle" }
            }
        }),
    );
    assert!(response["error"].is_object(), "should error with invalid 'to'");
}

#[test]
fn test_tool_move_nonexistent_target() {
    let tmp = setup_project();
    let t1 = create_positioned_task(&tmp, "Task", markplane_core::Priority::High, "a0");

    let response = send_request(
        &tmp,
        &json!({
            "jsonrpc": "2.0",
            "id": 408,
            "method": "tools/call",
            "params": {
                "name": "markplane_move",
                "arguments": { "id": t1, "after": "TASK-nope0" }
            }
        }),
    );
    assert!(response["error"].is_object(), "should error for nonexistent target");
}

#[test]
fn test_tool_move_non_task_error() {
    let tmp = setup_project();
    let root = tmp.path().join(".markplane");
    let project = markplane_core::Project::new(root);
    let epic = project
        .create_epic("Epic", markplane_core::Priority::High, None)
        .unwrap();

    let response = send_request(
        &tmp,
        &json!({
            "jsonrpc": "2.0",
            "id": 409,
            "method": "tools/call",
            "params": {
                "name": "markplane_move",
                "arguments": { "id": epic.id, "to": "top" }
            }
        }),
    );
    assert!(response["error"].is_object(), "should error for non-task");
}

#[test]
fn test_tool_update_rejects_invalid_field_for_type() {
    let tmp = setup_project();
    let root = tmp.path().join(".markplane");
    let project = markplane_core::Project::new(root);
    let plan = project.create_plan("Test plan", vec![], None).unwrap();

    // priority is not valid for plans
    let response = send_request(
        &tmp,
        &json!({
            "jsonrpc": "2.0",
            "id": 340,
            "method": "tools/call",
            "params": {
                "name": "markplane_update",
                "arguments": { "id": plan.id, "priority": "high" }
            }
        }),
    );
    assert!(response["error"].is_object());
}

// ── Template param ──────────────────────────────────────────────────

#[test]
fn test_tool_add_with_template() {
    let tmp = setup_project();
    let response = send_request(
        &tmp,
        &json!({
            "jsonrpc": "2.0",
            "id": 400,
            "method": "tools/call",
            "params": {
                "name": "markplane_add",
                "arguments": {
                    "title": "Bug report",
                    "type": "bug",
                    "template": "bug"
                }
            }
        }),
    );
    assert!(response["result"].is_object(), "Expected result, got: {:?}", response);

    let id = extract_id_from_response(&response);
    let content = std::fs::read_to_string(
        tmp.path().join(format!(".markplane/backlog/items/{}.md", id)),
    ).unwrap();
    assert!(content.contains("## Steps to Reproduce"));
}

#[test]
fn test_tool_plan_with_template() {
    let tmp = setup_project();
    let root = tmp.path().join(".markplane");
    let project = markplane_core::Project::new(root);

    let task = project
        .create_task(
            "Refactor test",
            "feature",
            markplane_core::Priority::Medium,
            markplane_core::Effort::Medium,
            None,
            vec![],
            None,
        )
        .unwrap();

    let response = send_request(
        &tmp,
        &json!({
            "jsonrpc": "2.0",
            "id": 401,
            "method": "tools/call",
            "params": {
                "name": "markplane_plan",
                "arguments": {
                    "task_id": task.id,
                    "template": "refactor"
                }
            }
        }),
    );
    assert!(response["result"].is_object(), "Expected result, got: {:?}", response);

    let plan_id = extract_id_from_response(&response);
    let content = std::fs::read_to_string(
        tmp.path().join(format!(".markplane/plans/items/{}.md", plan_id)),
    ).unwrap();
    assert!(content.contains("## Motivation"));
}

#[test]
fn test_resource_templates() {
    let tmp = setup_project();
    let response = send_request(
        &tmp,
        &json!({
            "jsonrpc": "2.0",
            "id": 402,
            "method": "resources/read",
            "params": {
                "uri": "markplane://templates"
            }
        }),
    );
    assert!(response["result"].is_object(), "Expected result, got: {:?}", response);

    let text = response["result"]["contents"][0]["text"].as_str().unwrap();
    assert!(text.contains("task:"));
    assert!(text.contains("epic:"));
    assert!(text.contains("plan:"));
    assert!(text.contains("note:"));
}
