use colored::Colorize;
use markplane_core::{validate_references, find_orphans, Project};

pub fn run(orphans: bool) -> anyhow::Result<()> {
    let project = Project::from_current_dir()?;

    // Check for broken references
    let broken = validate_references(&project)?;
    if broken.is_empty() {
        println!("{} No broken references found.", "✓".green());
    } else {
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

    if !broken.is_empty() {
        std::process::exit(1);
    }

    Ok(())
}
