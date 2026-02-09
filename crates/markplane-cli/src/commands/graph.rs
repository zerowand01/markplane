use colored::Colorize;
use markplane_core::{build_reference_graph, parse_id, Project};
use std::collections::HashSet;

pub fn run(id: String, depth: u32) -> anyhow::Result<()> {
    let project = Project::from_current_dir()?;
    let _ = parse_id(&id)?; // Validate ID format
    let graph = build_reference_graph(&project)?;

    println!("{}", format!("Dependency graph for {}", id).bold());
    println!();

    // Walk outgoing references (what this item depends on / references)
    let mut visited = HashSet::new();
    print_tree(&graph, &id, 0, depth, &mut visited, true);

    // Show reverse references (what references this item)
    println!();
    println!("{}", "Referenced by:".dimmed());
    let mut any_incoming = false;
    for (source_id, refs) in &graph {
        if refs.contains(&id) {
            println!("  {} → {}", source_id.cyan(), id);
            any_incoming = true;
        }
    }
    if !any_incoming {
        println!("  {}", "(none)".dimmed());
    }

    Ok(())
}

fn print_tree(
    graph: &std::collections::HashMap<String, Vec<String>>,
    id: &str,
    indent: u32,
    max_depth: u32,
    visited: &mut HashSet<String>,
    is_root: bool,
) {
    if indent > max_depth {
        return;
    }

    if visited.contains(id) {
        let prefix = "  ".repeat(indent as usize);
        println!("{}{} {}", prefix, "↻".yellow(), id.dimmed());
        return;
    }
    visited.insert(id.to_string());

    let prefix = "  ".repeat(indent as usize);
    if is_root {
        println!("{}{}", prefix, id.bold());
    } else {
        println!("{}└─ {}", prefix, id);
    }

    if let Some(refs) = graph.get(id) {
        for ref_id in refs {
            print_tree(graph, ref_id, indent + 1, max_depth, visited, false);
        }
    }
}
