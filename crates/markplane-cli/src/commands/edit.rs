use std::env;
use std::process::Command;

use anyhow::{bail, Context};
use markplane_core::Project;

pub fn run(id: String) -> anyhow::Result<()> {
    let project = Project::from_current_dir()?;
    let path = project.item_path(&id)?;

    let editor = env::var("EDITOR")
        .or_else(|_| env::var("VISUAL"))
        .unwrap_or_else(|_| "vi".to_string());

    // Run via `sh -c` so $EDITOR values with arguments work (e.g. "code --wait")
    let status = Command::new("sh")
        .arg("-c")
        .arg(format!("{} \"$1\"", editor))
        .arg("--") // argv[0] for sh
        .arg(&path)
        .status()
        .with_context(|| format!("failed to launch editor: {editor}"))?;

    if !status.success() {
        bail!("editor exited with {status}");
    }

    Ok(())
}
