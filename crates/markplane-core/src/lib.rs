pub mod models;
pub mod frontmatter;
pub mod project;
pub mod query;
pub mod references;
pub mod index;
pub mod context;
pub mod templates;
pub mod position;
pub mod error;

pub use error::{MarkplaneError, Result};
pub use models::{
    Task, TaskStatus, Config, Effort, Epic, EpicStatus, IdPrefix, ItemType,
    MarkplaneDocument, Note, NoteStatus, NoteType, Plan, PlanStatus, Priority,
    generate_random_id, parse_id,
};
pub use frontmatter::{parse_frontmatter, parse_frontmatter_raw, write_frontmatter};
pub use project::{
    Project, find_blocked_items, apply_tag_changes,
    Patch, TaskUpdate, EpicUpdate, PlanUpdate, NoteUpdate, UpdateFields,
};
pub use query::{QueryFilter, ScanScope};
pub use references::{extract_references, validate_references, find_orphans, BrokenReference, build_reference_graph};
pub use templates::render_template;
