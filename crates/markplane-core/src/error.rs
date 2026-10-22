use thiserror::Error;

#[derive(Error, Debug)]
pub enum MarkplaneError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("YAML parsing error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    #[error("Atomic write failed: {0}")]
    Persist(#[from] tempfile::PersistError),

    #[error("Item not found: {0}")]
    NotFound(String),

    #[error("Invalid ID format: {0}")]
    InvalidId(String),

    #[error("Invalid status transition: {from} -> {to}")]
    InvalidTransition { from: String, to: String },

    #[error("Invalid status: {0}")]
    InvalidStatus(String),

    #[error("Duplicate ID: {0}")]
    DuplicateId(String),

    #[error("Broken reference: {0}")]
    BrokenReference(String),

    #[error("Project not initialized: {0}")]
    NotInitialized(String),

    #[error("Config error: {0}")]
    Config(String),

    #[error("Link error: {0}")]
    InvalidLink(String),

    #[error("Frontmatter error: {0}")]
    Frontmatter(String),

    #[error("Invalid position key: {0}")]
    InvalidPosition(String),

    #[error("{}", data_version_message(*found, *supported))]
    DataVersion { found: u32, supported: u32 },
}

fn data_version_message(found: u32, supported: u32) -> String {
    if found > supported {
        format!(
            "Data version mismatch: .markplane/config.yaml is version {found}, but this \
             markplane binary supports version {supported}. Upgrade markplane to a newer release."
        )
    } else {
        format!(
            "Data version mismatch: .markplane/config.yaml is version {found}, which is older \
             than the version {supported} this markplane binary expects. This release cannot \
             upgrade it automatically."
        )
    }
}

pub type Result<T> = std::result::Result<T, MarkplaneError>;
