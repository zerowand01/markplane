pub mod context;
pub mod error;
pub mod frontmatter;
pub mod index;
pub mod links;
pub mod manifest;
pub mod models;
pub mod position;
pub mod project;
pub mod query;
pub mod references;
pub mod templates;

pub use error::{MarkplaneError, Result};
pub use frontmatter::{parse_frontmatter, parse_frontmatter_raw, write_frontmatter};
pub use links::{LinkAction, LinkRelation};
pub use models::{
    Config, Effort, Epic, EpicStatus, IdPrefix, MarkplaneDocument, Note, NoteStatus, Plan,
    PlanStatus, Priority, StatusCategory, Task, TaskWorkflow, WorkflowConfig, default_note_types,
    default_task_types, default_task_workflow, generate_random_id, parse_id,
};
pub use project::{
    EpicUpdate, MoveDirective, NoteUpdate, Patch, PlanUpdate, Project, TaskUpdate, UpdateFields,
    apply_tag_changes, find_blocked_items,
};
pub use query::{QueryFilter, ScanScope};
pub use references::{
    AsymmetricLink, BrokenReference, DependencyCycle, build_reference_graph, detect_cycles,
    extract_references, find_orphans, validate_reciprocal_links, validate_references,
    validate_task_statuses,
};
pub use templates::render_template;
