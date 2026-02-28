pub mod models;
pub mod frontmatter;
pub mod project;
pub mod query;
pub mod references;
pub mod index;
pub mod context;
pub mod templates;
pub mod manifest;
pub mod position;
pub mod error;
pub mod links;

pub use error::{MarkplaneError, Result};
pub use models::{
    Task, StatusCategory, TaskWorkflow, WorkflowConfig, Config, Effort, Epic,
    EpicStatus, IdPrefix, MarkplaneDocument, Note, NoteStatus, Plan, PlanStatus,
    Priority, generate_random_id, parse_id, default_task_types, default_note_types,
    default_task_workflow,
};
pub use frontmatter::{parse_frontmatter, parse_frontmatter_raw, write_frontmatter};
pub use project::{
    Project, find_blocked_items, apply_tag_changes,
    Patch, TaskUpdate, EpicUpdate, PlanUpdate, NoteUpdate, UpdateFields,
    MoveDirective,
};
pub use query::{QueryFilter, ScanScope};
pub use references::{extract_references, validate_references, validate_task_statuses, find_orphans, BrokenReference, build_reference_graph};
pub use templates::render_template;
pub use links::{LinkRelation, LinkAction};
