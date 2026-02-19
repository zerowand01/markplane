use std::path::PathBuf;

pub fn run(project: Option<PathBuf>) -> anyhow::Result<()> {
    crate::mcp::run(project)
}
