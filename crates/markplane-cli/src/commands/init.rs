use std::env;

use markplane_core::Project;

pub fn run(name: Option<String>, description: String, empty: bool) -> anyhow::Result<()> {
    let cwd = env::current_dir()?;
    let root = cwd.join(".markplane");

    let project_name = name.unwrap_or_else(|| {
        cwd.file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "my-project".to_string())
    });

    let project = Project::init(root, &project_name, &description)?;
    project.sync_all()?;

    println!("Initialized Markplane project: {}", project_name);

    if !empty {
        let ids = project.seed_starter_content()?;
        project.sync_all()?;

        println!();
        println!("  Seeded with starter content (1 epic, 2 tasks, 1 plan, 1 note)");
        println!();
        println!("  .markplane/");
        println!("  ├── config.yaml");
        println!("  ├── INDEX.md");
        println!("  ├── roadmap/          (EPIC-NNN)");
        println!("  ├── backlog/          (TASK-NNN)");
        println!("  ├── plans/            (PLAN-NNN)");
        println!("  ├── notes/            (NOTE-NNN)");
        println!("  ├── templates/");
        println!("  └── .context/");
        println!();
        println!("Next steps:");
        println!("  markplane ls                  # See your starter tasks");
        println!("  markplane show {}   # Review your setup checklist", ids[1]);
        println!("  markplane dashboard           # Project overview");
    } else {
        println!();
        println!("  .markplane/");
        println!("  ├── config.yaml");
        println!("  ├── INDEX.md");
        println!("  ├── roadmap/          (EPIC-NNN)");
        println!("  ├── backlog/          (TASK-NNN)");
        println!("  ├── plans/            (PLAN-NNN)");
        println!("  ├── notes/            (NOTE-NNN)");
        println!("  ├── templates/");
        println!("  └── .context/");
        println!();
        println!("Get started:");
        println!("  markplane add \"My first task\"");
        println!("  markplane ls");
    }

    Ok(())
}
