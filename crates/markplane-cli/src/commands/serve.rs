use std::collections::HashMap;
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
    Effort, Epic, EpicStatus, ItemType, MarkplaneDocument, Note, Plan, Priority,
    Project, QueryFilter, Task, TaskStatus, find_blocked_items, parse_id,
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

    // Ensure all tasks have position keys (handles migration from pre-position data)
    project.normalize_positions()?;

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
        .route("/api/epics", get(get_epics))
        .route("/api/epics/{id}", get(get_epic))
        .route("/api/plans", get(get_plans))
        .route("/api/plans/{id}", get(get_plan))
        .route("/api/notes", get(get_notes))
        .route("/api/notes/{id}", get(get_note))
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
        body: doc.body.clone(),
        task_count,
        done_count,
        progress,
        status_breakdown,
    }
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

// ── Summary API types ─────────────────────────────────────────────────────

#[derive(Serialize)]
struct SummaryResponse {
    name: String,
    description: String,
    counts: SummaryCounts,
    active_epics: Vec<EpicResponse>,
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

    let active_epics: Vec<_> = epics
        .iter()
        .filter(|e| e.frontmatter.status == EpicStatus::Active)
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
        .map(|t| task_to_response(t))
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
        active_epics,
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
        .create_task(&body.title, item_type, priority, effort, body.epic, body.tags)
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

async fn update_task(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(body): Json<UpdateTaskRequest>,
) -> Result<Json<ApiResponse<TaskResponse>>, (StatusCode, Json<ApiError>)> {
    parse_id(&id)
        .map_err(|_| error_response(StatusCode::BAD_REQUEST, "invalid_id", &format!("Invalid ID format: {}", id)))?;

    let mut doc: MarkplaneDocument<Task> = state
        .project
        .read_item(&id)
        .map_err(|e| error_response(StatusCode::NOT_FOUND, "not_found", &e.to_string()))?;

    if let Some(title) = body.title {
        doc.frontmatter.title = title;
    }
    if let Some(status) = &body.status {
        doc.frontmatter.status = status
            .parse()
            .map_err(|e: markplane_core::MarkplaneError| {
                error_response(StatusCode::BAD_REQUEST, "invalid_status", &e.to_string())
            })?;
    }
    if let Some(priority) = &body.priority {
        doc.frontmatter.priority = priority
            .parse()
            .map_err(|e: markplane_core::MarkplaneError| {
                error_response(StatusCode::BAD_REQUEST, "invalid_priority", &e.to_string())
            })?;
    }
    if let Some(effort) = &body.effort {
        doc.frontmatter.effort = effort
            .parse()
            .map_err(|e: markplane_core::MarkplaneError| {
                error_response(StatusCode::BAD_REQUEST, "invalid_effort", &e.to_string())
            })?;
    }
    if let Some(item_type) = &body.item_type {
        doc.frontmatter.item_type = item_type
            .parse()
            .map_err(|e: markplane_core::MarkplaneError| {
                error_response(StatusCode::BAD_REQUEST, "invalid_type", &e.to_string())
            })?;
    }
    if let Some(tags) = body.tags {
        doc.frontmatter.tags = tags;
    }
    if let Some(epic) = body.epic {
        doc.frontmatter.epic = if epic.is_empty() { None } else { Some(epic) };
    }
    if let Some(plan) = body.plan {
        doc.frontmatter.plan = if plan.is_empty() { None } else { Some(plan) };
    }
    if let Some(assignee) = body.assignee {
        doc.frontmatter.assignee = if assignee.is_empty() {
            None
        } else {
            Some(assignee)
        };
    }
    if let Some(position) = body.position {
        doc.frontmatter.position = if position.is_empty() {
            None
        } else {
            Some(position)
        };
    }
    if let Some(depends_on) = body.depends_on {
        doc.frontmatter.depends_on = depends_on;
    }
    if let Some(blocks) = body.blocks {
        doc.frontmatter.blocks = blocks;
    }

    doc.frontmatter.updated = chrono::Local::now().date_naive();

    state
        .project
        .write_item(&id, &doc)
        .map_err(|e| {
            error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "write_error",
                &e.to_string(),
            )
        })?;

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

async fn get_epics(
    State(state): State<Arc<AppState>>,
) -> Result<Json<ApiListResponse<EpicResponse>>, (StatusCode, Json<ApiError>)> {
    let epics = state
        .project
        .list_epics()
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
) -> Result<Json<ApiListResponse<PlanResponse>>, (StatusCode, Json<ApiError>)> {
    let plans = state
        .project
        .list_plans()
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
) -> Result<Json<ApiListResponse<NoteResponse>>, (StatusCode, Json<ApiError>)> {
    let notes = state
        .project
        .list_notes()
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

// ── Search ───────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct SearchParams {
    q: String,
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
    let tasks = state
        .project
        .list_tasks(&QueryFilter::default())
        .map_err(|e| error_response(StatusCode::INTERNAL_SERVER_ERROR, "query_error", &e.to_string()))?;
    for doc in &tasks {
        let fm = &doc.frontmatter;
        let title_lower = fm.title.to_lowercase();
        let body_lower = doc.body.to_lowercase();
        let title_match = title_lower.contains(&query);
        let body_match = body_lower.contains(&query);
        if title_match || body_match {
            let score = if title_match { 2.0 } else { 1.0 };
            let snippet = extract_snippet(&doc.body, &query);
            results.push(SearchResultResponse {
                id: fm.id.clone(),
                entity_type: "task".to_string(),
                title: fm.title.clone(),
                status: fm.status.to_string(),
                priority: Some(fm.priority.to_string()),
                snippet,
                score,
            });
        }
    }

    // Search epics
    let epics = state
        .project
        .list_epics()
        .map_err(|e| error_response(StatusCode::INTERNAL_SERVER_ERROR, "query_error", &e.to_string()))?;
    for doc in &epics {
        let fm = &doc.frontmatter;
        let title_lower = fm.title.to_lowercase();
        let body_lower = doc.body.to_lowercase();
        let title_match = title_lower.contains(&query);
        let body_match = body_lower.contains(&query);
        if title_match || body_match {
            let score = if title_match { 2.0 } else { 1.0 };
            let snippet = extract_snippet(&doc.body, &query);
            results.push(SearchResultResponse {
                id: fm.id.clone(),
                entity_type: "epic".to_string(),
                title: fm.title.clone(),
                status: fm.status.to_string(),
                priority: Some(fm.priority.to_string()),
                snippet,
                score,
            });
        }
    }

    // Search plans
    let plans = state
        .project
        .list_plans()
        .map_err(|e| error_response(StatusCode::INTERNAL_SERVER_ERROR, "query_error", &e.to_string()))?;
    for doc in &plans {
        let fm = &doc.frontmatter;
        let title_lower = fm.title.to_lowercase();
        let body_lower = doc.body.to_lowercase();
        let title_match = title_lower.contains(&query);
        let body_match = body_lower.contains(&query);
        if title_match || body_match {
            let score = if title_match { 2.0 } else { 1.0 };
            let snippet = extract_snippet(&doc.body, &query);
            results.push(SearchResultResponse {
                id: fm.id.clone(),
                entity_type: "plan".to_string(),
                title: fm.title.clone(),
                status: fm.status.to_string(),
                priority: None,
                snippet,
                score,
            });
        }
    }

    // Search notes
    let notes = state
        .project
        .list_notes()
        .map_err(|e| error_response(StatusCode::INTERNAL_SERVER_ERROR, "query_error", &e.to_string()))?;
    for doc in &notes {
        let fm = &doc.frontmatter;
        let title_lower = fm.title.to_lowercase();
        let body_lower = doc.body.to_lowercase();
        let title_match = title_lower.contains(&query);
        let body_match = body_lower.contains(&query);
        if title_match || body_match {
            let score = if title_match { 2.0 } else { 1.0 };
            let snippet = extract_snippet(&doc.body, &query);
            results.push(SearchResultResponse {
                id: fm.id.clone(),
                entity_type: "note".to_string(),
                title: fm.title.clone(),
                status: fm.status.to_string(),
                priority: None,
                snippet,
                score,
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

    // Only include nodes that have at least one edge (unless focused)
    if focus_id.is_none() {
        let mut connected: HashSet<String> = HashSet::new();
        for edge in &edges {
            connected.insert(edge.source.clone());
            connected.insert(edge.target.clone());
        }
        nodes_map.retain(|id, _| connected.contains(id));
    }

    let nodes: Vec<GraphNodeResponse> = nodes_map.into_values().collect();
    Ok(GraphResponse { nodes, edges })
}
