use std::collections::{HashMap, HashSet};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use axum::extract::{Path, Query, State};
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Json};
use axum::routing::{get, post};
use axum::Router;
use futures::SinkExt;
use futures::stream::StreamExt;
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;
use tower_http::cors::CorsLayer;
#[cfg(not(feature = "embed-ui"))]
use tower_http::services::ServeDir;

use markplane_core::{
    Effort, Epic, EpicStatus, EpicUpdate, ItemType, LinkAction, LinkRelation,
    MarkplaneDocument, MarkplaneError, Note, NoteType, NoteUpdate, Patch, Plan, PlanUpdate,
    Priority, Project, QueryFilter, Task, TaskStatus, TaskUpdate, find_blocked_items, parse_id,
};

#[cfg(feature = "embed-ui")]
use rust_embed::Embed;

#[cfg(feature = "embed-ui")]
#[derive(Embed)]
#[folder = "../markplane-web/ui/out"]
struct WebAssets;

struct AppState {
    project: Project,
    ws_tx: broadcast::Sender<String>,
}

pub async fn run(port: u16, open: bool, dev: bool) -> anyhow::Result<()> {
    let project = Project::from_current_dir()?;

    // Full sync on startup to ensure derived files are up-to-date
    project.sync_all()?;

    let (ws_tx, _) = broadcast::channel::<String>(256);

    let state = Arc::new(AppState {
        project,
        ws_tx: ws_tx.clone(),
    });

    // Start file watcher for WebSocket events
    let watch_root = state.project.root().to_path_buf();
    let ws_tx_clone = ws_tx.clone();
    tokio::spawn(async move {
        if let Err(e) = run_file_watcher(watch_root, ws_tx_clone).await {
            eprintln!("File watcher error: {}", e);
        }
    });

    let api = Router::new()
        .route("/api/summary", get(get_summary))
        .route("/api/tasks", get(get_tasks).post(create_task))
        .route(
            "/api/tasks/{id}",
            get(get_task).patch(update_task).delete(delete_task),
        )
        .route("/api/epics", get(get_epics).post(create_epic))
        .route("/api/epics/{id}", get(get_epic).patch(update_epic))
        .route("/api/plans", get(get_plans).post(create_plan))
        .route("/api/plans/{id}", get(get_plan).patch(update_plan))
        .route("/api/notes", get(get_notes).post(create_note))
        .route("/api/notes/{id}", get(get_note).patch(update_note))
        .route("/api/items/{id}/archive", post(post_archive_item))
        .route("/api/items/{id}/unarchive", post(post_unarchive_item))
        .route("/api/link", post(post_link))
        .route("/api/sync", post(post_sync))
        .route("/api/search", get(get_search))
        .route("/api/graph", get(get_graph_all))
        .route("/api/graph/{id}", get(get_graph_focused))
        .route("/ws", get(ws_handler))
        .with_state(state.clone());

    let app = if dev {
        // In dev mode, just serve the API — Next.js dev server proxies to us
        api.layer(CorsLayer::permissive())
    } else {
        // Try embedded assets first (when compiled with --features embed-ui),
        // fall back to filesystem serving for development builds.
        #[cfg(feature = "embed-ui")]
        {
            api.fallback(get(serve_embedded))
                .layer(CorsLayer::permissive())
        }
        #[cfg(not(feature = "embed-ui"))]
        {
            let ui_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../markplane-web/ui/out");
            let serve_dir = ServeDir::new(&ui_dir)
                .append_index_html_on_directories(true);
            api.fallback_service(serve_dir)
                .layer(CorsLayer::permissive())
        }
    };

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let url = format!("http://localhost:{}", port);

    println!("Markplane web UI starting on {}", url);
    if dev {
        println!("Running in dev mode — API only (use Next.js dev server for UI)");
    } else {
        #[cfg(feature = "embed-ui")]
        println!("Serving embedded UI from binary");
        #[cfg(not(feature = "embed-ui"))]
        println!("Serving static UI from crates/markplane-web/ui/out/");
    }
    println!("WebSocket available at ws://localhost:{}/ws", port);

    if open {
        let _ = open::that(&url);
    }

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

// ── Embedded Static File Server ──────────────────────────────────────────

#[cfg(feature = "embed-ui")]
async fn serve_embedded(
    uri: axum::http::Uri,
) -> impl IntoResponse {
    let path = uri.path().trim_start_matches('/');

    // Try the exact path first
    if let Some(content) = WebAssets::get(path) {
        let mime = mime_guess::from_path(path).first_or_octet_stream();
        return (
            StatusCode::OK,
            [(axum::http::header::CONTENT_TYPE, mime.as_ref().to_string())],
            content.data.into_owned(),
        ).into_response();
    }

    // For directory paths, try index.html
    let index_path = if path.is_empty() {
        "index.html".to_string()
    } else {
        format!("{}/index.html", path.trim_end_matches('/'))
    };
    if let Some(content) = WebAssets::get(&index_path) {
        return (
            StatusCode::OK,
            [(axum::http::header::CONTENT_TYPE, "text/html".to_string())],
            content.data.into_owned(),
        ).into_response();
    }

    // SPA fallback: serve root index.html for client-side routing
    if let Some(content) = WebAssets::get("index.html") {
        return (
            StatusCode::OK,
            [(axum::http::header::CONTENT_TYPE, "text/html".to_string())],
            content.data.into_owned(),
        ).into_response();
    }

    (StatusCode::NOT_FOUND, "Not found").into_response()
}

// ── File Watcher ─────────────────────────────────────────────────────────

async fn run_file_watcher(
    root: PathBuf,
    tx: broadcast::Sender<String>,
) -> anyhow::Result<()> {
    use notify_debouncer_mini::{new_debouncer, DebouncedEventKind};

    let (notify_tx, mut notify_rx) = tokio::sync::mpsc::channel(100);

    let mut debouncer = new_debouncer(
        Duration::from_millis(100),
        move |events: Result<Vec<notify_debouncer_mini::DebouncedEvent>, notify::Error>| {
            if let Ok(events) = events {
                for event in events {
                    if event.kind == DebouncedEventKind::Any {
                        let _ = notify_tx.blocking_send(event.path);
                    }
                }
            }
        },
    )?;

    debouncer
        .watcher()
        .watch(&root, notify::RecursiveMode::Recursive)?;

    // Keep debouncer alive
    let _debouncer = debouncer;

    while let Some(path) = notify_rx.recv().await {
        let path_str = path.to_string_lossy().to_string();

        // Skip non-markdown files (except config.yaml)
        if !path_str.ends_with(".md") && !path_str.ends_with("config.yaml") {
            continue;
        }

        // Skip .context/ directory changes (those are generated)
        if path_str.contains(".context") {
            continue;
        }

        // Skip INDEX.md changes (those are generated by sync)
        if path_str.ends_with("INDEX.md") {
            continue;
        }

        // Determine entity type and ID from path
        let event = if path_str.ends_with("config.yaml") {
            serde_json::json!({"type": "config_changed"})
        } else if let Some(filename) = path.file_stem() {
            let id = filename.to_string_lossy();
            if let Ok((prefix, _)) = parse_id(&id) {
                let entity = match prefix {
                    markplane_core::IdPrefix::Epic => "epic",
                    markplane_core::IdPrefix::Task => "task",
                    markplane_core::IdPrefix::Plan => "plan",
                    markplane_core::IdPrefix::Note => "note",
                };
                serde_json::json!({
                    "type": "file_changed",
                    "entity": entity,
                    "id": id,
                    "action": "modified"
                })
            } else {
                continue;
            }
        } else {
            continue;
        };

        let msg = event.to_string();
        let _ = tx.send(msg);
    }

    Ok(())
}

// ── WebSocket Handler ────────────────────────────────────────────────────

async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_ws(socket, state))
}

async fn handle_ws(socket: WebSocket, state: Arc<AppState>) {
    let (mut sender, mut receiver) = socket.split();

    // Send connected message
    let connected = serde_json::json!({"type": "connected", "version": "0.1.0"});
    let _ = sender.send(Message::Text(connected.to_string().into())).await;

    let mut rx = state.ws_tx.subscribe();

    // Forward broadcast events to this WebSocket client
    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if sender.send(Message::Text(msg.into())).await.is_err() {
                break;
            }
        }
    });

    // Consume (and ignore) incoming messages from client — keeps connection alive
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if matches!(msg, Message::Close(_)) {
                break;
            }
        }
    });

    // Wait for either task to finish
    tokio::select! {
        _ = &mut send_task => { recv_task.abort(); }
        _ = &mut recv_task => { send_task.abort(); }
    }
}

// ── API Types ─────────────────────────────────────────────────────────────

#[derive(Serialize)]
struct ApiResponse<T: Serialize> {
    data: T,
}

#[derive(Serialize)]
struct ApiListResponse<T: Serialize> {
    data: Vec<T>,
    meta: ListMeta,
}

#[derive(Serialize)]
struct ListMeta {
    total: usize,
}

#[derive(Serialize)]
struct ApiError {
    error: ErrorDetail,
}

#[derive(Serialize)]
struct ErrorDetail {
    code: String,
    message: String,
}

fn error_response(status: StatusCode, code: &str, message: &str) -> (StatusCode, Json<ApiError>) {
    (
        status,
        Json(ApiError {
            error: ErrorDetail {
                code: code.to_string(),
                message: message.to_string(),
            },
        }),
    )
}

/// Map a `MarkplaneError` to an HTTP error response.
fn map_core_error(e: MarkplaneError) -> (StatusCode, Json<ApiError>) {
    match &e {
        MarkplaneError::NotFound(_) => error_response(StatusCode::NOT_FOUND, "not_found", &e.to_string()),
        MarkplaneError::InvalidId(_) => error_response(StatusCode::BAD_REQUEST, "invalid_id", &e.to_string()),
        MarkplaneError::InvalidStatus(_) | MarkplaneError::InvalidTransition { .. } => {
            error_response(StatusCode::BAD_REQUEST, "invalid_status", &e.to_string())
        }
        MarkplaneError::InvalidLink(_) => error_response(StatusCode::BAD_REQUEST, "invalid_link", &e.to_string()),
        MarkplaneError::DuplicateId(_) => error_response(StatusCode::CONFLICT, "duplicate_id", &e.to_string()),
        MarkplaneError::BrokenReference(_) => error_response(StatusCode::BAD_REQUEST, "broken_reference", &e.to_string()),
        MarkplaneError::Config(_) | MarkplaneError::Frontmatter(_) => {
            error_response(StatusCode::BAD_REQUEST, "invalid_field", &e.to_string())
        }
        MarkplaneError::NotInitialized(_) | MarkplaneError::Io(_) | MarkplaneError::Yaml(_) => {
            error_response(StatusCode::INTERNAL_SERVER_ERROR, "internal_error", &e.to_string())
        }
    }
}

/// Diff two vectors to produce add/remove lists.
/// Converts the web UI's replacement semantics (full desired array) to core's add/remove semantics.
fn diff_vec(current: &[String], desired: &[String]) -> (Vec<String>, Vec<String>) {
    let to_add = desired.iter().filter(|d| !current.contains(d)).cloned().collect();
    let to_remove = current.iter().filter(|c| !desired.contains(c)).cloned().collect();
    (to_add, to_remove)
}

// ── Task API types ────────────────────────────────────────────────────────

#[derive(Serialize)]
struct TaskResponse {
    id: String,
    title: String,
    status: String,
    priority: String,
    #[serde(rename = "type")]
    item_type: String,
    effort: String,
    tags: Vec<String>,
    epic: Option<String>,
    plan: Option<String>,
    depends_on: Vec<String>,
    blocks: Vec<String>,
    assignee: Option<String>,
    position: Option<String>,
    created: String,
    updated: String,
    body: String,
}

fn task_to_response(doc: &MarkplaneDocument<Task>) -> TaskResponse {
    let fm = &doc.frontmatter;
    TaskResponse {
        id: fm.id.clone(),
        title: fm.title.clone(),
        status: fm.status.to_string(),
        priority: fm.priority.to_string(),
        item_type: fm.item_type.to_string(),
        effort: fm.effort.to_string(),
        tags: fm.tags.clone(),
        epic: fm.epic.clone(),
        plan: fm.plan.clone(),
        depends_on: fm.depends_on.clone(),
        blocks: fm.blocks.clone(),
        assignee: fm.assignee.clone(),
        position: fm.position.clone(),
        created: fm.created.to_string(),
        updated: fm.updated.to_string(),
        body: doc.body.clone(),
    }
}

#[derive(Deserialize)]
struct CreateTaskRequest {
    title: String,
    #[serde(rename = "type", default = "default_item_type")]
    item_type: String,
    #[serde(default = "default_priority")]
    priority: String,
    #[serde(default = "default_effort")]
    effort: String,
    #[serde(default)]
    epic: Option<String>,
    #[serde(default)]
    tags: Vec<String>,
}

fn default_item_type() -> String {
    "feature".to_string()
}
fn default_priority() -> String {
    "medium".to_string()
}
fn default_effort() -> String {
    "medium".to_string()
}

#[derive(Deserialize)]
struct CreateEpicRequest {
    title: String,
    #[serde(default = "default_priority")]
    priority: String,
}

#[derive(Deserialize)]
struct CreatePlanRequest {
    title: String,
    #[serde(default)]
    task_id: Option<String>,
}

#[derive(Deserialize)]
struct CreateNoteRequest {
    title: String,
    #[serde(rename = "type", default = "default_note_type")]
    note_type: String,
    #[serde(default)]
    tags: Vec<String>,
}

fn default_note_type() -> String {
    "research".to_string()
}

#[derive(Deserialize)]
struct UpdateTaskRequest {
    title: Option<String>,
    status: Option<String>,
    priority: Option<String>,
    effort: Option<String>,
    #[serde(rename = "type")]
    item_type: Option<String>,
    #[serde(default)]
    tags: Option<Vec<String>>,
    epic: Option<String>,
    plan: Option<String>,
    assignee: Option<String>,
    position: Option<String>,
    depends_on: Option<Vec<String>>,
    blocks: Option<Vec<String>>,
    body: Option<String>,
}

// ── Epic API types ────────────────────────────────────────────────────────

#[derive(Serialize)]
struct EpicResponse {
    id: String,
    title: String,
    status: String,
    priority: String,
    started: Option<String>,
    target: Option<String>,
    tags: Vec<String>,
    depends_on: Vec<String>,
    created: String,
    updated: String,
    body: String,
    task_count: usize,
    done_count: usize,
    progress: f64,
    status_breakdown: HashMap<String, usize>,
}

fn epic_to_response(
    doc: &MarkplaneDocument<Epic>,
    tasks: &[MarkplaneDocument<Task>],
) -> EpicResponse {
    let fm = &doc.frontmatter;
    let epic_tasks: Vec<_> = tasks
        .iter()
        .filter(|t| t.frontmatter.epic.as_deref() == Some(&fm.id))
        .collect();

    let task_count = epic_tasks.len();
    let done_count = epic_tasks
        .iter()
        .filter(|t| t.frontmatter.status == TaskStatus::Done)
        .count();
    let progress = if task_count > 0 {
        done_count as f64 / task_count as f64
    } else {
        0.0
    };

    let mut status_breakdown = HashMap::new();
    for t in &epic_tasks {
        *status_breakdown
            .entry(t.frontmatter.status.to_string())
            .or_insert(0) += 1;
    }

    EpicResponse {
        id: fm.id.clone(),
        title: fm.title.clone(),
        status: fm.status.to_string(),
        priority: fm.priority.to_string(),
        started: fm.started.map(|d| d.to_string()),
        target: fm.target.map(|d| d.to_string()),
        tags: fm.tags.clone(),
        depends_on: fm.depends_on.clone(),
        created: fm.created.to_string(),
        updated: fm.updated.to_string(),
        body: doc.body.clone(),
        task_count,
        done_count,
        progress,
        status_breakdown,
    }
}

#[derive(Deserialize)]
struct UpdateEpicRequest {
    title: Option<String>,
    status: Option<String>,
    priority: Option<String>,
    #[serde(default)]
    tags: Option<Vec<String>>,
    started: Option<String>,
    target: Option<String>,
    #[serde(default)]
    depends_on: Option<Vec<String>>,
    body: Option<String>,
}

// ── Plan API types ────────────────────────────────────────────────────────

#[derive(Serialize)]
struct PlanResponse {
    id: String,
    title: String,
    status: String,
    implements: Vec<String>,
    epic: Option<String>,
    created: String,
    updated: String,
    body: String,
}

fn plan_to_response(doc: &MarkplaneDocument<Plan>) -> PlanResponse {
    let fm = &doc.frontmatter;
    PlanResponse {
        id: fm.id.clone(),
        title: fm.title.clone(),
        status: fm.status.to_string(),
        implements: fm.implements.clone(),
        epic: fm.epic.clone(),
        created: fm.created.to_string(),
        updated: fm.updated.to_string(),
        body: doc.body.clone(),
    }
}

#[derive(Deserialize)]
struct UpdatePlanRequest {
    title: Option<String>,
    status: Option<String>,
    epic: Option<String>,
    body: Option<String>,
}

// ── Note API types ────────────────────────────────────────────────────────

#[derive(Serialize)]
struct NoteResponse {
    id: String,
    title: String,
    #[serde(rename = "type")]
    note_type: String,
    status: String,
    tags: Vec<String>,
    related: Vec<String>,
    created: String,
    updated: String,
    body: String,
}

fn note_to_response(doc: &MarkplaneDocument<Note>) -> NoteResponse {
    let fm = &doc.frontmatter;
    NoteResponse {
        id: fm.id.clone(),
        title: fm.title.clone(),
        note_type: fm.note_type.to_string(),
        status: fm.status.to_string(),
        tags: fm.tags.clone(),
        related: fm.related.clone(),
        created: fm.created.to_string(),
        updated: fm.updated.to_string(),
        body: doc.body.clone(),
    }
}

#[derive(Deserialize)]
struct UpdateNoteRequest {
    title: Option<String>,
    status: Option<String>,
    #[serde(rename = "type")]
    note_type: Option<String>,
    #[serde(default)]
    tags: Option<Vec<String>>,
    #[serde(default)]
    related: Option<Vec<String>>,
    body: Option<String>,
}

// ── Summary API types ─────────────────────────────────────────────────────

#[derive(Serialize)]
struct SummaryResponse {
    name: String,
    description: String,
    counts: SummaryCounts,
    now_epics: Vec<EpicResponse>,
    in_progress_tasks: Vec<TaskResponse>,
    blocked_tasks: Vec<TaskResponse>,
    recent_completions: Vec<TaskResponse>,
    next_up_tasks: Vec<TaskResponse>,
    context_summary: Option<String>,
    /// ISO 8601 timestamp of when .context/summary.md was last modified
    context_last_synced: Option<String>,
}

#[derive(Serialize)]
struct SummaryCounts {
    total: usize,
    in_progress: usize,
    planned: usize,
    backlog: usize,
    draft: usize,
    done: usize,
    blocked: usize,
}

// ── Handlers ──────────────────────────────────────────────────────────────

async fn get_summary(
    State(state): State<Arc<AppState>>,
) -> Result<Json<ApiResponse<SummaryResponse>>, (StatusCode, Json<ApiError>)> {
    let project = &state.project;

    let config = project
        .load_config()
        .map_err(|e| error_response(StatusCode::INTERNAL_SERVER_ERROR, "config_error", &e.to_string()))?;
    let tasks = project
        .list_tasks(&QueryFilter::default())
        .map_err(|e| error_response(StatusCode::INTERNAL_SERVER_ERROR, "query_error", &e.to_string()))?;
    let epics = project
        .list_epics()
        .map_err(|e| error_response(StatusCode::INTERNAL_SERVER_ERROR, "query_error", &e.to_string()))?;

    let blocked = find_blocked_items(&tasks);

    let in_progress: Vec<_> = tasks
        .iter()
        .filter(|t| t.frontmatter.status == TaskStatus::InProgress)
        .collect();
    let planned_count = tasks
        .iter()
        .filter(|t| t.frontmatter.status == TaskStatus::Planned)
        .count();
    let backlog_count = tasks
        .iter()
        .filter(|t| t.frontmatter.status == TaskStatus::Backlog)
        .count();
    let draft_count = tasks
        .iter()
        .filter(|t| t.frontmatter.status == TaskStatus::Draft)
        .count();
    let done_count = tasks
        .iter()
        .filter(|t| t.frontmatter.status == TaskStatus::Done)
        .count();

    let now_epics: Vec<_> = epics
        .iter()
        .filter(|e| e.frontmatter.status == EpicStatus::Now)
        .map(|e| epic_to_response(e, &tasks))
        .collect();

    let in_progress_tasks: Vec<_> = in_progress.iter().map(|t| task_to_response(t)).collect();
    let blocked_tasks: Vec<_> = blocked.iter().map(|t| task_to_response(t)).collect();

    // Recent completions: done items sorted by updated date (most recent first)
    let mut recent_done: Vec<_> = tasks
        .iter()
        .filter(|t| t.frontmatter.status == TaskStatus::Done)
        .collect();
    recent_done.sort_by(|a, b| b.frontmatter.updated.cmp(&a.frontmatter.updated));
    let recent_completions: Vec<_> = recent_done
        .iter()
        .take(5)
        .map(|t| task_to_response(t))
        .collect();

    // Next up: top 5 planned/backlog tasks (already priority-sorted by list_tasks)
    let next_up_tasks: Vec<_> = tasks
        .iter()
        .filter(|t| {
            t.frontmatter.status == TaskStatus::Planned
                || t.frontmatter.status == TaskStatus::Backlog
        })
        .take(5)
        .map(task_to_response)
        .collect();

    // Read .context/summary.md if it exists
    let context_path = project.root().join(".context").join("summary.md");
    let context_summary = std::fs::read_to_string(&context_path).ok();
    let context_last_synced = std::fs::metadata(&context_path)
        .and_then(|m| m.modified())
        .ok()
        .map(|t| {
            let dt: chrono::DateTime<chrono::Utc> = t.into();
            dt.to_rfc3339()
        });

    let summary = SummaryResponse {
        name: config.project.name,
        description: config.project.description,
        counts: SummaryCounts {
            total: tasks.len(),
            in_progress: in_progress.len(),
            planned: planned_count,
            backlog: backlog_count,
            draft: draft_count,
            done: done_count,
            blocked: blocked.len(),
        },
        now_epics,
        in_progress_tasks,
        blocked_tasks,
        recent_completions,
        next_up_tasks,
        context_summary,
        context_last_synced,
    };

    Ok(Json(ApiResponse { data: summary }))
}

#[derive(Deserialize)]
struct TaskQueryParams {
    status: Option<String>,
    priority: Option<String>,
    epic: Option<String>,
    tags: Option<String>,
    assignee: Option<String>,
    #[serde(rename = "type")]
    item_type: Option<String>,
    #[serde(default)]
    archived: bool,
}

async fn get_tasks(
    State(state): State<Arc<AppState>>,
    Query(params): Query<TaskQueryParams>,
) -> Result<Json<ApiListResponse<TaskResponse>>, (StatusCode, Json<ApiError>)> {
    let filter = QueryFilter {
        status: params.status.map(|s| super::parse_comma_list(&s)),
        priority: params.priority.map(|s| super::parse_comma_list(&s)),
        epic: params.epic,
        tags: params.tags.map(|s| super::parse_comma_list(&s)),
        assignee: params.assignee,
        item_type: params.item_type.map(|s| super::parse_comma_list(&s)),
        archived: params.archived,
    };

    let tasks = state
        .project
        .list_tasks(&filter)
        .map_err(|e| error_response(StatusCode::INTERNAL_SERVER_ERROR, "query_error", &e.to_string()))?;

    let total = tasks.len();
    let data: Vec<_> = tasks.iter().map(task_to_response).collect();

    Ok(Json(ApiListResponse {
        data,
        meta: ListMeta { total },
    }))
}

async fn get_task(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<TaskResponse>>, (StatusCode, Json<ApiError>)> {
    // Validate ID format
    parse_id(&id)
        .map_err(|_| error_response(StatusCode::BAD_REQUEST, "invalid_id", &format!("Invalid ID format: {}", id)))?;

    let doc: MarkplaneDocument<Task> = state
        .project
        .read_item(&id)
        .map_err(|e| error_response(StatusCode::NOT_FOUND, "not_found", &e.to_string()))?;

    Ok(Json(ApiResponse {
        data: task_to_response(&doc),
    }))
}

async fn create_task(
    State(state): State<Arc<AppState>>,
    Json(body): Json<CreateTaskRequest>,
) -> Result<(StatusCode, Json<ApiResponse<TaskResponse>>), (StatusCode, Json<ApiError>)> {
    let item_type: ItemType = body
        .item_type
        .parse()
        .map_err(|e: markplane_core::MarkplaneError| {
            error_response(StatusCode::BAD_REQUEST, "invalid_type", &e.to_string())
        })?;
    let priority: Priority = body
        .priority
        .parse()
        .map_err(|e: markplane_core::MarkplaneError| {
            error_response(StatusCode::BAD_REQUEST, "invalid_priority", &e.to_string())
        })?;
    let effort: Effort = body
        .effort
        .parse()
        .map_err(|e: markplane_core::MarkplaneError| {
            error_response(StatusCode::BAD_REQUEST, "invalid_effort", &e.to_string())
        })?;

    let task = state
        .project
        .create_task(&body.title, item_type, priority, effort, body.epic, body.tags, None)
        .map_err(|e| {
            error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "create_error",
                &e.to_string(),
            )
        })?;

    // Read the created task back to get the full document with body
    let doc: MarkplaneDocument<Task> = state
        .project
        .read_item(&task.id)
        .map_err(|e| {
            error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "read_error",
                &e.to_string(),
            )
        })?;

    Ok((
        StatusCode::CREATED,
        Json(ApiResponse {
            data: task_to_response(&doc),
        }),
    ))
}

async fn create_epic(
    State(state): State<Arc<AppState>>,
    Json(body): Json<CreateEpicRequest>,
) -> Result<(StatusCode, Json<ApiResponse<EpicResponse>>), (StatusCode, Json<ApiError>)> {
    let priority: Priority = body
        .priority
        .parse()
        .map_err(|e: MarkplaneError| {
            error_response(StatusCode::BAD_REQUEST, "invalid_priority", &e.to_string())
        })?;

    let epic = state
        .project
        .create_epic(&body.title, priority, None)
        .map_err(|e| {
            error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "create_error",
                &e.to_string(),
            )
        })?;

    let doc: MarkplaneDocument<Epic> = state
        .project
        .read_item(&epic.id)
        .map_err(|e| {
            error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "read_error",
                &e.to_string(),
            )
        })?;

    // New epic has no tasks yet
    let tasks: Vec<MarkplaneDocument<Task>> = vec![];
    Ok((
        StatusCode::CREATED,
        Json(ApiResponse {
            data: epic_to_response(&doc, &tasks),
        }),
    ))
}

async fn create_plan(
    State(state): State<Arc<AppState>>,
    Json(body): Json<CreatePlanRequest>,
) -> Result<(StatusCode, Json<ApiResponse<PlanResponse>>), (StatusCode, Json<ApiError>)> {
    let implements = match &body.task_id {
        Some(id) => vec![id.clone()],
        None => vec![],
    };

    let plan = state
        .project
        .create_plan(&body.title, implements, None, None)
        .map_err(|e| {
            error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "create_error",
                &e.to_string(),
            )
        })?;

    // If task_id provided, link the plan back to the task
    if let Some(ref task_id) = body.task_id
        && let Err(e) = state.project.link_items(
            task_id,
            &plan.id,
            LinkRelation::Plan,
            LinkAction::Add,
        )
    {
        eprintln!("Warning: failed to link plan {} to task {}: {}", plan.id, task_id, e);
    }

    let doc: MarkplaneDocument<Plan> = state
        .project
        .read_item(&plan.id)
        .map_err(|e| {
            error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "read_error",
                &e.to_string(),
            )
        })?;

    Ok((
        StatusCode::CREATED,
        Json(ApiResponse {
            data: plan_to_response(&doc),
        }),
    ))
}

async fn create_note(
    State(state): State<Arc<AppState>>,
    Json(body): Json<CreateNoteRequest>,
) -> Result<(StatusCode, Json<ApiResponse<NoteResponse>>), (StatusCode, Json<ApiError>)> {
    let note_type: NoteType = body
        .note_type
        .parse()
        .map_err(|e: MarkplaneError| {
            error_response(StatusCode::BAD_REQUEST, "invalid_type", &e.to_string())
        })?;

    let note = state
        .project
        .create_note(&body.title, note_type, body.tags, None)
        .map_err(|e| {
            error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "create_error",
                &e.to_string(),
            )
        })?;

    let doc: MarkplaneDocument<Note> = state
        .project
        .read_item(&note.id)
        .map_err(|e| {
            error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "read_error",
                &e.to_string(),
            )
        })?;

    Ok((
        StatusCode::CREATED,
        Json(ApiResponse {
            data: note_to_response(&doc),
        }),
    ))
}

async fn update_task(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(body): Json<UpdateTaskRequest>,
) -> Result<Json<ApiResponse<TaskResponse>>, (StatusCode, Json<ApiError>)> {
    parse_id(&id)
        .map_err(|_| error_response(StatusCode::BAD_REQUEST, "invalid_id", &format!("Invalid ID format: {}", id)))?;

    // Read current state for diffing tags and links
    let current: MarkplaneDocument<Task> = state.project.read_item(&id).map_err(map_core_error)?;
    let fm = &current.frontmatter;

    // ── Properties → TaskUpdate ──────────────────────────────────────
    let (add_tags, remove_tags) = if let Some(ref desired_tags) = body.tags {
        diff_vec(&fm.tags, desired_tags)
    } else {
        (vec![], vec![])
    };

    let assignee = match &body.assignee {
        None => Patch::Unchanged,
        Some(s) if s.is_empty() => Patch::Clear,
        Some(s) => Patch::Set(s.clone()),
    };
    let position = match &body.position {
        None => Patch::Unchanged,
        Some(s) if s.is_empty() => Patch::Clear,
        Some(s) => Patch::Set(s.clone()),
    };

    let has_changes = body.title.is_some() || body.status.is_some()
        || body.priority.is_some() || body.effort.is_some() || body.item_type.is_some()
        || !add_tags.is_empty() || !remove_tags.is_empty()
        || !matches!(assignee, Patch::Unchanged) || !matches!(position, Patch::Unchanged)
        || body.body.is_some();
    if has_changes {
        state.project.update_task(&id, &TaskUpdate {
            title: body.title,
            status: body.status,
            priority: body.priority,
            effort: body.effort,
            item_type: body.item_type,
            assignee,
            position,
            add_tags,
            remove_tags,
            body: body.body,
        }).map_err(map_core_error)?;
    }

    // ── Links → link_items() per change ──────────────────────────────

    // Epic (scalar): Add overwrites, Remove clears
    if let Some(ref desired_epic) = body.epic {
        let desired = if desired_epic.is_empty() { None } else { Some(desired_epic.as_str()) };
        let current_epic = fm.epic.as_deref();
        if desired != current_epic {
            match desired {
                Some(new) => {
                    // Add overwrites the epic field directly
                    state.project.link_items(&id, new, LinkRelation::Epic, LinkAction::Add)
                        .map_err(map_core_error)?;
                }
                None => {
                    // Clear: remove the current epic link
                    if let Some(old) = current_epic {
                        state.project.link_items(&id, old, LinkRelation::Epic, LinkAction::Remove)
                            .map_err(map_core_error)?;
                    }
                }
            }
        }
    }

    // Plan (scalar): Add handles old plan cleanup internally
    if let Some(ref desired_plan) = body.plan {
        let desired = if desired_plan.is_empty() { None } else { Some(desired_plan.as_str()) };
        let current_plan = fm.plan.as_deref();
        if desired != current_plan {
            match desired {
                Some(new) => {
                    // Add cleans up old plan's implements list and sets new plan
                    state.project.link_items(&id, new, LinkRelation::Plan, LinkAction::Add)
                        .map_err(map_core_error)?;
                }
                None => {
                    // Clear: remove the current plan link
                    if let Some(old) = current_plan {
                        state.project.link_items(&id, old, LinkRelation::Plan, LinkAction::Remove)
                            .map_err(map_core_error)?;
                    }
                }
            }
        }
    }

    // depends_on (array): diff
    if let Some(ref desired_deps) = body.depends_on {
        let (to_add, to_remove) = diff_vec(&fm.depends_on, desired_deps);
        // Remove first to avoid transient invalid states
        for dep in &to_remove {
            state.project.link_items(&id, dep, LinkRelation::DependsOn, LinkAction::Remove)
                .map_err(map_core_error)?;
        }
        for dep in &to_add {
            state.project.link_items(&id, dep, LinkRelation::DependsOn, LinkAction::Add)
                .map_err(map_core_error)?;
        }
    }

    // blocks (array): diff
    if let Some(ref desired_blocks) = body.blocks {
        let (to_add, to_remove) = diff_vec(&fm.blocks, desired_blocks);
        for blk in &to_remove {
            state.project.link_items(&id, blk, LinkRelation::Blocks, LinkAction::Remove)
                .map_err(map_core_error)?;
        }
        for blk in &to_add {
            state.project.link_items(&id, blk, LinkRelation::Blocks, LinkAction::Add)
                .map_err(map_core_error)?;
        }
    }

    // Re-read for response
    let doc: MarkplaneDocument<Task> = state.project.read_item(&id).map_err(map_core_error)?;
    Ok(Json(ApiResponse {
        data: task_to_response(&doc),
    }))
}

async fn delete_task(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<TaskResponse>>, (StatusCode, Json<ApiError>)> {
    parse_id(&id)
        .map_err(|_| error_response(StatusCode::BAD_REQUEST, "invalid_id", &format!("Invalid ID format: {}", id)))?;

    let doc: MarkplaneDocument<Task> = state
        .project
        .read_item(&id)
        .map_err(|e| error_response(StatusCode::NOT_FOUND, "not_found", &e.to_string()))?;

    let response = task_to_response(&doc);

    state
        .project
        .archive_item(&id)
        .map_err(|e| {
            error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "archive_error",
                &e.to_string(),
            )
        })?;

    Ok(Json(ApiResponse { data: response }))
}

// ── Archive / Unarchive ──────────────────────────────────────────────────

#[derive(Serialize)]
struct ArchiveResponse {
    success: bool,
    id: String,
}

async fn post_archive_item(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<ArchiveResponse>>, (StatusCode, Json<ApiError>)> {
    parse_id(&id)
        .map_err(|_| error_response(StatusCode::BAD_REQUEST, "invalid_id", &format!("Invalid ID format: {}", id)))?;

    state
        .project
        .archive_item(&id)
        .map_err(|e| error_response(StatusCode::INTERNAL_SERVER_ERROR, "archive_error", &e.to_string()))?;

    Ok(Json(ApiResponse {
        data: ArchiveResponse {
            success: true,
            id,
        },
    }))
}

async fn post_unarchive_item(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<ArchiveResponse>>, (StatusCode, Json<ApiError>)> {
    parse_id(&id)
        .map_err(|_| error_response(StatusCode::BAD_REQUEST, "invalid_id", &format!("Invalid ID format: {}", id)))?;

    state
        .project
        .unarchive_item(&id)
        .map_err(|e| error_response(StatusCode::INTERNAL_SERVER_ERROR, "unarchive_error", &e.to_string()))?;

    Ok(Json(ApiResponse {
        data: ArchiveResponse {
            success: true,
            id,
        },
    }))
}

#[derive(Deserialize)]
struct ArchivedQueryParam {
    #[serde(default)]
    archived: bool,
}

async fn get_epics(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ArchivedQueryParam>,
) -> Result<Json<ApiListResponse<EpicResponse>>, (StatusCode, Json<ApiError>)> {
    let epics = state
        .project
        .list_epics_filtered(params.archived)
        .map_err(|e| error_response(StatusCode::INTERNAL_SERVER_ERROR, "query_error", &e.to_string()))?;
    let tasks = state
        .project
        .list_tasks(&QueryFilter::default())
        .map_err(|e| error_response(StatusCode::INTERNAL_SERVER_ERROR, "query_error", &e.to_string()))?;

    let total = epics.len();
    let data: Vec<_> = epics.iter().map(|e| epic_to_response(e, &tasks)).collect();

    Ok(Json(ApiListResponse {
        data,
        meta: ListMeta { total },
    }))
}

async fn get_epic(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<EpicResponse>>, (StatusCode, Json<ApiError>)> {
    parse_id(&id)
        .map_err(|_| error_response(StatusCode::BAD_REQUEST, "invalid_id", &format!("Invalid ID format: {}", id)))?;

    let doc: MarkplaneDocument<Epic> = state
        .project
        .read_item(&id)
        .map_err(|e| error_response(StatusCode::NOT_FOUND, "not_found", &e.to_string()))?;

    let tasks = state
        .project
        .list_tasks(&QueryFilter::default())
        .map_err(|e| error_response(StatusCode::INTERNAL_SERVER_ERROR, "query_error", &e.to_string()))?;

    Ok(Json(ApiResponse {
        data: epic_to_response(&doc, &tasks),
    }))
}

async fn get_plans(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ArchivedQueryParam>,
) -> Result<Json<ApiListResponse<PlanResponse>>, (StatusCode, Json<ApiError>)> {
    let plans = state
        .project
        .list_plans_filtered(params.archived)
        .map_err(|e| error_response(StatusCode::INTERNAL_SERVER_ERROR, "query_error", &e.to_string()))?;

    let total = plans.len();
    let data: Vec<_> = plans.iter().map(plan_to_response).collect();

    Ok(Json(ApiListResponse {
        data,
        meta: ListMeta { total },
    }))
}

async fn get_plan(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<PlanResponse>>, (StatusCode, Json<ApiError>)> {
    parse_id(&id)
        .map_err(|_| error_response(StatusCode::BAD_REQUEST, "invalid_id", &format!("Invalid ID format: {}", id)))?;

    let doc: MarkplaneDocument<Plan> = state
        .project
        .read_item(&id)
        .map_err(|e| error_response(StatusCode::NOT_FOUND, "not_found", &e.to_string()))?;

    Ok(Json(ApiResponse {
        data: plan_to_response(&doc),
    }))
}

async fn get_notes(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ArchivedQueryParam>,
) -> Result<Json<ApiListResponse<NoteResponse>>, (StatusCode, Json<ApiError>)> {
    let notes = state
        .project
        .list_notes_filtered(params.archived)
        .map_err(|e| error_response(StatusCode::INTERNAL_SERVER_ERROR, "query_error", &e.to_string()))?;

    let total = notes.len();
    let data: Vec<_> = notes.iter().map(note_to_response).collect();

    Ok(Json(ApiListResponse {
        data,
        meta: ListMeta { total },
    }))
}

async fn get_note(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<NoteResponse>>, (StatusCode, Json<ApiError>)> {
    parse_id(&id)
        .map_err(|_| error_response(StatusCode::BAD_REQUEST, "invalid_id", &format!("Invalid ID format: {}", id)))?;

    let doc: MarkplaneDocument<Note> = state
        .project
        .read_item(&id)
        .map_err(|e| error_response(StatusCode::NOT_FOUND, "not_found", &e.to_string()))?;

    Ok(Json(ApiResponse {
        data: note_to_response(&doc),
    }))
}

async fn update_epic(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(body): Json<UpdateEpicRequest>,
) -> Result<Json<ApiResponse<EpicResponse>>, (StatusCode, Json<ApiError>)> {
    parse_id(&id)
        .map_err(|_| error_response(StatusCode::BAD_REQUEST, "invalid_id", &format!("Invalid ID format: {}", id)))?;

    // Read current state for diffing tags and links
    let current: MarkplaneDocument<Epic> = state.project.read_item(&id).map_err(map_core_error)?;
    let fm = &current.frontmatter;

    // Diff tags if provided
    let (add_tags, remove_tags) = if let Some(ref desired_tags) = body.tags {
        diff_vec(&fm.tags, desired_tags)
    } else {
        (vec![], vec![])
    };

    // Parse date fields to Patch<NaiveDate>
    let started = match body.started {
        None => Patch::Unchanged,
        Some(ref s) if s.is_empty() => Patch::Clear,
        Some(ref s) => Patch::Set(
            chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d")
                .map_err(|_| error_response(StatusCode::BAD_REQUEST, "invalid_date", "Invalid date format, expected YYYY-MM-DD"))?,
        ),
    };
    let target = match body.target {
        None => Patch::Unchanged,
        Some(ref s) if s.is_empty() => Patch::Clear,
        Some(ref s) => Patch::Set(
            chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d")
                .map_err(|_| error_response(StatusCode::BAD_REQUEST, "invalid_date", "Invalid date format, expected YYYY-MM-DD"))?,
        ),
    };

    // Properties + body
    let has_changes = body.title.is_some() || body.status.is_some() || body.priority.is_some()
        || !add_tags.is_empty() || !remove_tags.is_empty()
        || !matches!(started, Patch::Unchanged) || !matches!(target, Patch::Unchanged)
        || body.body.is_some();
    if has_changes {
        state.project.update_epic(&id, &EpicUpdate {
            title: body.title,
            status: body.status,
            priority: body.priority,
            add_tags,
            remove_tags,
            started,
            target,
            body: body.body,
        }).map_err(map_core_error)?;
    }

    // ── Links → link_items() per change ──────────────────────────────

    // depends_on (array): diff
    if let Some(ref desired_deps) = body.depends_on {
        let (to_add, to_remove) = diff_vec(&fm.depends_on, desired_deps);
        for dep in &to_remove {
            state.project.link_items(&id, dep, LinkRelation::DependsOn, LinkAction::Remove)
                .map_err(map_core_error)?;
        }
        for dep in &to_add {
            state.project.link_items(&id, dep, LinkRelation::DependsOn, LinkAction::Add)
                .map_err(map_core_error)?;
        }
    }

    // Re-read for response
    let doc: MarkplaneDocument<Epic> = state.project.read_item(&id).map_err(map_core_error)?;
    let tasks = state.project.list_tasks(&QueryFilter::default()).map_err(map_core_error)?;

    Ok(Json(ApiResponse {
        data: epic_to_response(&doc, &tasks),
    }))
}

async fn update_plan(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(body): Json<UpdatePlanRequest>,
) -> Result<Json<ApiResponse<PlanResponse>>, (StatusCode, Json<ApiError>)> {
    parse_id(&id)
        .map_err(|_| error_response(StatusCode::BAD_REQUEST, "invalid_id", &format!("Invalid ID format: {}", id)))?;

    // Read current state for link diffing
    let current: MarkplaneDocument<Plan> = state.project.read_item(&id).map_err(map_core_error)?;

    // Properties + body
    let has_changes = body.title.is_some() || body.status.is_some() || body.body.is_some();
    if has_changes {
        state.project.update_plan(&id, &PlanUpdate {
            title: body.title,
            status: body.status,
            body: body.body,
        }).map_err(map_core_error)?;
    }

    // ── Links → link_items() per change ──────────────────────────────

    // Epic (scalar): Add overwrites, Remove clears
    if let Some(ref desired_epic) = body.epic {
        let desired = if desired_epic.is_empty() { None } else { Some(desired_epic.as_str()) };
        let current_epic = current.frontmatter.epic.as_deref();
        if desired != current_epic {
            match desired {
                Some(new) => {
                    state.project.link_items(&id, new, LinkRelation::Epic, LinkAction::Add)
                        .map_err(map_core_error)?;
                }
                None => {
                    if let Some(old) = current_epic {
                        state.project.link_items(&id, old, LinkRelation::Epic, LinkAction::Remove)
                            .map_err(map_core_error)?;
                    }
                }
            }
        }
    }

    // Re-read for response
    let doc: MarkplaneDocument<Plan> = state.project.read_item(&id).map_err(map_core_error)?;
    Ok(Json(ApiResponse {
        data: plan_to_response(&doc),
    }))
}

async fn update_note(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(body): Json<UpdateNoteRequest>,
) -> Result<Json<ApiResponse<NoteResponse>>, (StatusCode, Json<ApiError>)> {
    parse_id(&id)
        .map_err(|_| error_response(StatusCode::BAD_REQUEST, "invalid_id", &format!("Invalid ID format: {}", id)))?;

    // Read current state for diffing tags and links
    let current: MarkplaneDocument<Note> = state.project.read_item(&id).map_err(map_core_error)?;
    let fm = &current.frontmatter;

    // ── Properties → NoteUpdate ──────────────────────────────────────
    let (add_tags, remove_tags) = if let Some(ref desired_tags) = body.tags {
        diff_vec(&fm.tags, desired_tags)
    } else {
        (vec![], vec![])
    };

    let has_changes = body.title.is_some() || body.status.is_some()
        || body.note_type.is_some()
        || !add_tags.is_empty() || !remove_tags.is_empty()
        || body.body.is_some();
    if has_changes {
        state.project.update_note(&id, &NoteUpdate {
            title: body.title,
            status: body.status,
            note_type: body.note_type,
            add_tags,
            remove_tags,
            body: body.body,
        }).map_err(map_core_error)?;
    }

    // ── Links → link_items() per change ──────────────────────────────

    // related (array): diff
    if let Some(ref desired_related) = body.related {
        let (to_add, to_remove) = diff_vec(&fm.related, desired_related);
        for rel in &to_remove {
            state.project.link_items(&id, rel, LinkRelation::Related, LinkAction::Remove)
                .map_err(map_core_error)?;
        }
        for rel in &to_add {
            state.project.link_items(&id, rel, LinkRelation::Related, LinkAction::Add)
                .map_err(map_core_error)?;
        }
    }

    // Re-read for response
    let doc: MarkplaneDocument<Note> = state.project.read_item(&id).map_err(map_core_error)?;
    Ok(Json(ApiResponse {
        data: note_to_response(&doc),
    }))
}

async fn post_sync(
    State(state): State<Arc<AppState>>,
) -> Result<Json<ApiResponse<SyncResponse>>, (StatusCode, Json<ApiError>)> {
    state
        .project
        .sync_all()
        .map_err(|e| error_response(StatusCode::INTERNAL_SERVER_ERROR, "sync_error", &e.to_string()))?;

    // Broadcast sync_complete event to all WebSocket clients
    let event = serde_json::json!({"type": "sync_complete"});
    let _ = state.ws_tx.send(event.to_string());

    Ok(Json(ApiResponse {
        data: SyncResponse {
            success: true,
            message: "Sync complete".to_string(),
        },
    }))
}

#[derive(Serialize)]
struct SyncResponse {
    success: bool,
    message: String,
}

// ── Link ─────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct LinkRequest {
    from: String,
    to: String,
    relation: String,
    #[serde(default)]
    remove: bool,
}

#[derive(Serialize)]
struct LinkResponse {
    from: String,
    to: String,
    relation: String,
    action: String,
}

async fn post_link(
    State(state): State<Arc<AppState>>,
    Json(req): Json<LinkRequest>,
) -> Result<Json<ApiResponse<LinkResponse>>, (StatusCode, Json<ApiError>)> {
    let relation: LinkRelation = req.relation.parse().map_err(|e: markplane_core::MarkplaneError| {
        error_response(StatusCode::BAD_REQUEST, "invalid_relation", &e.to_string())
    })?;
    let action = if req.remove {
        LinkAction::Remove
    } else {
        LinkAction::Add
    };

    state
        .project
        .link_items(&req.from, &req.to, relation, action)
        .map_err(|e| error_response(StatusCode::BAD_REQUEST, "link_error", &e.to_string()))?;

    let action_str = if req.remove { "removed" } else { "added" };

    Ok(Json(ApiResponse {
        data: LinkResponse {
            from: req.from,
            to: req.to,
            relation: relation.to_string(),
            action: action_str.to_string(),
        },
    }))
}

// ── Search ───────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct SearchParams {
    q: String,
    #[serde(default)]
    include_archived: bool,
}

#[derive(Serialize)]
struct SearchResultResponse {
    id: String,
    entity_type: String,
    title: String,
    status: String,
    priority: Option<String>,
    snippet: String,
    score: f64,
    archived: bool,
}

async fn get_search(
    State(state): State<Arc<AppState>>,
    Query(params): Query<SearchParams>,
) -> Result<Json<ApiListResponse<SearchResultResponse>>, (StatusCode, Json<ApiError>)> {
    let query = params.q.to_lowercase();
    if query.len() < 2 {
        return Ok(Json(ApiListResponse {
            data: vec![],
            meta: ListMeta { total: 0 },
        }));
    }

    let mut results: Vec<SearchResultResponse> = Vec::new();

    // Search tasks
    let mut tasks = state
        .project
        .list_tasks(&QueryFilter::default())
        .map_err(|e| error_response(StatusCode::INTERNAL_SERVER_ERROR, "query_error", &e.to_string()))?;
    let archived_task_ids: HashSet<String> = if params.include_archived {
        let archived = state.project.list_tasks(&QueryFilter { archived: true, ..Default::default() })
            .map_err(|e| error_response(StatusCode::INTERNAL_SERVER_ERROR, "query_error", &e.to_string()))?;
        let ids = archived.iter().map(|d| d.frontmatter.id.clone()).collect();
        tasks.extend(archived);
        ids
    } else {
        HashSet::new()
    };
    for doc in &tasks {
        let fm = &doc.frontmatter;
        let id_match = fm.id.to_lowercase().contains(&query);
        let title_match = fm.title.to_lowercase().contains(&query);
        let tag_match = fm.tags.iter().any(|t| t.to_lowercase().contains(&query));
        let assignee_match = fm.assignee.as_ref().is_some_and(|a| a.to_lowercase().contains(&query));
        let body_match = doc.body.to_lowercase().contains(&query);
        if id_match || title_match || tag_match || assignee_match || body_match {
            let score = if id_match { 3.0 } else if title_match { 2.0 } else if tag_match || assignee_match { 1.5 } else { 1.0 };
            let snippet = extract_snippet(&doc.body, &query);
            results.push(SearchResultResponse {
                id: fm.id.clone(),
                entity_type: "task".to_string(),
                title: fm.title.clone(),
                status: fm.status.to_string(),
                priority: Some(fm.priority.to_string()),
                snippet,
                score,
                archived: archived_task_ids.contains(&fm.id),
            });
        }
    }

    // Search epics
    let mut epics = state
        .project
        .list_epics()
        .map_err(|e| error_response(StatusCode::INTERNAL_SERVER_ERROR, "query_error", &e.to_string()))?;
    let archived_epic_ids: HashSet<String> = if params.include_archived {
        let archived = state.project.list_epics_filtered(true)
            .map_err(|e| error_response(StatusCode::INTERNAL_SERVER_ERROR, "query_error", &e.to_string()))?;
        let ids = archived.iter().map(|d| d.frontmatter.id.clone()).collect();
        epics.extend(archived);
        ids
    } else {
        HashSet::new()
    };
    for doc in &epics {
        let fm = &doc.frontmatter;
        let id_match = fm.id.to_lowercase().contains(&query);
        let title_match = fm.title.to_lowercase().contains(&query);
        let tag_match = fm.tags.iter().any(|t| t.to_lowercase().contains(&query));
        let body_match = doc.body.to_lowercase().contains(&query);
        if id_match || title_match || tag_match || body_match {
            let score = if id_match { 3.0 } else if title_match { 2.0 } else if tag_match { 1.5 } else { 1.0 };
            let snippet = extract_snippet(&doc.body, &query);
            results.push(SearchResultResponse {
                id: fm.id.clone(),
                entity_type: "epic".to_string(),
                title: fm.title.clone(),
                status: fm.status.to_string(),
                priority: Some(fm.priority.to_string()),
                snippet,
                score,
                archived: archived_epic_ids.contains(&fm.id),
            });
        }
    }

    // Search plans
    let mut plans = state
        .project
        .list_plans()
        .map_err(|e| error_response(StatusCode::INTERNAL_SERVER_ERROR, "query_error", &e.to_string()))?;
    let archived_plan_ids: HashSet<String> = if params.include_archived {
        let archived = state.project.list_plans_filtered(true)
            .map_err(|e| error_response(StatusCode::INTERNAL_SERVER_ERROR, "query_error", &e.to_string()))?;
        let ids = archived.iter().map(|d| d.frontmatter.id.clone()).collect();
        plans.extend(archived);
        ids
    } else {
        HashSet::new()
    };
    for doc in &plans {
        let fm = &doc.frontmatter;
        let id_match = fm.id.to_lowercase().contains(&query);
        let title_match = fm.title.to_lowercase().contains(&query);
        let body_match = doc.body.to_lowercase().contains(&query);
        if id_match || title_match || body_match {
            let score = if id_match { 3.0 } else if title_match { 2.0 } else { 1.0 };
            let snippet = extract_snippet(&doc.body, &query);
            results.push(SearchResultResponse {
                id: fm.id.clone(),
                entity_type: "plan".to_string(),
                title: fm.title.clone(),
                status: fm.status.to_string(),
                priority: None,
                snippet,
                score,
                archived: archived_plan_ids.contains(&fm.id),
            });
        }
    }

    // Search notes
    let mut notes = state
        .project
        .list_notes()
        .map_err(|e| error_response(StatusCode::INTERNAL_SERVER_ERROR, "query_error", &e.to_string()))?;
    let archived_note_ids: HashSet<String> = if params.include_archived {
        let archived = state.project.list_notes_filtered(true)
            .map_err(|e| error_response(StatusCode::INTERNAL_SERVER_ERROR, "query_error", &e.to_string()))?;
        let ids = archived.iter().map(|d| d.frontmatter.id.clone()).collect();
        notes.extend(archived);
        ids
    } else {
        HashSet::new()
    };
    for doc in &notes {
        let fm = &doc.frontmatter;
        let id_match = fm.id.to_lowercase().contains(&query);
        let title_match = fm.title.to_lowercase().contains(&query);
        let tag_match = fm.tags.iter().any(|t| t.to_lowercase().contains(&query));
        let body_match = doc.body.to_lowercase().contains(&query);
        if id_match || title_match || tag_match || body_match {
            let score = if id_match { 3.0 } else if title_match { 2.0 } else if tag_match { 1.5 } else { 1.0 };
            let snippet = extract_snippet(&doc.body, &query);
            results.push(SearchResultResponse {
                id: fm.id.clone(),
                entity_type: "note".to_string(),
                title: fm.title.clone(),
                status: fm.status.to_string(),
                priority: None,
                snippet,
                score,
                archived: archived_note_ids.contains(&fm.id),
            });
        }
    }

    // Sort by score descending, then by ID
    results.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.id.cmp(&b.id))
    });

    let total = results.len();
    Ok(Json(ApiListResponse {
        data: results,
        meta: ListMeta { total },
    }))
}

/// Extract a text snippet around the first match of `query` in `body`.
fn extract_snippet(body: &str, query: &str) -> String {
    let lower = body.to_lowercase();
    if let Some(pos) = lower.find(query) {
        let start = pos.saturating_sub(40);
        let end = (pos + query.len() + 80).min(body.len());
        // Find safe UTF-8 boundaries
        let start = body.floor_char_boundary(start);
        let end = body.ceil_char_boundary(end);
        let mut snippet = body[start..end].to_string();
        // Replace newlines with spaces for cleaner display
        snippet = snippet.replace('\n', " ");
        if start > 0 {
            snippet.insert_str(0, "...");
        }
        if end < body.len() {
            snippet.push_str("...");
        }
        snippet
    } else {
        // No body match — return first ~120 chars of body
        let end = (120).min(body.len());
        let end = body.ceil_char_boundary(end);
        let mut snippet = body[..end].replace('\n', " ");
        if end < body.len() {
            snippet.push_str("...");
        }
        snippet
    }
}

// ── Graph ────────────────────────────────────────────────────────────────

#[derive(Serialize)]
struct GraphNodeResponse {
    id: String,
    #[serde(rename = "type")]
    node_type: String,
    title: String,
    status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    priority: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    tags: Vec<String>,
}

#[derive(Serialize)]
struct GraphEdgeResponse {
    source: String,
    target: String,
    relation: String,
}

#[derive(Serialize)]
struct GraphResponse {
    nodes: Vec<GraphNodeResponse>,
    edges: Vec<GraphEdgeResponse>,
}

async fn get_graph_all(
    State(state): State<Arc<AppState>>,
) -> Result<Json<ApiResponse<GraphResponse>>, (StatusCode, Json<ApiError>)> {
    let graph = build_graph(&state.project, None)
        .map_err(|e| error_response(StatusCode::INTERNAL_SERVER_ERROR, "graph_error", &e.to_string()))?;
    Ok(Json(ApiResponse { data: graph }))
}

async fn get_graph_focused(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<GraphResponse>>, (StatusCode, Json<ApiError>)> {
    parse_id(&id)
        .map_err(|_| error_response(StatusCode::BAD_REQUEST, "invalid_id", &format!("Invalid ID format: {}", id)))?;
    let graph = build_graph(&state.project, Some(&id))
        .map_err(|e| error_response(StatusCode::INTERNAL_SERVER_ERROR, "graph_error", &e.to_string()))?;
    Ok(Json(ApiResponse { data: graph }))
}

fn build_graph(
    project: &Project,
    focus_id: Option<&str>,
) -> anyhow::Result<GraphResponse> {
    use std::collections::HashSet;

    let mut nodes_map: HashMap<String, GraphNodeResponse> = HashMap::new();
    let mut edges: Vec<GraphEdgeResponse> = Vec::new();

    // Collect tasks
    let tasks = project.list_tasks(&QueryFilter::default())?;
    for doc in &tasks {
        let fm = &doc.frontmatter;
        nodes_map.insert(
            fm.id.clone(),
            GraphNodeResponse {
                id: fm.id.clone(),
                node_type: "task".to_string(),
                title: fm.title.clone(),
                status: fm.status.to_string(),
                priority: Some(fm.priority.to_string()),
                tags: fm.tags.clone(),
            },
        );
        for dep in &fm.depends_on {
            edges.push(GraphEdgeResponse {
                source: dep.clone(),
                target: fm.id.clone(),
                relation: "depends_on".to_string(),
            });
        }
        for blk in &fm.blocks {
            edges.push(GraphEdgeResponse {
                source: fm.id.clone(),
                target: blk.clone(),
                relation: "blocks".to_string(),
            });
        }
        if let Some(epic) = &fm.epic {
            edges.push(GraphEdgeResponse {
                source: epic.clone(),
                target: fm.id.clone(),
                relation: "epic".to_string(),
            });
        }
    }

    // Collect epics
    let epics = project.list_epics()?;
    for doc in &epics {
        let fm = &doc.frontmatter;
        nodes_map.insert(
            fm.id.clone(),
            GraphNodeResponse {
                id: fm.id.clone(),
                node_type: "epic".to_string(),
                title: fm.title.clone(),
                status: fm.status.to_string(),
                priority: Some(fm.priority.to_string()),
                tags: fm.tags.clone(),
            },
        );
        for dep in &fm.depends_on {
            edges.push(GraphEdgeResponse {
                source: dep.clone(),
                target: fm.id.clone(),
                relation: "depends_on".to_string(),
            });
        }
    }

    // Collect plans
    let plans = project.list_plans()?;
    for doc in &plans {
        let fm = &doc.frontmatter;
        nodes_map.insert(
            fm.id.clone(),
            GraphNodeResponse {
                id: fm.id.clone(),
                node_type: "plan".to_string(),
                title: fm.title.clone(),
                status: fm.status.to_string(),
                priority: None,
                tags: vec![],
            },
        );
        for imp in &fm.implements {
            edges.push(GraphEdgeResponse {
                source: fm.id.clone(),
                target: imp.clone(),
                relation: "implements".to_string(),
            });
        }
        if let Some(epic) = &fm.epic {
            edges.push(GraphEdgeResponse {
                source: epic.clone(),
                target: fm.id.clone(),
                relation: "epic".to_string(),
            });
        }
    }

    // Collect notes
    let notes = project.list_notes()?;
    for doc in &notes {
        let fm = &doc.frontmatter;
        nodes_map.insert(
            fm.id.clone(),
            GraphNodeResponse {
                id: fm.id.clone(),
                node_type: "note".to_string(),
                title: fm.title.clone(),
                status: fm.status.to_string(),
                priority: None,
                tags: vec![],
            },
        );
        for rel in &fm.related {
            edges.push(GraphEdgeResponse {
                source: fm.id.clone(),
                target: rel.clone(),
                relation: "related".to_string(),
            });
        }
    }

    // Only keep edges where both source and target exist as nodes
    edges.retain(|e| nodes_map.contains_key(&e.source) && nodes_map.contains_key(&e.target));

    // If focused on a specific ID, filter to only reachable nodes within 2 hops
    if let Some(focus) = focus_id {
        let mut reachable: HashSet<String> = HashSet::new();
        reachable.insert(focus.to_string());

        // BFS 2 hops (both directions)
        for _ in 0..2 {
            let current: Vec<String> = reachable.iter().cloned().collect();
            for id in &current {
                for edge in &edges {
                    if edge.source == *id {
                        reachable.insert(edge.target.clone());
                    }
                    if edge.target == *id {
                        reachable.insert(edge.source.clone());
                    }
                }
            }
        }

        edges.retain(|e| reachable.contains(&e.source) && reachable.contains(&e.target));
        nodes_map.retain(|id, _| reachable.contains(id));
    }

    let nodes: Vec<GraphNodeResponse> = nodes_map.into_values().collect();
    Ok(GraphResponse { nodes, edges })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diff_vec_no_change() {
        let current = vec!["a".to_string(), "b".to_string()];
        let desired = vec!["a".to_string(), "b".to_string()];
        let (add, remove) = diff_vec(&current, &desired);
        assert!(add.is_empty());
        assert!(remove.is_empty());
    }

    #[test]
    fn test_diff_vec_additions_only() {
        let current = vec!["a".to_string()];
        let desired = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let (add, remove) = diff_vec(&current, &desired);
        assert_eq!(add, vec!["b", "c"]);
        assert!(remove.is_empty());
    }

    #[test]
    fn test_diff_vec_removals_only() {
        let current = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let desired = vec!["a".to_string()];
        let (add, remove) = diff_vec(&current, &desired);
        assert!(add.is_empty());
        assert_eq!(remove, vec!["b", "c"]);
    }

    #[test]
    fn test_diff_vec_mixed() {
        let current = vec!["a".to_string(), "b".to_string()];
        let desired = vec!["b".to_string(), "c".to_string()];
        let (add, remove) = diff_vec(&current, &desired);
        assert_eq!(add, vec!["c"]);
        assert_eq!(remove, vec!["a"]);
    }

    #[test]
    fn test_diff_vec_empty_to_some() {
        let current: Vec<String> = vec![];
        let desired = vec!["a".to_string(), "b".to_string()];
        let (add, remove) = diff_vec(&current, &desired);
        assert_eq!(add, vec!["a", "b"]);
        assert!(remove.is_empty());
    }

    #[test]
    fn test_diff_vec_some_to_empty() {
        let current = vec!["a".to_string(), "b".to_string()];
        let desired: Vec<String> = vec![];
        let (add, remove) = diff_vec(&current, &desired);
        assert!(add.is_empty());
        assert_eq!(remove, vec!["a", "b"]);
    }
}
