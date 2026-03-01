mod init;
mod add;
mod show;
mod edit;
mod ls;
mod status;
mod sync;
mod start;
mod done;
mod promote;
mod plan;
mod epic;
mod note;
mod update;
mod link;
mod check;
mod stale;
mod archive;
mod unarchive;
mod context;
mod formatting;
mod metrics;
mod graph;
mod claude_md;
mod dashboard;
mod serve;
mod mcp;

use std::path::PathBuf;
use clap::Subcommand;

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize a new .markplane/ structure
    Init {
        /// Project name (defaults to current directory name)
        #[arg(long)]
        name: Option<String>,
        /// Project description
        #[arg(long, default_value = "")]
        description: String,
        /// Skip starter content (create empty project)
        #[arg(long)]
        empty: bool,
    },

    /// Create a new task
    Add {
        /// Title of the item
        title: String,
        /// Task type (configurable in config.yaml, default: first in task_types list)
        #[arg(long)]
        r#type: Option<String>,
        /// Priority level
        #[arg(long, default_value = "medium")]
        priority: String,
        /// Effort estimate
        #[arg(long, default_value = "medium")]
        effort: String,
        /// Parent epic ID
        #[arg(long)]
        epic: Option<String>,
        /// Comma-separated tags
        #[arg(long)]
        tags: Option<String>,
        /// Template name override (e.g. "bug", "default")
        #[arg(long)]
        template: Option<String>,
    },

    /// Show details of an item
    Show {
        /// Item ID (e.g. TASK-042)
        id: String,
    },

    /// Open an item in $EDITOR
    Edit {
        /// Item ID (e.g. TASK-042)
        id: String,
    },

    /// List items
    Ls {
        /// Item kind to list: tasks (default), epics, plans, notes
        #[command(subcommand)]
        kind: Option<LsKind>,

        /// Filter by status (comma-separated)
        #[arg(long)]
        status: Option<String>,
        /// Filter by priority (comma-separated)
        #[arg(long)]
        priority: Option<String>,
        /// Filter by epic ID
        #[arg(long)]
        epic: Option<String>,
        /// Filter by tags (comma-separated)
        #[arg(long)]
        tags: Option<String>,
        /// Filter by assignee
        #[arg(long)]
        assignee: Option<String>,
        /// Filter by item type (comma-separated)
        #[arg(long)]
        r#type: Option<String>,
        /// List archived items instead of active ones
        #[arg(long)]
        archived: bool,
    },

    /// Update the status of an item
    Status {
        /// Item ID (e.g. TASK-042)
        id: String,
        /// New status value
        new_status: String,
    },

    /// Regenerate INDEX.md files and .context/ summaries
    Sync {
        /// Also normalize position keys (rewrites source files)
        #[arg(long)]
        normalize: bool,
    },

    /// Start working on an item (sets status to in-progress and assigns to you)
    Start {
        /// Item ID (e.g. TASK-042)
        id: String,
        /// Assignee name (defaults to $USER or "me")
        #[arg(long)]
        user: Option<String>,
    },

    /// Mark an item as done
    Done {
        /// Item ID (e.g. TASK-042)
        id: String,
    },

    /// Promote a note to a task
    Promote {
        /// Note ID (e.g. NOTE-007)
        id: String,
        /// Priority for the new task
        #[arg(long, default_value = "medium")]
        priority: String,
        /// Effort estimate for the new task
        #[arg(long, default_value = "medium")]
        effort: String,
    },

    /// Create a linked implementation plan for a task
    Plan {
        /// Task ID (e.g. TASK-042)
        id: String,
        /// Plan title (defaults to "Implementation plan for <item title>")
        #[arg(long)]
        title: Option<String>,
        /// Template name override (e.g. "refactor", "implementation")
        #[arg(long)]
        template: Option<String>,
    },

    /// Create a new epic
    Epic {
        /// Epic title
        title: String,
        /// Priority level
        #[arg(long, default_value = "medium")]
        priority: String,
    },

    /// Create a new note
    Note {
        /// Note title
        title: String,
        /// Note type (configurable in config.yaml, default: first in note_types list)
        #[arg(long)]
        r#type: Option<String>,
        /// Comma-separated tags
        #[arg(long)]
        tags: Option<String>,
        /// Template name override (e.g. "research", "analysis")
        #[arg(long)]
        template: Option<String>,
    },

    /// Update properties on any item
    Update {
        /// Item ID (e.g. TASK-042, EPIC-001, PLAN-003, NOTE-007)
        id: String,
        /// New title
        #[arg(long)]
        title: Option<String>,
        /// New status
        #[arg(long)]
        status: Option<String>,
        /// New priority (tasks and epics only)
        #[arg(long)]
        priority: Option<String>,
        /// New effort size (tasks only)
        #[arg(long)]
        effort: Option<String>,
        /// New item type (tasks only: feature, bug, enhancement, chore, research, spike)
        #[arg(long)]
        r#type: Option<String>,
        /// Set assignee (tasks only)
        #[arg(long)]
        assignee: Option<String>,
        /// Clear assignee
        #[arg(long)]
        clear_assignee: bool,
        /// Set position key (tasks only)
        #[arg(long)]
        position: Option<String>,
        /// Clear position
        #[arg(long)]
        clear_position: bool,
        /// Comma-separated tags to add
        #[arg(long)]
        add_tag: Option<String>,
        /// Comma-separated tags to remove
        #[arg(long)]
        remove_tag: Option<String>,
        /// Set started date (epics only, YYYY-MM-DD)
        #[arg(long)]
        started: Option<String>,
        /// Clear started date
        #[arg(long)]
        clear_started: bool,
        /// Set target date (epics only, YYYY-MM-DD)
        #[arg(long)]
        target: Option<String>,
        /// Clear target date
        #[arg(long)]
        clear_target: bool,
        /// New note type (notes only, configurable in config.yaml)
        #[arg(long)]
        note_type: Option<String>,
    },

    /// Link two items with a relationship
    Link {
        /// Source item ID
        from: String,
        /// Target item ID
        to: String,
        /// Relationship type: blocks, depends-on, epic, plan, implements, related
        #[arg(long, short = 'r')]
        relation: String,
        /// Remove the link instead of adding it
        #[arg(long)]
        remove: bool,
    },

    /// Validate cross-references and find broken links
    Check {
        /// Also show orphan items (no incoming references)
        #[arg(long)]
        orphans: bool,
    },

    /// List items not updated in N days
    Stale {
        /// Number of days threshold
        #[arg(long, default_value = "30")]
        days: u32,
    },

    /// Archive an item or all completed items
    Archive {
        /// Item ID to archive (e.g. TASK-042)
        id: Option<String>,
        /// Archive all completed items across all types
        #[arg(long)]
        all_done: bool,
        /// Preview what would be archived
        #[arg(long)]
        dry_run: bool,
    },

    /// Restore an archived item back to active
    Unarchive {
        /// Item ID to restore (e.g. TASK-042)
        id: String,
    },

    /// Regenerate .context/ files
    Context {
        /// Focus on a specific context view (active-work, blocked, metrics, summary)
        #[arg(long)]
        focus: Option<String>,
    },

    /// Show project metrics and statistics
    Metrics,

    /// Show dependency graph for an item
    Graph {
        /// Item ID (e.g. TASK-042)
        id: String,
        /// Maximum depth to traverse
        #[arg(long, default_value = "3")]
        depth: u32,
    },

    /// Output CLAUDE.md integration snippet
    ClaudeMd,

    /// Show project dashboard overview
    Dashboard,

    /// Start the web UI server
    Serve {
        /// Port to listen on
        #[arg(long, default_value = "4200")]
        port: u16,
        /// Open browser automatically
        #[arg(long)]
        open: bool,
        /// Dev mode: API only, no static files
        #[arg(long)]
        dev: bool,
    },

    /// Run the MCP (Model Context Protocol) server over stdio
    Mcp {
        /// Path to the project directory (defaults to current directory)
        #[arg(long)]
        project: Option<PathBuf>,
    },
}

#[derive(Subcommand)]
pub enum LsKind {
    /// List epics
    Epics,
    /// List plans
    Plans,
    /// List notes
    Notes,
}

pub fn execute(cmd: Commands) -> anyhow::Result<()> {
    match cmd {
        Commands::Init { name, description, empty } => init::run(name, description, empty),
        Commands::Add {
            title,
            r#type,
            priority,
            effort,
            epic,
            tags,
            template,
        } => add::run(title, r#type, priority, effort, epic, tags, template),
        Commands::Show { id } => show::run(id),
        Commands::Edit { id } => edit::run(id),
        Commands::Ls {
            kind,
            status,
            priority,
            epic,
            tags,
            assignee,
            r#type,
            archived,
        } => ls::run(kind, status, priority, epic, tags, assignee, r#type, archived),
        Commands::Status { id, new_status } => status::run(id, new_status),
        Commands::Sync { normalize } => sync::run(normalize),
        Commands::Start { id, user } => start::run(id, user),
        Commands::Done { id } => done::run(id),
        Commands::Promote { id, priority, effort } => promote::run(id, priority, effort),
        Commands::Plan { id, title, template } => plan::run(id, title, template),
        Commands::Epic { title, priority } => epic::run(title, priority),
        Commands::Note { title, r#type, tags, template } => note::run(title, r#type, tags, template),
        Commands::Update {
            id, title, status, priority, effort, r#type, assignee,
            clear_assignee, position, clear_position, add_tag, remove_tag,
            started, clear_started, target, clear_target, note_type,
        } => update::run(
            id, title, status, priority, effort, r#type, assignee,
            clear_assignee, position, clear_position, add_tag, remove_tag,
            started, clear_started, target, clear_target, note_type,
        ),
        Commands::Link {
            from,
            to,
            relation,
            remove,
        } => link::run(from, to, relation, remove),
        Commands::Check { orphans } => check::run(orphans),
        Commands::Stale { days } => stale::run(days),
        Commands::Archive { id, all_done, dry_run } => archive::run(id, all_done, dry_run),
        Commands::Unarchive { id } => unarchive::run(id),
        Commands::Context { focus } => context::run(focus),
        Commands::Metrics => metrics::run(),
        Commands::Graph { id, depth } => graph::run(id, depth),
        Commands::ClaudeMd => claude_md::run(),
        Commands::Dashboard => dashboard::run(),
        Commands::Serve { port, open, dev } => {
            let rt = tokio::runtime::Runtime::new()?;
            rt.block_on(serve::run(port, open, dev))
        }
        Commands::Mcp { project } => mcp::run(project),
    }
}

/// Split a comma-separated string into a Vec of trimmed strings.
pub fn parse_comma_list(s: &str) -> Vec<String> {
    s.split(',')
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .collect()
}
