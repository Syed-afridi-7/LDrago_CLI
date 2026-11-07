pub mod commands;
pub mod config;

use clap::{Parser, Subcommand};

#[derive(Parser, Debug, Clone, PartialEq)]
#[command(
    name = "ldrago",
    author = "LDrago Team",
    version = env!("CARGO_PKG_VERSION"),
    about = "LDrago CLI — Multi-engine local AI orchestrator",
    long_about = "LDrago is a modular, production-ready AI CLI featuring two custom neural architectures built from scratch in PyTorch (Transformer & SSM) alongside local Ollama integration."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug, Clone, PartialEq)]
pub enum Commands {
    /// Run diagnostic probes across all three AI engines
    Doctor,
    /// Inspect and manage registered model engines
    Model {
        #[command(subcommand)]
        action: ModelCommands,
    },
    /// Inspect workspace context, directory tree, manifest, or keyword search
    Context {
        #[command(subcommand)]
        action: commands::context::ContextCommands,
    },
    /// Generate a grounded execution plan validated against JSON schema and security policies
    Plan {
        /// High-level objective to accomplish
        objective: String,
        /// Target brain: coding (Transformer), design (SSM), or general
        #[arg(short, long, default_value = "coding")]
        brain: String,
    },
    /// Display real-time dynamic development roadmap and completion dashboard
    Progress,
    /// Send an instruction or prompt to a brain
    Ask {
        /// Text instruction or prompt to evaluate
        prompt: String,
        /// Target brain: coding (Transformer), design (SSM), general (Ollama), or auto
        #[arg(short, long, default_value = "auto")]
        brain: String,
        /// Maximum number of tokens to generate
        #[arg(short, long, default_value_t = 64)]
        max_tokens: u32,
        /// Sampling temperature (0.0 to 1.0)
        #[arg(short, long, default_value_t = 0.7)]
        temperature: f32,
    },
    /// Train an LDrago brain model using local PyTorch training harness
    Train {
        /// Target brain: coding (Transformer) or design (SSM)
        brain: String,
        /// Run micro-training on a tiny batch for verification
        #[arg(long)]
        micro: bool,
        /// Number of training steps
        #[arg(short, long, default_value_t = 10)]
        steps: u32,
    },
    /// Print LDrago version and system architecture information
    Version,
}

#[derive(Subcommand, Debug, Clone, PartialEq)]
pub enum ModelCommands {
    /// List all registered model engines and their status
    List,
}
