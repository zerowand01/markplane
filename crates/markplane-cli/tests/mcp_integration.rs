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

/// Send multiple requests in sequence and return responses.
fn send_requests(tmp: &TempDir, requests: &[Value]) -> Vec<Value> {
    let mut input = String::new();
    for req in requests {
        input.push_str(&serde_json::to_string(req).unwrap());
        input.push('\n');
    }

    let output = mcp_cmd()
        .arg("--project")
        .arg(tmp.path().to_str().unwrap())
        .write_stdin(input)
        .output()
        .expect("failed to run markplane mcp");

    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| serde_json::from_str(line.trim()).expect("failed to parse response"))
        .collect()
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
    assert!(tool_names.contains(&"markplane_stale"));
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
    assert_eq!(result["id"], "TASK-001");
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

// ── Tool: markplane_show ─────────────────────────────────────────────────

#[test]
fn test_tool_show() {
    let tmp = setup_project();
    // Add an item first
    send_request(
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

    let response = send_request(
        &tmp,
        &json!({
            "jsonrpc": "2.0",
            "id": 30,
            "method": "tools/call",
            "params": {
                "name": "markplane_show",
                "arguments": { "id": "TASK-001" }
            }
        }),
    );

    assert!(response["error"].is_null());
    let text = response["result"]["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("TASK-001"));
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

// ── Tool: markplane_update ───────────────────────────────────────────────

#[test]
fn test_tool_update() {
    let tmp = setup_project();
    send_request(
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

    // Update status
    let response = send_request(
        &tmp,
        &json!({
            "jsonrpc": "2.0",
            "id": 50,
            "method": "tools/call",
            "params": {
                "name": "markplane_update",
                "arguments": { "id": "TASK-001", "status": "in-progress" }
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
                "arguments": { "id": "TASK-001" }
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
    send_request(
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

    let response = send_request(
        &tmp,
        &json!({
            "jsonrpc": "2.0",
            "id": 60,
            "method": "tools/call",
            "params": {
                "name": "markplane_start",
                "arguments": { "id": "TASK-001" }
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
    send_request(
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

    let response = send_request(
        &tmp,
        &json!({
            "jsonrpc": "2.0",
            "id": 70,
            "method": "tools/call",
            "params": {
                "name": "markplane_done",
                "arguments": { "id": "TASK-001" }
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
fn test_tool_context_for_item() {
    let tmp = setup_project();
    send_request(
        &tmp,
        &json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": {
                "name": "markplane_add",
                "arguments": { "title": "Context target" }
            }
        }),
    );

    let response = send_request(
        &tmp,
        &json!({
            "jsonrpc": "2.0",
            "id": 82,
            "method": "tools/call",
            "params": {
                "name": "markplane_context",
                "arguments": { "item": "TASK-001" }
            }
        }),
    );

    assert!(response["error"].is_null());
    let text = response["result"]["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("TASK-001"));
    assert!(text.contains("Context target"));
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
    assert!(text.contains("All cross-references are valid"));
}

// ── Tool: markplane_stale ────────────────────────────────────────────────

#[test]
fn test_tool_stale() {
    let tmp = setup_project();
    // Items created today won't be stale
    send_request(
        &tmp,
        &json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": {
                "name": "markplane_add",
                "arguments": { "title": "Fresh item" }
            }
        }),
    );

    let response = send_request(
        &tmp,
        &json!({
            "jsonrpc": "2.0",
            "id": 100,
            "method": "tools/call",
            "params": {
                "name": "markplane_stale",
                "arguments": { "days": 1 }
            }
        }),
    );

    assert!(response["error"].is_null());
    let text = response["result"]["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("No stale items"));
}

// ── Tool: markplane_graph ────────────────────────────────────────────────

#[test]
fn test_tool_graph() {
    let tmp = setup_project();
    // Create linked items
    send_request(
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
    send_request(
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
    send_request(
        &tmp,
        &json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "tools/call",
            "params": {
                "name": "markplane_link",
                "arguments": { "from": "TASK-001", "to": "TASK-002", "relation": "blocks" }
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
                "arguments": { "id": "TASK-001" }
            }
        }),
    );

    assert!(response["error"].is_null());
    let text = response["result"]["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("Reference Graph for TASK-001"));
    assert!(text.contains("TASK-002"));
}

// ── Tool: markplane_promote ──────────────────────────────────────────────

#[test]
fn test_tool_promote() {
    let tmp = setup_project();
    // Create a note first (directly using core)
    let root = tmp.path().join(".markplane");
    let project = markplane_core::Project::new(root);
    project
        .create_note("Good idea", markplane_core::NoteType::Idea, vec!["cool".to_string()])
        .unwrap();

    let response = send_request(
        &tmp,
        &json!({
            "jsonrpc": "2.0",
            "id": 120,
            "method": "tools/call",
            "params": {
                "name": "markplane_promote",
                "arguments": { "note_id": "NOTE-001", "priority": "high" }
            }
        }),
    );

    assert!(response["error"].is_null());
    let text = response["result"]["content"][0]["text"].as_str().unwrap();
    let result: Value = serde_json::from_str(text).unwrap();
    assert_eq!(result["id"], "TASK-001");
    assert_eq!(result["promoted_from"], "NOTE-001");
}

// ── Tool: markplane_plan ─────────────────────────────────────────────────

#[test]
fn test_tool_plan() {
    let tmp = setup_project();
    send_request(
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

    let response = send_request(
        &tmp,
        &json!({
            "jsonrpc": "2.0",
            "id": 130,
            "method": "tools/call",
            "params": {
                "name": "markplane_plan",
                "arguments": { "task_id": "TASK-001", "title": "My plan" }
            }
        }),
    );

    assert!(response["error"].is_null());
    let text = response["result"]["content"][0]["text"].as_str().unwrap();
    let result: Value = serde_json::from_str(text).unwrap();
    assert_eq!(result["id"], "PLAN-001");
    assert_eq!(result["implements"], "TASK-001");
}

// ── Tool: markplane_link ─────────────────────────────────────────────────

#[test]
fn test_tool_link_blocks() {
    let tmp = setup_project();
    let responses = send_requests(
        &tmp,
        &[
            json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "tools/call",
                "params": {
                    "name": "markplane_add",
                    "arguments": { "title": "A" }
                }
            }),
            json!({
                "jsonrpc": "2.0",
                "id": 2,
                "method": "tools/call",
                "params": {
                    "name": "markplane_add",
                    "arguments": { "title": "B" }
                }
            }),
            json!({
                "jsonrpc": "2.0",
                "id": 3,
                "method": "tools/call",
                "params": {
                    "name": "markplane_link",
                    "arguments": {
                        "from": "TASK-001",
                        "to": "TASK-002",
                        "relation": "blocks"
                    }
                }
            }),
        ],
    );

    let link_resp = &responses[2];
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
    project
        .create_task(
            "Resource test",
            markplane_core::ItemType::Feature,
            markplane_core::Priority::Medium,
            markplane_core::Effort::Small,
            None,
            vec![],
        )
        .unwrap();

    let response = send_request(
        &tmp,
        &json!({
            "jsonrpc": "2.0",
            "id": 203,
            "method": "resources/read",
            "params": { "uri": "markplane://task/TASK-001" }
        }),
    );

    assert!(response["error"].is_null());
    let text = response["result"]["contents"][0]["text"]
        .as_str()
        .unwrap();
    assert!(text.contains("TASK-001"));
    assert!(text.contains("Resource test"));
}

#[test]
fn test_resource_epic_item() {
    let tmp = setup_project();
    let root = tmp.path().join(".markplane");
    let project = markplane_core::Project::new(root);
    project
        .create_epic("Epic resource test", markplane_core::Priority::High)
        .unwrap();

    let response = send_request(
        &tmp,
        &json!({
            "jsonrpc": "2.0",
            "id": 204,
            "method": "resources/read",
            "params": { "uri": "markplane://epic/EPIC-001" }
        }),
    );

    assert!(response["error"].is_null());
    let text = response["result"]["contents"][0]["text"]
        .as_str()
        .unwrap();
    assert!(text.contains("EPIC-001"));
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
    project
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
            "params": { "uri": "markplane://plan/PLAN-001" }
        }),
    );

    assert!(response["error"].is_null());
    let text = response["result"]["contents"][0]["text"]
        .as_str()
        .unwrap();
    assert!(text.contains("PLAN-001"));
    assert!(text.contains("Plan resource test"));
}

#[test]
fn test_resource_plan_wrong_prefix() {
    let tmp = setup_project();
    let root = tmp.path().join(".markplane");
    let project = markplane_core::Project::new(root);
    project
        .create_task(
            "Not a plan",
            markplane_core::ItemType::Feature,
            markplane_core::Priority::Medium,
            markplane_core::Effort::Small,
            None,
            vec![],
        )
        .unwrap();

    let response = send_request(
        &tmp,
        &json!({
            "jsonrpc": "2.0",
            "id": 211,
            "method": "resources/read",
            "params": { "uri": "markplane://plan/TASK-001" }
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
    project
        .create_note(
            "Note resource test",
            markplane_core::NoteType::Research,
            vec![],
        )
        .unwrap();

    let response = send_request(
        &tmp,
        &json!({
            "jsonrpc": "2.0",
            "id": 212,
            "method": "resources/read",
            "params": { "uri": "markplane://note/NOTE-001" }
        }),
    );

    assert!(response["error"].is_null());
    let text = response["result"]["contents"][0]["text"]
        .as_str()
        .unwrap();
    assert!(text.contains("NOTE-001"));
    assert!(text.contains("Note resource test"));
}

#[test]
fn test_resource_note_wrong_prefix() {
    let tmp = setup_project();
    let root = tmp.path().join(".markplane");
    let project = markplane_core::Project::new(root);
    project
        .create_task(
            "Not a note",
            markplane_core::ItemType::Feature,
            markplane_core::Priority::Medium,
            markplane_core::Effort::Small,
            None,
            vec![],
        )
        .unwrap();

    let response = send_request(
        &tmp,
        &json!({
            "jsonrpc": "2.0",
            "id": 213,
            "method": "resources/read",
            "params": { "uri": "markplane://note/TASK-001" }
        }),
    );

    assert!(response["error"].is_object());
    assert!(response["error"]["message"]
        .as_str()
        .unwrap()
        .contains("Expected NOTE-"));
}
