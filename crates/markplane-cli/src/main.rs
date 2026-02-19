mod commands;
mod mcp;

use clap::Parser;

#[derive(Parser)]
#[command(name = "markplane", about = "AI-native project management", version)]
struct Cli {
    #[command(subcommand)]
    command: commands::Commands,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    commands::execute(cli.command)
}
