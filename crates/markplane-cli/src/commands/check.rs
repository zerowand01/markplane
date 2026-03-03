use colored::Colorize;
use markplane_core::{
    LinkAction, LinkRelation, Project, detect_cycles, find_orphans, validate_reciprocal_links,
    validate_references, validate_task_statuses,
};

fn field_to_relation(forward_field: &str) -> Option<LinkRelation> {
    match forward_field {
        "blocks" => Some(LinkRelation::Blocks),
        "depends_on" => Some(LinkRelation::DependsOn),
        "plan" => Some(LinkRelation::Plan),
        "implements" => Some(LinkRelation::Implements),
        "related" => Some(LinkRelation::Related),
        _ => None,
    }
}

pub fn run(orphans: bool, fix: bool) -> anyhow::Result<()> {
    let project = Project::from_current_dir()?;

    // Check for broken references
    let broken = validate_references(&project)?;
    if broken.is_empty() {
        println!("{} No broken references found.", "✓".green());
    } else {
        println!("{} {} broken reference(s):\n", "✗".red(), broken.len());
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

    // Check for asymmetric reciprocal links
    let asymmetric = validate_reciprocal_links(&project)?;
    if asymmetric.is_empty() {
        println!("{} All reciprocal links are symmetric.", "✓".green());
    } else {
        println!("\n{} {} asymmetric link(s):\n", "✗".red(), asymmetric.len());
        for link in &asymmetric {
            println!(
                "  {} has {}: {} but {} is missing {}: {}",
                link.source_id.yellow(),
                link.forward_field,
                link.target_id,
                link.target_id.yellow(),
                link.missing_field,
                link.source_id,
            );
        }

        if fix {
            println!();
            let mut repaired = 0;
            for link in &asymmetric {
                if let Some(relation) = field_to_relation(&link.forward_field) {
                    match project.link_items(
                        &link.source_id,
                        &link.target_id,
                        relation,
                        LinkAction::Add,
                    ) {
                        Ok(()) => {
                            repaired += 1;
                            println!(
                                "  {} Repaired {} {} → {}",
                                "✓".green(),
                                link.forward_field,
                                link.source_id,
                                link.target_id,
                            );
                        }
                        Err(e) => {
                            println!(
                                "  {} Failed to repair {} {} → {}: {}",
                                "✗".red(),
                                link.forward_field,
                                link.source_id,
                                link.target_id,
                                e,
                            );
                        }
                    }
                }
            }
            println!(
                "\nRepaired {} of {} asymmetric link(s).",
                repaired,
                asymmetric.len()
            );
        }
    }

    // Check for dependency cycles
    let cycles = detect_cycles(&project)?;
    if cycles.is_empty() {
        println!("{} No dependency cycles found.", "✓".green());
    } else {
        println!("\n{} {} dependency cycle(s):\n", "✗".red(), cycles.len());
        for cycle in &cycles {
            println!("  {}", cycle.path.join(" → "));
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

    // Broken refs, invalid statuses, and cycles are never auto-fixable
    let unfixable = broken.len() + invalid_statuses.len() + cycles.len();
    if unfixable > 0 {
        return Err(anyhow::anyhow!(
            "Found {} issue(s)",
            unfixable + asymmetric.len()
        ));
    }
    // Asymmetric links are only an error if --fix wasn't used
    if !asymmetric.is_empty() && !fix {
        return Err(anyhow::anyhow!(
            "Found {} asymmetric link(s)",
            asymmetric.len()
        ));
    }

    Ok(())
}
