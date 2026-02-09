pub fn run() -> anyhow::Result<()> {
    let snippet = r#"## Project Management
This project uses Markplane for project management. Key files:
- `.markplane/INDEX.md` - Navigation entry point
- `.markplane/.context/summary.md` - Current project state
- `.markplane/backlog/INDEX.md` - All work items
- `.markplane/plans/INDEX.md` - Implementation plans
When working on a task, read the relevant backlog item and its linked plan first."#;

    println!("{}", snippet);

    Ok(())
}
