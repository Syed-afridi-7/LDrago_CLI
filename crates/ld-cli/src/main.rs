use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "ldrago",
    author,
    version,
    about = "LDrago CLI - Local Twin AI Platform",
    long_about = "A high-performance local AI orchestrator connecting custom neural networks and local models."
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Check system health and backend connectivity
    Doctor,
    /// Model management commands
    Model {
        #[command(subcommand)]
        action: ModelCommands,
    },
    /// Ask a question to one of the brains
    Ask {
        /// Prompt to send
        prompt: String,
        /// Brain to use (coding, design, general)
        #[arg(short, long, default_value = "coding")]
        brain: String,
    },
    /// Print version information
    Version,
}

#[derive(Subcommand, Debug)]
enum ModelCommands {
    /// List available models and brains
    List,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Doctor) => {
            println!("Running LDrago system diagnostic...");
        }
        Some(Commands::Model { action }) => match action {
            ModelCommands::List => {
                println!("Listing available models...");
            }
        },
        Some(Commands::Ask { prompt, brain }) => {
            println!("Sending prompt to brain [{brain}]: {prompt}");
        }
        Some(Commands::Version) | None => {
            println!("ldrago {}", env!("CARGO_PKG_VERSION"));
        }
    }

    Ok(())
}
