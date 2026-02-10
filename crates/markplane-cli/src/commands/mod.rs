mod init;
mod add;
mod show;
mod ls;
mod status;
mod sync;
mod start;
mod done;
mod promote;
mod plan;
mod epic;
mod note;
mod assign;
mod link;
mod tag;
mod check;
mod stale;
mod archive;
mod context;
mod formatting;
mod metrics;
mod graph;
mod claude_md;
mod dashboard;

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
    },

    /// Create a new backlog item
    Add {
        /// Title of the item
        title: String,
        /// Item type
        #[arg(long, default_value = "feature")]
        r#type: String,
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
    },

    /// Show details of an item
    Show {
        /// Item ID (e.g. BACK-042)
        id: String,
    },

    /// List items
    Ls {
        /// Item kind to list: backlog (default), epics, plans, notes
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
    },

    /// Update the status of an item
    Status {
        /// Item ID (e.g. BACK-042)
        id: String,
        /// New status value
        new_status: String,
    },

    /// Regenerate INDEX.md files and .context/ summaries
    Sync,

    /// Start working on an item (sets status to in-progress and assigns to you)
    Start {
        /// Item ID (e.g. BACK-042)
        id: String,
        /// Assignee name (defaults to $USER or "me")
        #[arg(long)]
        user: Option<String>,
    },

    /// Mark an item as done
    Done {
        /// Item ID (e.g. BACK-042)
        id: String,
    },

    /// Promote a note to a backlog item
    Promote {
        /// Note ID (e.g. NOTE-007)
        id: String,
        /// Priority for the new backlog item
        #[arg(long, default_value = "medium")]
        priority: String,
        /// Effort estimate for the new backlog item
        #[arg(long, default_value = "medium")]
        effort: String,
    },

    /// Create a linked implementation plan for a backlog item
    Plan {
        /// Backlog item ID (e.g. BACK-042)
        id: String,
        /// Plan title (defaults to "Implementation plan for <item title>")
        #[arg(long)]
        title: Option<String>,
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
        /// Note type (research, analysis, idea, decision, meeting)
        #[arg(long, default_value = "idea")]
        r#type: String,
        /// Comma-separated tags
        #[arg(long)]
        tags: Option<String>,
    },

    /// Assign an item to a user
    Assign {
        /// Item ID (e.g. BACK-042)
        id: String,
        /// User to assign (e.g. @daniel)
        user: String,
    },

    /// Add a dependency link between items
    Link {
        /// Source item ID
        id: String,
        /// Target item that the source blocks
        #[arg(long)]
        blocks: Option<String>,
        /// Target item that the source depends on
        #[arg(long)]
        depends_on: Option<String>,
    },

    /// Add tags to an item
    Tag {
        /// Item ID (e.g. BACK-042)
        id: String,
        /// Comma-separated tags to add
        tags: String,
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

    /// Move done/cancelled items to archive directories
    Archive {
        /// Preview what would be archived without making changes
        #[arg(long)]
        dry_run: bool,
    },

    /// Regenerate .context/ files
    Context {
        /// Generate focused context for a specific item
        #[arg(long)]
        item: Option<String>,
        /// Generate focused context for a domain/tag
        #[arg(long)]
        focus: Option<String>,
    },

    /// Show project metrics and statistics
    Metrics,

    /// Show dependency graph for an item
    Graph {
        /// Item ID (e.g. BACK-042)
        id: String,
        /// Maximum depth to traverse
        #[arg(long, default_value = "3")]
        depth: u32,
    },

    /// Output CLAUDE.md integration snippet
    ClaudeMd,

    /// Show project dashboard overview
    Dashboard,
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
        Commands::Init { name, description } => init::run(name, description),
        Commands::Add {
            title,
            r#type,
            priority,
            effort,
            epic,
            tags,
        } => add::run(title, r#type, priority, effort, epic, tags),
        Commands::Show { id } => show::run(id),
        Commands::Ls {
            kind,
            status,
            priority,
            epic,
            tags,
            assignee,
            r#type,
        } => ls::run(kind, status, priority, epic, tags, assignee, r#type),
        Commands::Status { id, new_status } => status::run(id, new_status),
        Commands::Sync => sync::run(),
        Commands::Start { id, user } => start::run(id, user),
        Commands::Done { id } => done::run(id),
        Commands::Promote { id, priority, effort } => promote::run(id, priority, effort),
        Commands::Plan { id, title } => plan::run(id, title),
        Commands::Epic { title, priority } => epic::run(title, priority),
        Commands::Note { title, r#type, tags } => note::run(title, r#type, tags),
        Commands::Assign { id, user } => assign::run(id, user),
        Commands::Link {
            id,
            blocks,
            depends_on,
        } => link::run(id, blocks, depends_on),
        Commands::Tag { id, tags } => tag::run(id, tags),
        Commands::Check { orphans } => check::run(orphans),
        Commands::Stale { days } => stale::run(days),
        Commands::Archive { dry_run } => archive::run(dry_run),
        Commands::Context { item, focus } => context::run(item, focus),
        Commands::Metrics => metrics::run(),
        Commands::Graph { id, depth } => graph::run(id, depth),
        Commands::ClaudeMd => claude_md::run(),
        Commands::Dashboard => dashboard::run(),
    }
}

/// Split a comma-separated string into a Vec of trimmed strings.
pub fn parse_comma_list(s: &str) -> Vec<String> {
    s.split(',')
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .collect()
}
