use colored::Colorize;
use markplane_core::{validate_references, validate_task_statuses, find_orphans, Project};

pub fn run(orphans: bool) -> anyhow::Result<()> {
    let project = Project::from_current_dir()?;
    let mut has_errors = false;

    // Check for broken references
    let broken = validate_references(&project)?;
    if broken.is_empty() {
        println!("{} No broken references found.", "✓".green());
    } else {
        has_errors = true;
        println!(
            "{} {} broken reference(s):\n",
            "✗".red(),
            broken.len()
        );
        for br in &broken {
            println!(
                "  {} references {} (not found)",
                br.source_file.dimmed(),
                br.target_id.red()
            );
        }
    }

    // Check for invalid task statuses
    let invalid_statuses = validate_task_statuses(&project)?;
    if invalid_statuses.is_empty() {
        println!("{} All task statuses are valid.", "✓".green());
    } else {
        has_errors = true;
        println!(
            "\n{} {} invalid task status(es):\n",
            "✗".red(),
            invalid_statuses.len()
        );
        for br in &invalid_statuses {
            println!(
                "  {} has unknown status (not in configured workflow)",
                br.source_file.dimmed(),
            );
        }
    }

    if orphans {
        println!();
        let orphan_list = find_orphans(&project)?;
        if orphan_list.is_empty() {
            println!("{} No orphan items found.", "✓".green());
        } else {
            println!(
                "{} {} orphan item(s) (no incoming references):\n",
                "!".yellow(),
                orphan_list.len()
            );
            for id in &orphan_list {
                println!("  {}", id);
            }
        }
    }

    if has_errors {
        let total = broken.len() + invalid_statuses.len();
        return Err(anyhow::anyhow!("Found {} issue(s)", total));
    }

    Ok(())
}
