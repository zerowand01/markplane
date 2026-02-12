use std::env;

use markplane_core::Project;

pub fn run(name: Option<String>, description: String) -> anyhow::Result<()> {
    let cwd = env::current_dir()?;
    let root = cwd.join(".markplane");

    let project_name = name.unwrap_or_else(|| {
        cwd.file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "my-project".to_string())
    });

    Project::init(root, &project_name, &description)?;

    println!("Initialized Markplane project: {}", project_name);
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

    Ok(())
}
